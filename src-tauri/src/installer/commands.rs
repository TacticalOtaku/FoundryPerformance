use super::layout;
use super::mode::{self, InstallState, Mode};
use super::ops::{self, InstallRequest, Step};
use super::{dircheck, version};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
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

fn launch(exe: &Path, args: &[String]) -> Result<(), String> {
    std::process::Command::new(exe)
        .args(args)
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
        launch(&target, &[])?;
        exit_soon(&app, 900);
    }
    Ok(())
}

#[tauri::command]
pub fn open_installed(app: AppHandle) -> Result<(), String> {
    let (dir, _) = ops::existing().ok_or_else(|| "install.err.launch".to_string())?;
    launch(&dir.join(layout::EXE_NAME), &[])?;
    exit_soon(&app, 200);
    Ok(())
}

#[tauri::command]
pub fn uninstall(app: AppHandle, wipe_data: bool) -> Result<(), String> {
    ops::uninstall(wipe_data).map_err(String::from)?;
    exit_soon(&app, 1500);
    Ok(())
}

/// Обновление: манифест последней проверки (кнопка ставит ровно то, что показали), флаг отмены
/// от «Пропустить» и признак замены exe — пока он поднят, сплэш закрыть нельзя.
#[derive(Default)]
pub struct UpdateState {
    manifest: Mutex<Option<super::update::Manifest>>,
    cancelled: AtomicBool,
    applying: AtomicBool,
}

impl UpdateState {
    pub fn applying(&self) -> bool {
        self.applying.load(Ordering::SeqCst)
    }
}

pub const UPDATED_FLAG: &str = "--updated";

/// Аргументы нового exe после автообновления: исходные (`--portable`, `--open …`) плюс `--updated`.
/// Без них портативная сборка без маркера стартовала бы установщиком, а запуск по адресу терял бы адрес.
pub fn relaunch_args(original: impl Iterator<Item = String>) -> Vec<String> {
    let mut args: Vec<String> = original.filter(|a| a != UPDATED_FLAG).collect();
    args.push(UPDATED_FLAG.to_string());
    args
}

/// Сплэш ждёт ленту 4 с, фоновая проверка — 8 с.
fn check_timeout(requested: Option<u64>) -> u64 {
    requested.unwrap_or(8).clamp(1, 30)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDto {
    version: String,
    notes: String,
}

#[tauri::command]
pub async fn check_update(state: State<'_, UpdateState>, timeout_s: Option<u64>) -> Result<Option<UpdateDto>, String> {
    if !super::update::checks_enabled() {
        return Ok(None);
    }
    // Ошибку отдаём наверх: ручная проверка должна отличать «нет связи» от «обновлений нет»
    let m = super::update::fetch_manifest(check_timeout(timeout_s)).await?;
    if !super::update::is_newer(version::current(), &m) {
        return Ok(None);
    }
    let dto = UpdateDto { version: m.version.clone(), notes: m.notes.clone() };
    *state.manifest.lock().expect("update state poisoned") = Some(m);
    Ok(Some(dto))
}

#[tauri::command]
pub fn cancel_update(state: State<'_, UpdateState>) {
    state.cancelled.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub async fn apply_update(app: AppHandle, state: State<'_, UpdateState>) -> Result<(), String> {
    state.cancelled.store(false, Ordering::SeqCst);
    let m = state.manifest.lock().expect("update state poisoned").clone().ok_or_else(|| "update.err.download".to_string())?;
    let bytes = super::update::download(&m.url, &state.cancelled, |pct| {
        let _ = app.emit("update-progress", pct);
    })
    .await?;
    let result = install_update(&state, &bytes, &m);
    if result.is_err() {
        state.applying.store(false, Ordering::SeqCst);
    }
    result?;
    exit_soon(&app, 300);
    Ok(())
}

/// Проверки, замена exe и запуск новой версии. С этого места сплэш не закрыть.
fn install_update(state: &UpdateState, bytes: &[u8], m: &super::update::Manifest) -> Result<(), String> {
    let not_cancelled = || {
        if state.cancelled.load(Ordering::SeqCst) {
            Err("update.err.cancelled".to_string())
        } else {
            Ok(())
        }
    };
    not_cancelled()?;
    state.applying.store(true, Ordering::SeqCst);
    super::update::verify(bytes, m, super::update::PUBLIC_KEY)?;
    if !super::update::is_newer(version::current(), m) {
        return Err("update.err.version".into());
    }
    not_cancelled()?;
    let exe = std::env::current_exe().map_err(|_| "update.err.apply".to_string())?;
    super::update::apply(bytes, &exe)?;
    launch(&exe, &relaunch_args(std::env::args().skip(1)))
}

#[cfg(test)]
mod tests {
    use super::super::update::Manifest;
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn relaunch_keeps_args_and_marks_updated() {
        let got = relaunch_args(s(&["--portable", "--open", "https://vtt.example.com/"]).into_iter());
        assert_eq!(got, s(&["--portable", "--open", "https://vtt.example.com/", "--updated"]));
    }

    #[test]
    fn relaunch_does_not_repeat_updated() {
        assert_eq!(relaunch_args(s(&["--updated", "--portable"]).into_iter()), s(&["--portable", "--updated"]));
    }

    #[test]
    fn check_timeout_defaults_and_clamps() {
        assert_eq!(check_timeout(None), 8);
        assert_eq!(check_timeout(Some(4)), 4);
        assert_eq!(check_timeout(Some(0)), 1);
        assert_eq!(check_timeout(Some(600)), 30);
    }

    #[test]
    fn cancelled_update_never_installs() {
        let state = UpdateState::default();
        state.cancelled.store(true, Ordering::SeqCst);
        let m = Manifest { version: "9.9.9".into(), notes: String::new(), url: String::new(), sha256: String::new(), signature: String::new() };
        assert_eq!(install_update(&state, b"x", &m), Err("update.err.cancelled".to_string()));
        assert!(!state.applying());
    }
}
