# Custom Installer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Самоустанавливающийся `FoundryPerformance.exe` с экранами установки/удаления в стиле «Пульта» вместо NSIS и с заделом под автообновление.

**Architecture:** Режим (`launcher` / `install` / `uninstall`) определяется в Rust до создания окна. Модуль `installer/` делает проверку папки, копирование через `selfreplace`, `install.json`, ярлыки (IShellLinkW), реестр HKCU (winreg), поиск запущенных экземпляров (ToolHelp32), самоудаление, проверку WebView2. Экраны — Svelte в том же окне, прогресс — события Tauri.

**Tech Stack:** Rust (windows-rs 0.61, winreg 0.55, tauri-plugin-dialog 2), Svelte 5, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-26-installer-design.md`

## Global Constraints

- Имя установленного файла: `FoundryPerformance.exe`; маркеры рядом с ним: `install.json`, `portable`.
- Папка по умолчанию: `%LOCALAPPDATA%\Programs\FoundryPerformance`. Только HKCU, без прав администратора, без самоповышения.
- Удаление стирает **только наши файлы** (`FoundryPerformance.exe`, `.old`, `.new`, `install.json`), `rmdir` без `/S`.
- Мировые/пользовательские данные (`%APPDATA%\FoundryPerformance`, `%LOCALAPPDATA%\FoundryPerformance`) удаляются только по явному переключателю.
- В debug-сборке (`npm run tauri dev`) по умолчанию режим `launcher`; экран установки — аргументом `--installer`.
- Ошибки из Rust в UI — i18n-ключи (`install.err.*`, `uninstall.err.*`).
- Все правила Global Constraints основного плана (`2026-09-26-foundry-performance.md`) действуют.

---

### Task 1: Версии, раскладка файлов, определение режима

**Files:**
- Create: `src-tauri/src/installer/mod.rs`, `src-tauri/src/installer/version.rs`, `src-tauri/src/installer/layout.rs`, `src-tauri/src/installer/mode.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod installer;`)

**Interfaces — Produces:**
- `version::Version(u32,u32,u32)` (`Ord`, `Display`), `Version::parse(&str) -> Option<Version>`, `version::current() -> Version`
- `layout::{EXE_NAME, INSTALL_MARKER, PORTABLE_MARKER}`, `layout::default_dir() -> PathBuf`, `layout::InstallManifest { schema, version, installed_at, desktop_shortcut }`, `layout::read_manifest(&Path) -> Option<InstallManifest>`, `layout::write_manifest(&Path, &InstallManifest) -> io::Result<()>`
- `mode::Mode { Launcher, Install, Uninstall }`, `mode::detect(args: &[String], exe_dir: &Path, dev: bool) -> Mode`, `mode::InstallState { Fresh, Upgrade, Current }`, `mode::install_state(current: Version, existing: Option<Version>) -> InstallState`

- [ ] Тесты (в каждом файле `#[cfg(test)]`):
  - version: `parse("0.1.0") == Some(Version(0,1,0))`, `parse("v1.2") == Some(Version(1,2,0))`, `parse("1.2.3-beta") == Some(Version(1,2,3))`, `parse("a.b")`/`parse("1.2.3.4")` → `None`; `Version(0,2,0) > Version(0,1,9)`.
  - layout: manifest roundtrip в tempdir; `read_manifest` отсутствующего → `None`.
  - mode: `--uninstall` → Uninstall (даже при маркерах); маркер `install.json` → Launcher; маркер `portable` → Launcher; `--portable`/`--open` → Launcher; пустая папка → Install; `dev=true` без `--installer` → Launcher, с `--installer` → Install; `install_state`: None→Fresh, старше→Upgrade, равная/новее→Current.
- [ ] Запустить — FAIL; реализовать; PASS; commit `feat(installer): versions, layout and mode detection`.

Реализация:
```rust
// version.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u32, pub u32, pub u32);
impl Version {
    pub fn parse(s: &str) -> Option<Version> {
        let core = s.trim().trim_start_matches('v').split(['-', '+']).next()?;
        let parts: Vec<&str> = core.split('.').collect();
        if parts.is_empty() || parts.len() > 3 { return None; }
        let n = |i: usize| parts.get(i).map_or(Some(0), |p| p.parse::<u32>().ok());
        Some(Version(n(0)?, n(1)?, n(2)?))
    }
}
impl std::fmt::Display for Version { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}.{}.{}", self.0, self.1, self.2) } }
pub fn current() -> Version { Version::parse(env!("CARGO_PKG_VERSION")).expect("valid crate version") }

// layout.rs
pub const EXE_NAME: &str = "FoundryPerformance.exe";
pub const INSTALL_MARKER: &str = "install.json";
pub const PORTABLE_MARKER: &str = "portable";
pub fn default_dir() -> PathBuf { local_appdata().join("Programs").join("FoundryPerformance") }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] #[serde(rename_all = "camelCase")]
pub struct InstallManifest { pub schema: u32, pub version: String, pub installed_at: u64, pub desktop_shortcut: bool }
pub fn read_manifest(dir: &Path) -> Option<InstallManifest> { serde_json::from_str(&fs::read_to_string(dir.join(INSTALL_MARKER)).ok()?).ok() }
pub fn write_manifest(dir: &Path, m: &InstallManifest) -> io::Result<()> { fs::write(dir.join(INSTALL_MARKER), serde_json::to_string_pretty(m).map_err(io::Error::other)?) }

// mode.rs
pub fn detect(args: &[String], exe_dir: &Path, dev: bool) -> Mode {
    let has = |f: &str| args.iter().any(|a| a == f);
    if has("--uninstall") { return Mode::Uninstall; }
    if dev && !has("--installer") { return Mode::Launcher; }
    if has("--portable") || has("--open") || exe_dir.join(PORTABLE_MARKER).exists() || exe_dir.join(INSTALL_MARKER).exists() { return Mode::Launcher; }
    Mode::Install
}
pub fn install_state(current: Version, existing: Option<Version>) -> InstallState {
    match existing { None => InstallState::Fresh, Some(v) if v < current => InstallState::Upgrade, Some(_) => InstallState::Current }
}
```

---

### Task 2: Проверка папки и `selfreplace`

**Files:** Create `src-tauri/src/installer/dircheck.rs`, `src-tauri/src/installer/selfreplace.rs`

**Interfaces — Produces:**
- `dircheck::check_static(dir: &Path) -> Result<(), &'static str>` (без изменений на диске: абсолютный, не корень, не системная)
- `dircheck::check_writable(dir: &Path) -> Result<(), &'static str>` (создаёт папку, пишет пробный файл)
- `selfreplace::replace(src: &Path, target: &Path) -> io::Result<()>`, `selfreplace::rollback(target: &Path) -> io::Result<()>`, `selfreplace::cleanup(target: &Path)`

- [ ] Тесты: `check_static("relative")` → `install.err.notAbsolute`; `check_static("C:\\")` → `install.err.root`; `check_static(%WINDIR%\\Temp)` → `install.err.system`; tempdir → Ok; `check_writable(tempdir/"a/b")` создаёт и Ok. `replace` в пустую цель → файл с содержимым src, `.old` нет; поверх существующего → новое содержимое, `.old` со старым; `cleanup` удаляет `.old`; `rollback` возвращает старое; `replace(x, x)` → Ok без изменений.
- [ ] FAIL → реализовать → PASS → commit `feat(installer): directory checks and self-replace`.

Реализация:
```rust
// dircheck.rs
pub fn check_static(dir: &Path) -> Result<(), &'static str> {
    if !dir.is_absolute() { return Err("install.err.notAbsolute"); }
    if dir.parent().is_none() { return Err("install.err.root"); }
    let lower = |p: &Path| p.to_string_lossy().to_lowercase();
    if let Some(win) = std::env::var_os("WINDIR") {
        if lower(dir).starts_with(&lower(Path::new(&win))) { return Err("install.err.system"); }
    }
    Ok(())
}
pub fn check_writable(dir: &Path) -> Result<(), &'static str> {
    check_static(dir)?;
    fs::create_dir_all(dir).map_err(|_| "install.err.noAccess")?;
    let probe = dir.join(".fp-write-test");
    fs::write(&probe, b"ok").map_err(|_| "install.err.noAccess")?;
    let _ = fs::remove_file(probe);
    Ok(())
}

// selfreplace.rs
fn suffixed(p: &Path, s: &str) -> PathBuf { let mut o = p.as_os_str().to_owned(); o.push(s); PathBuf::from(o) }
pub fn replace(src: &Path, target: &Path) -> io::Result<()> {
    if same_file(src, target) { return Ok(()); }
    let new = suffixed(target, ".new");
    let old = suffixed(target, ".old");
    fs::copy(src, &new)?;
    let had_old = target.exists();
    if had_old { let _ = fs::remove_file(&old); fs::rename(target, &old)?; }   // запущенный exe переименовать можно
    if let Err(e) = fs::rename(&new, target) {
        if had_old { let _ = fs::rename(&old, target); }
        let _ = fs::remove_file(&new);
        return Err(e);
    }
    Ok(())
}
pub fn rollback(target: &Path) -> io::Result<()> { let old = suffixed(target, ".old"); if old.exists() { let _ = fs::remove_file(target); fs::rename(old, target)?; } Ok(()) }
pub fn cleanup(target: &Path) { let _ = fs::remove_file(suffixed(target, ".old")); let _ = fs::remove_file(suffixed(target, ".new")); }
fn same_file(a: &Path, b: &Path) -> bool { matches!((fs::canonicalize(a), fs::canonicalize(b)), (Ok(x), Ok(y)) if x == y) }
```

---

### Task 3: Реестр, ярлыки, процессы, WebView2, самоудаление

**Files:** Create `src-tauri/src/installer/registry.rs`, `shortcuts.rs`, `procs.rs`, `webview2.rs`, `selfdelete.rs`; Modify `src-tauri/Cargo.toml`

Cargo:
```toml
winreg = "0.55"
tauri-plugin-dialog = "2"
windows = { version = "0.61", features = ["Win32_Foundation", "Win32_Graphics_Dxgi", "Win32_Globalization", "Win32_System_Com", "Win32_UI_Shell", "Win32_UI_WindowsAndMessaging", "Win32_System_Threading", "Win32_System_Diagnostics_ToolHelp"] }
```

**Interfaces — Produces:**
- `registry::{APP_KEY, UNINSTALL_KEY}`, `enum RegValue { Str(String), Dword(u32) }`, `registry::uninstall_values(exe, dir, version, size_kb) -> Vec<(&'static str, RegValue)>`, `registry::app_values(dir, version)`, `registry::write_install(exe, dir, version, size_kb) -> io::Result<()>`, `registry::read_install() -> Option<(PathBuf, String)>`, `registry::delete_install()`
- `shortcuts::{LINK_NAME, start_menu_dir(), desktop_dir()}`, `shortcuts::create(link, target, workdir) -> windows::core::Result<()>`
- `procs::is_inside(image: &Path, dir: &Path) -> bool`, `procs::running_from(dir: &Path) -> Vec<u32>` (без текущего PID)
- `webview2::valid_pv(&str) -> bool`, `webview2::installed() -> bool`, `webview2::prompt_download(lang: &str)`
- `selfdelete::command(dir: &Path) -> String`, `selfdelete::spawn(dir: &Path) -> io::Result<()>`

- [ ] Тесты (чистые части): `uninstall_values` содержит `UninstallString = "\"<exe>\" --uninstall"`, `DisplayIcon = <exe>`, `InstallLocation = <dir>`, `EstimatedSize = Dword(size)`, `NoModify/NoRepair = Dword(1)`; `is_inside` регистронезависим и не путает `...\Foundry` с `...\FoundryPerformance`; `valid_pv("")`/`valid_pv("0.0.0.0")` → false, `"128.0.2739.42"` → true; `selfdelete::command` содержит `rmdir "<dir>"`, не содержит `/S`, удаляет ровно четыре наших файла; `shortcuts::create` в tempdir создаёт `.lnk` (реальный COM, тест обычный).
- [ ] FAIL → реализовать → PASS → commit `feat(installer): registry, shortcuts, process check, WebView2 check, self-delete`.

Реализация ключевых мест:
```rust
// registry.rs
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
fn write_key(path: &str, values: &[(&str, RegValue)]) -> io::Result<()> {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(path)?;
    for (name, v) in values { match v { RegValue::Str(s) => key.set_value(name, s)?, RegValue::Dword(d) => key.set_value(name, d)? } }
    Ok(())
}

// shortcuts.rs
pub fn create(link: &Path, target: &Path, workdir: &Path) -> windows::core::Result<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let sl: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        sl.SetPath(&HSTRING::from(target.as_os_str()))?;
        sl.SetWorkingDirectory(&HSTRING::from(workdir.as_os_str()))?;
        sl.SetIconLocation(&HSTRING::from(target.as_os_str()), 0)?;
        sl.SetDescription(&HSTRING::from("Foundry Performance"))?;
        sl.cast::<IPersistFile>()?.Save(&HSTRING::from(link.as_os_str()), true)
    }
}
fn known(id: &GUID) -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s.map(PathBuf::from)
    }
}

// selfdelete.rs — только наши файлы, rmdir без /S; ping вместо timeout (у скрытой консоли нет stdin)
pub fn command(dir: &Path) -> String {
    let d = dir.display();
    format!("ping -n 3 127.0.0.1 >NUL & del /F /Q \"{d}\\FoundryPerformance.exe\" \"{d}\\FoundryPerformance.exe.old\" \"{d}\\FoundryPerformance.exe.new\" \"{d}\\install.json\" & rmdir \"{d}\"")
}
pub fn spawn(dir: &Path) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    std::process::Command::new("cmd").raw_arg(format!("/C {}", command(dir))).creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS).spawn().map(|_| ())
}
```
`procs::running_from` — `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)`, `Process32FirstW/NextW`, `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)`, `QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, …)`, `CloseHandle`; фильтр `is_inside` и `pid != std::process::id()`.
`webview2::installed` — `pv` в `HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}`, `HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{…}`, `HKCU\Software\Microsoft\EdgeUpdate\Clients\{…}`; `prompt_download` — `MessageBoxW(MB_YESNO | MB_ICONWARNING)` → `ShellExecuteW("open", "https://go.microsoft.com/fwlink/p/?LinkId=2124703")`.

---

### Task 4: Сценарии установки/удаления, команды, интеграция в запуск

**Files:** Create `src-tauri/src/installer/ops.rs`, `src-tauri/src/installer/commands.rs`; Modify `src-tauri/src/lib.rs`, `src-tauri/build.rs`, `src-tauri/capabilities/launcher.json`

**Interfaces — Produces (команды, JS camelCase):**
- `get_mode() -> ModeDto { mode: "launcher"|"install"|"uninstall", install: Option<InstallInfo { currentVersion, defaultDir, existingDir?, existingVersion?, state: "fresh"|"upgrade"|"current" }> }`
- `pick_install_dir(current: String) -> Option<String>`
- `check_install_dir(dir: String) -> Result<(), String>` (статическая проверка)
- `install(opts: { dir, desktop, launch }) -> Result<(), String>`; события `install-progress { step: "check"|"copy"|"shortcuts"|"register"|"done", pct }`
- `open_installed() -> Result<(), String>`
- `uninstall(wipeData: bool) -> Result<(), String>`

- [ ] `ops::install`: check_writable → `running_from` пусто (иначе `install.err.running`) → `selfreplace::replace` → `write_manifest` → ярлык «Пуск» (+ рабочий стол или удаление старого ярлыка на столе) → `registry::write_install` → Done. Любая ошибка после копирования: удалить созданные ярлыки, `selfreplace::rollback`.
- [ ] `ops::uninstall(wipe)`: папка из `registry::read_install()` или папка текущего exe; `running_from` (кроме себя) → `uninstall.err.running`; удалить ярлыки, ключи; при `wipe` — `%APPDATA%\FoundryPerformance`, `%LOCALAPPDATA%\FoundryPerformance`; `selfdelete::spawn`.
- [ ] `lib.rs`: в начале `run()` — если `!webview2::installed()` → `prompt_download` и выход; режим `mode::detect(&args, exe_dir, cfg!(debug_assertions))`; `app.manage(ModeState)`; трей и `--open` только в `Launcher`; в `Launcher` — `selfreplace::cleanup(current_exe)`; регистрация `tauri_plugin_dialog::init()` и новых команд.
- [ ] `build.rs` + `launcher.json`: `get_mode`, `pick_install_dir`, `check_install_dir`, `install`, `open_installed`, `uninstall` (+ `allow-*`).
- [ ] `cargo test`, `cargo clippy -D warnings` → PASS; commit `feat(installer): install/uninstall flows and commands`.

---

### Task 5: Экраны установки и удаления

**Files:** Create `src/screens/InstallScreen.svelte`, `src/screens/UninstallScreen.svelte`; Modify `src/lib/api.ts`, `src/lib/mock.ts`, `src/lib/types.ts`, `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`, `src/App.svelte`, `package.json` (`@tauri-apps/api` уже есть)

- [ ] `types.ts`: `Mode`, `InstallInfo`, `ModeDto`, `InstallStep`.
- [ ] `api.ts`: `setupApi = { getMode, pickDir, checkDir, install, openInstalled, uninstall, onProgress(cb) }` (`listen` из `@tauri-apps/api/event`); `mock.ts` — режим из `?mode=install|uninstall` для превью, имитация прогресса.
- [ ] i18n (RU/EN, parity-тест покрывает): `install.title` «Поставим на место», `install.lead`, `install.dir`, `install.browse`, `install.dirHint` («{size} МБ · права администратора не нужны»), `install.desktop`, `install.launch`, `install.go` «УСТАНОВИТЬ», `install.upgrade` «ОБНОВИТЬ {from} → {to}», `install.reinstall`, `install.openInstalled`, `install.done`, `install.step.check|copy|shortcuts|register|done`, `install.err.notAbsolute|root|system|noAccess|running|copy|shortcuts|register`, `uninstall.title` «Выключаем пульт?», `uninstall.wipe`, `uninstall.go`, `uninstall.cancel`, `uninstall.done`, `uninstall.err.running`, `setup.retry`.
- [ ] `InstallScreen.svelte` по эскизу спецификации: слева заголовок, описание, поле папки (`checkDir` на изменение, ошибка под полем), «ОБЗОР…», тумблеры (`Toggle`), кнопка (стиль `LaunchButton`); справа `Lcd` (процент + шаг), декоративная ручка (угол = −60° + 120°·pct/100), список шагов ✓/●/○. Состояния `fresh`/`upgrade`/`current`.
- [ ] `UninstallScreen.svelte`: заголовок, тумблер «удалить данные», «УДАЛИТЬ» (красная), «ОТМЕНА» (закрывает окно), после успеха — «Удалено» и авто-выход (Rust).
- [ ] `App.svelte`: сначала `getMode()`; `install` → InstallScreen, `uninstall` → UninstallScreen, иначе текущая логика (`app.load()` только в launcher).
- [ ] Превью в браузере (`?mode=install`, `?mode=uninstall`), обе темы; `npm run check`, `npx vitest run`; commit `feat(ui): install and uninstall screens`.

---

### Task 6: Сборка дистрибутива и ручная проверка

**Files:** Create `scripts/dist.ps1`; Delete `scripts/package-portable.ps1`; Modify `package.json`, `src-tauri/tauri.conf.json`, `.gitignore`, `README.md`

- [ ] `tauri.conf.json`: `bundle.active = false`, убрать `targets` и `windows.nsis`/`webviewInstallMode` (иконки оставить — нужны для ресурса exe).
- [ ] `package.json`: `"dist": "tauri build && powershell -ExecutionPolicy Bypass -File scripts/dist.ps1"`, убрать `portable`; `.gitignore`: `dist-release/`.
- [ ] `scripts/dist.ps1`: копия exe в `dist-release/FoundryPerformance-<v>.exe`; zip `FoundryPerformance-<v>-portable.zip` = `FoundryPerformance.exe` + пустой `portable`.
- [ ] README: раздел «Установка» (один exe, выбор папки, удаление через «Приложения Windows», portable-архив), «Сборка» (`npm run dist`).
- [ ] Ручная проверка (фиксировать результат): установка по умолчанию и в свою папку; ярлыки «Пуск»/рабочий стол; запись в «Приложениях Windows»; повторный запуск установщика → `current`; установщик с подменённой версией (временно `0.1.1` в `tauri.conf.json`/`Cargo.toml`) → `upgrade` и замена поверх запущенного; удаление без данных (серверы сохранились) и с данными; папка с чужим файлом не удаляется.
- [ ] Commit `chore: ship self-installing exe instead of NSIS`.
