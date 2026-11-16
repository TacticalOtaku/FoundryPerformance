//! Глубокий модуль установки, удаления и пост-обновленческой очистки при запуске.
use super::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const EXE_NAME: &str = "FoundryPerformance.exe";
pub const EXE_OLD_SUFFIX: &str = ".old";
pub const EXE_NEW_SUFFIX: &str = ".new";
/// Лежит рядом с установленным exe; его наличие = «запущен из папки установки».
pub const INSTALL_MARKER: &str = "install.json";
/// Пустой файл рядом с exe = portable-режим, установщик не показывается.
pub const PORTABLE_MARKER: &str = "portable";
pub const LINK_NAME: &str = "Foundry Performance.lnk";
pub const UPDATED_FLAG: &str = "--updated";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Step {
    Check,
    Copy,
    Shortcuts,
    Register,
    Done,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallInfo {
    pub current_version: String,
    pub default_dir: String,
    pub existing_dir: Option<String>,
    pub existing_version: Option<String>,
    pub state: InstallState,
}

pub struct InstallRequest {
    pub dir: PathBuf,
    pub desktop: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallManifest {
    pub schema: u32,
    pub version: String,
    pub installed_at: u64,
    pub desktop_shortcut: bool,
}

/// Порт операционной системы для установщика (реестр, ярлыки, процессы, отложенное удаление).
pub trait InstallHost: Send + Sync {
    fn windows_dir(&self) -> Option<PathBuf>;
    fn default_install_dir(&self) -> PathBuf;
    fn current_exe(&self) -> Option<PathBuf>;
    fn data_dirs(&self) -> Vec<PathBuf>;
    fn running_from(&self, dir: &Path) -> Vec<u32>;
    fn wait_until_alone(&self, dir: &Path, timeout: Duration);
    fn start_menu_dir(&self) -> Option<PathBuf>;
    fn desktop_dir(&self) -> Option<PathBuf>;
    fn create_shortcut(&self, link: &Path, target: &Path, workdir: &Path) -> Result<(), String>;
    fn read_install(&self) -> Option<(PathBuf, String)>;
    fn write_install(&self, exe: &Path, dir: &Path, version: &str, size_kb: u32) -> Result<(), String>;
    fn set_version(&self, version: &str);
    fn delete_install(&self);
    fn spawn_self_delete(&self, dir: &Path, cmd: &str) -> io::Result<()>;
    fn launch(&self, exe: &Path, args: &[String]) -> Result<(), String>;
}

/// Единый список наших файлов в папке установки — используется и при замене, и при самоудалении.
pub fn managed_files() -> [String; 4] {
    [
        EXE_NAME.to_string(),
        format!("{EXE_NAME}{EXE_OLD_SUFFIX}"),
        format!("{EXE_NAME}{EXE_NEW_SUFFIX}"),
        INSTALL_MARKER.to_string(),
    ]
}

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
pub fn replace_exe(src: &Path, target: &Path) -> io::Result<()> {
    if same_file(src, target) {
        return Ok(());
    }
    let new = suffixed(target, EXE_NEW_SUFFIX);
    let old = suffixed(target, EXE_OLD_SUFFIX);
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

/// Возвращает `Ok(true)`, если предыдущий `.old` восстановлен.
pub fn rollback_exe(target: &Path) -> io::Result<bool> {
    let old = suffixed(target, EXE_OLD_SUFFIX);
    if old.exists() {
        let _ = fs::remove_file(target);
        fs::rename(old, target)?;
        return Ok(true);
    }
    Ok(false)
}

/// Убирает хвосты прошлой замены; `.old` может быть занят старым процессом — тогда в следующий раз.
pub fn cleanup_exe(target: &Path) {
    let _ = fs::remove_file(suffixed(target, EXE_OLD_SUFFIX));
    let _ = fs::remove_file(suffixed(target, EXE_NEW_SUFFIX));
}

/// В debug-сборке (`tauri dev`) по умолчанию лаунчер; экран установки — по `--installer`.
pub fn detect_mode(args: &[String], exe_dir: &Path, dev: bool) -> Mode {
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

/// Из «Обзора» обычно выбирают родительскую папку (`D:\Games`) — дописываем свою подпапку.
pub fn for_picked(picked: &Path) -> PathBuf {
    let named_ours = picked.file_name().is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("FoundryPerformance"));
    if named_ours {
        picked.to_path_buf()
    } else {
        picked.join("FoundryPerformance")
    }
}

pub fn read_manifest(dir: &Path) -> Option<InstallManifest> {
    serde_json::from_str(&fs::read_to_string(dir.join(INSTALL_MARKER)).ok()?).ok()
}

pub fn write_manifest(dir: &Path, m: &InstallManifest) -> io::Result<()> {
    fs::write(dir.join(INSTALL_MARKER), serde_json::to_string_pretty(m).map_err(io::Error::other)?)
}

/// После автообновления в `install.json` осталась старая версия — поправляем.
/// `Ok(true)` — версия изменилась (надо обновить и реестр).
pub fn bump_manifest(dir: &Path, version: &str) -> io::Result<bool> {
    match read_manifest(dir) {
        Some(m) if m.version != version => {
            write_manifest(dir, &InstallManifest { version: version.to_string(), ..m })?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Удаляет только наши файлы из канонического layout и затем пустую папку (`rmdir` без `/S`):
/// чужие файлы в папке установки переживут удаление вместе с папкой.
/// `ping` вместо `timeout` — у скрытой консоли нет stdin, и `timeout` сразу падает.
pub fn selfdelete_command(dir: &Path) -> String {
    let d = dir.display();
    let files = managed_files().map(|f| format!("\"{d}\\{f}\"")).join(" ");
    format!("ping -n 3 127.0.0.1 >NUL & del /F /Q {files} & rmdir \"{d}\"")
}

pub struct Installer {
    mode: Mode,
    version: Version,
    host: Arc<dyn InstallHost>,
}

impl Installer {
    pub fn new(mode: Mode, version: Version, host: Arc<dyn InstallHost>) -> Self {
        Self { mode, version, host }
    }

    pub fn production(args: &[String], exe: Option<&Path>) -> Self {
        let exe_dir = exe.and_then(Path::parent).map(Path::to_path_buf).unwrap_or_default();
        let mode = detect_mode(args, &exe_dir, cfg!(debug_assertions));
        Self::new(mode, Version::current(), Arc::new(super::WinInstallHost))
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn info(&self) -> InstallInfo {
        let existing = self.existing();
        let state = install_state(self.version, existing.as_ref().and_then(|(_, v)| Version::parse(v)));
        InstallInfo {
            current_version: self.version.to_string(),
            default_dir: existing.as_ref().map_or_else(|| self.host.default_install_dir(), |(d, _)| d.clone()).display().to_string(),
            existing_dir: existing.as_ref().map(|(d, _)| d.display().to_string()),
            existing_version: existing.map(|(_, v)| v),
            state,
        }
    }

    /// Существующая установка: папка из реестра, если exe в ней на месте.
    pub fn existing(&self) -> Option<(PathBuf, String)> {
        self.host.read_install().filter(|(dir, _)| dir.join(EXE_NAME).exists())
    }

    /// Проверка без изменений на диске — для подсказки в поле ввода.
    pub fn check_static(&self, dir: &Path) -> Result<(), &'static str> {
        if !dir.is_absolute() {
            return Err("install.err.notAbsolute");
        }
        if dir.parent().is_none() {
            return Err("install.err.root");
        }
        let lower = |p: &Path| p.to_string_lossy().to_lowercase();
        if let Some(win) = self.host.windows_dir() {
            if lower(dir).starts_with(&lower(&win)) {
                return Err("install.err.system");
            }
        }
        Ok(())
    }

    /// Полная проверка перед установкой: создаёт папку и пишет пробный файл.
    pub fn check_writable(&self, dir: &Path) -> Result<(), &'static str> {
        self.check_static(dir)?;
        fs::create_dir_all(dir).map_err(|_| "install.err.noAccess")?;
        let probe = dir.join(".fp-write-test");
        fs::write(&probe, b"ok").map_err(|_| "install.err.noAccess")?;
        let _ = fs::remove_file(probe);
        Ok(())
    }

    /// Пост-обновленческая уборка при старте лаунчера: ожидание старого процесса, удаление `.old`/`.new`,
    /// синхронизация версии в `install.json` и «Приложениях Windows».
    pub fn finish_update_startup(&self, args: &[String], exe: Option<&Path>) {
        let exe_dir = exe.and_then(Path::parent).unwrap_or_else(|| Path::new(""));
        if args.iter().any(|a| a == UPDATED_FLAG) {
            self.host.wait_until_alone(exe_dir, Duration::from_secs(5));
        }
        if let Some(e) = exe {
            cleanup_exe(e);
        }
        let version = self.version.to_string();
        if bump_manifest(exe_dir, &version).unwrap_or(false) {
            self.host.set_version(&version);
        }
    }

    /// Возвращает путь к установленному exe; ошибка — i18n-ключ. Системные шаги откатываются при сбое.
    pub fn install(&self, req: &InstallRequest, src_exe: &Path, progress: &dyn Fn(Step, u8)) -> Result<PathBuf, &'static str> {
        progress(Step::Check, 5);
        self.check_writable(&req.dir)?;
        if !self.host.running_from(&req.dir).is_empty() {
            return Err("install.err.running");
        }

        progress(Step::Copy, 30);
        let target = req.dir.join(EXE_NAME);
        let prev_manifest = read_manifest(&req.dir);
        replace_exe(src_exe, &target).map_err(|_| "install.err.copy")?;
        let manifest = InstallManifest {
            schema: 1,
            version: self.version.to_string(),
            installed_at: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
            desktop_shortcut: req.desktop,
        };
        let undo_copy = || {
            let restored = rollback_exe(&target).unwrap_or(false);
            match &prev_manifest {
                Some(m) => {
                    let _ = write_manifest(&req.dir, m);
                }
                None => {
                    if !restored {
                        let _ = fs::remove_file(&target);
                    }
                    let _ = fs::remove_file(req.dir.join(INSTALL_MARKER));
                }
            }
        };
        if write_manifest(&req.dir, &manifest).is_err() {
            undo_copy();
            return Err("install.err.copy");
        }

        progress(Step::Shortcuts, 60);
        let links = match self.make_links(&target, &req.dir, req.desktop) {
            Ok(l) => l,
            Err(e) => {
                undo_copy();
                return Err(e);
            }
        };

        progress(Step::Register, 85);
        let size_kb = fs::metadata(&target).map(|m| (m.len() / 1024) as u32).unwrap_or(0);
        if self.host.write_install(&target, &req.dir, &manifest.version, size_kb).is_err() {
            for l in &links {
                let _ = fs::remove_file(l);
            }
            undo_copy();
            return Err("install.err.register");
        }

        progress(Step::Done, 100);
        Ok(target)
    }

    fn make_links(&self, target: &Path, dir: &Path, desktop: bool) -> Result<Vec<PathBuf>, &'static str> {
        let start = self.host.start_menu_dir().ok_or("install.err.shortcuts")?.join(LINK_NAME);
        self.host.create_shortcut(&start, target, dir).map_err(|_| "install.err.shortcuts")?;
        let mut made = vec![start];
        if let Some(desk) = self.host.desktop_dir().map(|d| d.join(LINK_NAME)) {
            if desktop {
                if self.host.create_shortcut(&desk, target, dir).is_err() {
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

    pub fn uninstall(&self, wipe_data: bool) -> Result<(), &'static str> {
        let dir = self
            .existing()
            .map(|(d, _)| d)
            .or_else(|| self.host.current_exe()?.parent().map(Path::to_path_buf))
            .ok_or("uninstall.err.files")?;
        if !self.host.running_from(&dir).is_empty() {
            return Err("uninstall.err.running");
        }
        for base in [self.host.start_menu_dir(), self.host.desktop_dir()].into_iter().flatten() {
            let _ = fs::remove_file(base.join(LINK_NAME));
        }
        self.host.delete_install();
        if wipe_data {
            for data_dir in self.host.data_dirs() {
                let _ = fs::remove_dir_all(data_dir);
            }
        }
        // Файлы удаляем, только если это действительно наша папка установки
        if dir.join(INSTALL_MARKER).exists() {
            let cmd = selfdelete_command(&dir);
            self.host.spawn_self_delete(&dir, &cmd).map_err(|_| "uninstall.err.files")?;
        }
        Ok(())
    }

    pub fn open_installed(&self) -> Result<(), String> {
        let (dir, _) = self.existing().ok_or_else(|| "install.err.launch".to_string())?;
        self.host.launch(&dir.join(EXE_NAME), &[])
    }

    pub fn launch(&self, exe: &Path, args: &[String]) -> Result<(), String> {
        self.host.launch(exe, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeHost {
        root: PathBuf,
        windows: Option<PathBuf>,
        running: Mutex<Vec<u32>>,
        waited: Mutex<Vec<PathBuf>>,
        fail_shortcut_at: Mutex<Option<usize>>,
        shortcut_calls: Mutex<usize>,
        fail_register: Mutex<bool>,
        registry: Mutex<Option<(PathBuf, String)>>,
        version_updates: Mutex<Vec<String>>,
        self_deletes: Mutex<Vec<(PathBuf, String)>>,
        launched: Mutex<Vec<(PathBuf, Vec<String>)>>,
    }

    impl FakeHost {
        fn new(root: &Path) -> Arc<Self> {
            let start = root.join("start_menu");
            let desk = root.join("desktop");
            let appdata = root.join("appdata").join("FoundryPerformance");
            let local = root.join("localappdata").join("FoundryPerformance");
            fs::create_dir_all(&start).unwrap();
            fs::create_dir_all(&desk).unwrap();
            fs::create_dir_all(&appdata).unwrap();
            fs::create_dir_all(&local).unwrap();
            Arc::new(Self {
                root: root.to_path_buf(),
                windows: Some(PathBuf::from(r"C:\Windows")),
                ..Default::default()
            })
        }
    }

    impl InstallHost for FakeHost {
        fn windows_dir(&self) -> Option<PathBuf> {
            self.windows.clone()
        }
        fn default_install_dir(&self) -> PathBuf {
            self.root.join("Programs").join("FoundryPerformance")
        }
        fn current_exe(&self) -> Option<PathBuf> {
            Some(self.root.join("src").join(EXE_NAME))
        }
        fn data_dirs(&self) -> Vec<PathBuf> {
            vec![
                self.root.join("appdata").join("FoundryPerformance"),
                self.root.join("localappdata").join("FoundryPerformance"),
            ]
        }
        fn running_from(&self, _dir: &Path) -> Vec<u32> {
            self.running.lock().unwrap().clone()
        }
        fn wait_until_alone(&self, dir: &Path, _timeout: Duration) {
            self.waited.lock().unwrap().push(dir.to_path_buf());
        }
        fn start_menu_dir(&self) -> Option<PathBuf> {
            Some(self.root.join("start_menu"))
        }
        fn desktop_dir(&self) -> Option<PathBuf> {
            Some(self.root.join("desktop"))
        }
        fn create_shortcut(&self, link: &Path, target: &Path, _workdir: &Path) -> Result<(), String> {
            let mut n = self.shortcut_calls.lock().unwrap();
            *n += 1;
            if *self.fail_shortcut_at.lock().unwrap() == Some(*n) {
                return Err("shortcut err".into());
            }
            fs::write(link, target.display().to_string()).map_err(|e| e.to_string())
        }
        fn read_install(&self) -> Option<(PathBuf, String)> {
            self.registry.lock().unwrap().clone()
        }
        fn write_install(&self, _exe: &Path, dir: &Path, version: &str, _size_kb: u32) -> Result<(), String> {
            if *self.fail_register.lock().unwrap() {
                return Err("reg err".into());
            }
            *self.registry.lock().unwrap() = Some((dir.to_path_buf(), version.to_string()));
            Ok(())
        }
        fn set_version(&self, version: &str) {
            self.version_updates.lock().unwrap().push(version.to_string());
        }
        fn delete_install(&self) {
            *self.registry.lock().unwrap() = None;
        }
        fn spawn_self_delete(&self, dir: &Path, cmd: &str) -> io::Result<()> {
            self.self_deletes.lock().unwrap().push((dir.to_path_buf(), cmd.to_string()));
            Ok(())
        }
        fn launch(&self, exe: &Path, args: &[String]) -> Result<(), String> {
            self.launched.lock().unwrap().push((exe.to_path_buf(), args.to_vec()));
            Ok(())
        }
    }

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn dir_validation_rejects_relative_root_and_system_and_writes_probe() {
        let tmp = tempfile::tempdir().unwrap();
        let inst = Installer::new(Mode::Install, Version(0, 3, 0), FakeHost::new(tmp.path()));

        assert_eq!(inst.check_static(Path::new("relative\\dir")), Err("install.err.notAbsolute"));
        assert_eq!(inst.check_static(Path::new("C:\\")), Err("install.err.root"));
        assert_eq!(inst.check_static(Path::new(r"C:\Windows\Temp")), Err("install.err.system"));

        let nested = tmp.path().join("a").join("b");
        assert_eq!(inst.check_writable(&nested), Ok(()));
        assert!(nested.is_dir());
        assert!(!nested.join(".fp-write-test").exists());
    }

    #[test]
    fn mode_detection_and_install_states() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join(INSTALL_MARKER), "{}").unwrap();
        assert_eq!(detect_mode(&args(&["--uninstall"]), d.path(), false), Mode::Uninstall);

        let fresh = tempfile::tempdir().unwrap();
        assert_eq!(detect_mode(&args(&[]), fresh.path(), false), Mode::Install);
        assert_eq!(detect_mode(&args(&["--portable"]), fresh.path(), false), Mode::Launcher);
        assert_eq!(detect_mode(&args(&["--open", "x"]), fresh.path(), false), Mode::Launcher);
        assert_eq!(detect_mode(&args(&[]), fresh.path(), true), Mode::Launcher);
        assert_eq!(detect_mode(&args(&["--installer"]), fresh.path(), true), Mode::Install);
        fs::write(fresh.path().join(PORTABLE_MARKER), "").unwrap();
        assert_eq!(detect_mode(&args(&[]), fresh.path(), false), Mode::Launcher);

        let cur = Version(0, 2, 0);
        assert_eq!(install_state(cur, None), InstallState::Fresh);
        assert_eq!(install_state(cur, Some(Version(0, 1, 0))), InstallState::Upgrade);
        assert_eq!(install_state(cur, Some(cur)), InstallState::Current);
        assert_eq!(install_state(cur, Some(Version(0, 3, 0))), InstallState::Current);
    }

    #[test]
    fn selfdelete_shares_canonical_layout_filenames_and_never_recurses() {
        let c = selfdelete_command(Path::new(r"D:\Games"));
        for f in managed_files() {
            assert!(c.contains(&format!("\"D:\\Games\\{f}\"")), "{f} missing in {c}");
        }
        assert!(c.ends_with("rmdir \"D:\\Games\""));
        assert!(!c.to_lowercase().contains("/s"));
        assert_eq!(c.matches("\"D:\\Games").count(), managed_files().len() + 1);
    }

    #[test]
    fn picked_folder_and_manifest_roundtrip() {
        assert_eq!(for_picked(Path::new(r"D:\Games")), PathBuf::from(r"D:\Games\FoundryPerformance"));
        assert_eq!(for_picked(Path::new(r"D:\Games\FoundryPerformance")), PathBuf::from(r"D:\Games\FoundryPerformance"));
        assert_eq!(for_picked(Path::new(r"D:\Games\foundryperformance")), PathBuf::from(r"D:\Games\foundryperformance"));

        let d = tempfile::tempdir().unwrap();
        assert_eq!(read_manifest(d.path()), None);
        let m = InstallManifest { schema: 1, version: "0.1.0".into(), installed_at: 42, desktop_shortcut: true };
        write_manifest(d.path(), &m).unwrap();
        assert_eq!(read_manifest(d.path()), Some(m));
    }

    #[test]
    fn exe_replace_rollback_cleanup_and_self_noop() {
        let d = tempfile::tempdir().unwrap();
        let src = d.path().join("download.exe");
        fs::write(&src, "NEW").unwrap();
        let target = d.path().join("install").join(EXE_NAME);
        fs::create_dir_all(target.parent().unwrap()).unwrap();

        replace_exe(&src, &target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "NEW");
        assert!(!suffixed(&target, EXE_OLD_SUFFIX).exists());

        fs::write(&target, "OLD").unwrap();
        replace_exe(&src, &target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "NEW");
        assert_eq!(fs::read_to_string(suffixed(&target, EXE_OLD_SUFFIX)).unwrap(), "OLD");
        assert!(rollback_exe(&target).unwrap());
        assert_eq!(fs::read_to_string(&target).unwrap(), "OLD");

        fs::write(suffixed(&target, EXE_OLD_SUFFIX), "X").unwrap();
        fs::write(suffixed(&target, EXE_NEW_SUFFIX), "Y").unwrap();
        cleanup_exe(&target);
        assert!(!suffixed(&target, EXE_OLD_SUFFIX).exists());
        assert!(!suffixed(&target, EXE_NEW_SUFFIX).exists());

        replace_exe(&target, &target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "OLD");
    }

    #[test]
    fn install_rolls_back_exe_manifest_and_shortcuts_on_failures() {
        let tmp = tempfile::tempdir().unwrap();
        let host = FakeHost::new(tmp.path());
        let src = tmp.path().join("new.exe");
        fs::write(&src, "NEW_EXE").unwrap();
        let dir = tmp.path().join("install");
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join(EXE_NAME);
        let old_manifest = InstallManifest { schema: 1, version: "0.2.0".into(), installed_at: 7, desktop_shortcut: true };

        // 1. Настройка существующей установки + сбой на втором ярлыке (рабочий стол) -> откат exe, manifest и ярлыка Пуск
        fs::write(&target, "OLD_EXE").unwrap();
        write_manifest(&dir, &old_manifest).unwrap();
        *host.fail_shortcut_at.lock().unwrap() = Some(2);
        let inst = Installer::new(Mode::Install, Version(0, 3, 0), host.clone());
        let err = inst.install(&InstallRequest { dir: dir.clone(), desktop: true }, &src, &|_, _| ()).unwrap_err();
        assert_eq!(err, "install.err.shortcuts");
        assert_eq!(fs::read_to_string(&target).unwrap(), "OLD_EXE");
        assert_eq!(read_manifest(&dir), Some(old_manifest.clone()));
        assert!(!host.start_menu_dir().unwrap().join(LINK_NAME).exists());
        assert!(!host.desktop_dir().unwrap().join(LINK_NAME).exists());

        // 2. Сбой записи в реестр -> удаляются оба ярлыка, восстанавливаются прежний exe и manifest
        *host.fail_shortcut_at.lock().unwrap() = None;
        *host.shortcut_calls.lock().unwrap() = 0;
        *host.fail_register.lock().unwrap() = true;
        let err = inst.install(&InstallRequest { dir: dir.clone(), desktop: true }, &src, &|_, _| ()).unwrap_err();
        assert_eq!(err, "install.err.register");
        assert_eq!(fs::read_to_string(&target).unwrap(), "OLD_EXE");
        assert_eq!(read_manifest(&dir), Some(old_manifest));
        assert!(!host.start_menu_dir().unwrap().join(LINK_NAME).exists());
        assert!(!host.desktop_dir().unwrap().join(LINK_NAME).exists());

        // 3. Сбой при чистой установке убирает и сам target, и install.json
        let fresh_dir = tmp.path().join("fresh");
        let err = inst.install(&InstallRequest { dir: fresh_dir.clone(), desktop: false }, &src, &|_, _| ()).unwrap_err();
        assert_eq!(err, "install.err.register");
        assert!(!fresh_dir.join(EXE_NAME).exists());
        assert!(!fresh_dir.join(INSTALL_MARKER).exists());

        // 4. Запущенный процесс блокирует установку до копирования
        *host.fail_register.lock().unwrap() = false;
        *host.running.lock().unwrap() = vec![42];
        assert_eq!(inst.install(&InstallRequest { dir: dir.clone(), desktop: true }, &src, &|_, _| ()).unwrap_err(), "install.err.running");
        assert_eq!(fs::read_to_string(&target).unwrap(), "OLD_EXE");
    }

    #[test]
    fn uninstall_blocks_when_running_and_handles_wipe_data() {
        let tmp = tempfile::tempdir().unwrap();
        let host = FakeHost::new(tmp.path());
        let inst = Installer::new(Mode::Uninstall, Version(0, 3, 0), host.clone());
        let dir = tmp.path().join("install");
        let src = tmp.path().join("app.exe");
        fs::write(&src, "EXE").unwrap();

        inst.install(&InstallRequest { dir: dir.clone(), desktop: true }, &src, &|_, _| ()).unwrap();
        let user_data = host.data_dirs()[0].join("settings.json");
        fs::write(&user_data, "{}").unwrap();

        // Блокируется, пока процесс в папке жив
        *host.running.lock().unwrap() = vec![99];
        assert_eq!(inst.uninstall(true), Err("uninstall.err.running"));
        assert!(user_data.exists());
        assert!(host.read_install().is_some());

        // Удаление с wipe_data=false сохраняет данные пользователя, но сносит ярлыки, реестр и планирует удаление наших файлов
        *host.running.lock().unwrap() = vec![];
        inst.uninstall(false).unwrap();
        assert!(user_data.exists());
        assert!(host.read_install().is_none());
        assert!(!host.start_menu_dir().unwrap().join(LINK_NAME).exists());
        assert!(!host.desktop_dir().unwrap().join(LINK_NAME).exists());
        assert_eq!(host.self_deletes.lock().unwrap().len(), 1);

        // Повторная установка и удаление с wipe_data=true удаляет данные пользователя
        inst.install(&InstallRequest { dir: dir.clone(), desktop: false }, &src, &|_, _| ()).unwrap();
        fs::write(&user_data, "{}").unwrap();
        inst.uninstall(true).unwrap();
        for d in host.data_dirs() {
            assert!(!d.exists());
        }
    }

    #[test]
    fn startup_housekeeping_waits_cleans_bumps_manifest_and_syncs_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let host = FakeHost::new(tmp.path());
        let dir = tmp.path().join("install");
        fs::create_dir_all(&dir).unwrap();
        let exe = dir.join(EXE_NAME);
        fs::write(&exe, "NEW").unwrap();
        fs::write(suffixed(&exe, EXE_OLD_SUFFIX), "OLD").unwrap();
        fs::write(suffixed(&exe, EXE_NEW_SUFFIX), "NEW").unwrap();
        write_manifest(&dir, &InstallManifest { schema: 1, version: "0.2.0".into(), installed_at: 1, desktop_shortcut: false }).unwrap();

        let inst = Installer::new(Mode::Launcher, Version(0, 3, 0), host.clone());
        inst.finish_update_startup(&args(&["--updated"]), Some(&exe));

        assert_eq!(host.waited.lock().unwrap().as_slice(), &[dir.as_path()]);
        assert!(!suffixed(&exe, EXE_OLD_SUFFIX).exists());
        assert!(!suffixed(&exe, EXE_NEW_SUFFIX).exists());
        assert_eq!(read_manifest(&dir).unwrap().version, "0.3.0");
        assert_eq!(host.version_updates.lock().unwrap().as_slice(), &["0.3.0".to_string()]);

        // Повторный запуск с актуальной версией без --updated не ждёт и не трогает реестр
        host.waited.lock().unwrap().clear();
        host.version_updates.lock().unwrap().clear();
        inst.finish_update_startup(&[], Some(&exe));
        assert!(host.waited.lock().unwrap().is_empty());
        assert!(host.version_updates.lock().unwrap().is_empty());
    }
}
