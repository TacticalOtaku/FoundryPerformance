use std::fs;
use std::path::Path;

/// Проверка без изменений на диске — для подсказки в поле ввода.
pub fn check_static(dir: &Path) -> Result<(), &'static str> {
    if !dir.is_absolute() {
        return Err("install.err.notAbsolute");
    }
    if dir.parent().is_none() {
        return Err("install.err.root");
    }
    let lower = |p: &Path| p.to_string_lossy().to_lowercase();
    if let Some(win) = std::env::var_os("WINDIR") {
        if lower(dir).starts_with(&lower(Path::new(&win))) {
            return Err("install.err.system");
        }
    }
    Ok(())
}

/// Полная проверка перед установкой: создаёт папку и пишет пробный файл.
pub fn check_writable(dir: &Path) -> Result<(), &'static str> {
    check_static(dir)?;
    fs::create_dir_all(dir).map_err(|_| "install.err.noAccess")?;
    let probe = dir.join(".fp-write-test");
    fs::write(&probe, b"ok").map_err(|_| "install.err.noAccess")?;
    let _ = fs::remove_file(probe);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rejects_relative_root_and_system() {
        assert_eq!(check_static(Path::new("relative\\dir")), Err("install.err.notAbsolute"));
        assert_eq!(check_static(Path::new("C:\\")), Err("install.err.root"));
        let win = PathBuf::from(std::env::var_os("WINDIR").unwrap());
        assert_eq!(check_static(&win.join("Temp")), Err("install.err.system"));
    }

    #[test]
    fn accepts_regular_folder_and_creates_it() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(check_static(d.path()), Ok(()));
        let nested = d.path().join("a").join("b");
        assert_eq!(check_writable(&nested), Ok(()));
        assert!(nested.is_dir());
        assert!(!nested.join(".fp-write-test").exists());
    }
}
