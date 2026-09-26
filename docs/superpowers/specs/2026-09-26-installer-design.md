# Foundry Performance — собственный установщик (дизайн)

Дата: 2026-09-26 · Статус: утверждён

## 1. Цель

Заменить стандартный NSIS-установщик собственным, в стиле «Пульта», и заложить архитектуру
под будущее автообновление. Автообновление (загрузка из ленты релизов) в этот объём **не входит**.

## 2. Подход: самоустанавливающийся exe

Распространяется один файл `FoundryPerformance-<версия>.exe`. Режим определяется при старте,
в Rust, до создания окна:

| Условие | Режим |
|---|---|
| аргумент `--uninstall` | `uninstall` — экран «Удаление» |
| рядом с exe есть `install.json` (запущен из папки установки) | `launcher` |
| аргумент `--portable` или рядом с exe есть файл `portable` | `launcher` |
| аргумент `--open <url>` | `launcher` (как сейчас, для отладки и замеров) |
| иначе | `install` — экран «Установка» |

В режиме `install` экран показывает одно из состояний:
- `fresh` — не установлено → «Установить»;
- `upgrade` — установлена более старая версия → «Обновить vX → vY»;
- `current` — установлена та же или более новая → «Открыть установленную» / «Переустановить».

Существующая установка находится по `HKCU\Software\FoundryPerformance\InstallDir`
и `install.json` в этой папке.

## 3. Установка

1. **Папка.** По умолчанию `%LOCALAPPDATA%\Programs\FoundryPerformance`, выбор — системным
   диалогом (`tauri-plugin-dialog`, `blocking_pick_folder` в async-команде).
   Проверка: путь абсолютный; не корень диска; не `C:\Windows`; в папку можно писать
   (пробный файл). Нет прав → понятная ошибка «выберите другую папку»; самоповышения прав нет.
2. **Запущенный экземпляр.** Если из целевой папки уже работает `FoundryPerformance.exe`
   (ToolHelp32 + `QueryFullProcessImageNameW`) — «Закройте Foundry Performance» + «Повторить».
3. **Копирование.** Модуль `selfreplace`: запись во временный файл `FoundryPerformance.exe.new`,
   существующий exe переименовывается в `.old`, `.new` → `FoundryPerformance.exe`.
   `.old` удаляется при следующем старте установленной копии.
4. **`install.json`** в папке установки: `{ schema, version, installedAt, desktopShortcut }`.
5. **Ярлыки** (IShellLinkW + IPersistFile): меню «Пуск»
   (`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Foundry Performance.lnk`) и, по выбору,
   рабочий стол (`SHGetKnownFolderPath(FOLDERID_Desktop)` — учитывает OneDrive).
6. **Реестр (HKCU, `winreg`):**
   - `Software\FoundryPerformance`: `InstallDir`, `Version`;
   - `Software\Microsoft\Windows\CurrentVersion\Uninstall\FoundryPerformance`: `DisplayName`,
     `DisplayVersion`, `DisplayIcon`, `Publisher`, `InstallLocation`,
     `UninstallString = "<exe>" --uninstall`, `EstimatedSize` (КБ), `NoModify=1`, `NoRepair=1`.
7. **Завершение.** «Запустить после установки» (по умолчанию вкл.) → запуск установленного exe
   и выход установщика; иначе — экран «Готово».

Прогресс — события `install-progress { step, pct }` (Tauri Emitter): `check`, `copy`,
`shortcuts`, `register`, `done`. Ошибка любого шага — событие с ключом i18n, частично
созданное откатывается (ярлыки/реестр удаляются, `.old` возвращается на место).

## 4. Удаление (`--uninstall`)

Экран «Удаление»: переключатель «Удалить также мои серверы, настройки и кэш игры» (выкл.).
1. Удалять нельзя, пока запущен экземпляр из папки установки (кроме самого себя) — тот же
   экран «Закройте…».
2. Удалить ярлыки, ключи реестра; при включённом переключателе —
   `%APPDATA%\FoundryPerformance` и `%LOCALAPPDATA%\FoundryPerformance`.
3. Самоудаление после выхода: отсоединённый `cmd /C` с паузой 2 с удаляет **только наши файлы**
   (`FoundryPerformance.exe`, `.old`, `.new`, `install.json`) и затем `rmdir` **без** `/S` —
   пустая папка удаляется, папка с чужими файлами остаётся. Так нельзя стереть лишнее, даже
   если пользователь поставил программу прямо в `D:\Games`.

## 5. Интерфейс

- Новые экраны `InstallScreen.svelte`, `UninstallScreen.svelte` в том же окне без рамки.
- Установка: заголовок «Поставим на место», описание, поле папки + «Обзор…», тумблеры
  «Ярлык на рабочем столе» (вкл.), «Запустить после установки» (вкл.), кнопка «УСТАНОВИТЬ»
  (стиль `LaunchButton`). Справа: ЖК с процентом и названием шага, ручка поворачивается по
  мере прогресса, список шагов (✓ / ● / ○).
- Удаление: «Выключаем пульт?», тумблер удаления данных, красная «УДАЛИТЬ», «ОТМЕНА».
- Все строки — RU/EN (`install.*`, `uninstall.*`).

## 6. Нет WebView2

До создания окна Rust проверяет `pv` в
`HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}`
и в `HKCU\Software\Microsoft\EdgeUpdate\Clients\{…}`. Нет ни там, ни там — `MessageBoxW`
«Нужен компонент Microsoft WebView2. Открыть страницу загрузки?» → `ShellExecuteW` на
`https://go.microsoft.com/fwlink/p/?LinkId=2124703`, выход.

## 7. Сборка и поставка

- NSIS отключается (`bundle.active = false`).
- `npm run dist` → `dist-release/FoundryPerformance-<версия>.exe` и
  `dist-release/FoundryPerformance-<версия>-portable.zip` (exe + пустой файл `portable`).
- Скрипт `package-portable.ps1` заменяется на `scripts/dist.ps1`.

## 8. Автообновление через GitHub Releases (утверждено 2026-09-26)

- Репозиторий: `TacticalOtaku/FoundryPerformance` (публичный). Лента:
  `https://github.com/TacticalOtaku/FoundryPerformance/releases/latest/download/latest.json`
  = `{ version, notes, url, sha256, signature }`.
- Подпись — `tauri signer` (формат minisign, ed25519): `signature` = base64 текста `.sig`,
  подписанный комментарий содержит `version:<версия>` (`--app-version`). Публичный ключ —
  `src-tauri/update.pub` (вшит в exe); закрытый ключ и пароль — только в секретах GitHub
  (`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`) и у владельца.
- Проверки перед заменой (любая неудача — ничего не заменяется, ключ `update.err.*` на ЖК):
  `url` начинается с `https://github.com/TacticalOtaku/FoundryPerformance/releases/download/`;
  SHA-256 файла = `sha256`; подпись верна для вшитого ключа; `version:` в подписанном
  комментарии = `version` ленты; версия ленты новее текущей.
- Поведение: лаунчер (release-сборка) при старте тихо читает ленту → под ЖК кнопка
  «ОБНОВЛЕНИЕ X.Y.Z ▶» (подсказка — `notes`) → по нажатию загрузка с процентами на ЖК →
  проверки → `selfreplace` → запуск новой версии и выход. Никаких фоновых установок.
- Новая версия при первом старте синхронизирует `install.json` и `DisplayVersion` в реестре.
- CI: `.github/workflows/release.yml` на тег `v*` (windows-latest): проверка тега = версия,
  тесты, `npm run dist`, подпись exe, `latest.json`, публикация релиза (exe, portable-zip,
  latest.json). Версия поднимается `npm run version:set X.Y.Z` (package.json, Cargo.toml,
  tauri.conf.json).

## 9. Тестирование

- Rust unit: сравнение версий; определение режима (по аргументам и файлам в tempdir);
  проверка папки (корень диска, системная, недоступная); `selfreplace` на tempdir
  (новая установка, замена, очистка `.old`, откат при ошибке); сборка значений реестра
  (чистая функция → список пар ключ/значение); команда самоудаления (строка, только наши файлы).
- Ручная: установка в папку по умолчанию и в свою, повторный запуск установщика (current),
  «обновление» с подменённой версией, удаление с данными и без, проверка «Приложений Windows».
