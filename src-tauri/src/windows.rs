use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tauri::webview::{NewWindowFeatures, NewWindowResponse};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Wry};

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
    // Без явного размера (обычная ссылка target=_blank) — разворачиваем, как основное окно игры
    let maximize = features.size().is_none();
    let blank = "about:blank".parse().expect("valid url");
    let mut b = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(blank))
        .window_features(features)
        .title("Foundry Performance")
        .maximized(maximize)
        .disable_drag_drop_handler()
        .general_autofill_enabled(false)
        .on_document_title_changed(|w, title| {
            let _ = w.set_title(&format!("{title} — Foundry Performance"));
        });
    if let Some(script) = &init {
        b = b.initialization_script(script.as_str());
    }
    with_child_windows(b, app, init).build()
}

pub fn open_launcher(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(LAUNCHER) {
        w.show()?;
        w.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, LAUNCHER, WebviewUrl::App("index.html".into()))
        .title("Foundry Performance")
        .inner_size(900.0, 620.0)
        .min_inner_size(820.0, 580.0)
        .decorations(false)
        .center()
        .build()?;
    Ok(())
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
        .maximized(true)
        .data_directory(l.data_dir)
        .additional_browser_args(&l.browser_args)
        .disable_drag_drop_handler()
        .general_autofill_enabled(false);
    if let Some(script) = &init {
        b = b.initialization_script(script.as_str());
    }
    with_child_windows(b, app, init).build()?;
    Ok(())
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
