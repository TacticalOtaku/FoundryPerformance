//! Сценарии установки и удаления. Системные шаги откатываются, если дальше что-то сломалось.
use super::layout::{self, InstallManifest, EXE_NAME, INSTALL_MARKER};
use super::shortcuts::{self, LINK_NAME};
use super::{dircheck, procs, registry, selfdelete, selfreplace, version};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Step {
    Check,
    Copy,
    Shortcuts,
    Register,
    Done,
}

pub struct InstallRequest {
    pub dir: PathBuf,
    pub desktop: bool,
}

/// Возвращает путь к установленному exe; ошибка — i18n-ключ.
pub fn install(req: &InstallRequest, src_exe: &Path, progress: &dyn Fn(Step, u8)) -> Result<PathBuf, &'static str> {
    progress(Step::Check, 5);
    dircheck::check_writable(&req.dir)?;
    if !procs::running_from(&req.dir).is_empty() {
        return Err("install.err.running");
    }

    progress(Step::Copy, 30);
    let target = req.dir.join(EXE_NAME);
    let prev_manifest = layout::read_manifest(&req.dir);
    selfreplace::replace(src_exe, &target).map_err(|_| "install.err.copy")?;
    let manifest = InstallManifest {
        schema: 1,
        version: version::current().to_string(),
        installed_at: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
        desktop_shortcut: req.desktop,
    };
    let undo_copy = || {
        let _ = selfreplace::rollback(&target);
        match &prev_manifest {
            Some(m) => {
                let _ = layout::write_manifest(&req.dir, m);
            }
            None => {
                let _ = fs::remove_file(&target);
                let _ = fs::remove_file(req.dir.join(INSTALL_MARKER));
            }
        }
    };
    if layout::write_manifest(&req.dir, &manifest).is_err() {
        undo_copy();
        return Err("install.err.copy");
    }

    progress(Step::Shortcuts, 60);
    let links = match make_links(&target, &req.dir, req.desktop) {
        Ok(l) => l,
        Err(e) => {
            undo_copy();
            return Err(e);
        }
    };

    progress(Step::Register, 85);
    let size_kb = fs::metadata(&target).map(|m| (m.len() / 1024) as u32).unwrap_or(0);
    if registry::write_install(&target, &req.dir, &manifest.version, size_kb).is_err() {
        for l in &links {
            let _ = fs::remove_file(l);
        }
        undo_copy();
        return Err("install.err.register");
    }

    progress(Step::Done, 100);
    Ok(target)
}

fn make_links(target: &Path, dir: &Path, desktop: bool) -> Result<Vec<PathBuf>, &'static str> {
    let start = shortcuts::start_menu_dir().ok_or("install.err.shortcuts")?.join(LINK_NAME);
    shortcuts::create(&start, target, dir).map_err(|_| "install.err.shortcuts")?;
    let mut made = vec![start];
    if let Some(desk) = shortcuts::desktop_dir().map(|d| d.join(LINK_NAME)) {
        if desktop {
            if shortcuts::create(&desk, target, dir).is_err() {
                for l in &made {
                    let _ = fs::remove_file(l);
                }
                return Err("install.err.shortcuts");
            }
            made.push(desk);
        } else {
            // переустановка с выключенным ярлыком — убираем старый
            let _ = fs::remove_file(desk);
        }
    }
    Ok(made)
}

/// Существующая установка: папка из реестра, если exe в ней на месте.
pub fn existing() -> Option<(PathBuf, String)> {
    registry::read_install().filter(|(dir, _)| dir.join(EXE_NAME).exists())
}

pub fn uninstall(wipe_data: bool) -> Result<(), &'static str> {
    let dir = existing()
        .map(|(d, _)| d)
        .or_else(|| std::env::current_exe().ok()?.parent().map(Path::to_path_buf))
        .ok_or("uninstall.err.files")?;
    if !procs::running_from(&dir).is_empty() {
        return Err("uninstall.err.running");
    }
    for base in [shortcuts::start_menu_dir(), shortcuts::desktop_dir()].into_iter().flatten() {
        let _ = fs::remove_file(base.join(LINK_NAME));
    }
    registry::delete_install();
    if wipe_data {
        for var in ["APPDATA", "LOCALAPPDATA"] {
            if let Some(base) = std::env::var_os(var) {
                let _ = fs::remove_dir_all(PathBuf::from(base).join("FoundryPerformance"));
            }
        }
    }
    // Файлы удаляем, только если это действительно наша папка установки
    if dir.join(INSTALL_MARKER).exists() {
        selfdelete::spawn(&dir).map_err(|_| "uninstall.err.files")?;
    }
    Ok(())
}
