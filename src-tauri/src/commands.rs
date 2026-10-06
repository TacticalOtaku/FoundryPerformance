use crate::model::*;
use crate::state::{AppState, Data};
use crate::{agent, cache, engine_flags, gpu, locale, probe, profile, store, telemetry, windows};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State, Webview};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    Normal,
    /// Без агента, стандартные флаги — чтобы исключить наше влияние.
    Safe,
    /// Стандартные флаги + агент только для измерений.
    Baseline,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDto {
    settings: Settings,
    servers: Vec<Server>,
    stats: BTreeMap<String, ServerStats>,
    presets: BTreeMap<ProfileId, Levers>,
    gpu: Option<gpu::GpuInfo>,
    gpu_check_current: bool,
    recommended: ProfileId,
    notice: Option<Notice>,
    locale: &'static str,
    version: String,
}

pub fn build_launch(settings: &Settings, server: &Server, mode: LaunchMode) -> Result<windows::GameLaunch, String> {
    let url = probe::normalize_url(&server.url)?;
    let presets = profile::resolve_all(settings, Some(server));
    let boot = agent::Boot {
        server_id: &server.id,
        profile_id: profile::active_id(settings, Some(server)),
        presets: &presets,
        locale: locale::effective(settings.locale),
        measure_only: mode == LaunchMode::Baseline,
    };
    Ok(windows::GameLaunch {
        url,
        title: format!("{} — Foundry Performance", server.name),
        browser_args: match mode {
            LaunchMode::Normal => engine_flags::build(&settings.engine),
            LaunchMode::Safe | LaunchMode::Baseline => engine_flags::safe(),
        },
        init_script: match mode {
            LaunchMode::Safe => None,
            LaunchMode::Normal | LaunchMode::Baseline => Some(agent::script(&boot)),
        },
        data_dir: store::engine_dir(),
    })
}

/// Адрес игры узнаёт только агент — из интерфейса его не принимаем.
/// Сохраняется, пока адрес входа тот же; сменили адрес — узнаем заново при следующем входе.
pub fn merge_server(stored: Option<&Server>, incoming: Server) -> Server {
    let game_url = stored.filter(|s| s.url == incoming.url).and_then(|s| s.game_url.clone());
    Server { game_url, ..incoming }
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

fn start_game(app: &AppHandle, state: &AppState, server: &Server, mode: LaunchMode) -> Result<(), String> {
    let launch = {
        let mut d = state.data.lock().expect("state poisoned");
        d.current_server = Some(server.id.clone());
        d.current_mode = mode;
        d.session_fallback_done = false;
        d.session_origins = telemetry::session_origins(server);
        build_launch(&d.settings, server, mode)?
    };
    windows::open_game(app, launch).map_err(|e| {
        eprintln!("[foundry-performance] open_game failed: {e}");
        "notice.launchFailed".to_string()
    })
}

/// Запуск из командной строки: `--open <url> [--safe|--baseline]`.
pub fn launch_adhoc(app: &AppHandle, state: &AppState, url: &str, mode: LaunchMode) -> Result<(), String> {
    let server = Server { id: "adhoc".into(), name: url.to_string(), url: url.to_string(), profile: None, overrides: Overrides::default(), game_url: None };
    start_game(app, state, &server, mode)
}

#[tauri::command]
pub fn get_state(app: AppHandle, state: State<'_, AppState>) -> StateDto {
    let d = state.data.lock().expect("state poisoned");
    StateDto {
        settings: d.settings.clone(),
        servers: d.servers.clone(),
        stats: d.stats.clone(),
        presets: profile::presets(),
        gpu: gpu::best(&state.adapters).cloned(),
        gpu_check_current: gpu_check_current(&d.settings, &state.adapters),
        recommended: gpu::recommend(gpu::best(&state.adapters)),
        notice: d.notice.clone(),
        locale: locale::effective(d.settings.locale),
        version: app.package_info().version.to_string(),
    }
}

#[tauri::command]
pub fn save_server(state: State<'_, AppState>, server: Server) -> Result<Vec<Server>, String> {
    let name = server.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 60 || server.id.trim().is_empty() {
        return Err("err.nameRequired".into());
    }
    let url = probe::normalize_url(&server.url)?;
    let mut d = state.data.lock().expect("state poisoned");
    let clean = merge_server(d.servers.iter().find(|s| s.id == server.id), Server { name, url: url.to_string(), ..server });
    match d.servers.iter_mut().find(|s| s.id == clean.id) {
        Some(s) => *s = clean,
        None => d.servers.push(clean),
    }
    state.store.save_servers(&d.servers).map_err(|_| "err.saveFailed".to_string())?;
    Ok(d.servers.clone())
}

#[tauri::command]
pub fn delete_server(state: State<'_, AppState>, id: String) -> Result<Vec<Server>, String> {
    let mut d = state.data.lock().expect("state poisoned");
    d.servers.retain(|s| s.id != id);
    d.stats.remove(&id);
    state.store.save_servers(&d.servers).map_err(|_| "err.saveFailed".to_string())?;
    let _ = state.store.save_stats(&d.stats);
    Ok(d.servers.clone())
}

/// Вердикт проверки ускорения пишет только лаунчер: копия настроек в интерфейсе может быть старше.
pub fn merge_settings(stored: &Settings, incoming: Settings) -> Settings {
    Settings { schema: SCHEMA, gpu_check: stored.gpu_check.clone(), ..incoming }
}

/// Вердикт последней проверки относится к текущим бэкенду и видеокарте.
pub fn gpu_check_current(settings: &Settings, adapters: &[gpu::GpuInfo]) -> bool {
    let adapter = gpu::best(adapters).map_or("", |g| g.name.as_str());
    settings.gpu_check.as_ref().is_some_and(|c| c.is_current(settings.engine.angle, adapter))
}

/// Рендерер WebGL самого лаунчера: доступность GPU до запуска игры. Ничего не сохраняет.
pub fn classify_launcher(renderer: &str, adapters: &[gpu::GpuInfo]) -> Result<Verdict, String> {
    if renderer.len() > 512 {
        return Err("rejected".into());
    }
    Ok(gpu::classify(renderer, adapters))
}

#[tauri::command]
pub fn classify_renderer(state: State<'_, AppState>, renderer: String) -> Result<Verdict, String> {
    classify_launcher(&renderer, &state.adapters)
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    let mut d = state.data.lock().expect("state poisoned");
    d.settings = merge_settings(&d.settings, settings);
    state.store.save_settings(&d.settings).map_err(|_| "err.saveFailed".to_string())?;
    Ok(d.settings.clone())
}

#[tauri::command]
pub async fn probe_server(url: String) -> probe::ProbeResult {
    probe::probe(&url).await
}

/// async: на Windows создание окна из синхронной команды может взаимно заблокироваться.
#[tauri::command]
pub async fn launch(app: AppHandle, state: State<'_, AppState>, server_id: String, safe_mode: bool) -> Result<(), String> {
    let server = {
        let d = state.data.lock().expect("state poisoned");
        d.servers.iter().find(|s| s.id == server_id).cloned().ok_or_else(|| "err.serverMissing".to_string())?
    };
    let mode = if safe_mode { LaunchMode::Safe } else { LaunchMode::Normal };
    start_game(&app, &state, &server, mode)
}

#[tauri::command]
pub fn clear_notice(state: State<'_, AppState>) {
    state.data.lock().expect("state poisoned").notice = None;
}

/// Окна игры открывают и чужие страницы (ссылки из чата), а IPC им доступен.
/// Поэтому отчёт принимается только со страниц этой сессии, а новый адрес игры —
/// только о самой странице и только если по нему действительно отвечает Foundry.
#[tauri::command]
pub async fn report_telemetry(webview: Webview, state: State<'_, AppState>, report: telemetry::Report) -> Result<Option<Verdict>, String> {
    if !windows::is_game(webview.label()) || !telemetry::validate(&report) {
        return Err("rejected".into());
    }
    let page = webview.url().map_err(|_| "rejected".to_string())?;
    if let telemetry::Report::FoundryUrl { url } = &report {
        if !telemetry::describes_page(url, &page) || !probe::probe(url).await.foundry {
            return Err("rejected".into());
        }
        let mut d = state.data.lock().expect("state poisoned");
        if !d.session_origins.contains(&page.origin()) {
            d.session_origins.push(page.origin());
        }
    } else if !telemetry::page_trusted(&page, &state.data.lock().expect("state poisoned").session_origins) {
        return Err("rejected".into());
    }
    let mut guard = state.data.lock().expect("state poisoned");
    let d = &mut *guard;
    let Some(server_id) = d.current_server.clone() else { return Ok(None) };
    if let telemetry::Report::Gpu { renderer } = &report {
        let persist = d.current_mode == LaunchMode::Normal;
        let (verdict, fx) = telemetry::apply_gpu(renderer, &state.adapters, persist, !d.session_fallback_done, &mut d.settings);
        d.session_fallback_done |= fx.fallback;
        commit(&state, d, fx);
        return Ok(Some(verdict));
    }
    if matches!(report, telemetry::Report::WebglLost { early: true }) {
        if d.session_fallback_done {
            return Ok(None);
        }
        d.session_fallback_done = true;
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|t| t.as_secs()).unwrap_or(0);
    let fx = telemetry::apply(&report, &server_id, now, &mut d.stats, &mut d.servers, &mut d.settings);
    commit(&state, d, fx);
    Ok(None)
}

/// Сохраняет то, что изменил отчёт агента.
fn commit(state: &AppState, d: &mut Data, fx: telemetry::Effects) {
    if fx.stats {
        let _ = state.store.save_stats(&d.stats);
    }
    if fx.servers {
        let _ = state.store.save_servers(&d.servers);
    }
    if fx.settings {
        let _ = state.store.save_settings(&d.settings);
    }
    if fx.notice.is_some() {
        d.notice = fx.notice;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srv() -> Server {
        Server { id: "a1".into(), name: "Страд".into(), url: "vtt.example.com/game".into(), profile: Some(ProfileId::Potato), overrides: Overrides::default(), game_url: None }
    }

    #[test]
    fn normal_launch_uses_tuned_flags_and_agent() {
        let l = build_launch(&Settings::default(), &srv(), LaunchMode::Normal).unwrap();
        assert_eq!(l.url.as_str(), "https://vtt.example.com/");
        assert!(l.browser_args.contains("--use-angle=d3d11"));
        let script = l.init_script.unwrap();
        assert!(script.contains("\"profileId\":\"potato\""));
        assert!(script.contains("\"measureOnly\":false"));
        assert!(l.title.contains("Страд"));
    }

    #[test]
    fn safe_launch_has_no_agent_and_stock_flags() {
        let l = build_launch(&Settings::default(), &srv(), LaunchMode::Safe).unwrap();
        assert!(l.init_script.is_none());
        assert_eq!(l.browser_args, engine_flags::safe());
    }

    #[test]
    fn baseline_launch_measures_only() {
        let l = build_launch(&Settings::default(), &srv(), LaunchMode::Baseline).unwrap();
        assert_eq!(l.browser_args, engine_flags::safe());
        assert!(l.init_script.unwrap().contains("\"measureOnly\":true"));
    }

    #[test]
    fn invalid_url_is_rejected() {
        let s = Server { url: "::::".into(), ..srv() };
        assert_eq!(build_launch(&Settings::default(), &s, LaunchMode::Normal).unwrap_err(), "err.urlInvalid");
    }

    #[test]
    fn saving_keeps_discovered_game_url_until_address_changes() {
        let stored = Server { url: "https://www.sqyre.app/games/x/".into(), game_url: Some("https://x.sqyre.app/".into()), ..srv() };
        let renamed = merge_server(Some(&stored), Server { name: "Новое имя".into(), game_url: None, ..stored.clone() });
        assert_eq!(renamed.game_url.as_deref(), Some("https://x.sqyre.app/"));
        let moved = merge_server(Some(&stored), Server { url: "https://other.host/".into(), ..stored.clone() });
        assert_eq!(moved.game_url, None);
        let fresh = merge_server(None, Server { game_url: Some("https://spoofed/".into()), ..srv() });
        assert_eq!(fresh.game_url, None);
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
    fn saving_settings_keeps_the_launchers_gpu_check() {
        let check = GpuCheck { verdict: Verdict::Software, renderer: "r".into(), angle: AngleBackend::D3d11, adapter: "RTX".into() };
        let stored = Settings { gpu_check: Some(check.clone()), ..Settings::default() };
        // интерфейс прислал старую копию без проверки и с другим профилем
        let incoming = Settings { profile: ProfileId::Potato, schema: 0, gpu_check: None, ..Settings::default() };
        let merged = merge_settings(&stored, incoming);
        assert_eq!(merged.gpu_check, Some(check));
        assert_eq!(merged.profile, ProfileId::Potato);
        assert_eq!(merged.schema, SCHEMA);
    }

    #[test]
    fn gpu_check_is_current_only_for_the_same_backend_and_gpu() {
        let rtx = gpu::GpuInfo { name: "RTX".into(), vram_mb: 12288, vendor_id: 0x10DE };
        let check = GpuCheck { verdict: Verdict::Unknown, renderer: "r".into(), angle: AngleBackend::D3d11, adapter: "RTX".into() };
        let s = Settings { gpu_check: Some(check), ..Settings::default() };
        assert!(gpu_check_current(&s, std::slice::from_ref(&rtx)));
        let switched = Settings { engine: EngineSettings { angle: AngleBackend::Gl, ..EngineSettings::default() }, ..s.clone() };
        assert!(!gpu_check_current(&switched, std::slice::from_ref(&rtx)));
        assert!(!gpu_check_current(&Settings::default(), &[rtx]));
    }

    #[test]
    fn launcher_renderer_is_classified_and_length_limited() {
        let rtx = gpu::GpuInfo { name: "RTX".into(), vram_mb: 12288, vendor_id: 0x10DE };
        let nv = "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 (0x00002F04) Direct3D11 vs_5_0 ps_5_0, D3D11)";
        assert_eq!(classify_launcher(nv, std::slice::from_ref(&rtx)), Ok(Verdict::Hardware { backend: Some(AngleBackend::D3d11) }));
        assert!(classify_launcher(&"x".repeat(513), &[rtx]).is_err());
    }
}

/// F11 в игре: полный экран без рамки для того окна, где нажали.
#[tauri::command]
pub fn toggle_fullscreen(window: tauri::WebviewWindow) -> Result<(), String> {
    if !windows::is_game(window.label()) {
        return Err("not a game window".into());
    }
    let on = window.is_fullscreen().map_err(|e| e.to_string())?;
    window.set_fullscreen(!on).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cache_size() -> u64 {
    tauri::async_runtime::spawn_blocking(|| cache::size(&store::engine_dir())).await.unwrap_or(0)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearedDto {
    freed: u64,
    pending: bool,
}

#[tauri::command]
pub async fn clear_cache(app: AppHandle) -> Result<ClearedDto, String> {
    // Во время игры WebView2 держит эти файлы и пишет в них
    if app.webview_windows().keys().any(|l| windows::is_game(l)) {
        return Err("cache.err.gameOpen".into());
    }
    let r = tauri::async_runtime::spawn_blocking(|| cache::clear(&store::engine_dir())).await.map_err(|_| "cache.err.failed".to_string())?;
    Ok(ClearedDto { freed: r.freed, pending: r.pending })
}
