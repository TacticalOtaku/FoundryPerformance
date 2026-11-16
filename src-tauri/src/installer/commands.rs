//! Тонкий слой Tauri-команд поверх `Installer` и `Updater`.
use super::install::{self, InstallInfo, InstallRequest, Installer, Mode, Step};
use super::updater::{self, UpdateDto, Updater};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeDto {
    mode: Mode,
    install: Option<InstallInfo>,
}

/// Выход чуть позже ответа, чтобы интерфейс успел показать «Готово».
fn exit_soon(app: &AppHandle, ms: u64) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(ms));
        app.exit(0);
    });
}

#[tauri::command]
pub fn get_mode(state: State<'_, Installer>) -> ModeDto {
    let mode = state.mode();
    ModeDto { mode, install: (mode == Mode::Install).then(|| state.info()) }
}

#[tauri::command]
pub async fn pick_install_dir(app: AppHandle, current: String) -> Option<String> {
    let mut dialog = app.dialog().file();
    // Диалог открываем в ближайшей существующей папке
    if let Some(start) = Path::new(current.trim()).ancestors().find(|p| p.is_dir()) {
        dialog = dialog.set_directory(start);
    }
    let picked = dialog.blocking_pick_folder()?.into_path().ok()?;
    Some(install::for_picked(&picked).display().to_string())
}

#[tauri::command]
pub fn check_install_dir(state: State<'_, Installer>, dir: String) -> Result<(), String> {
    state.check_static(Path::new(dir.trim())).map_err(String::from)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOpts {
    dir: String,
    desktop: bool,
    launch: bool,
}

#[derive(Serialize, Clone)]
struct Progress {
    step: Step,
    pct: u8,
}

#[tauri::command]
pub async fn install(app: AppHandle, state: State<'_, Installer>, opts: InstallOpts) -> Result<(), String> {
    let src = std::env::current_exe().map_err(|_| "install.err.copy".to_string())?;
    let req = InstallRequest { dir: PathBuf::from(opts.dir.trim()), desktop: opts.desktop };
    let emit = |step: Step, pct: u8| {
        let _ = app.emit("install-progress", Progress { step, pct });
    };
    let target = state.install(&req, &src, &emit).map_err(String::from)?;
    if opts.launch {
        state.launch(&target, &[])?;
        exit_soon(&app, 900);
    }
    Ok(())
}

#[tauri::command]
pub fn open_installed(app: AppHandle, state: State<'_, Installer>) -> Result<(), String> {
    state.open_installed()?;
    exit_soon(&app, 200);
    Ok(())
}

#[tauri::command]
pub fn uninstall(app: AppHandle, state: State<'_, Installer>, wipe_data: bool) -> Result<(), String> {
    state.uninstall(wipe_data).map_err(String::from)?;
    exit_soon(&app, 1500);
    Ok(())
}

#[tauri::command]
pub async fn check_update(state: State<'_, Updater>, timeout_s: Option<u64>) -> Result<Option<UpdateDto>, String> {
    state.check(timeout_s).await.map_err(String::from)
}

#[tauri::command]
pub fn cancel_update(state: State<'_, Updater>) {
    state.cancel();
}

#[tauri::command]
pub async fn apply_update(app: AppHandle, state: State<'_, Updater>, installer: State<'_, Installer>) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|_| "update.err.apply".to_string())?;
    state
        .apply(&exe, |pct| {
            let _ = app.emit("update-progress", pct);
        })
        .await
        .map_err(String::from)?;
    if let Err(e) = installer.launch(&exe, &updater::relaunch_args(std::env::args().skip(1))) {
        state.clear_applying();
        return Err(e);
    }
    exit_soon(&app, 300);
    Ok(())
}
