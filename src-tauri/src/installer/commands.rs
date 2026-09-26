use super::layout;
use super::mode::{self, InstallState, Mode};
use super::ops::{self, InstallRequest, Step};
use super::{dircheck, version};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

pub struct ModeState {
    pub mode: Mode,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallInfo {
    current_version: String,
    default_dir: String,
    existing_dir: Option<String>,
    existing_version: Option<String>,
    state: InstallState,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeDto {
    mode: Mode,
    install: Option<InstallInfo>,
}

fn install_info() -> InstallInfo {
    let existing = ops::existing();
    let current = version::current();
    let state = mode::install_state(current, existing.as_ref().and_then(|(_, v)| version::Version::parse(v)));
    InstallInfo {
        current_version: current.to_string(),
        default_dir: existing.as_ref().map_or_else(layout::default_dir, |(d, _)| d.clone()).display().to_string(),
        existing_dir: existing.as_ref().map(|(d, _)| d.display().to_string()),
        existing_version: existing.map(|(_, v)| v),
        state,
    }
}

/// Выход чуть позже ответа, чтобы интерфейс успел показать «Готово».
fn exit_soon(app: &AppHandle, ms: u64) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(ms));
        app.exit(0);
    });
}

fn launch(exe: &Path) -> Result<(), String> {
    std::process::Command::new(exe)
        .current_dir(exe.parent().unwrap_or(Path::new(".")))
        .spawn()
        .map(|_| ())
        .map_err(|_| "install.err.launch".to_string())
}

#[tauri::command]
pub fn get_mode(state: State<'_, ModeState>) -> ModeDto {
    ModeDto { mode: state.mode, install: (state.mode == Mode::Install).then(install_info) }
}

#[tauri::command]
pub async fn pick_install_dir(app: AppHandle, current: String) -> Option<String> {
    let mut dialog = app.dialog().file();
    // Диалог открываем в ближайшей существующей папке
    if let Some(start) = Path::new(current.trim()).ancestors().find(|p| p.is_dir()) {
        dialog = dialog.set_directory(start);
    }
    let picked = dialog.blocking_pick_folder()?.into_path().ok()?;
    Some(layout::for_picked(&picked).display().to_string())
}

#[tauri::command]
pub fn check_install_dir(dir: String) -> Result<(), String> {
    dircheck::check_static(Path::new(dir.trim())).map_err(String::from)
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
pub async fn install(app: AppHandle, opts: InstallOpts) -> Result<(), String> {
    let src = std::env::current_exe().map_err(|_| "install.err.copy".to_string())?;
    let req = InstallRequest { dir: PathBuf::from(opts.dir.trim()), desktop: opts.desktop };
    let emit = |step: Step, pct: u8| {
        let _ = app.emit("install-progress", Progress { step, pct });
    };
    let target = ops::install(&req, &src, &emit).map_err(String::from)?;
    if opts.launch {
        launch(&target)?;
        exit_soon(&app, 900);
    }
    Ok(())
}

#[tauri::command]
pub fn open_installed(app: AppHandle) -> Result<(), String> {
    let (dir, _) = ops::existing().ok_or_else(|| "install.err.launch".to_string())?;
    launch(&dir.join(layout::EXE_NAME))?;
    exit_soon(&app, 200);
    Ok(())
}

#[tauri::command]
pub fn uninstall(app: AppHandle, wipe_data: bool) -> Result<(), String> {
    ops::uninstall(wipe_data).map_err(String::from)?;
    exit_soon(&app, 1500);
    Ok(())
}
