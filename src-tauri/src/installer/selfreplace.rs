//! Замена exe новой версией. На этом же держится будущее автообновление.
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn suffixed(p: &Path, s: &str) -> PathBuf {
    let mut o = p.as_os_str().to_owned();
    o.push(s);
    PathBuf::from(o)
}

fn same_file(a: &Path, b: &Path) -> bool {
    matches!((fs::canonicalize(a), fs::canonicalize(b)), (Ok(x), Ok(y)) if x == y)
}

/// Копирует `src` в `target`. Прежний `target` остаётся рядом как `.old`:
/// запущенный exe Windows удалить не даёт, а переименовать — даёт.
pub fn replace(src: &Path, target: &Path) -> io::Result<()> {
    if same_file(src, target) {
        return Ok(());
    }
    let new = suffixed(target, ".new");
    let old = suffixed(target, ".old");
    fs::copy(src, &new)?;
    let had_old = target.exists();
    if had_old {
        let _ = fs::remove_file(&old);
        fs::rename(target, &old)?;
    }
    if let Err(e) = fs::rename(&new, target) {
        if had_old {
            let _ = fs::rename(&old, target);
        }
        let _ = fs::remove_file(&new);
        return Err(e);
    }
    Ok(())
}

pub fn rollback(target: &Path) -> io::Result<()> {
    let old = suffixed(target, ".old");
    if old.exists() {
        let _ = fs::remove_file(target);
        fs::rename(old, target)?;
    }
    Ok(())
}

/// Убирает хвосты прошлой замены; `.old` может быть занят старым процессом — тогда в следующий раз.
pub fn cleanup(target: &Path) {
    let _ = fs::remove_file(suffixed(target, ".old"));
    let _ = fs::remove_file(suffixed(target, ".new"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let src = d.path().join("download.exe");
        fs::write(&src, "NEW").unwrap();
        let target = d.path().join("install").join("FoundryPerformance.exe");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        (d, src, target)
    }

    #[test]
    fn fresh_install_copies_without_old() {
        let (_d, src, target) = setup();
        replace(&src, &target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "NEW");
        assert!(!suffixed(&target, ".old").exists());
        assert!(!suffixed(&target, ".new").exists());
    }

    #[test]
    fn replace_keeps_previous_as_old_until_cleanup() {
        let (_d, src, target) = setup();
        fs::write(&target, "OLD").unwrap();
        replace(&src, &target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "NEW");
        assert_eq!(fs::read_to_string(suffixed(&target, ".old")).unwrap(), "OLD");
        cleanup(&target);
        assert!(!suffixed(&target, ".old").exists());
    }

    #[test]
    fn rollback_restores_previous() {
        let (_d, src, target) = setup();
        fs::write(&target, "OLD").unwrap();
        replace(&src, &target).unwrap();
        rollback(&target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "OLD");
    }

    #[test]
    fn replacing_itself_is_a_noop() {
        let (_d, _src, target) = setup();
        fs::write(&target, "SAME").unwrap();
        replace(&target, &target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "SAME");
        assert!(!suffixed(&target, ".old").exists());
    }
}
