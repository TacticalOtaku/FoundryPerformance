use crate::windows;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

pub fn create(app: &AppHandle, lang: &str) -> tauri::Result<()> {
    let (hud, launcher, quit) = if lang == "ru" {
        ("Показать / скрыть HUD", "Вернуться в лаунчер", "Выход")
    } else {
        ("Toggle HUD", "Back to launcher", "Quit")
    };
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "hud", hud, true, None::<&str>)?,
            &MenuItem::with_id(app, "launcher", launcher, true, None::<&str>)?,
            &MenuItem::with_id(app, "quit", quit, true, None::<&str>)?,
        ],
    )?;
    let mut builder = TrayIconBuilder::with_id("main").tooltip("Foundry Performance").menu(&menu).show_menu_on_left_click(true);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "hud" => {
                if let Some(w) = app.get_webview_window(windows::GAME) {
                    let _ = w.eval("window.__FP__ && window.__FP__.toggleHud()");
                }
            }
            "launcher" => match app.get_webview_window(windows::GAME) {
                // закрытие игры само откроет лаунчер (см. lib.rs, WindowEvent::Destroyed)
                Some(w) => {
                    let _ = w.destroy();
                }
                None => {
                    let _ = windows::open_launcher(app);
                }
            },
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
