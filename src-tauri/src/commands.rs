//! Тонкий слой Tauri-команд поверх `Launcher` и `WindowCoordinator`.
use crate::launcher::{Launcher, Report, StateDto};
use crate::model::{Server, Settings};
use crate::probe::ProbeResult;
use crate::windows::{self, WindowCoordinator};
use crate::{cache, store};
use serde::Serialize;
use tauri::{AppHandle, Manager, State, Webview};

#[tauri::command]
pub fn get_state(app: AppHandle, state: State<'_, Launcher>) -> StateDto {
    state.state_dto(app.package_info().version.to_string())
}

#[tauri::command]
pub fn save_server(state: State<'_, Launcher>, server: Server) -> Result<Vec<Server>, String> {
    state.save_server(server)
}

#[tauri::command]
pub fn delete_server(state: State<'_, Launcher>, id: String) -> Result<Vec<Server>, String> {
    state.delete_server(&id)
}

#[tauri::command]
pub fn save_settings(state: State<'_, Launcher>, settings: Settings) -> Result<Settings, String> {
    state.save_settings(settings)
}

#[tauri::command]
pub async fn probe_server(app: AppHandle, url: String) -> ProbeResult {
    app.state::<Launcher>().probe_server(&url).await
}

/// async: на Windows создание окна из синхронной команды может взаимно заблокироваться.
#[tauri::command]
pub async fn launch(
    app: AppHandle,
    state: State<'_, Launcher>,
    windows: State<'_, WindowCoordinator>,
    server_id: String,
    safe_mode: bool,
) -> Result<(), String> {
    windows.launch_server(&windows::tauri_host(&app), &state, &server_id, safe_mode)
}

#[tauri::command]
pub fn clear_notice(state: State<'_, Launcher>) {
    state.clear_notice();
}

/// Окна игры открывают и чужие страницы (ссылки из чата), а IPC им доступен.
/// Вся проверка источника и обновление состояния идут через `Launcher`.
#[tauri::command]
pub async fn report_telemetry(webview: Webview, state: State<'_, Launcher>, report: Report) -> Result<(), String> {
    if !windows::is_game(webview.label()) {
        return Err("rejected".into());
    }
    let page = webview.url().map_err(|_| "rejected".to_string())?;
    state.report_telemetry(&page, report).await
}

/// F11 в игре: полный экран без рамки для того окна, где нажали.
#[tauri::command]
pub fn toggle_fullscreen(window: tauri::WebviewWindow) -> Result<(), String> {
    if !windows::is_game(window.label()) {
        return Err("not a game window".into());
    }
    let on = window.is_fullscreen().map_err(|e| e.to_string())?;
    window.set_fullscreen(!on).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cache_size() -> u64 {
    tauri::async_runtime::spawn_blocking(|| cache::size(&store::engine_dir())).await.unwrap_or(0)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearedDto {
    freed: u64,
    pending: bool,
}

#[tauri::command]
pub async fn clear_cache(app: AppHandle) -> Result<ClearedDto, String> {
    // Во время игры WebView2 держит эти файлы и пишет в них
    if app.webview_windows().keys().any(|l| windows::is_game(l)) {
        return Err("cache.err.gameOpen".into());
    }
    let r = tauri::async_runtime::spawn_blocking(|| cache::clear(&store::engine_dir())).await.map_err(|_| "cache.err.failed".to_string())?;
    Ok(ClearedDto { freed: r.freed, pending: r.pending })
}
