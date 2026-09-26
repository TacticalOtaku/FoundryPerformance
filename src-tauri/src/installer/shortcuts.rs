use std::path::{Path, PathBuf};
use windows::core::{Interface, GUID, HSTRING};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{IShellLinkW, SHGetKnownFolderPath, ShellLink, FOLDERID_Desktop, FOLDERID_Programs, KF_FLAG_DEFAULT};

pub const LINK_NAME: &str = "Foundry Performance.lnk";

fn known(id: &GUID) -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s.map(PathBuf::from)
    }
}

/// «Пуск → Программы» текущего пользователя.
pub fn start_menu_dir() -> Option<PathBuf> {
    known(&FOLDERID_Programs)
}

/// Рабочий стол через Known Folders — учитывает перенос в OneDrive.
pub fn desktop_dir() -> Option<PathBuf> {
    known(&FOLDERID_Desktop)
}

pub fn create(link: &Path, target: &Path, workdir: &Path) -> windows::core::Result<()> {
    unsafe {
        // S_FALSE / RPC_E_CHANGED_MODE — COM уже инициализирован в потоке, это нормально
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let sl: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        sl.SetPath(&HSTRING::from(target.as_os_str()))?;
        sl.SetWorkingDirectory(&HSTRING::from(workdir.as_os_str()))?;
        sl.SetIconLocation(&HSTRING::from(target.as_os_str()), 0)?;
        sl.SetDescription(&HSTRING::from("Foundry Performance"))?;
        sl.cast::<IPersistFile>()?.Save(&HSTRING::from(link.as_os_str()), true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_link_file() {
        let d = tempfile::tempdir().unwrap();
        let target = std::env::current_exe().unwrap();
        let link = d.path().join(LINK_NAME);
        create(&link, &target, target.parent().unwrap()).unwrap();
        assert!(link.metadata().unwrap().len() > 0);
    }

    #[test]
    fn known_folders_resolve() {
        assert!(start_menu_dir().unwrap().is_dir());
        assert!(desktop_dir().unwrap().is_dir());
    }
}
