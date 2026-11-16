pub mod cache;
pub mod commands;
pub mod engine_flags;
pub mod installer;
pub mod launcher;
pub mod model;
pub mod probe;
pub mod profile;
pub mod splash;
pub mod store;
pub mod tray;
pub mod windows;

use installer::{Installer, Mode, Updater};
use launcher::Launcher;
use tauri::{Manager, RunEvent, WindowEvent};
use windows::{CloseDecision, WindowCoordinator};

pub fn run() {
    // Без WebView2 окно не нарисовать — спрашиваем системным диалогом и выходим
    if !installer::win::webview2_installed() {
        installer::win::prompt_webview2(launcher::system_locale());
        return;
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let exe = std::env::current_exe().ok();
    let installer = Installer::production(&args, exe.as_deref());
    let mode = installer.mode();
    if mode == Mode::Launcher {
        installer.finish_update_startup(&args, exe.as_deref());
        // Кэш, который в прошлый раз был занят WebView2: окна игры ещё нет — файлы свободны
        cache::finish_pending(&store::engine_dir());
    }

    let mut builder = tauri::Builder::default();
    // Два лаунчера держат в памяти свои копии серверов и настроек и затирают файлы друг друга.
    // Установщик и удаление сюда не входят: при запущенном лаунчере они сами скажут «закройте программу».
    if mode == Mode::Launcher {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let app = app.clone();
            // async: создание окна прямо из обработчика может взаимно заблокироваться на Windows
            tauri::async_runtime::spawn(async move {
                app.state::<WindowCoordinator>()
                    .handle_second_launch(&windows::tauri_host(&app), &app.state::<Launcher>(), argv);
            });
        }));
    }
    let app = builder
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            app.manage(installer);
            app.manage(Updater::production());
            app.manage(WindowCoordinator::new(store::Store::default_dir(), &args));
            let launcher = Launcher::new(store::Store::new(store::Store::default_dir()));
            let lang = launcher.lang();
            app.manage(launcher);
            // Установщик и удаление — только окно, без трея и игры
            if mode != Mode::Launcher {
                windows::open_launcher(app.handle())?;
                return Ok(());
            }
            tray::create(app.handle(), lang)?;
            // Сначала сплэш обновления; лаунчер или игру по адресу он откроет сам
            windows::open_splash(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| match (window.label(), event) {
            (_, WindowEvent::Moved(_) | WindowEvent::Resized(_)) => {
                if let Some(coord) = window.try_state::<WindowCoordinator>() {
                    coord.track(window);
                }
            }
            (label, WindowEvent::CloseRequested { api, .. }) => {
                let applying = window.try_state::<Updater>().is_some_and(|s| s.applying());
                let decision = window
                    .try_state::<WindowCoordinator>()
                    .map_or(CloseDecision::Ignore, |coord| coord.close_decision(label, applying));
                match decision {
                    CloseDecision::Prevent => api.prevent_close(),
                    CloseDecision::Exit => window.app_handle().exit(0),
                    CloseDecision::PersistAndExit => {
                        if let Some(coord) = window.try_state::<WindowCoordinator>() {
                            coord.persist();
                        }
                        window.app_handle().exit(0);
                    }
                    CloseDecision::Ignore => {}
                }
            }
            (label, WindowEvent::Destroyed) if windows::is_game(label) || label == windows::LAUNCHER => {
                let app = window.app_handle();
                if let (Some(coord), Some(launcher)) = (app.try_state::<WindowCoordinator>(), app.try_state::<Launcher>()) {
                    coord.handle_destroyed(&windows::tauri_host(app), &launcher, label);
                }
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
            installer::commands::apply_update,
            installer::commands::cancel_update,
            splash::splash_flags,
            splash::splash_done,
            splash::restart_to_update
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
