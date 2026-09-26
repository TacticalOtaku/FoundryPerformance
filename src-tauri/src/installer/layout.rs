use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const EXE_NAME: &str = "FoundryPerformance.exe";
/// Лежит рядом с установленным exe; его наличие = «запущен из папки установки».
pub const INSTALL_MARKER: &str = "install.json";
/// Пустой файл рядом с exe = portable-режим, установщик не показывается.
pub const PORTABLE_MARKER: &str = "portable";

pub fn default_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("Programs").join("FoundryPerformance")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallManifest {
    pub schema: u32,
    pub version: String,
    pub installed_at: u64,
    pub desktop_shortcut: bool,
}

pub fn read_manifest(dir: &Path) -> Option<InstallManifest> {
    serde_json::from_str(&fs::read_to_string(dir.join(INSTALL_MARKER)).ok()?).ok()
}

pub fn write_manifest(dir: &Path, m: &InstallManifest) -> io::Result<()> {
    fs::write(dir.join(INSTALL_MARKER), serde_json::to_string_pretty(m).map_err(io::Error::other)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip() {
        let d = tempfile::tempdir().unwrap();
        let m = InstallManifest { schema: 1, version: "0.1.0".into(), installed_at: 42, desktop_shortcut: true };
        write_manifest(d.path(), &m).unwrap();
        assert_eq!(read_manifest(d.path()), Some(m));
    }

    #[test]
    fn missing_manifest_is_none() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(read_manifest(d.path()), None);
    }

    #[test]
    fn default_dir_is_per_user_programs() {
        let d = default_dir();
        assert!(d.ends_with("Programs/FoundryPerformance") || d.ends_with("Programs\\FoundryPerformance"));
    }
}
