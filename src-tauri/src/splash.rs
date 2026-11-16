//! Тонкий Tauri-адаптер команд сплэша поверх `WindowCoordinator`.
use crate::launcher::Launcher;
use crate::windows::{self, WindowCoordinator};
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Serialize)]
pub struct SplashFlags {
    updated: Option<String>,
}

#[tauri::command]
pub fn splash_flags(windows: State<'_, WindowCoordinator>) -> SplashFlags {
    SplashFlags { updated: windows.take_updated() }
}

/// async: на Windows создание окна из синхронной команды может взаимно заблокироваться.
#[tauri::command]
pub async fn splash_done(app: AppHandle, windows: State<'_, WindowCoordinator>, launcher: State<'_, Launcher>) -> Result<(), String> {
    windows.finish_splash(&windows::tauri_host(&app), &launcher);
    Ok(())
}

/// Пилюля «Обновить до X»: тот же сплэш, после него — снова лаунчер. async — по той же причине.
#[tauri::command]
pub async fn restart_to_update(app: AppHandle, windows: State<'_, WindowCoordinator>) -> Result<(), String> {
    windows.restart_to_update(&windows::tauri_host(&app))
}
