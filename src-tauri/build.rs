fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_state",
            "save_server",
            "delete_server",
            "save_settings",
            "probe_server",
            "launch",
            "clear_notice",
            "report_telemetry",
            "toggle_fullscreen",
            "cache_size",
            "clear_cache",
            "get_mode",
            "pick_install_dir",
            "check_install_dir",
            "install",
            "open_installed",
            "uninstall",
            "check_update",
            "apply_update",
        ]),
    ))
    .expect("failed to run tauri-build");
}
