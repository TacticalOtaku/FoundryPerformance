use std::path::PathBuf;
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub const LAUNCHER: &str = "main";
pub const GAME: &str = "game";

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
    let mut b = WebviewWindowBuilder::new(app, GAME, WebviewUrl::External(l.url))
        .title(l.title)
        .inner_size(1280.0, 800.0)
        .maximized(true)
        .data_directory(l.data_dir)
        .additional_browser_args(&l.browser_args)
        .disable_drag_drop_handler()
        .general_autofill_enabled(false)
        .on_new_window(|_url, _features| NewWindowResponse::Allow);
    if let Some(script) = l.init_script {
        b = b.initialization_script(script);
    }
    b.build()?;
    Ok(())
}
