use crate::window_state::{self, Placement, Rect, Role, Saved, Snapshot};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tauri::webview::{NewWindowFeatures, NewWindowResponse};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window, Wry};

pub const LAUNCHER: &str = "main";
pub const GAME: &str = "game";

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

/// Память о положении окон: читается при старте, пишется при закрытии окна.
pub struct WindowMemory {
    dir: PathBuf,
    saved: Mutex<Saved>,
    /// Метка окна, которое сейчас играет роль `Popup`.
    popup: Mutex<Option<String>>,
}

impl WindowMemory {
    pub fn new(dir: PathBuf) -> WindowMemory {
        let saved = Mutex::new(window_state::load(&dir));
        WindowMemory { dir, saved, popup: Mutex::new(None) }
    }

    pub fn role_of(&self, label: &str) -> Option<Role> {
        match label {
            LAUNCHER => Some(Role::Launcher),
            GAME => Some(Role::Game),
            l if self.popup.lock().expect("popup poisoned").as_deref() == Some(l) => Some(Role::Popup),
            _ => None,
        }
    }

    /// Роль `Popup` получает первое окно без размера, пока прежнее такое окно открыто — нет.
    fn claim_popup(&self, app: &AppHandle, label: &str) -> bool {
        let mut popup = self.popup.lock().expect("popup poisoned");
        if popup.as_deref().is_some_and(|l| app.get_webview_window(l).is_some()) {
            return false;
        }
        *popup = Some(label.to_string());
        true
    }

    /// Вызывается на каждое движение и смену размера отслеживаемого окна.
    pub fn track(&self, window: &Window) {
        let Some(role) = self.role_of(window.label()) else { return };
        let (Ok(pos), Ok(size)) = (window.outer_position(), window.inner_size()) else { return };
        let snap = Snapshot {
            rect: Rect { x: pos.x, y: pos.y, w: size.width, h: size.height },
            maximized: window.is_maximized().unwrap_or(false),
            fullscreen: window.is_fullscreen().unwrap_or(false),
            minimized: window.is_minimized().unwrap_or(false),
        };
        let monitors = monitors(window.available_monitors().unwrap_or_default());
        let mut saved = self.saved.lock().expect("window state poisoned");
        if let Some(p) = window_state::merge(saved.0.get(&role).copied(), snap, &monitors) {
            saved.0.insert(role, p);
        }
    }

    pub fn persist(&self) {
        let saved = self.saved.lock().expect("window state poisoned");
        if let Err(e) = window_state::save(&self.dir, &saved) {
            eprintln!("[foundry-performance] window state not saved: {e}");
        }
    }

    /// Сохранённое место, если оно ещё видно на подключённых мониторах.
    fn placement(&self, role: Role, window: &WebviewWindow) -> Option<Placement> {
        let p = self.saved.lock().expect("window state poisoned").0.get(&role).copied()?;
        window_state::fits(&p, &monitors(window.available_monitors().ok()?)).then_some(p)
    }
}

fn monitors(list: Vec<tauri::Monitor>) -> Vec<Rect> {
    list.iter().map(|m| Rect { x: m.position().x, y: m.position().y, w: m.size().width, h: m.size().height }).collect()
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

fn memory(app: &AppHandle) -> Option<tauri::State<'_, WindowMemory>> {
    app.try_state::<WindowMemory>()
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
    let remembered = no_size && memory(app).is_some_and(|m| m.claim_popup(app, &label));
    let blank = "about:blank".parse().expect("valid url");
    let mut b = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(blank))
        .window_features(features)
        .title("Foundry Performance")
        .maximized(no_size && !remembered)
        .visible(!remembered)
        .disable_drag_drop_handler()
        .general_autofill_enabled(false)
        .on_document_title_changed(|w, title| {
            let _ = w.set_title(&format!("{title} — Foundry Performance"));
        });
    if let Some(script) = &init {
        b = b.initialization_script(script.as_str());
    }
    let window = with_child_windows(b, app, init).build()?;
    if remembered {
        let placement = memory(app).and_then(|m| m.placement(Role::Popup, &window));
        place_and_show(&window, placement, true)?;
    }
    Ok(window)
}

pub fn open_launcher(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(LAUNCHER) {
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
    let placement = memory(app).and_then(|m| m.placement(Role::Launcher, &window));
    place_and_show(&window, placement, false)
}

#[derive(Debug)]
pub struct GameLaunch {
    pub url: url::Url,
    pub title: String,
    pub browser_args: String,
    pub init_script: Option<String>,
    pub data_dir: PathBuf,
}

/// Отдельная папка данных = отдельный процесс браузера WebView2 со своими флагами.
pub fn open_game(app: &AppHandle, l: GameLaunch) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(GAME) {
        w.set_focus()?;
        return Ok(());
    }
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
    let placement = memory(app).and_then(|m| m.placement(Role::Game, &window));
    place_and_show(&window, placement, true)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
