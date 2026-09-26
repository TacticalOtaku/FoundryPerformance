use crate::model::*;
use crate::state::AppState;
use crate::{agent, engine_flags, gpu, locale, probe, profile, store, telemetry, windows};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State, Webview};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    Normal,
    /// Без агента, стандартные флаги — чтобы исключить наше влияние.
    Safe,
    /// Стандартные флаги + агент только для измерений.
    Baseline,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDto {
    settings: Settings,
    servers: Vec<Server>,
    stats: BTreeMap<String, ServerStats>,
    presets: BTreeMap<ProfileId, Levers>,
    gpu: Option<gpu::GpuInfo>,
    recommended: ProfileId,
    notice: Option<Notice>,
    locale: &'static str,
    version: String,
}

pub fn build_launch(settings: &Settings, server: &Server, mode: LaunchMode) -> Result<windows::GameLaunch, String> {
    let url = probe::normalize_url(&server.url)?;
    let presets = profile::resolve_all(settings, Some(server));
    let boot = agent::Boot {
        server_id: &server.id,
        profile_id: profile::active_id(settings, Some(server)),
        presets: &presets,
        locale: locale::effective(settings.locale),
        measure_only: mode == LaunchMode::Baseline,
    };
    Ok(windows::GameLaunch {
        url,
        title: format!("{} — Foundry Performance", server.name),
        browser_args: match mode {
            LaunchMode::Normal => engine_flags::build(&settings.engine),
            LaunchMode::Safe | LaunchMode::Baseline => engine_flags::safe(),
        },
        init_script: match mode {
            LaunchMode::Safe => None,
            LaunchMode::Normal | LaunchMode::Baseline => Some(agent::script(&boot)),
        },
        data_dir: store::engine_dir(),
    })
}

pub fn parse_cli(args: impl Iterator<Item = String>) -> Option<(String, LaunchMode)> {
    let mut url = None;
    let mut mode = LaunchMode::Normal;
    let mut it = args;
    while let Some(a) = it.next() {
        match a.as_str() {
            "--open" => url = it.next(),
            "--safe" => mode = LaunchMode::Safe,
            "--baseline" => mode = LaunchMode::Baseline,
            _ => {}
        }
    }
    url.map(|u| (u, mode))
}

fn start_game(app: &AppHandle, state: &AppState, server: &Server, mode: LaunchMode) -> Result<(), String> {
    let launch = {
        let mut d = state.data.lock().expect("state poisoned");
        d.current_server = Some(server.id.clone());
        d.session_fallback_done = false;
        build_launch(&d.settings, server, mode)?
    };
    windows::open_game(app, launch).map_err(|e| {
        eprintln!("[foundry-performance] open_game failed: {e}");
        "notice.launchFailed".to_string()
    })
}

/// Запуск из командной строки: `--open <url> [--safe|--baseline]`.
pub fn launch_adhoc(app: &AppHandle, state: &AppState, url: &str, mode: LaunchMode) -> Result<(), String> {
    let server = Server { id: "adhoc".into(), name: url.to_string(), url: url.to_string(), profile: None, overrides: Overrides::default() };
    start_game(app, state, &server, mode)
}

#[tauri::command]
pub fn get_state(app: AppHandle, state: State<'_, AppState>) -> StateDto {
    let d = state.data.lock().expect("state poisoned");
    StateDto {
        settings: d.settings.clone(),
        servers: d.servers.clone(),
        stats: d.stats.clone(),
        presets: profile::presets(),
        gpu: state.gpu.clone(),
        recommended: gpu::recommend(state.gpu.as_ref()),
        notice: d.notice.clone(),
        locale: locale::effective(d.settings.locale),
        version: app.package_info().version.to_string(),
    }
}

#[tauri::command]
pub fn save_server(state: State<'_, AppState>, server: Server) -> Result<Vec<Server>, String> {
    let name = server.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 60 || server.id.trim().is_empty() {
        return Err("err.nameRequired".into());
    }
    let url = probe::normalize_url(&server.url)?;
    let clean = Server { name, url: url.to_string(), ..server };
    let mut d = state.data.lock().expect("state poisoned");
    match d.servers.iter_mut().find(|s| s.id == clean.id) {
        Some(s) => *s = clean,
        None => d.servers.push(clean),
    }
    state.store.save_servers(&d.servers).map_err(|_| "err.saveFailed".to_string())?;
    Ok(d.servers.clone())
}

#[tauri::command]
pub fn delete_server(state: State<'_, AppState>, id: String) -> Result<Vec<Server>, String> {
    let mut d = state.data.lock().expect("state poisoned");
    d.servers.retain(|s| s.id != id);
    d.stats.remove(&id);
    state.store.save_servers(&d.servers).map_err(|_| "err.saveFailed".to_string())?;
    let _ = state.store.save_stats(&d.stats);
    Ok(d.servers.clone())
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    let mut d = state.data.lock().expect("state poisoned");
    d.settings = Settings { schema: SCHEMA, ..settings };
    state.store.save_settings(&d.settings).map_err(|_| "err.saveFailed".to_string())?;
    Ok(d.settings.clone())
}

#[tauri::command]
pub async fn probe_server(url: String) -> probe::ProbeResult {
    probe::probe(&url).await
}

/// async: на Windows создание окна из синхронной команды может взаимно заблокироваться.
#[tauri::command]
pub async fn launch(app: AppHandle, state: State<'_, AppState>, server_id: String, safe_mode: bool) -> Result<(), String> {
    let server = {
        let d = state.data.lock().expect("state poisoned");
        d.servers.iter().find(|s| s.id == server_id).cloned().ok_or_else(|| "err.serverMissing".to_string())?
    };
    let mode = if safe_mode { LaunchMode::Safe } else { LaunchMode::Normal };
    start_game(&app, &state, &server, mode)
}

#[tauri::command]
pub fn clear_notice(state: State<'_, AppState>) {
    state.data.lock().expect("state poisoned").notice = None;
}

#[tauri::command]
pub fn report_telemetry(webview: Webview, state: State<'_, AppState>, report: telemetry::Report) -> Result<(), String> {
    if !windows::is_game(webview.label()) || !telemetry::validate(&report) {
        return Err("rejected".into());
    }
    let mut guard = state.data.lock().expect("state poisoned");
    let d = &mut *guard;
    let Some(server_id) = d.current_server.clone() else { return Ok(()) };
    if matches!(report, telemetry::Report::WebglLost { early: true }) {
        if d.session_fallback_done {
            return Ok(());
        }
        d.session_fallback_done = true;
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|t| t.as_secs()).unwrap_or(0);
    let fx = telemetry::apply(&report, &server_id, now, &mut d.stats, &mut d.servers, &mut d.settings);
    if fx.stats {
        let _ = state.store.save_stats(&d.stats);
    }
    if fx.servers {
        let _ = state.store.save_servers(&d.servers);
    }
    if fx.settings {
        let _ = state.store.save_settings(&d.settings);
    }
    if fx.notice.is_some() {
        d.notice = fx.notice;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srv() -> Server {
        Server { id: "a1".into(), name: "Страд".into(), url: "vtt.example.com/game".into(), profile: Some(ProfileId::Potato), overrides: Overrides::default() }
    }

    #[test]
    fn normal_launch_uses_tuned_flags_and_agent() {
        let l = build_launch(&Settings::default(), &srv(), LaunchMode::Normal).unwrap();
        assert_eq!(l.url.as_str(), "https://vtt.example.com/");
        assert!(l.browser_args.contains("--use-angle=d3d11"));
        let script = l.init_script.unwrap();
        assert!(script.contains("\"profileId\":\"potato\""));
        assert!(script.contains("\"measureOnly\":false"));
        assert!(l.title.contains("Страд"));
    }

    #[test]
    fn safe_launch_has_no_agent_and_stock_flags() {
        let l = build_launch(&Settings::default(), &srv(), LaunchMode::Safe).unwrap();
        assert!(l.init_script.is_none());
        assert_eq!(l.browser_args, engine_flags::safe());
    }

    #[test]
    fn baseline_launch_measures_only() {
        let l = build_launch(&Settings::default(), &srv(), LaunchMode::Baseline).unwrap();
        assert_eq!(l.browser_args, engine_flags::safe());
        assert!(l.init_script.unwrap().contains("\"measureOnly\":true"));
    }

    #[test]
    fn invalid_url_is_rejected() {
        let s = Server { url: "::::".into(), ..srv() };
        assert_eq!(build_launch(&Settings::default(), &s, LaunchMode::Normal).unwrap_err(), "err.urlInvalid");
    }

    #[test]
    fn parse_cli_args() {
        let a = |v: &[&str]| parse_cli(v.iter().map(|s| s.to_string()));
        assert_eq!(a(&["--open", "vtt.x"]), Some(("vtt.x".to_string(), LaunchMode::Normal)));
        assert_eq!(a(&["--safe", "--open", "vtt.x"]), Some(("vtt.x".to_string(), LaunchMode::Safe)));
        assert_eq!(a(&["--open", "vtt.x", "--baseline"]), Some(("vtt.x".to_string(), LaunchMode::Baseline)));
        assert_eq!(a(&[]), None);
    }
}
