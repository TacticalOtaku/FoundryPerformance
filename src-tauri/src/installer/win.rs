//! Windows-реализация `InstallHost` (реестр, ярлыки COM, процессы ToolHelp32,
//! отложенное самоудаление через cmd) и ранняя проверка Microsoft WebView2 Runtime.
use super::install::InstallHost;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use windows::core::{Interface, GUID, HSTRING, PWSTR};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Shell::{IShellLinkW, SHGetKnownFolderPath, ShellExecuteW, ShellLink, FOLDERID_Desktop, FOLDERID_Programs, KF_FLAG_DEFAULT};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, IDYES, MB_ICONWARNING, MB_YESNO, SW_SHOWNORMAL};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_SET_VALUE};
use winreg::RegKey;

pub const APP_KEY: &str = r"Software\FoundryPerformance";
/// Запись в «Приложениях Windows» (HKCU — без прав администратора).
pub const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\FoundryPerformance";

const WEBVIEW2_CLIENT: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
const WEBVIEW2_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";

pub struct WinInstallHost;

impl InstallHost for WinInstallHost {
    fn windows_dir(&self) -> Option<PathBuf> {
        std::env::var_os("WINDIR").map(PathBuf::from)
    }

    fn default_install_dir(&self) -> PathBuf {
        let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        base.join("Programs").join("FoundryPerformance")
    }

    fn current_exe(&self) -> Option<PathBuf> {
        std::env::current_exe().ok()
    }

    fn data_dirs(&self) -> Vec<PathBuf> {
        ["APPDATA", "LOCALAPPDATA"]
            .into_iter()
            .filter_map(std::env::var_os)
            .map(|base| PathBuf::from(base).join("FoundryPerformance"))
            .collect()
    }

    fn running_from(&self, dir: &Path) -> Vec<u32> {
        running_from(dir)
    }

    fn wait_until_alone(&self, dir: &Path, timeout: Duration) {
        wait_until_alone(dir, timeout);
    }

    fn start_menu_dir(&self) -> Option<PathBuf> {
        known_folder(&FOLDERID_Programs)
    }

    fn desktop_dir(&self) -> Option<PathBuf> {
        known_folder(&FOLDERID_Desktop)
    }

    fn create_shortcut(&self, link: &Path, target: &Path, workdir: &Path) -> Result<(), String> {
        create_link(link, target, workdir).map_err(|e| e.to_string())
    }

    fn read_install(&self) -> Option<(PathBuf, String)> {
        let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(APP_KEY).ok()?;
        let dir: String = key.get_value("InstallDir").ok()?;
        let version: String = key.get_value("Version").ok()?;
        Some((PathBuf::from(dir), version))
    }

    fn write_install(&self, exe: &Path, dir: &Path, version: &str, size_kb: u32) -> Result<(), String> {
        write_key(APP_KEY, &app_values(dir, version)).map_err(|e| e.to_string())?;
        write_key(UNINSTALL_KEY, &uninstall_values(exe, dir, version, size_kb)).map_err(|e| e.to_string())
    }

    fn set_version(&self, version: &str) {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(k) = hkcu.open_subkey_with_flags(APP_KEY, KEY_SET_VALUE) {
            let _ = k.set_value("Version", &version.to_string());
        }
        if let Ok(k) = hkcu.open_subkey_with_flags(UNINSTALL_KEY, KEY_SET_VALUE) {
            let _ = k.set_value("DisplayVersion", &version.to_string());
        }
    }

    fn delete_install(&self) {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(UNINSTALL_KEY);
        let _ = hkcu.delete_subkey_all(APP_KEY);
    }

    fn spawn_self_delete(&self, _dir: &Path, cmd: &str) -> io::Result<()> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        std::process::Command::new("cmd")
            .raw_arg(format!("/C {cmd}"))
            .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
            .spawn()
            .map(|_| ())
    }

    fn launch(&self, exe: &Path, args: &[String]) -> Result<(), String> {
        std::process::Command::new(exe)
            .args(args)
            .current_dir(exe.parent().unwrap_or(Path::new(".")))
            .spawn()
            .map(|_| ())
            .map_err(|_| "install.err.launch".to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum RegValue {
    Str(String),
    Dword(u32),
}

fn uninstall_values(exe: &Path, dir: &Path, version: &str, size_kb: u32) -> Vec<(&'static str, RegValue)> {
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

fn app_values(dir: &Path, version: &str) -> Vec<(&'static str, RegValue)> {
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

fn known_folder(id: &GUID) -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s.map(PathBuf::from)
    }
}

fn create_link(link: &Path, target: &Path, workdir: &Path) -> windows::core::Result<()> {
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

fn is_inside(image: &Path, dir: &Path) -> bool {
    let norm = |p: &Path| p.to_string_lossy().to_lowercase().trim_end_matches('\\').to_string();
    let (img, d) = (norm(image), norm(dir));
    img.len() > d.len() && img.starts_with(&d) && img[d.len()..].starts_with('\\')
}

fn image_path(pid: u32) -> Option<PathBuf> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
        let _ = CloseHandle(h);
        ok.then(|| PathBuf::from(String::from_utf16_lossy(&buf[..len as usize])))
    }
}

/// PID процессов, чей exe лежит в `dir` (кроме нас самих): их надо закрыть перед заменой файлов.
fn running_from(dir: &Path) -> Vec<u32> {
    let me = std::process::id();
    let mut found = Vec::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return found };
        let mut entry = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut more = Process32FirstW(snap, &mut entry).is_ok();
        while more {
            let pid = entry.th32ProcessID;
            if pid != me && image_path(pid).is_some_and(|p| is_inside(&p, dir)) {
                found.push(pid);
            }
            more = Process32NextW(snap, &mut entry).is_ok();
        }
        let _ = CloseHandle(snap);
    }
    found
}

/// После автообновления прежний процесс живёт ещё ~300 мс: ждём его, иначе защита от
/// второго запуска примет новую версию за дубль и закроет её.
fn wait_until_alone(dir: &Path, timeout: Duration) {
    let until = Instant::now() + timeout;
    while !running_from(dir).is_empty() && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub fn valid_webview2_pv(pv: &str) -> bool {
    !pv.is_empty() && pv != "0.0.0.0"
}

/// Официальный способ проверки WebView2 Runtime: значение `pv` в EdgeUpdate\Clients.
pub fn webview2_installed() -> bool {
    let places = [
        (HKEY_LOCAL_MACHINE, format!(r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{WEBVIEW2_CLIENT}")),
        (HKEY_LOCAL_MACHINE, format!(r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{WEBVIEW2_CLIENT}")),
        (HKEY_CURRENT_USER, format!(r"Software\Microsoft\EdgeUpdate\Clients\{WEBVIEW2_CLIENT}")),
    ];
    places.iter().any(|(root, path)| {
        RegKey::predef(*root)
            .open_subkey(path)
            .and_then(|k| k.get_value::<String, _>("pv"))
            .is_ok_and(|pv| valid_webview2_pv(&pv))
    })
}

/// Окна ещё нет (без WebView2 его не нарисовать) — спрашиваем системным диалогом.
pub fn prompt_webview2(lang: &str) {
    let (text, caption) = if lang == "ru" {
        ("Для работы нужен компонент Microsoft WebView2.\n\nОткрыть страницу загрузки?", "Foundry Performance")
    } else {
        ("Microsoft WebView2 Runtime is required.\n\nOpen the download page?", "Foundry Performance")
    };
    unsafe {
        if MessageBoxW(None, &HSTRING::from(text), &HSTRING::from(caption), MB_YESNO | MB_ICONWARNING) == IDYES {
            ShellExecuteW(None, &HSTRING::from("open"), &HSTRING::from(WEBVIEW2_URL), None, None, SW_SHOWNORMAL);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(v: &'a [(&str, RegValue)], name: &str) -> &'a RegValue {
        &v.iter().find(|(n, _)| *n == name).unwrap().1
    }

    #[test]
    fn default_dir_is_per_user_programs() {
        let d = WinInstallHost.default_install_dir();
        assert!(d.ends_with("Programs/FoundryPerformance") || d.ends_with("Programs\\FoundryPerformance"));
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

    #[test]
    fn creates_a_link_file() {
        let d = tempfile::tempdir().unwrap();
        let target = std::env::current_exe().unwrap();
        let link = d.path().join(super::super::install::LINK_NAME);
        create_link(&link, &target, target.parent().unwrap()).unwrap();
        assert!(link.metadata().unwrap().len() > 0);
    }

    #[test]
    fn known_folders_resolve() {
        assert!(WinInstallHost.start_menu_dir().unwrap().is_dir());
        assert!(WinInstallHost.desktop_dir().unwrap().is_dir());
    }

    #[test]
    fn inside_is_case_insensitive_and_respects_boundaries() {
        let dir = Path::new(r"C:\Programs\FoundryPerformance");
        assert!(is_inside(Path::new(r"c:\programs\foundryperformance\FoundryPerformance.exe"), dir));
        assert!(!is_inside(Path::new(r"C:\Programs\FoundryPerformanceX\a.exe"), dir));
        assert!(!is_inside(Path::new(r"C:\Programs\Foundry\a.exe"), Path::new(r"C:\Programs\Foundry\sub")));
    }

    #[test]
    fn waits_until_the_other_process_from_the_folder_exits() {
        let d = tempfile::tempdir().unwrap();
        let exe = d.path().join("old.exe");
        std::fs::copy(Path::new(&std::env::var_os("WINDIR").unwrap()).join(r"System32\PING.EXE"), &exe).unwrap();
        // ~1 с: один промежуток между двумя пингами
        let mut child = std::process::Command::new(&exe).args(["-n", "2", "127.0.0.1"]).stdout(std::process::Stdio::null()).spawn().unwrap();
        wait_until_alone(d.path(), Duration::from_secs(10));
        assert!(child.try_wait().unwrap().is_some(), "returned while the old process was still running");
        assert!(running_from(d.path()).is_empty());
    }

    #[test]
    fn current_process_is_never_reported() {
        let me = std::env::current_exe().unwrap();
        assert!(running_from(me.parent().unwrap()).is_empty());
    }

    #[test]
    fn webview2_version_strings_and_runtime_check() {
        assert!(!valid_webview2_pv(""));
        assert!(!valid_webview2_pv("0.0.0.0"));
        assert!(valid_webview2_pv("128.0.2739.42"));
        assert!(webview2_installed());
    }
}
