pub mod agent;
pub mod commands;
pub mod engine_flags;
pub mod gpu;
pub mod locale;
pub mod model;
pub mod probe;
pub mod profile;
pub mod state;
pub mod store;
pub mod telemetry;
pub mod tray;
pub mod windows;

use state::{AppState, Data};
use std::sync::Mutex;
use tauri::{Manager, RunEvent, WindowEvent};

pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let store = store::Store::new(store::Store::default_dir());
            let gpu = gpu::detect();
            let (settings, n1) = store.load_settings(gpu::recommend(gpu.as_ref()));
            let (servers, n2) = store.load_servers();
            let (stats, n3) = store.load_stats();
            let lang = locale::effective(settings.locale);
            app.manage(AppState {
                store,
                gpu,
                data: Mutex::new(Data {
                    settings,
                    servers,
                    stats,
                    notice: n1.or(n2).or(n3),
                    current_server: None,
                    session_fallback_done: false,
                }),
            });
            tray::create(app.handle(), lang)?;
            match commands::parse_cli(std::env::args().skip(1)) {
                Some((url, mode)) => commands::launch_adhoc(app.handle(), &app.state::<AppState>(), &url, mode)?,
                None => windows::open_launcher(app.handle())?,
            }
            Ok(())
        })
        .on_window_event(|window, event| match (window.label(), event) {
            // Пользователь закрыл лаунчер крестиком — выходим. Программный destroy()
            // при запуске игры CloseRequested не порождает.
            (windows::LAUNCHER, WindowEvent::CloseRequested { .. }) => window.app_handle().exit(0),
            (label, WindowEvent::Destroyed) if windows::is_game(label) => {
                let app = window.app_handle();
                let open = app.webview_windows();
                // Sqyre и поп-ауты держат несколько окон игры — ждём закрытия последнего
                if !windows::last_game_closed(open.keys().map(String::as_str).filter(|l| *l != label)) {
                    return;
                }
                if let Some(s) = app.try_state::<AppState>() {
                    s.data.lock().expect("state poisoned").current_server = None;
                }
                let _ = windows::open_launcher(app);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::save_server,
            commands::delete_server,
            commands::save_settings,
            commands::probe_server,
            commands::launch,
            commands::clear_notice,
            commands::report_telemetry
        ])
        .build(tauri::generate_context!())
        .expect("error while building Foundry Performance");

    // Между закрытием лаунчера и открытием игры окон может не остаться — не выходим.
    // Явный выход — только app.exit(0) (крестик лаунчера или «Выход» в трее).
    app.run(|_app, event| {
        if let RunEvent::ExitRequested { code: None, api, .. } = event {
            api.prevent_exit();
        }
    });
}
