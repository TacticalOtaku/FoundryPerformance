//! Сплэш обновления: окно перед лаунчером или игрой, проверяет и ставит обновление.
use crate::commands::{self, LaunchMode};
use crate::installer::commands::UPDATED_FLAG;
use crate::installer::version;
use crate::state::AppState;
use crate::windows;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

/// Что открыть, когда сплэш закончит.
#[derive(Debug, Clone, PartialEq)]
pub enum AfterSplash {
    Launcher,
    Adhoc { url: String, mode: LaunchMode },
}

pub struct SplashState {
    after: Mutex<AfterSplash>,
    /// Версия, до которой только что обновились (`--updated`); отдаётся один раз, чтобы сплэш,
    /// открытый пилюлей в том же процессе, не показал «Обновлено» повторно.
    updated: Mutex<Option<String>>,
}

impl SplashState {
    pub fn new(args: &[String]) -> SplashState {
        let after = match commands::parse_cli(args.iter().cloned()) {
            Some((url, mode)) => AfterSplash::Adhoc { url, mode },
            None => AfterSplash::Launcher,
        };
        let updated = args.iter().any(|a| a == UPDATED_FLAG).then(|| version::current().to_string());
        SplashState { after: Mutex::new(after), updated: Mutex::new(updated) }
    }

    fn take_after(&self) -> AfterSplash {
        std::mem::replace(&mut *self.after.lock().expect("splash state poisoned"), AfterSplash::Launcher)
    }

    fn take_updated(&self) -> Option<String> {
        self.updated.lock().expect("splash state poisoned").take()
    }
}

#[derive(Serialize)]
pub struct SplashFlags {
    updated: Option<String>,
}

#[tauri::command]
pub fn splash_flags(state: State<'_, SplashState>) -> SplashFlags {
    SplashFlags { updated: state.take_updated() }
}

/// async: на Windows создание окна из синхронной команды может взаимно заблокироваться.
#[tauri::command]
pub async fn splash_done(app: AppHandle, state: State<'_, SplashState>, data: State<'_, AppState>) -> Result<(), String> {
    let game = match state.take_after() {
        AfterSplash::Adhoc { url, mode } => commands::launch_adhoc(&app, &data, &url, mode)
            .map_err(|e| eprintln!("[foundry-performance] launch after splash failed: {e}"))
            .is_ok(),
        AfterSplash::Launcher => false,
    };
    // Игра не открылась — показываем лаунчер, чтобы не остаться без окон
    if !game {
        if let Err(e) = windows::open_launcher(&app) {
            eprintln!("[foundry-performance] open_launcher after splash failed: {e}");
        }
    }
    if let Some(w) = app.get_webview_window(windows::SPLASH) {
        let _ = w.destroy();
    }
    Ok(())
}

/// Пилюля «Обновить до X»: тот же сплэш, после него — снова лаунчер. async — по той же причине.
#[tauri::command]
pub async fn restart_to_update(app: AppHandle, state: State<'_, SplashState>) -> Result<(), String> {
    *state.after.lock().expect("splash state poisoned") = AfterSplash::Launcher;
    windows::open_splash(&app).map_err(|e| e.to_string())?;
    // Программный destroy() не порождает CloseRequested — приложение не выходит
    if let Some(w) = app.get_webview_window(windows::LAUNCHER) {
        let _ = w.destroy();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::LaunchMode;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn plain_start_opens_launcher() {
        let s = SplashState::new(&args(&["--portable"]));
        assert_eq!(s.take_after(), AfterSplash::Launcher);
        assert_eq!(s.take_updated(), None);
    }

    #[test]
    fn address_start_launches_game_after_splash() {
        let s = SplashState::new(&args(&["--open", "https://vtt.example.com/", "--safe"]));
        assert_eq!(s.take_after(), AfterSplash::Adhoc { url: "https://vtt.example.com/".into(), mode: LaunchMode::Safe });
    }

    #[test]
    fn after_is_taken_once() {
        let s = SplashState::new(&args(&["--open", "https://vtt.example.com/"]));
        assert!(matches!(s.take_after(), AfterSplash::Adhoc { .. }));
        assert_eq!(s.take_after(), AfterSplash::Launcher);
    }

    #[test]
    fn updated_version_is_given_once() {
        let s = SplashState::new(&args(&["--portable", "--updated"]));
        assert_eq!(s.take_updated(), Some(crate::installer::version::current().to_string()));
        assert_eq!(s.take_updated(), None);
    }
}
