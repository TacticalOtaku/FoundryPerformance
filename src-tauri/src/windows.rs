//! Координатор окон и их положений: жизненный цикл сплэша, лаунчера, окон игры,
//! возврат из трея и запоминание координат на мониторах Windows.
use crate::installer::{Version, UPDATED_FLAG};
use crate::launcher::{self, LaunchMode, Launcher};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tauri::webview::{NewWindowFeatures, NewWindowResponse};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window, Wry};

pub const LAUNCHER: &str = "main";
pub const GAME: &str = "game";
pub const SPLASH: &str = "splash";

/// Сколько окна должно остаться на каком-нибудь мониторе, чтобы за него можно было взяться.
const MIN_VISIBLE_W: i64 = 120;
const MIN_VISIBLE_H: i64 = 60;
/// Больше любого реального монитора — такой размер мог прийти только из испорченного файла.
const MAX_SIDE: u32 = 16_384;
/// Размер «обычного» окна, если его видели только развёрнутым.
const FALLBACK_W: u32 = 1280;
const FALLBACK_H: u32 = 800;
/// Развёрнутое окно Windows вылезает за край монитора на ширину рамки — сдвигаем внутрь.
const MAXIMIZED_INSET: i32 = 16;

static CHILD_SEQ: AtomicU32 = AtomicU32::new(1);

/// Окно игры: основное (`game`) или открытое из неё (`game-N`).
pub fn is_game(label: &str) -> bool {
    label == GAME || label.starts_with("game-")
}

pub fn child_label(n: u32) -> String {
    format!("game-{n}")
}

/// Лаунчер возвращается, только когда закрыто последнее окно игры.
pub fn last_game_closed<'a>(mut remaining: impl Iterator<Item = &'a str>) -> bool {
    !remaining.any(is_game)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Launcher,
    Game,
    /// Первое окно без заданного размера, открытое игрой (у Sqyre — сам мир).
    Popup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    fn overlap(&self, o: &Rect) -> (i64, i64) {
        let w = (self.x as i64 + self.w as i64).min(o.x as i64 + o.w as i64) - (self.x as i64).max(o.x as i64);
        let h = (self.y as i64 + self.h as i64).min(o.y as i64 + o.h as i64) - (self.y as i64).max(o.y as i64);
        (w.max(0), h.max(0))
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && (x as i64) < self.x as i64 + self.w as i64 && (y as i64) < self.y as i64 + self.h as i64
    }
}

/// «Обычный» прямоугольник окна плюс поверх него — развёрнуто ли и на весь ли экран.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub maximized: bool,
    pub fullscreen: bool,
}

impl Placement {
    pub fn rect(&self) -> Rect {
        Rect { x: self.x, y: self.y, w: self.w, h: self.h }
    }
}

/// Текущее состояние окна, как его отдаёт Tauri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    pub rect: Rect,
    pub maximized: bool,
    pub fullscreen: bool,
    pub minimized: bool,
}

/// Окно открываем на прежнем месте, только если оно заметно видно хотя бы на одном мониторе.
pub fn fits(p: &Placement, monitors: &[Rect]) -> bool {
    (1..=MAX_SIDE).contains(&p.w)
        && (1..=MAX_SIDE).contains(&p.h)
        && monitors.iter().any(|m| {
            let (w, h) = p.rect().overlap(m);
            w >= MIN_VISIBLE_W && h >= MIN_VISIBLE_H
        })
}

fn monitor_of(x: i32, y: i32, monitors: &[Rect]) -> Option<usize> {
    monitors.iter().position(|m| m.contains(x, y))
}

/// Новое запоминаемое положение после движения или смены размера окна.
/// Свёрнутое окно ничего не меняет; развёрнутое хранит прежний обычный прямоугольник,
/// но переезжает на тот монитор, где его развернули.
pub fn merge(prev: Option<Placement>, s: Snapshot, monitors: &[Rect]) -> Option<Placement> {
    if s.minimized {
        return prev;
    }
    if !s.maximized && !s.fullscreen {
        return Some(Placement { x: s.rect.x, y: s.rect.y, w: s.rect.w, h: s.rect.h, maximized: false, fullscreen: false });
    }
    let (inside_x, inside_y) = (s.rect.x + MAXIMIZED_INSET, s.rect.y + MAXIMIZED_INSET);
    let base = match prev {
        Some(p) if monitor_of(p.x + MAXIMIZED_INSET, p.y + MAXIMIZED_INSET, monitors) == monitor_of(inside_x, inside_y, monitors) => p,
        Some(p) => Placement { x: inside_x, y: inside_y, ..p },
        None => Placement { x: inside_x, y: inside_y, w: FALLBACK_W, h: FALLBACK_H, maximized: false, fullscreen: false },
    };
    Some(Placement { maximized: s.maximized, fullscreen: s.fullscreen, ..base })
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Saved(BTreeMap<Role, Placement>);

fn state_file_path(dir: &Path) -> PathBuf {
    dir.join("windows.json")
}

/// Нет файла или он битый — просто открываем окна как в первый раз.
fn load_saved(dir: &Path) -> Saved {
    fs::read_to_string(state_file_path(dir)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn save_state(dir: &Path, saved: &Saved) -> std::io::Result<()> {
    crate::store::write_atomic(&state_file_path(dir), &serde_json::to_vec_pretty(saved)?)
}

/// Что открыть, когда сплэш закончит.
#[derive(Debug, Clone, PartialEq)]
pub enum AfterSplash {
    Launcher,
    Adhoc { url: String, mode: LaunchMode },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseDecision {
    Prevent,
    Exit,
    PersistAndExit,
    Ignore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnDecision {
    CloseGames(Vec<String>),
    FocusSplash,
    OpenLauncher,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecondLaunchDecision {
    Focus(String),
    LaunchAdhoc { url: String, mode: LaunchMode },
    OpenLauncher,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DestroyOutcome {
    persist: bool,
    end_game_session: bool,
    open_launcher: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChildPlan {
    claimed_popup: bool,
    maximized: bool,
    visible: bool,
}

#[derive(Debug)]
pub struct GameLaunch {
    pub url: url::Url,
    pub title: String,
    pub browser_args: String,
    pub init_script: Option<String>,
    pub data_dir: PathBuf,
}

/// Абстракция оконной системы: позволяет проверять переходы и `Popup`-инварианты без `AppHandle`.
pub trait WindowHost {
    fn open_labels(&self) -> Vec<String>;
    fn focus(&self, label: &str) -> bool;
    fn destroy(&self, label: &str);
    fn open_launcher(&self) -> Result<(), String>;
    fn open_splash(&self) -> Result<(), String>;
    fn open_game(&self, launch: GameLaunch) -> Result<(), String>;
}

struct CoordinatorInner {
    saved: Saved,
    /// Метка окна, которое сейчас играет роль `Popup`.
    popup: Option<String>,
    after_splash: AfterSplash,
    /// Версия, до которой только что обновились (`--updated`); отдаётся сплэшу один раз.
    updated: Option<String>,
}

impl CoordinatorInner {
    fn new(saved: Saved, args: &[String]) -> Self {
        let after_splash = match launcher::parse_cli(args.iter().cloned()) {
            Some((url, mode)) => AfterSplash::Adhoc { url, mode },
            None => AfterSplash::Launcher,
        };
        let updated = args.iter().any(|a| a == UPDATED_FLAG).then(|| Version::current().to_string());
        Self { saved, popup: None, after_splash, updated }
    }

    fn role_of(&self, label: &str) -> Option<Role> {
        match label {
            LAUNCHER => Some(Role::Launcher),
            GAME => Some(Role::Game),
            l if self.popup.as_deref() == Some(l) => Some(Role::Popup),
            _ => None,
        }
    }

    /// Роль `Popup` получает первое окно без заданного размера, пока прежнее такое окно открыто — нет.
    fn claim_popup(&mut self, label: &str, no_size: bool, open: &[String]) -> ChildPlan {
        let busy = self.popup.as_deref().is_some_and(|l| open.iter().any(|o| o == l));
        let claimed_popup = no_size && !busy;
        if claimed_popup {
            self.popup = Some(label.to_string());
        }
        ChildPlan { claimed_popup, maximized: no_size && !claimed_popup, visible: !claimed_popup }
    }

    fn on_destroyed(&mut self, label: &str, open: &[String]) -> DestroyOutcome {
        if self.popup.as_deref() == Some(label) {
            self.popup = None;
        }
        let relevant = label == LAUNCHER || is_game(label);
        if !is_game(label) {
            return DestroyOutcome { persist: relevant, end_game_session: false, open_launcher: false };
        }
        let last = last_game_closed(open.iter().map(String::as_str).filter(|l| *l != label));
        DestroyOutcome { persist: true, end_game_session: last, open_launcher: last }
    }

    fn plan_return_to_launcher(open: &[String]) -> ReturnDecision {
        let games: Vec<String> = open.iter().filter(|l| is_game(l)).cloned().collect();
        if !games.is_empty() {
            ReturnDecision::CloseGames(games)
        } else if open.iter().any(|l| l == SPLASH) {
            ReturnDecision::FocusSplash
        } else {
            ReturnDecision::OpenLauncher
        }
    }

    fn existing_game(open: &[String]) -> Option<String> {
        open.iter()
            .find(|l| l.as_str() == GAME)
            .or_else(|| open.iter().find(|l| is_game(l)))
            .cloned()
    }

    fn plan_second_launch(open: &[String], cli: Option<(String, LaunchMode)>) -> SecondLaunchDecision {
        if open.iter().any(|l| l == SPLASH) {
            return SecondLaunchDecision::Focus(SPLASH.to_string());
        }
        if let Some((url, mode)) = cli {
            return SecondLaunchDecision::LaunchAdhoc { url, mode };
        }
        if let Some(game) = Self::existing_game(open) {
            return SecondLaunchDecision::Focus(game);
        }
        SecondLaunchDecision::OpenLauncher
    }

    fn plan_close_requested(label: &str, update_applying: bool) -> CloseDecision {
        match label {
            SPLASH if update_applying => CloseDecision::Prevent,
            SPLASH => CloseDecision::Exit,
            LAUNCHER => CloseDecision::PersistAndExit,
            _ => CloseDecision::Ignore,
        }
    }
}

/// Единый координатор жизненного цикла окон и сохранения их положений.
pub struct WindowCoordinator {
    dir: PathBuf,
    inner: Mutex<CoordinatorInner>,
}

impl WindowCoordinator {
    pub fn new(dir: PathBuf, args: &[String]) -> Self {
        let saved = load_saved(&dir);
        Self { dir, inner: Mutex::new(CoordinatorInner::new(saved, args)) }
    }

    pub fn take_updated(&self) -> Option<String> {
        self.inner.lock().expect("window coordinator poisoned").updated.take()
    }

    pub fn role_of(&self, label: &str) -> Option<Role> {
        self.inner.lock().expect("window coordinator poisoned").role_of(label)
    }

    /// Вызывается на каждое движение и смену размера отслеживаемого окна.
    pub fn track(&self, window: &Window) {
        let (Ok(pos), Ok(size)) = (window.outer_position(), window.inner_size()) else { return };
        let snap = Snapshot {
            rect: Rect { x: pos.x, y: pos.y, w: size.width, h: size.height },
            maximized: window.is_maximized().unwrap_or(false),
            fullscreen: window.is_fullscreen().unwrap_or(false),
            minimized: window.is_minimized().unwrap_or(false),
        };
        let monitors = monitors(window.available_monitors().unwrap_or_default());
        let mut inner = self.inner.lock().expect("window coordinator poisoned");
        let Some(role) = inner.role_of(window.label()) else { return };
        if let Some(p) = merge(inner.saved.0.get(&role).copied(), snap, &monitors) {
            inner.saved.0.insert(role, p);
        }
    }

    pub fn persist(&self) {
        let inner = self.inner.lock().expect("window coordinator poisoned");
        if let Err(e) = save_state(&self.dir, &inner.saved) {
            eprintln!("[foundry-performance] window state not saved: {e}");
        }
    }

    /// Сохранённое место для роли, если оно ещё видно на подключённых мониторах.
    pub fn placement_for(&self, role: Role, monitors: &[Rect]) -> Option<Placement> {
        let inner = self.inner.lock().expect("window coordinator poisoned");
        let p = inner.saved.0.get(&role).copied()?;
        fits(&p, monitors).then_some(p)
    }

    fn plan_child(&self, label: &str, no_size: bool, open: &[String]) -> ChildPlan {
        self.inner.lock().expect("window coordinator poisoned").claim_popup(label, no_size, open)
    }

    /// Переход «Лаунчер → Игра»: закрывает `main`, только если открыто новое окно игры.
    pub fn launch_server<H: WindowHost>(&self, host: &H, launcher: &Launcher, server_id: &str, safe_mode: bool) -> Result<(), String> {
        let opened = launcher.launch_server(server_id, safe_mode, |game| self.open_or_focus_game(host, game))?;
        if opened {
            host.destroy(LAUNCHER);
        }
        Ok(())
    }

    /// Запуск по `--open <url>`: закрывает `main` (если был открыт) только при открытии нового окна игры.
    pub fn launch_adhoc<H: WindowHost>(&self, host: &H, launcher: &Launcher, url: &str, mode: LaunchMode) -> Result<(), String> {
        let opened = launcher.launch_adhoc(url, mode, |game| self.open_or_focus_game(host, game))?;
        if opened {
            host.destroy(LAUNCHER);
        }
        Ok(())
    }

    fn open_or_focus_game<H: WindowHost>(&self, host: &H, game: GameLaunch) -> Result<bool, String> {
        let open = host.open_labels();
        if let Some(existing) = CoordinatorInner::existing_game(&open) {
            if host.focus(&existing) {
                return Ok(false);
            }
        }
        host.open_game(game).map(|_| true)
    }

    /// Переход «Сплэш → Лаунчер или Игра». Если запуск игры по `--open` не удался — открывает лаунчер.
    pub fn finish_splash<H: WindowHost>(&self, host: &H, launcher: &Launcher) {
        let after = {
            let mut inner = self.inner.lock().expect("window coordinator poisoned");
            std::mem::replace(&mut inner.after_splash, AfterSplash::Launcher)
        };
        let game = match after {
            AfterSplash::Adhoc { url, mode } => self
                .launch_adhoc(host, launcher, &url, mode)
                .map_err(|e| eprintln!("[foundry-performance] launch after splash failed: {e}"))
                .is_ok(),
            AfterSplash::Launcher => false,
        };
        if !game {
            if let Err(e) = host.open_launcher() {
                eprintln!("[foundry-performance] open_launcher after splash failed: {e}");
            }
        }
        host.destroy(SPLASH);
    }

    /// Переход «Лаунчер → Сплэш обновления».
    pub fn restart_to_update<H: WindowHost>(&self, host: &H) -> Result<(), String> {
        self.inner.lock().expect("window coordinator poisoned").after_splash = AfterSplash::Launcher;
        host.open_splash()?;
        // Программный destroy() не порождает CloseRequested — приложение не выходит
        host.destroy(LAUNCHER);
        Ok(())
    }

    /// Переход из трея «Вернуться в лаунчер».
    pub fn return_to_launcher<H: WindowHost>(&self, host: &H) {
        let open = host.open_labels();
        match CoordinatorInner::plan_return_to_launcher(&open) {
            ReturnDecision::CloseGames(games) => {
                // Закрытие последнего окна игры само завершит сессию и откроет лаунчер (см. on_destroyed)
                for label in games {
                    host.destroy(&label);
                }
            }
            ReturnDecision::FocusSplash => {
                host.focus(SPLASH);
            }
            ReturnDecision::OpenLauncher => {
                let _ = host.open_launcher();
            }
        }
    }

    /// Повторный запуск процесса (single-instance): поднимает активное окно или запускает `--open`.
    pub fn handle_second_launch<H: WindowHost>(&self, host: &H, launcher: &Launcher, argv: Vec<String>) {
        let open = host.open_labels();
        match CoordinatorInner::plan_second_launch(&open, launcher::parse_cli(argv.into_iter().skip(1))) {
            SecondLaunchDecision::Focus(label) => {
                host.focus(&label);
            }
            SecondLaunchDecision::LaunchAdhoc { url, mode } => {
                if let Err(e) = self.launch_adhoc(host, launcher, &url, mode) {
                    eprintln!("[foundry-performance] launch from second start failed: {e}");
                }
            }
            SecondLaunchDecision::OpenLauncher => {
                if let Err(e) = host.open_launcher() {
                    eprintln!("[foundry-performance] open_launcher on second start failed: {e}");
                }
            }
        }
    }

    /// Реакция на закрытие окна (`WindowEvent::Destroyed`): сохраняет положения и, если закрылось
    /// последнее окно игры, завершает игровую сессию и возвращает лаунчер.
    pub fn handle_destroyed<H: WindowHost>(&self, host: &H, launcher: &Launcher, label: &str) {
        let open = host.open_labels();
        let outcome = self.inner.lock().expect("window coordinator poisoned").on_destroyed(label, &open);
        if outcome.persist {
            self.persist();
        }
        if outcome.end_game_session {
            launcher.end_game_session();
        }
        if outcome.open_launcher {
            let _ = host.open_launcher();
        }
    }

    pub fn close_decision(&self, label: &str, update_applying: bool) -> CloseDecision {
        CoordinatorInner::plan_close_requested(label, update_applying)
    }
}

pub fn tauri_host<'a>(app: &'a AppHandle) -> TauriWindowHost<'a> {
    TauriWindowHost { app }
}

pub struct TauriWindowHost<'a> {
    app: &'a AppHandle,
}

impl WindowHost for TauriWindowHost<'_> {
    fn open_labels(&self) -> Vec<String> {
        self.app.webview_windows().into_keys().collect()
    }

    fn focus(&self, label: &str) -> bool {
        let Some(w) = self.app.get_webview_window(label) else { return false };
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        true
    }

    fn destroy(&self, label: &str) {
        if let Some(w) = self.app.get_webview_window(label) {
            let _ = w.destroy();
        }
    }

    fn open_launcher(&self) -> Result<(), String> {
        open_launcher(self.app).map_err(|e| e.to_string())
    }

    fn open_splash(&self) -> Result<(), String> {
        open_splash(self.app).map_err(|e| e.to_string())
    }

    fn open_game(&self, launch: GameLaunch) -> Result<(), String> {
        open_game(self.app, launch).map_err(|e| {
            eprintln!("[foundry-performance] open_game failed: {e}");
            "notice.launchFailed".to_string()
        })
    }
}

fn monitors(list: Vec<tauri::Monitor>) -> Vec<Rect> {
    list.iter()
        .map(|m| Rect { x: m.position().x, y: m.position().y, w: m.size().width, h: m.size().height })
        .collect()
}

/// Окно создаётся скрытым; ставим на прежнее место (или разворачиваем) и показываем.
fn place_and_show(window: &WebviewWindow, placement: Option<Placement>, maximize_by_default: bool) -> tauri::Result<()> {
    match placement {
        Some(p) => {
            window.set_position(PhysicalPosition::new(p.x, p.y))?;
            window.set_size(PhysicalSize::new(p.w, p.h))?;
            if p.maximized {
                window.maximize()?;
            }
            if p.fullscreen {
                window.set_fullscreen(true)?;
            }
        }
        None if maximize_by_default => window.maximize()?,
        None => {}
    }
    window.show()
}

fn coordinator(app: &AppHandle) -> Option<tauri::State<'_, WindowCoordinator>> {
    app.try_state::<WindowCoordinator>()
}

fn saved_placement(app: &AppHandle, role: Role, window: &WebviewWindow) -> Option<Placement> {
    let list = monitors(window.available_monitors().ok()?);
    coordinator(app).and_then(|c| c.placement_for(role, &list))
}

type GameBuilder<'a> = WebviewWindowBuilder<'a, Wry, AppHandle>;

/// Окна, которые страница открывает сама (Sqyre открывает мир во втором окне, модули — поп-ауты),
/// создаём как свои: `window_features` даёт им то же окружение WebView2 (флаги, папка данных),
/// а мы добавляем тот же агент. С `NewWindowResponse::Allow` WebView2 открыл бы «чужое» окно без агента.
fn with_child_windows<'a>(b: GameBuilder<'a>, app: &AppHandle, init: Option<Arc<String>>) -> GameBuilder<'a> {
    let app = app.clone();
    b.on_new_window(move |_url, features| match open_child(&app, features, init.clone()) {
        Ok(window) => NewWindowResponse::Create { window },
        Err(e) => {
            eprintln!("[foundry-performance] child window failed: {e}");
            NewWindowResponse::Allow
        }
    })
}

fn open_child(app: &AppHandle, features: NewWindowFeatures, init: Option<Arc<String>>) -> tauri::Result<WebviewWindow> {
    let label = child_label(CHILD_SEQ.fetch_add(1, Ordering::Relaxed));
    // Без явного размера (обычная ссылка target=_blank) — разворачиваем, как основное окно игры.
    // Окна модулей с заданным размером (листы персонажей) не запоминаем: их место решает модуль.
    let no_size = features.size().is_none();
    let open: Vec<String> = app.webview_windows().into_keys().collect();
    let plan = coordinator(app).map_or(ChildPlan { claimed_popup: false, maximized: no_size, visible: true }, |c| {
        c.plan_child(&label, no_size, &open)
    });
    let blank = "about:blank".parse().expect("valid url");
    let mut b = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(blank))
        .window_features(features)
        .title("Foundry Performance")
        .maximized(plan.maximized)
        .visible(plan.visible)
        .disable_drag_drop_handler()
        .general_autofill_enabled(false)
        .on_document_title_changed(|w, title| {
            let _ = w.set_title(&format!("{title} — Foundry Performance"));
        });
    if let Some(script) = &init {
        b = b.initialization_script(script.as_str());
    }
    let window = with_child_windows(b, app, init).build()?;
    if plan.claimed_popup {
        let placement = saved_placement(app, Role::Popup, &window);
        place_and_show(&window, placement, true)?;
    }
    Ok(window)
}

pub fn open_launcher(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(LAUNCHER) {
        let _ = w.unminimize();
        w.show()?;
        w.set_focus()?;
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(app, LAUNCHER, WebviewUrl::App("index.html".into()))
        .title("Foundry Performance")
        .inner_size(900.0, 620.0)
        .min_inner_size(820.0, 580.0)
        .decorations(false)
        .center()
        .visible(false)
        .build()?;
    let placement = saved_placement(app, Role::Launcher, &window);
    place_and_show(&window, placement, false)
}

/// Сплэш обновления: маленькое окно без рамки, в панели задач — экран ошибки ждёт клика
/// и не должен теряться под другими окнами. Положение не запоминается.
pub fn open_splash(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(SPLASH) {
        let _ = w.unminimize();
        w.show()?;
        return w.set_focus();
    }
    let window = WebviewWindowBuilder::new(app, SPLASH, WebviewUrl::App("index.html".into()))
        .title("Foundry Performance")
        .inner_size(300.0, 340.0)
        .resizable(false)
        .maximizable(false)
        .decorations(false)
        .center()
        .visible(false)
        .build()?;
    window.show()?;
    window.set_focus()
}

/// Отдельная папка данных = отдельный процесс браузера WebView2 со своими флагами.
fn open_game(app: &AppHandle, l: GameLaunch) -> tauri::Result<()> {
    let init = l.init_script.map(Arc::new);
    let mut b = WebviewWindowBuilder::new(app, GAME, WebviewUrl::External(l.url))
        .title(l.title)
        .inner_size(1280.0, 800.0)
        .visible(false)
        .data_directory(l.data_dir)
        .additional_browser_args(&l.browser_args)
        .disable_drag_drop_handler()
        .general_autofill_enabled(false);
    if let Some(script) = &init {
        b = b.initialization_script(script.as_str());
    }
    let window = with_child_windows(b, app, init).build()?;
    let placement = saved_placement(app, Role::Game, &window);
    place_and_show(&window, placement, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launcher::{FoundryProbe, Report, SystemLocale};
    use crate::model::{Overrides, ProfileId, Server};
    use crate::probe::ProbeResult;
    use crate::store::Store;
    use std::cell::RefCell;
    use std::future::Future;
    use std::pin::Pin;

    const PRIMARY: Rect = Rect { x: 0, y: 0, w: 1920, h: 1080 };
    const SECOND: Rect = Rect { x: 1920, y: 0, w: 2560, h: 1440 };

    fn normal(x: i32, y: i32, w: u32, h: u32) -> Snapshot {
        Snapshot { rect: Rect { x, y, w, h }, maximized: false, fullscreen: false, minimized: false }
    }

    fn placed(x: i32, y: i32) -> Placement {
        Placement { x, y, w: 1000, h: 700, maximized: false, fullscreen: false }
    }

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    struct StubProbe;
    impl FoundryProbe for StubProbe {
        fn probe(&self, _url: String) -> Pin<Box<dyn Future<Output = ProbeResult> + Send>> {
            Box::pin(async { ProbeResult::default() })
        }
    }

    struct StubLocale;
    impl SystemLocale for StubLocale {
        fn lang(&self) -> &'static str {
            "en"
        }
    }

    fn make_launcher_and_coord(cli: &[&str]) -> (tempfile::TempDir, Launcher, WindowCoordinator) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        let l = Launcher::with_ports(store, dir.path().join("engine"), Arc::new(StubProbe), Arc::new(StubLocale));
        l.save_server(Server {
            id: "a1".into(),
            name: "Strahd".into(),
            url: "https://vtt.example.com/".into(),
            profile: Some(ProfileId::Balance),
            overrides: Overrides::default(),
            game_url: None,
        })
        .unwrap();
        l.save_server(Server {
            id: "a2".into(),
            name: "Other".into(),
            url: "https://other.example.com/".into(),
            profile: Some(ProfileId::Potato),
            overrides: Overrides::default(),
            game_url: None,
        })
        .unwrap();
        let c = WindowCoordinator::new(dir.path().to_path_buf(), &args(cli));
        (dir, l, c)
    }

    #[derive(Default)]
    struct FakeHost {
        open: RefCell<Vec<String>>,
        focused: RefCell<Vec<String>>,
        destroyed: RefCell<Vec<String>>,
        fail_open_game: bool,
        fail_open_splash: bool,
    }

    impl FakeHost {
        fn with_open(labels: &[&str]) -> Self {
            Self { open: RefCell::new(labels.iter().map(|s| s.to_string()).collect()), ..Default::default() }
        }

        fn has(&self, label: &str) -> bool {
            self.open.borrow().iter().any(|l| l == label)
        }
    }

    impl WindowHost for FakeHost {
        fn open_labels(&self) -> Vec<String> {
            self.open.borrow().clone()
        }

        fn focus(&self, label: &str) -> bool {
            if self.has(label) {
                self.focused.borrow_mut().push(label.to_string());
                true
            } else {
                false
            }
        }

        fn destroy(&self, label: &str) {
            self.open.borrow_mut().retain(|l| l != label);
            self.destroyed.borrow_mut().push(label.to_string());
        }

        fn open_launcher(&self) -> Result<(), String> {
            if !self.has(LAUNCHER) {
                self.open.borrow_mut().push(LAUNCHER.to_string());
            }
            Ok(())
        }

        fn open_splash(&self) -> Result<(), String> {
            if self.fail_open_splash {
                return Err("splash failed".into());
            }
            if !self.has(SPLASH) {
                self.open.borrow_mut().push(SPLASH.to_string());
            }
            Ok(())
        }

        fn open_game(&self, _launch: GameLaunch) -> Result<(), String> {
            if self.fail_open_game {
                return Err("notice.launchFailed".into());
            }
            if !self.has(GAME) {
                self.open.borrow_mut().push(GAME.to_string());
            }
            Ok(())
        }
    }

    #[test]
    fn game_labels() {
        assert!(is_game(GAME));
        assert!(is_game(&child_label(3)));
        assert!(!is_game(LAUNCHER));
        assert!(!is_game("gamer"));
    }

    #[test]
    fn child_labels_are_unique_and_valid() {
        let a = child_label(1);
        let b = child_label(2);
        assert_ne!(a, b);
        assert!(a.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    }

    #[test]
    fn launcher_returns_only_after_last_game_window() {
        assert!(!last_game_closed(["main", "game-1"].into_iter()));
        assert!(last_game_closed(["main"].into_iter()));
        assert!(last_game_closed(std::iter::empty()));
    }

    #[test]
    fn popup_is_claimed_only_by_first_unsized_child_while_open() {
        let (_d, l, c) = make_launcher_and_coord(&[]);
        let host = FakeHost::with_open(&[GAME]);

        // Размерное дочернее окно никогда не забирает роль Popup
        let sized = c.plan_child("game-1", false, &host.open_labels());
        assert!(!sized.claimed_popup);
        assert!(!sized.maximized);
        assert!(sized.visible);
        assert_eq!(c.role_of("game-1"), None);

        // Первое безразмерное дочернее окно занимает роль Popup
        let first = c.plan_child("game-2", true, &host.open_labels());
        assert!(first.claimed_popup);
        assert!(!first.maximized);
        assert!(!first.visible);
        assert_eq!(c.role_of("game-2"), Some(Role::Popup));
        host.open.borrow_mut().push("game-2".into());

        // Пока game-2 открыто, второе безразмерное окно роль Popup не получает
        let second = c.plan_child("game-3", true, &host.open_labels());
        assert!(!second.claimed_popup);
        assert!(second.maximized);
        assert!(second.visible);
        assert_eq!(c.role_of("game-3"), None);

        // Закрытие game-2 освобождает роль Popup для следующего безразмерного окна
        host.open.borrow_mut().retain(|x| x != "game-2");
        c.handle_destroyed(&host, &l, "game-2");
        assert_eq!(c.role_of("game-2"), None);
        let next = c.plan_child("game-4", true, &host.open_labels());
        assert!(next.claimed_popup);
        assert_eq!(c.role_of("game-4"), Some(Role::Popup));
    }

    #[tokio::test]
    async fn launch_destroys_launcher_only_when_new_game_opens_and_preserves_on_focus_or_failure() {
        let (_d, l, c) = make_launcher_and_coord(&[]);

        // Ошибка открытия игры оставляет лаунчер открытым, а last_server и сессию нетронутыми
        let fail_host = FakeHost { fail_open_game: true, ..FakeHost::with_open(&[LAUNCHER]) };
        assert_eq!(c.launch_server(&fail_host, &l, "a1", false).unwrap_err(), "notice.launchFailed");
        assert!(fail_host.has(LAUNCHER));
        assert!(!fail_host.has(GAME));
        assert_eq!(l.state_dto("0.3.0".into()).settings.last_server, None);

        // Успешный запуск открывает игру, фиксирует сессию и уничтожает окно main
        let ok_host = FakeHost::with_open(&[LAUNCHER]);
        c.launch_server(&ok_host, &l, "a1", false).unwrap();
        assert!(!ok_host.has(LAUNCHER));
        assert!(ok_host.has(GAME));
        assert_eq!(l.state_dto("0.3.0".into()).settings.last_server.as_deref(), Some("a1"));

        // Если окно игры уже открыто, повторный запуск (например, пока main снова открыт) лишь
        // поднимает существующее окно игры и не трогает main, last_server и текущую сессию
        ok_host.open.borrow_mut().push(LAUNCHER.into());
        c.launch_server(&ok_host, &l, "a2", false).unwrap();
        assert!(ok_host.has(LAUNCHER));
        assert_eq!(ok_host.focused.borrow().as_slice(), &[GAME.to_string()]);
        assert_eq!(l.state_dto("0.3.0".into()).settings.last_server.as_deref(), Some("a1"));
        l.report_telemetry(&"https://vtt.example.com/game".parse().unwrap(), Report::Session { avg: 60.0, low1: 40.0, profile: ProfileId::Balance })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn multi_window_game_close_and_tray_return_to_launcher_clear_session_only_on_last_window() {
        let (_d, l, c) = make_launcher_and_coord(&[]);
        let host = FakeHost::with_open(&[LAUNCHER]);
        c.launch_server(&host, &l, "a1", false).unwrap();
        host.open.borrow_mut().push("game-1".into());

        // Возврат из трея закрывает оба окна игры
        c.return_to_launcher(&host);
        assert_eq!(host.destroyed.borrow().iter().filter(|x| is_game(x)).count(), 2);

        // Имитация последовательности WindowEvent::Destroyed: после первого окна game-1 ещё живо
        host.open.borrow_mut().push("game-1".into());
        c.handle_destroyed(&host, &l, GAME);
        assert!(!host.has(LAUNCHER));
        l.report_telemetry(&"https://vtt.example.com/game".parse().unwrap(), Report::Session { avg: 55.0, low1: 35.0, profile: ProfileId::Balance })
            .await
            .unwrap();

        // После закрытия последнего окна игры сессия завершается, и открывается лаунчер
        host.open.borrow_mut().clear();
        c.handle_destroyed(&host, &l, "game-1");
        assert!(host.has(LAUNCHER));
        assert!(l
            .report_telemetry(&"https://vtt.example.com/game".parse().unwrap(), Report::Session { avg: 55.0, low1: 35.0, profile: ProfileId::Balance })
            .await
            .is_err());

        // Если игр нет, возврат из трея просто открывает/поднимает лаунчер
        host.open.borrow_mut().clear();
        c.return_to_launcher(&host);
        assert!(host.has(LAUNCHER));
    }

    #[test]
    fn splash_and_update_transitions_preserve_fallback_and_order() {
        // Обычный старт: splash -> main, затем restart_to_update: main -> splash
        let (_d, l, c) = make_launcher_and_coord(&["--portable", "--updated"]);
        assert_eq!(c.take_updated(), Some(Version::current().to_string()));
        assert_eq!(c.take_updated(), None);

        let host = FakeHost::with_open(&[SPLASH]);
        c.finish_splash(&host, &l);
        assert!(host.has(LAUNCHER));
        assert!(!host.has(SPLASH));

        // Если открытие сплэша упало, лаунчер не уничтожается
        let bad_splash_host = FakeHost { fail_open_splash: true, ..FakeHost::with_open(&[LAUNCHER]) };
        assert!(c.restart_to_update(&bad_splash_host).is_err());
        assert!(bad_splash_host.has(LAUNCHER));

        c.restart_to_update(&host).unwrap();
        assert!(host.has(SPLASH));
        assert!(!host.has(LAUNCHER));

        // Сплэш с --open при успехе открывает игру, а при ошибке падает обратно на лаунчер
        let (_d2, l2, c2) = make_launcher_and_coord(&["--open", "https://vtt.example.com/", "--safe"]);
        let adhoc_ok = FakeHost::with_open(&[SPLASH]);
        c2.finish_splash(&adhoc_ok, &l2);
        assert!(adhoc_ok.has(GAME));
        assert!(!adhoc_ok.has(LAUNCHER));
        assert!(!adhoc_ok.has(SPLASH));

        let (_d3, l3, c3) = make_launcher_and_coord(&["--open", "::::"]);
        let adhoc_fail = FakeHost::with_open(&[SPLASH]);
        c3.finish_splash(&adhoc_fail, &l3);
        assert!(adhoc_fail.has(LAUNCHER));
        assert!(!adhoc_fail.has(GAME));
        assert!(!adhoc_fail.has(SPLASH));
    }

    #[test]
    fn close_requested_blocks_only_applying_splash() {
        assert_eq!(CoordinatorInner::plan_close_requested(SPLASH, true), CloseDecision::Prevent);
        assert_eq!(CoordinatorInner::plan_close_requested(SPLASH, false), CloseDecision::Exit);
        assert_eq!(CoordinatorInner::plan_close_requested(LAUNCHER, true), CloseDecision::PersistAndExit);
        assert_eq!(CoordinatorInner::plan_close_requested(GAME, false), CloseDecision::Ignore);
    }

    #[test]
    fn window_on_a_connected_monitor_fits() {
        assert!(fits(&placed(100, 100), &[PRIMARY]));
        assert!(fits(&placed(2500, 300), &[PRIMARY, SECOND]));
    }

    #[test]
    fn window_on_a_disconnected_monitor_does_not_fit() {
        assert!(!fits(&placed(2500, 300), &[PRIMARY]));
    }

    #[test]
    fn a_sliver_at_the_edge_is_not_enough() {
        assert!(!fits(&placed(1870, 100), &[PRIMARY]));
        assert!(fits(&placed(1700, 100), &[PRIMARY]));
    }

    #[test]
    fn zero_or_absurd_size_never_fits() {
        assert!(!fits(&Placement { w: 0, ..placed(10, 10) }, &[PRIMARY]));
        assert!(!fits(&Placement { h: 65_496, ..placed(10, 10) }, &[PRIMARY]));
    }

    #[test]
    fn normal_window_is_remembered_as_is() {
        let p = merge(None, normal(40, 50, 1200, 800), &[PRIMARY]).unwrap();
        assert_eq!(p, Placement { x: 40, y: 50, w: 1200, h: 800, maximized: false, fullscreen: false });
    }

    #[test]
    fn minimizing_keeps_the_previous_placement() {
        let prev = Some(placed(10, 10));
        let s = Snapshot { rect: Rect { x: -32000, y: -32000, w: 160, h: 28 }, minimized: true, ..normal(0, 0, 0, 0) };
        assert_eq!(merge(prev, s, &[PRIMARY]), prev);
    }

    #[test]
    fn maximizing_keeps_the_normal_rect() {
        let s = Snapshot { maximized: true, ..normal(-8, -8, 1936, 1056) };
        let p = merge(Some(placed(200, 150)), s, &[PRIMARY]).unwrap();
        assert_eq!((p.x, p.y, p.w, p.maximized), (200, 150, 1000, true));
    }

    #[test]
    fn maximized_on_another_monitor_moves_there() {
        let s = Snapshot { maximized: true, ..normal(1912, -8, 2576, 1416) };
        let p = merge(Some(placed(200, 150)), s, &[PRIMARY, SECOND]).unwrap();
        assert!(SECOND.contains(p.x, p.y), "{p:?}");
        assert_eq!((p.w, p.h, p.maximized), (1000, 700, true));
    }

    #[test]
    fn first_seen_maximized_gets_a_fallback_size_on_that_monitor() {
        let s = Snapshot { maximized: true, ..normal(1912, -8, 2576, 1416) };
        let p = merge(None, s, &[PRIMARY, SECOND]).unwrap();
        assert!(SECOND.contains(p.x, p.y));
        assert_eq!((p.w, p.h), (FALLBACK_W, FALLBACK_H));
        assert!(fits(&p, &[PRIMARY, SECOND]));
    }

    #[test]
    fn fullscreen_is_remembered() {
        let s = Snapshot { fullscreen: true, ..normal(0, 0, 1920, 1080) };
        assert!(merge(Some(placed(10, 10)), s, &[PRIMARY]).unwrap().fullscreen);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let mut saved = Saved::default();
        saved.0.insert(Role::Popup, placed(5, 6));
        save_state(dir.path(), &saved).unwrap();
        assert_eq!(load_saved(dir.path()), saved);
        fs::write(state_file_path(dir.path()), "{broken").unwrap();
        assert_eq!(load_saved(dir.path()), Saved::default());
    }
}
