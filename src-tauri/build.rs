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
        ]),
    ))
    .expect("failed to run tauri-build");
}
