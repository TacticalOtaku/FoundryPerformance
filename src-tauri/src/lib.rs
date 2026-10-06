pub mod agent;
pub mod cache;
pub mod commands;
pub mod engine_flags;
pub mod gpu;
pub mod installer;
pub mod locale;
pub mod model;
pub mod probe;
pub mod profile;
pub mod state;
pub mod store;
pub mod telemetry;
pub mod tray;
pub mod window_state;
pub mod windows;

use installer::mode::Mode;
use state::{AppState, Data};
use std::sync::Mutex;
use tauri::{Manager, RunEvent, WindowEvent};

pub fn run() {
    // Без WebView2 окно не нарисовать — спрашиваем системным диалогом и выходим
    if !installer::webview2::installed() {
        installer::webview2::prompt_download(locale::effective(model::Locale::Auto));
        return;
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let exe = std::env::current_exe().ok();
    let exe_dir = exe.as_deref().and_then(std::path::Path::parent).map(std::path::Path::to_path_buf).unwrap_or_default();
    let mode = installer::mode::detect(&args, &exe_dir, cfg!(debug_assertions));
    if mode == Mode::Launcher {
        if let Some(e) = &exe {
            installer::selfreplace::cleanup(e);
        }
        // Кэш, который в прошлый раз был занят WebView2: окна игры ещё нет — файлы свободны
        cache::finish_pending(&store::engine_dir());
        // Первый запуск после автообновления: версия в install.json и «Приложениях Windows»
        let version = installer::version::current().to_string();
        if installer::layout::bump_manifest(&exe_dir, &version).unwrap_or(false) {
            installer::registry::set_version(&version);
        }
    }

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            app.manage(installer::commands::ModeState { mode });
            app.manage(installer::commands::UpdateState::default());
            app.manage(windows::WindowMemory::new(store::Store::default_dir()));
            let store = store::Store::new(store::Store::default_dir());
            let adapters = gpu::detect_all();
            let (settings, n1) = store.load_settings(gpu::recommend(gpu::best(&adapters)));
            let (servers, n2) = store.load_servers();
            let (stats, n3) = store.load_stats();
            let lang = locale::effective(settings.locale);
            app.manage(AppState {
                store,
                adapters,
                data: Mutex::new(Data {
                    settings,
                    servers,
                    stats,
                    notice: n1.or(n2).or(n3),
                    current_server: None,
                    current_mode: commands::LaunchMode::Normal,
                    session_fallback_done: false,
                    session_origins: Vec::new(),
                }),
            });
            // Установщик и удаление — только окно, без трея и игры
            if mode != Mode::Launcher {
                windows::open_launcher(app.handle())?;
                return Ok(());
            }
            tray::create(app.handle(), lang)?;
            match commands::parse_cli(args.clone().into_iter()) {
                Some((url, launch_mode)) => commands::launch_adhoc(app.handle(), &app.state::<AppState>(), &url, launch_mode)?,
                None => windows::open_launcher(app.handle())?,
            }
            Ok(())
        })
        .on_window_event(|window, event| match (window.label(), event) {
            (_, WindowEvent::Moved(_) | WindowEvent::Resized(_)) => {
                if let Some(m) = window.try_state::<windows::WindowMemory>() {
                    m.track(window);
                }
            }
            // Пользователь закрыл лаунчер крестиком — выходим. Программный destroy()
            // при запуске игры CloseRequested не порождает.
            (windows::LAUNCHER, WindowEvent::CloseRequested { .. }) => {
                if let Some(m) = window.try_state::<windows::WindowMemory>() {
                    m.persist();
                }
                window.app_handle().exit(0)
            }
            (label, WindowEvent::Destroyed) if windows::is_game(label) || label == windows::LAUNCHER => {
                if let Some(m) = window.try_state::<windows::WindowMemory>() {
                    m.persist();
                }
                if label == windows::LAUNCHER {
                    return;
                }
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
            commands::report_telemetry,
            commands::toggle_fullscreen,
            commands::cache_size,
            commands::clear_cache,
            installer::commands::get_mode,
            installer::commands::pick_install_dir,
            installer::commands::check_install_dir,
            installer::commands::install,
            installer::commands::open_installed,
            installer::commands::uninstall,
            installer::commands::check_update,
            installer::commands::apply_update
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
