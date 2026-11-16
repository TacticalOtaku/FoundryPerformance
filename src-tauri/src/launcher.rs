//! Ядро лаунчера: настройки, слоты, статистика, игровая сессия, телеметрия и локаль.
use crate::model::*;
use crate::probe::{self, ProbeResult};
use crate::store::Store;
use crate::windows::GameLaunch;
use crate::{engine_flags, profile, store};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub const AGENT_JS: &str = include_str!("../../agent/dist/agent.js");

const LANG_RUSSIAN: u16 = 0x19;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    Normal,
    /// Стандартные флаги, агент только убирает свои настройки Foundry — чтобы исключить наше влияние.
    Safe,
    /// Стандартные флаги + агент только для измерений.
    Baseline,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDto {
    pub settings: Settings,
    pub servers: Vec<Server>,
    pub stats: BTreeMap<String, ServerStats>,
    pub presets: BTreeMap<ProfileId, Levers>,
    pub notice: Option<Notice>,
    pub locale: &'static str,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Report {
    Session { avg: f32, low1: f32, profile: ProfileId },
    Bench { avg: f32, low1: f32, min: f32, profile: ProfileId },
    ProfileChanged { profile: ProfileId },
    WebglLost { early: bool },
    /// Агент загрузился в мир: это настоящий адрес Foundry (для хостингов он отличается от адреса входа).
    FoundryUrl { url: String },
}

#[derive(Debug, Default, PartialEq)]
struct Effects {
    stats: bool,
    servers: bool,
    settings: bool,
    notice: Option<Notice>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Boot<'a> {
    server_id: &'a str,
    profile_id: ProfileId,
    presets: &'a BTreeMap<ProfileId, Levers>,
    locale: &'a str,
    measure_only: bool,
    /// Безопасный режим: агент только убирает свои core-настройки из localStorage.
    safe: bool,
}

fn agent_script(boot: &Boot) -> String {
    let json = serde_json::to_string(boot).expect("Boot is always serializable");
    format!("window.__FP_BOOT__ = {json};\n{AGENT_JS}")
}

/// Порт HTTP-проверки серверов (в продакшене — живой `probe::probe`, в тестах — заглушка).
pub trait FoundryProbe: Send + Sync {
    fn probe(&self, url: String) -> Pin<Box<dyn Future<Output = ProbeResult> + Send>>;
}

pub struct LiveFoundryProbe;

impl FoundryProbe for LiveFoundryProbe {
    fn probe(&self, url: String) -> Pin<Box<dyn Future<Output = ProbeResult> + Send>> {
        Box::pin(async move { probe::probe(&url).await })
    }
}

/// Порт определения системного языка Windows.
pub trait SystemLocale: Send + Sync {
    fn lang(&self) -> &'static str;
}

pub struct WinSystemLocale;

impl SystemLocale for WinSystemLocale {
    fn lang(&self) -> &'static str {
        let lang = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
        if lang & 0x3ff == LANG_RUSSIAN {
            "ru"
        } else {
            "en"
        }
    }
}

pub fn system_locale() -> &'static str {
    WinSystemLocale.lang()
}

struct GameSession {
    server_id: String,
    /// Источники страниц, от которых в этой сессии принимаются отчёты агента:
    /// адрес входа, известный адрес игры и адрес, подтверждённый проверкой Foundry.
    origins: Vec<url::Origin>,
    /// Откат ANGLE выполняется не больше одного раза за сессию.
    fallback_done: bool,
}

impl GameSession {
    fn for_server(server: &Server) -> Self {
        Self { server_id: server.id.clone(), origins: session_origins(server), fallback_done: false }
    }
}

struct LauncherInner {
    settings: Settings,
    servers: Vec<Server>,
    stats: BTreeMap<String, ServerStats>,
    notice: Option<Notice>,
    session: Option<GameSession>,
}

/// Глубокий модуль лаунчера: инкапсулирует состояние, игровую сессию, телеметрию и запись на диск.
pub struct Launcher {
    store: Store,
    engine_dir: PathBuf,
    probe: Arc<dyn FoundryProbe>,
    locale: Arc<dyn SystemLocale>,
    inner: Mutex<LauncherInner>,
}

impl Launcher {
    pub fn new(store: Store) -> Self {
        Self::with_ports(store, store::engine_dir(), Arc::new(LiveFoundryProbe), Arc::new(WinSystemLocale))
    }

    pub fn with_ports(store: Store, engine_dir: PathBuf, probe: Arc<dyn FoundryProbe>, locale: Arc<dyn SystemLocale>) -> Self {
        let (settings, n1) = store.load_settings();
        let (servers, n2) = store.load_servers();
        let (stats, n3) = store.load_stats();
        let inner = LauncherInner { settings, servers, stats, notice: n1.or(n2).or(n3), session: None };
        Self { store, engine_dir, probe, locale, inner: Mutex::new(inner) }
    }

    fn effective_locale(&self, pref: Locale) -> &'static str {
        match pref {
            Locale::Ru => "ru",
            Locale::En => "en",
            Locale::Auto => self.locale.lang(),
        }
    }

    pub fn lang(&self) -> &'static str {
        let d = self.inner.lock().expect("launcher poisoned");
        self.effective_locale(d.settings.locale)
    }

    pub fn state_dto(&self, version: String) -> StateDto {
        let d = self.inner.lock().expect("launcher poisoned");
        StateDto {
            settings: d.settings.clone(),
            servers: d.servers.clone(),
            stats: d.stats.clone(),
            presets: profile::presets(),
            notice: d.notice.clone(),
            locale: self.effective_locale(d.settings.locale),
            version,
        }
    }

    pub fn clear_notice(&self) {
        self.inner.lock().expect("launcher poisoned").notice = None;
    }

    pub fn save_server(&self, server: Server) -> Result<Vec<Server>, String> {
        let name = server.name.trim().to_string();
        if name.is_empty() || name.chars().count() > 60 || server.id.trim().is_empty() {
            return Err("err.nameRequired".into());
        }
        let url = probe::normalize_url(&server.url)?;
        let mut d = self.inner.lock().expect("launcher poisoned");
        let clean = merge_server(d.servers.iter().find(|s| s.id == server.id), Server { name, url: url.to_string(), ..server });
        let mut next = d.servers.clone();
        match next.iter_mut().find(|s| s.id == clean.id) {
            Some(s) => *s = clean,
            None => next.push(clean),
        }
        self.store.save_servers(&next).map_err(|_| "err.saveFailed".to_string())?;
        d.servers = next;
        Ok(d.servers.clone())
    }

    pub fn delete_server(&self, id: &str) -> Result<Vec<Server>, String> {
        let mut d = self.inner.lock().expect("launcher poisoned");
        let mut next = d.servers.clone();
        next.retain(|s| s.id != id);
        self.store.save_servers(&next).map_err(|_| "err.saveFailed".to_string())?;
        d.servers = next;
        if d.stats.remove(id).is_some() {
            let _ = self.store.save_stats(&d.stats);
        }
        if forget_server(&mut d.settings, id) {
            let _ = self.store.save_settings(&d.settings);
        }
        Ok(d.servers.clone())
    }

    pub fn save_settings(&self, settings: Settings) -> Result<Settings, String> {
        let mut d = self.inner.lock().expect("launcher poisoned");
        let next = merge_settings(&d.settings, settings);
        self.store.save_settings(&next).map_err(|_| "err.saveFailed".to_string())?;
        d.settings = next;
        Ok(d.settings.clone())
    }

    pub fn probe_server(&self, url: &str) -> Pin<Box<dyn Future<Output = ProbeResult> + Send>> {
        self.probe.probe(url.to_string())
    }

    /// Запускает сохранённый слот. `open` возвращает `Ok(true)`, если открыто новое окно игры,
    /// и `Ok(false)`, если поднято уже открытое — тогда сессия и `last_server` не трогаются.
    pub fn launch_server(&self, server_id: &str, safe_mode: bool, open: impl FnOnce(GameLaunch) -> Result<bool, String>) -> Result<bool, String> {
        let server = {
            let d = self.inner.lock().expect("launcher poisoned");
            d.servers.iter().find(|s| s.id == server_id).cloned().ok_or_else(|| "err.serverMissing".to_string())?
        };
        let mode = if safe_mode { LaunchMode::Safe } else { LaunchMode::Normal };
        self.start_game(&server, mode, true, open)
    }

    /// Запуск из командной строки: `--open <url> [--safe|--baseline]`.
    pub fn launch_adhoc(&self, url: &str, mode: LaunchMode, open: impl FnOnce(GameLaunch) -> Result<bool, String>) -> Result<bool, String> {
        let server = Server {
            id: "adhoc".into(),
            name: url.to_string(),
            url: url.to_string(),
            profile: None,
            overrides: Overrides::default(),
            game_url: None,
        };
        self.start_game(&server, mode, false, open)
    }

    fn start_game(
        &self,
        server: &Server,
        mode: LaunchMode,
        remember_last: bool,
        open: impl FnOnce(GameLaunch) -> Result<bool, String>,
    ) -> Result<bool, String> {
        let launch = {
            let d = self.inner.lock().expect("launcher poisoned");
            self.build_launch(&d.settings, server, mode)?
        };
        let opened = open(launch)?;
        if !opened {
            return Ok(false);
        }
        let mut d = self.inner.lock().expect("launcher poisoned");
        d.session = Some(GameSession::for_server(server));
        // При следующем открытии лаунчер предложит продолжить с этого сервера
        if remember_last && d.settings.last_server.as_deref() != Some(server.id.as_str()) {
            d.settings.last_server = Some(server.id.clone());
            let _ = self.store.save_settings(&d.settings);
        }
        Ok(true)
    }

    pub fn end_game_session(&self) {
        self.inner.lock().expect("launcher poisoned").session = None;
    }

    fn build_launch(&self, settings: &Settings, server: &Server, mode: LaunchMode) -> Result<GameLaunch, String> {
        let url = probe::normalize_url(&server.url)?;
        let presets = profile::resolve_all(settings, Some(server));
        let boot = Boot {
            server_id: &server.id,
            profile_id: profile::active_id(settings, Some(server)),
            presets: &presets,
            locale: self.effective_locale(settings.locale),
            measure_only: mode == LaunchMode::Baseline,
            safe: mode == LaunchMode::Safe,
        };
        Ok(GameLaunch {
            url,
            title: format!("{} — Foundry Performance", server.name),
            browser_args: match mode {
                LaunchMode::Normal => engine_flags::build(&settings.engine),
                LaunchMode::Safe | LaunchMode::Baseline => engine_flags::safe(),
            },
            init_script: Some(agent_script(&boot)),
            data_dir: self.engine_dir.clone(),
        })
    }

    /// Принимает телеметрию только от доверенных страниц активной игровой сессии (или от новой
    /// страницы самого мира при подтверждённом `FoundryUrl`), с одним захватом мьютекса.
    pub async fn report_telemetry(&self, page: &url::Url, report: Report) -> Result<(), String> {
        if !validate_report(&report) {
            return Err("rejected".into());
        }
        if let Report::FoundryUrl { url } = &report {
            if !describes_page(url, page) || !self.probe.probe(url.clone()).await.foundry {
                return Err("rejected".into());
            }
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|t| t.as_secs()).unwrap_or(0);
        let mut guard = self.inner.lock().expect("launcher poisoned");
        let d = &mut *guard;
        let Some(session) = d.session.as_mut() else {
            return Err("rejected".into());
        };
        if let Report::FoundryUrl { .. } = &report {
            if !session.origins.contains(&page.origin()) {
                session.origins.push(page.origin());
            }
        } else if !page_trusted(page, &session.origins) {
            return Err("rejected".into());
        }
        if matches!(report, Report::WebglLost { early: true }) {
            if session.fallback_done {
                return Ok(());
            }
            session.fallback_done = true;
        }
        let server_id = session.server_id.clone();
        let fx = apply_report(&report, &server_id, now, &mut d.stats, &mut d.servers, &mut d.settings);
        if fx.stats {
            let _ = self.store.save_stats(&d.stats);
        }
        if fx.servers {
            let _ = self.store.save_servers(&d.servers);
        }
        if fx.settings {
            let _ = self.store.save_settings(&d.settings);
        }
        if fx.notice.is_some() {
            d.notice = fx.notice;
        }
        Ok(())
    }
}

pub fn parse_cli(args: impl Iterator<Item = String>) -> Option<(String, LaunchMode)> {
    let mut url = None;
    let mut mode = LaunchMode::Normal;
    let mut it = args;
    while let Some(a) = it.next() {
        match a.as_str() {
            "--open" => url = it.next(),
            "--safe" => mode = LaunchMode::Safe,
            "--baseline" => mode = LaunchMode::Baseline,
            _ => {}
        }
    }
    url.map(|u| (u, mode))
}

/// Адрес игры узнаёт только агент — из интерфейса его не принимаем.
/// Сохраняется, пока адрес входа тот же; сменили адрес — узнаем заново при следующем входе.
fn merge_server(stored: Option<&Server>, incoming: Server) -> Server {
    let game_url = stored.filter(|s| s.url == incoming.url).and_then(|s| s.game_url.clone());
    Server { game_url, ..incoming }
}

/// Последний запущенный сервер пишет только лаунчер: копия настроек в интерфейсе может быть старше.
fn merge_settings(stored: &Settings, incoming: Settings) -> Settings {
    Settings { schema: SCHEMA, last_server: stored.last_server.clone(), ..incoming }
}

/// Удалённый сервер больше не «последний запущенный». `true` — настройки изменились.
fn forget_server(settings: &mut Settings, id: &str) -> bool {
    if settings.last_server.as_deref() != Some(id) {
        return false;
    }
    settings.last_server = None;
    true
}

fn fps_ok(x: f32) -> bool {
    x.is_finite() && (0.0..=1000.0).contains(&x)
}

fn validate_report(r: &Report) -> bool {
    match r {
        Report::Session { avg, low1, .. } => fps_ok(*avg) && fps_ok(*low1),
        Report::Bench { avg, low1, min, .. } => fps_ok(*avg) && fps_ok(*low1) && fps_ok(*min),
        Report::ProfileChanged { .. } | Report::WebglLost { .. } => true,
        Report::FoundryUrl { url } => {
            url.len() <= 2048 && (url.starts_with("https://") || url.starts_with("http://")) && probe::normalize_url(url).is_ok()
        }
    }
}

/// Откуда в этой сессии ждём отчёты: адрес входа и уже известный адрес игры.
fn session_origins(server: &Server) -> Vec<url::Origin> {
    let mut out: Vec<url::Origin> = Vec::new();
    for u in [Some(server.url.as_str()), server.game_url.as_deref()].into_iter().flatten() {
        let Ok(u) = probe::normalize_url(u) else { continue };
        // Страница игры хостинга доверенная сразу — проба её /api/status может быть закрыта
        for o in [Some(u.origin()), probe::hosted_foundry(&u).map(|h| h.origin())].into_iter().flatten() {
            if !out.contains(&o) {
                out.push(o);
            }
        }
    }
    out
}

fn page_trusted(page: &url::Url, origins: &[url::Origin]) -> bool {
    origins.contains(&page.origin())
}

/// Страница может сообщить только свой собственный адрес.
fn describes_page(reported: &str, page: &url::Url) -> bool {
    probe::normalize_url(reported).is_ok_and(|u| u.origin() == page.origin())
}

fn apply_report(
    report: &Report,
    server_id: &str,
    now: u64,
    stats: &mut BTreeMap<String, ServerStats>,
    servers: &mut [Server],
    settings: &mut Settings,
) -> Effects {
    let mut fx = Effects::default();
    match *report {
        Report::Session { avg, low1, profile } => {
            let s = stats.entry(server_id.to_string()).or_default();
            s.last_session = Some(FpsSummary { avg, low1, profile, at: now });
            fx.stats = true;
        }
        Report::Bench { avg, low1, min, profile } => {
            stats.entry(server_id.to_string()).or_default().last_bench = Some(BenchResult { avg, low1, min, profile, at: now });
            fx.stats = true;
        }
        Report::ProfileChanged { profile } => {
            if let Some(s) = servers.iter_mut().find(|s| s.id == server_id) {
                s.profile = Some(profile);
                fx.servers = true;
            }
        }
        Report::WebglLost { early: true } => {
            let from = settings.engine.angle;
            settings.engine.angle = from.next();
            fx.settings = true;
            fx.notice = Some(
                Notice::new("notice.engineFallback")
                    .with("from", from.flag_value())
                    .with("to", settings.engine.angle.flag_value()),
            );
        }
        Report::WebglLost { early: false } => {}
        Report::FoundryUrl { ref url } => {
            let Some(s) = servers.iter_mut().find(|s| s.id == server_id) else { return fx };
            let (Ok(game), Ok(slot)) = (probe::normalize_url(url), probe::normalize_url(&s.url)) else { return fx };
            // Прямой адрес Foundry в слоте — отдельный адрес игры не нужен
            let game = (game != slot).then(|| game.to_string());
            if s.game_url != game {
                s.game_url = game;
                fx.servers = true;
            }
        }
    }
    fx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct StubProbe {
        results: Mutex<HashMap<String, ProbeResult>>,
        calls: AtomicUsize,
    }

    impl StubProbe {
        fn new() -> Self {
            Self { results: Mutex::new(HashMap::new()), calls: AtomicUsize::new(0) }
        }

        fn insert(&self, url: &str, result: ProbeResult) {
            self.results.lock().unwrap().insert(url.to_string(), result);
        }
    }

    impl FoundryProbe for StubProbe {
        fn probe(&self, url: String) -> Pin<Box<dyn Future<Output = ProbeResult> + Send>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let res = self.results.lock().unwrap().get(&url).cloned().unwrap_or_default();
            Box::pin(async move { res })
        }
    }

    struct StubLocale(&'static str);

    impl SystemLocale for StubLocale {
        fn lang(&self) -> &'static str {
            self.0
        }
    }

    fn foundry_ok() -> ProbeResult {
        ProbeResult { reachable: true, foundry: true, active: true, ..ProbeResult::default() }
    }

    fn make_launcher(locale: &'static str) -> (tempfile::TempDir, Arc<StubProbe>, Launcher) {
        let dir = tempfile::tempdir().unwrap();
        let probe = Arc::new(StubProbe::new());
        let l = Launcher::with_ports(Store::new(dir.path().to_path_buf()), dir.path().join("engine"), probe.clone(), Arc::new(StubLocale(locale)));
        (dir, probe, l)
    }

    fn srv(id: &str, url: &str) -> Server {
        Server { id: id.into(), name: "Страд".into(), url: url.into(), profile: Some(ProfileId::Potato), overrides: Overrides::default(), game_url: None }
    }

    #[test]
    fn normal_safe_and_baseline_launch_builds() {
        let (_d, _p, l) = make_launcher("ru");
        let s = srv("a1", "vtt.example.com/game");
        let settings = Settings::default();

        let normal = l.build_launch(&settings, &s, LaunchMode::Normal).unwrap();
        assert_eq!(normal.url.as_str(), "https://vtt.example.com/");
        assert!(normal.browser_args.contains("--use-angle=d3d11"));
        assert!(normal.title.contains("Страд"));
        let script = normal.init_script.unwrap();
        assert!(script.starts_with("window.__FP_BOOT__ = {"));
        assert!(script.contains("\"profileId\":\"potato\""));
        assert!(script.contains("\"measureOnly\":false"));
        assert!(script.contains("\"locale\":\"ru\""));
        assert!(script.ends_with(AGENT_JS));

        let safe = l.build_launch(&settings, &s, LaunchMode::Safe).unwrap();
        assert_eq!(safe.browser_args, engine_flags::safe());
        assert!(safe.init_script.unwrap().contains("\"safe\":true"));

        let base = l.build_launch(&settings, &s, LaunchMode::Baseline).unwrap();
        assert_eq!(base.browser_args, engine_flags::safe());
        assert!(base.init_script.unwrap().contains("\"measureOnly\":true"));

        let bad = srv("b", "::::");
        assert_eq!(l.build_launch(&settings, &bad, LaunchMode::Normal).unwrap_err(), "err.urlInvalid");
    }

    #[test]
    fn parse_cli_args() {
        let a = |v: &[&str]| parse_cli(v.iter().map(|s| s.to_string()));
        assert_eq!(a(&["--open", "vtt.x"]), Some(("vtt.x".to_string(), LaunchMode::Normal)));
        assert_eq!(a(&["--safe", "--open", "vtt.x"]), Some(("vtt.x".to_string(), LaunchMode::Safe)));
        assert_eq!(a(&["--open", "vtt.x", "--baseline"]), Some(("vtt.x".to_string(), LaunchMode::Baseline)));
        assert_eq!(a(&[]), None);
    }

    #[test]
    fn locale_respects_preference_and_injected_system_locale() {
        let (_d, _p, ru) = make_launcher("ru");
        assert_eq!(ru.lang(), "ru");
        ru.save_settings(Settings { locale: Locale::En, ..Settings::default() }).unwrap();
        assert_eq!(ru.lang(), "en");

        let (_d2, _p2, en) = make_launcher("en");
        assert_eq!(en.lang(), "en");
        en.save_settings(Settings { locale: Locale::Ru, ..Settings::default() }).unwrap();
        assert_eq!(en.lang(), "ru");
    }

    #[test]
    fn save_server_validates_normalizes_and_preserves_game_url_until_address_changes() {
        let (d, _p, l) = make_launcher("en");

        assert_eq!(l.save_server(srv("a1", "https://x")).map(|_| ()), Ok(()));
        assert_eq!(l.save_server(Server { name: "   ".into(), ..srv("x", "https://x") }).unwrap_err(), "err.nameRequired");
        assert_eq!(l.save_server(srv("x", "::::")).unwrap_err(), "err.urlInvalid");

        // Преднамеренный game_url при создании нового слота не принимается
        l.save_server(Server { game_url: Some("https://spoofed/".into()), ..srv("s1", "https://www.sqyre.app/games/x/") }).unwrap();
        let persisted = Store::new(d.path().to_path_buf()).load_servers().0;
        let s1 = persisted.iter().find(|s| s.id == "s1").unwrap();
        assert_eq!(s1.game_url, None);

        // Имитируем, что агент ранее открыл game_url, и переименовываем слот без смены url
        l.inner.lock().unwrap().servers.iter_mut().find(|s| s.id == "s1").unwrap().game_url = Some("https://x.sqyre.app/".into());
        let list = l.save_server(Server { name: "Новое имя".into(), game_url: None, ..srv("s1", "https://www.sqyre.app/games/x/") }).unwrap();
        assert_eq!(list.iter().find(|s| s.id == "s1").unwrap().game_url.as_deref(), Some("https://x.sqyre.app/"));

        // При смене адреса входа game_url сбрасывается и сохраняется на диск
        let list = l.save_server(srv("s1", "https://other.host/")).unwrap();
        assert_eq!(list.iter().find(|s| s.id == "s1").unwrap().game_url, None);
        let reloaded = Store::new(d.path().to_path_buf()).load_servers().0;
        assert_eq!(reloaded.iter().find(|s| s.id == "s1").unwrap().game_url, None);
    }

    #[test]
    fn save_settings_preserves_last_server_and_persists_to_disk() {
        let (d, _p, l) = make_launcher("en");
        l.save_server(srv("a1", "https://vtt.example.com/")).unwrap();
        l.launch_server("a1", false, |_| Ok(true)).unwrap();

        let saved = l.save_settings(Settings { profile: ProfileId::Potato, last_server: None, ..Settings::default() }).unwrap();
        assert_eq!(saved.last_server.as_deref(), Some("a1"));
        assert_eq!(saved.profile, ProfileId::Potato);

        let disk = Store::new(d.path().to_path_buf()).load_settings().0;
        assert_eq!(disk.last_server.as_deref(), Some("a1"));
        assert_eq!(disk.profile, ProfileId::Potato);
    }

    #[tokio::test]
    async fn delete_server_cascades_across_servers_stats_and_last_server() {
        let (d, _p, l) = make_launcher("en");
        l.save_server(srv("a1", "https://vtt.example.com/")).unwrap();
        l.save_server(srv("a2", "https://other.example.com/")).unwrap();
        l.launch_server("a1", false, |_| Ok(true)).unwrap();

        let page: url::Url = "https://vtt.example.com/game".parse().unwrap();
        l.report_telemetry(&page, Report::Session { avg: 55.0, low1: 33.0, profile: ProfileId::Balance }).await.unwrap();
        assert!(Store::new(d.path().to_path_buf()).load_stats().0.contains_key("a1"));

        // Удаление другого слота оставляет last_server и статистику a1
        l.delete_server("a2").unwrap();
        assert_eq!(l.state_dto("0.3.0".into()).settings.last_server.as_deref(), Some("a1"));

        // Удаление активного last_server сбрасывает слот, статистику и last_server на диске
        let remaining = l.delete_server("a1").unwrap();
        assert!(remaining.is_empty());
        let store = Store::new(d.path().to_path_buf());
        assert!(store.load_servers().0.is_empty());
        assert!(!store.load_stats().0.contains_key("a1"));
        assert_eq!(store.load_settings().0.last_server, None);
    }

    #[tokio::test]
    async fn launch_failures_and_existing_focus_leave_session_and_last_server_untouched() {
        let (d, _p, l) = make_launcher("en");
        l.save_server(srv("a1", "https://vtt.example.com/")).unwrap();
        l.save_server(srv("a2", "https://two.example.com/")).unwrap();
        assert!(l.launch_server("a1", false, |_| Ok(true)).unwrap());

        // Ошибка открытия окна не портит активную сессию и last_server
        let err = l.launch_server("a2", false, |_| Err("notice.launchFailed".into())).unwrap_err();
        assert_eq!(err, "notice.launchFailed");
        assert_eq!(l.state_dto("0.3.0".into()).settings.last_server.as_deref(), Some("a1"));
        assert_eq!(Store::new(d.path().to_path_buf()).load_settings().0.last_server.as_deref(), Some("a1"));

        // Поднятие уже открытого окна (Ok(false)) тоже не меняет сессию и last_server
        assert!(!l.launch_server("a2", false, |_| Ok(false)).unwrap());
        assert_eq!(l.state_dto("0.3.0".into()).settings.last_server.as_deref(), Some("a1"));

        // А отчёт от a1 по-прежнему принимается, от a2 — отклоняется
        l.report_telemetry(&"https://vtt.example.com/game".parse().unwrap(), Report::Session { avg: 60.0, low1: 45.0, profile: ProfileId::Quality })
            .await
            .unwrap();
        assert!(l
            .report_telemetry(&"https://two.example.com/game".parse().unwrap(), Report::Session { avg: 60.0, low1: 45.0, profile: ProfileId::Quality })
            .await
            .is_err());

        // Закрытие последнего окна сбрасывает всю сессию целиком
        l.end_game_session();
        assert!(l
            .report_telemetry(&"https://vtt.example.com/game".parse().unwrap(), Report::Session { avg: 60.0, low1: 45.0, profile: ProfileId::Quality })
            .await
            .is_err());
    }

    #[tokio::test]
    async fn telemetry_enforces_origins_promotes_foundry_url_and_guards_webgl_fallback_once() {
        let (d, probe, l) = make_launcher("en");
        l.save_server(srv("a1", "https://www.sqyre.app/games/x/")).unwrap();
        l.launch_server("a1", false, |_| Ok(true)).unwrap();

        let evil: url::Url = "https://evil.example/x".parse().unwrap();
        let custom_game: url::Url = "https://custom.vtt.host/game?session=1".parse().unwrap();

        // Непроверенный источник и невалидные числа отвергаются без записи
        assert!(l.report_telemetry(&evil, Report::Session { avg: 50.0, low1: 30.0, profile: ProfileId::Balance }).await.is_err());
        assert!(l
            .report_telemetry(&"https://www.sqyre.app/games/x/".parse().unwrap(), Report::Session { avg: f32::NAN, low1: 30.0, profile: ProfileId::Balance })
            .await
            .is_err());
        assert!(l
            .report_telemetry(&custom_game, Report::FoundryUrl { url: "https://evil.example/".into() })
            .await
            .is_err());
        assert!(l
            .report_telemetry(&custom_game, Report::FoundryUrl { url: "javascript:alert(1)".into() })
            .await
            .is_err());

        // Неподтверждённый пробой FoundryUrl не даёт доверия источнику
        assert!(l
            .report_telemetry(&custom_game, Report::FoundryUrl { url: "https://custom.vtt.host/game".into() })
            .await
            .is_err());
        assert!(l.report_telemetry(&custom_game, Report::Session { avg: 50.0, low1: 30.0, profile: ProfileId::Balance }).await.is_err());

        // Подтверждённый пробой FoundryUrl сохраняет game_url и добавляет источник в сессию
        probe.insert("https://custom.vtt.host/game", foundry_ok());
        l.report_telemetry(&custom_game, Report::FoundryUrl { url: "https://custom.vtt.host/game".into() })
            .await
            .unwrap();
        let store = Store::new(d.path().to_path_buf());
        assert_eq!(store.load_servers().0[0].game_url.as_deref(), Some("https://custom.vtt.host/"));

        // Теперь статистика, смена профиля и бенч с этого источника принимаются и сохраняются
        l.report_telemetry(&custom_game, Report::Session { avg: 58.0, low1: 40.0, profile: ProfileId::Balance }).await.unwrap();
        l.report_telemetry(&custom_game, Report::Bench { avg: 41.2, low1: 20.5, min: 12.0, profile: ProfileId::Potato }).await.unwrap();
        l.report_telemetry(&custom_game, Report::ProfileChanged { profile: ProfileId::Potato }).await.unwrap();

        let stats = store.load_stats().0;
        assert_eq!(stats["a1"].last_session.unwrap().avg, 58.0);
        assert_eq!(stats["a1"].last_bench.unwrap().min, 12.0);
        assert_eq!(store.load_servers().0[0].profile, Some(ProfileId::Potato));

        // Ранний WebglLost откатывает ANGLE один раз за сессию; повторный в этой же сессии — no-op
        l.report_telemetry(&custom_game, Report::WebglLost { early: true }).await.unwrap();
        l.report_telemetry(&custom_game, Report::WebglLost { early: true }).await.unwrap();
        let dto = l.state_dto("0.3.0".into());
        assert_eq!(dto.settings.engine.angle, AngleBackend::D3d11on12);
        assert_eq!(store.load_settings().0.engine.angle, AngleBackend::D3d11on12);
        let notice = dto.notice.unwrap();
        assert_eq!((notice.key.as_str(), notice.params["to"].as_str()), ("notice.engineFallback", "d3d11on12"));

        // Новая сессия снова разрешает один откат ANGLE
        l.end_game_session();
        l.launch_server("a1", false, |_| Ok(true)).unwrap();
        l.report_telemetry(&custom_game, Report::WebglLost { early: true }).await.unwrap();
        assert_eq!(l.state_dto("0.3.0".into()).settings.engine.angle, AngleBackend::Gl);
    }

    #[tokio::test]
    async fn probe_server_delegates_to_injected_probe_port() {
        let (_d, probe, l) = make_launcher("en");
        probe.insert("https://vtt.example.com/", foundry_ok());
        assert!(l.probe_server("https://vtt.example.com/").await.foundry);
        assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn parses_agent_report_json() {
        let r: Report = serde_json::from_str(r#"{"kind":"bench","avg":41.2,"low1":20.5,"min":12.0,"profile":"potato"}"#).unwrap();
        assert_eq!(r, Report::Bench { avg: 41.2, low1: 20.5, min: 12.0, profile: ProfileId::Potato });
        let r: Report = serde_json::from_str(r#"{"kind":"webglLost","early":true}"#).unwrap();
        assert_eq!(r, Report::WebglLost { early: true });
    }

    #[test]
    fn sqyre_game_host_is_trusted_from_launch() {
        let s = srv("a", "https://www.sqyre.app/games/aldarionv210-3c11b9b6/");
        let origins = session_origins(&s);
        assert!(page_trusted(&"https://aldarionv210-3c11b9b6.games.sqyre.app/game".parse().unwrap(), &origins));
        assert!(!page_trusted(&"https://other.games.sqyre.app/game".parse().unwrap(), &origins));
    }
}
