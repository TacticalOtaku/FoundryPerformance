use std::io;
use std::path::Path;

/// Удаляет только наши файлы и затем пустую папку (`rmdir` без `/S`):
/// чужие файлы в папке установки переживут удаление вместе с папкой.
/// `ping` вместо `timeout` — у скрытой консоли нет stdin, и `timeout` сразу падает.
pub fn command(dir: &Path) -> String {
    let d = dir.display();
    format!(
        "ping -n 3 127.0.0.1 >NUL & del /F /Q \"{d}\\FoundryPerformance.exe\" \"{d}\\FoundryPerformance.exe.old\" \"{d}\\FoundryPerformance.exe.new\" \"{d}\\install.json\" & rmdir \"{d}\""
    )
}

/// Запущенный exe сам себя удалить не может — поручаем это отсоединённому cmd после выхода.
pub fn spawn(dir: &Path) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    std::process::Command::new("cmd")
        .raw_arg(format!("/C {}", command(dir)))
        .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletes_only_our_files_and_never_recurses() {
        let c = command(Path::new(r"D:\Games"));
        for f in ["FoundryPerformance.exe", "FoundryPerformance.exe.old", "FoundryPerformance.exe.new", "install.json"] {
            assert!(c.contains(&format!("\"D:\\Games\\{f}\"")), "{f} missing in {c}");
        }
        assert!(c.ends_with("rmdir \"D:\\Games\""));
        assert!(!c.to_lowercase().contains("/s"));
        // ровно 4 наших файла + rmdir — никаких других путей
        assert_eq!(c.matches("\"D:\\Games").count(), 5);
    }
}
