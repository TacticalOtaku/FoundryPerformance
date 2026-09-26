use std::io;
use std::path::{Path, PathBuf};
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

pub const APP_KEY: &str = r"Software\FoundryPerformance";
/// Запись в «Приложениях Windows» (HKCU — без прав администратора).
pub const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\FoundryPerformance";

#[derive(Debug, Clone, PartialEq)]
pub enum RegValue {
    Str(String),
    Dword(u32),
}

pub fn uninstall_values(exe: &Path, dir: &Path, version: &str, size_kb: u32) -> Vec<(&'static str, RegValue)> {
    let exe_s = exe.display().to_string();
    vec![
        ("DisplayName", RegValue::Str("Foundry Performance".into())),
        ("DisplayVersion", RegValue::Str(version.into())),
        ("DisplayIcon", RegValue::Str(exe_s.clone())),
        ("Publisher", RegValue::Str("Foundry Performance".into())),
        ("InstallLocation", RegValue::Str(dir.display().to_string())),
        ("UninstallString", RegValue::Str(format!("\"{exe_s}\" --uninstall"))),
        ("EstimatedSize", RegValue::Dword(size_kb)),
        ("NoModify", RegValue::Dword(1)),
        ("NoRepair", RegValue::Dword(1)),
    ]
}

pub fn app_values(dir: &Path, version: &str) -> Vec<(&'static str, RegValue)> {
    vec![
        ("InstallDir", RegValue::Str(dir.display().to_string())),
        ("Version", RegValue::Str(version.into())),
    ]
}

fn write_key(path: &str, values: &[(&str, RegValue)]) -> io::Result<()> {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(path)?;
    for (name, v) in values {
        match v {
            RegValue::Str(s) => key.set_value(name, s)?,
            RegValue::Dword(d) => key.set_value(name, d)?,
        }
    }
    Ok(())
}

pub fn write_install(exe: &Path, dir: &Path, version: &str, size_kb: u32) -> io::Result<()> {
    write_key(APP_KEY, &app_values(dir, version))?;
    write_key(UNINSTALL_KEY, &uninstall_values(exe, dir, version, size_kb))
}

/// Папка и версия существующей установки, если она ещё на месте.
pub fn read_install() -> Option<(PathBuf, String)> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(APP_KEY).ok()?;
    let dir: String = key.get_value("InstallDir").ok()?;
    let version: String = key.get_value("Version").ok()?;
    Some((PathBuf::from(dir), version))
}

pub fn delete_install() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let _ = hkcu.delete_subkey_all(UNINSTALL_KEY);
    let _ = hkcu.delete_subkey_all(APP_KEY);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(v: &'a [(&str, RegValue)], name: &str) -> &'a RegValue {
        &v.iter().find(|(n, _)| *n == name).unwrap().1
    }

    #[test]
    fn uninstall_entry_points_back_to_the_exe() {
        let exe = Path::new(r"C:\Users\u\AppData\Local\Programs\FoundryPerformance\FoundryPerformance.exe");
        let dir = exe.parent().unwrap();
        let v = uninstall_values(exe, dir, "0.1.0", 6200);
        assert_eq!(get(&v, "UninstallString"), &RegValue::Str(format!("\"{}\" --uninstall", exe.display())));
        assert_eq!(get(&v, "DisplayIcon"), &RegValue::Str(exe.display().to_string()));
        assert_eq!(get(&v, "InstallLocation"), &RegValue::Str(dir.display().to_string()));
        assert_eq!(get(&v, "DisplayVersion"), &RegValue::Str("0.1.0".into()));
        assert_eq!(get(&v, "EstimatedSize"), &RegValue::Dword(6200));
        assert_eq!(get(&v, "NoModify"), &RegValue::Dword(1));
        assert_eq!(get(&v, "NoRepair"), &RegValue::Dword(1));
    }

    #[test]
    fn app_key_remembers_dir_and_version() {
        let v = app_values(Path::new(r"D:\Games\FP"), "0.2.0");
        assert_eq!(get(&v, "InstallDir"), &RegValue::Str(r"D:\Games\FP".into()));
        assert_eq!(get(&v, "Version"), &RegValue::Str("0.2.0".into()));
    }
}
