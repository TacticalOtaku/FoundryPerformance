use super::layout::{INSTALL_MARKER, PORTABLE_MARKER};
use super::version::Version;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Launcher,
    Install,
    Uninstall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallState {
    Fresh,
    Upgrade,
    Current,
}

/// В debug-сборке (`tauri dev`) по умолчанию лаунчер; экран установки — по `--installer`.
pub fn detect(args: &[String], exe_dir: &Path, dev: bool) -> Mode {
    let has = |f: &str| args.iter().any(|a| a == f);
    if has("--uninstall") {
        return Mode::Uninstall;
    }
    if dev && !has("--installer") {
        return Mode::Launcher;
    }
    if has("--portable") || has("--open") || exe_dir.join(PORTABLE_MARKER).exists() || exe_dir.join(INSTALL_MARKER).exists() {
        return Mode::Launcher;
    }
    Mode::Install
}

pub fn install_state(current: Version, existing: Option<Version>) -> InstallState {
    match existing {
        None => InstallState::Fresh,
        Some(v) if v < current => InstallState::Upgrade,
        Some(_) => InstallState::Current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn uninstall_flag_wins() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join(INSTALL_MARKER), "{}").unwrap();
        assert_eq!(detect(&args(&["--uninstall"]), d.path(), false), Mode::Uninstall);
    }

    #[test]
    fn markers_and_flags_mean_launcher() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(detect(&args(&[]), d.path(), false), Mode::Install);
        assert_eq!(detect(&args(&["--portable"]), d.path(), false), Mode::Launcher);
        assert_eq!(detect(&args(&["--open", "x"]), d.path(), false), Mode::Launcher);
        fs::write(d.path().join(PORTABLE_MARKER), "").unwrap();
        assert_eq!(detect(&args(&[]), d.path(), false), Mode::Launcher);
        let d2 = tempfile::tempdir().unwrap();
        fs::write(d2.path().join(INSTALL_MARKER), "{}").unwrap();
        assert_eq!(detect(&args(&[]), d2.path(), false), Mode::Launcher);
    }

    #[test]
    fn dev_build_opens_launcher_unless_asked() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(detect(&args(&[]), d.path(), true), Mode::Launcher);
        assert_eq!(detect(&args(&["--installer"]), d.path(), true), Mode::Install);
    }

    #[test]
    fn install_states() {
        let cur = Version(0, 2, 0);
        assert_eq!(install_state(cur, None), InstallState::Fresh);
        assert_eq!(install_state(cur, Some(Version(0, 1, 0))), InstallState::Upgrade);
        assert_eq!(install_state(cur, Some(cur)), InstallState::Current);
        assert_eq!(install_state(cur, Some(Version(0, 3, 0))), InstallState::Current);
    }
}
