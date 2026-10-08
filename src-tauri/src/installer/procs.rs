use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use windows::core::PWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};

pub fn is_inside(image: &Path, dir: &Path) -> bool {
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
pub fn running_from(dir: &Path) -> Vec<u32> {
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
pub fn wait_until_alone(dir: &Path, timeout: Duration) {
    let until = Instant::now() + timeout;
    while !running_from(dir).is_empty() && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
