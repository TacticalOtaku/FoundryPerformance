# Проверка GPU-ускорения (дизайн)

> **Заменено** `2026-10-07-intent-redesign-design.md` (дизайн «Намерение», значок видеокарты в духе FLC). Документ оставлен для истории.

Дата: 2026-10-06 · Статус: утверждён в обсуждении, ждёт вычитки спеки

Подпроект 3 из задачи «зависимости → GPU → Tactile → установщик». Визуальное оформление
значка делает подпроект 2 (Tactile); здесь — данные, логика и минимальная честная индикация
в текущем интерфейсе.

## 1. Проблема

Лаунчер показывает, какая видеокарта **стоит** в системе (DXGI, адаптер с наибольшей VRAM),
но не знает, на чём игра **рисует**. Если WebView2 молча ушёл в программный рендер
(SwiftShader, Microsoft Basic Render Driver) или на ноутбуке с двумя GPU взял встроенную,
FPS падает в разы, а паспорт по-прежнему показывает «RTX 5070».

Уже есть и не меняется: флаги `--force-high-performance-gpu`, `--ignore-gpu-blocklist`,
`--enable-gpu-rasterization`, `--enable-zero-copy`; выбор бэкенда ANGLE; откат ANGLE при
раннем `webglLost` (не чаще раза за сессию).

**Критерий успеха:** после первого запуска игры лаунчер показывает реальный статус
ускорения; при программном рендере следующий запуск идёт на другом бэкенде ANGLE без
действий игрока; при встроенной видеокарте вместо дискретной игрок видит, что делать.

**Не входит:** живые метрики (загрузка GPU, занятая VRAM), проба WebGL до запуска игры,
запись настроек графики Windows (`UserGpuPreferences`), финальный вид значка.

## 2. Вердикты

`gpu::classify(renderer: &str, adapters: &[GpuInfo]) -> Verdict`, чистая функция.

| Вердикт | Условие | Реакция |
|---|---|---|
| `Hardware { backend }` | нет программных маркеров; производитель рендерера совпадает с производителем лучшего адаптера (или лучший адаптер неизвестен) | ничего, значок «ВКЛ» |
| `Software` | строка содержит `SwiftShader`, `Basic Render Driver` или `llvmpipe` (без учёта регистра) | откат ANGLE на `next()` к следующему запуску, уведомление `notice.softwareRender`, тост в игре |
| `WrongGpu { backend }` | производитель рендерера — один из адаптеров, но не лучший (по VRAM) | тост в игре + подсказка в лаунчере; настройки не меняются |
| `Unknown` | пустая строка, нет `ANGLE (`, производитель не распознан | ничего, значок нейтральный |

В JSON вердикт — `#[serde(tag = "kind", rename_all = "camelCase")]`:
`{"kind":"hardware","backend":"d3d11"}`, `{"kind":"software"}`,
`{"kind":"wrongGpu","backend":"d3d11"}`, `{"kind":"unknown"}`; `backend` может быть `null`.

Разбор строки ANGLE:
- Производитель — первое поле после `ANGLE (` до запятой: `NVIDIA`/`NVIDIA Corporation` → 0x10DE,
  `AMD`/`ATI Technologies Inc.` → 0x1002, `Intel` → 0x8086. `Google`, `Microsoft` → программные
  (попадают в `Software` по маркерам).
- Бэкенд — по подстроке: `Direct3D11`/`D3D11` → d3d11, `D3D11on12` → d3d11on12, `OpenGL` → gl,
  `Vulkan` → vulkan; иначе `None`.
- Device id не сверяется: в варианте OpenGL его нет.

Единственная видеокарта Intel — это `Hardware`, не `WrongGpu`.

Примеры строк для тестов:
- `ANGLE (NVIDIA, NVIDIA GeForce GTX 1060 6GB (0x00001C03) Direct3D11 vs_5_0 ps_5_0, D3D11)`
- `ANGLE (Intel, Intel(R) UHD Graphics 620 (0x00005917) Direct3D11 vs_5_0 ps_5_0, D3D11)`
- `ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver)`
- `ANGLE (Microsoft, Microsoft Basic Render Driver (0x0000008C) Direct3D11 vs_5_0 ps_5_0, D3D11)`
- `ANGLE (NVIDIA Corporation, NVIDIA GeForce GTX 1060 6GB/PCIe/SSE2, OpenGL 4.5.0)`
- `ANGLE (NVIDIA, Vulkan 1.3.277 (NVIDIA GeForce RTX 5070 (0x00002F04)), NVIDIA)`

## 3. Поток данных

### Агент (`agent/src`)
- Новый `gpu.ts`: `rendererName(renderer)` — чтение `UNMASKED_RENDERER_WEBGL` через
  `WEBGL_debug_renderer_info`; переносится из `diag.ts`, F10 использует его же.
- `bridge.ts`: рядом с `send` появляется `ask<T>(report): Promise<T | null>` — ждёт ответ
  `report_telemetry`, при отказе IPC возвращает `null` и обновляет `ipcStatus` как `send`.
- `main.ts`: отчёт `foundryUrl` отправляется через `ask`, и проверка GPU ждёт его ответа —
  на хостингах страница игры становится доверенной только после этого отчёта. Проверка
  запускается в `ready` после `attachTicker()` и, если канваса тогда не было, в следующем
  `canvasReady` (только после `ready`: в Foundry `canvasReady` срабатывает раньше `ready`).
  Один раз за загрузку мира: если имя рендерера получено — `ask({kind: "gpu", renderer})`;
  при ответе `software` или `wrongGpu` — `ui.notifications.warn(текст, {permanent: true})`.
  Работает и в базовом замере.
- i18n агента: `gpuSoftware` («Игра рисует без видеокарты. Движок уже переключён —
  перезапусти игру»), `gpuWrong` («Игра рисует на встроенной видеокарте. Выбери дискретную видеокарту
  основной в панели NVIDIA или AMD»), RU и EN. Настройка графики Windows для нашего exe не
  годится: рисует GPU-процесс `msedgewebview2.exe` из папки с номером версии WebView2.

### Rust (`src-tauri/src`)
- `gpu.rs`: `detect()` → `detect_all() -> Vec<GpuInfo>` (все аппаратные адаптеры);
  `best(&[GpuInfo]) -> Option<&GpuInfo>` — наибольшая VRAM. `AppState.gpu` заменяется на
  `AppState.adapters: Vec<GpuInfo>`; паспорт и `recommend` берут `best`.
  Плюс `Verdict`, `classify`, разбор строки (раздел 2).
- `telemetry.rs`: `Report::Gpu { renderer: String }`; `validate` отклоняет `renderer`
  длиннее 512 символов. Общий `apply` его не трогает; отдельная чистая функция
  `apply_gpu(renderer, adapters, persist, fallback_allowed, settings) -> (Verdict, Effects)`:
  - `persist` (запуск `LaunchMode::Normal`) — сохраняет `settings.gpu_check`, при `Software` и
    `fallback_allowed` делает откат ANGLE и уведомление; `Effects.fallback = true`, и команда
    выставляет тот же флаг `session_fallback_done`, что и ранний `webglLost`;
  - без `persist` (`LaunchMode::Baseline`, стоковые флаги) только возвращает вердикт.
- `state.rs`: `Data.current_mode: LaunchMode` — выставляется в `start_game` рядом с
  `current_server`. (`LaunchMode::Safe` агента не имеет и отчётов не шлёт.)
- `commands.rs`: `report_telemetry` возвращает `Result<Option<Verdict>, String>`; для всех
  отчётов, кроме `gpu`, — `None`. Проверки происхождения страницы не меняются.

### Хранение
`Settings.gpu_check: Option<GpuCheck>` с `#[serde(default)]` — старые файлы читаются без миграции.

```rust
pub struct GpuCheck {
    pub verdict: Verdict,
    pub renderer: String,
    pub angle: AngleBackend, // выбранный в настройках на момент проверки
    pub adapter: String,     // имя лучшего адаптера на момент проверки
}
```

`GpuCheck::is_current(&self, angle, adapter) -> bool` — проверка устарела, если с тех пор
сменили бэкенд (вручную или откатом) или видеокарту. Устаревшая показывается как «не проверено».

### Лаунчер (временно, до Tactile)
- `gpuCheck` приходит в составе `settings`; `StateDto` получает `gpuCheckCurrent: boolean`.
- `save_settings` сохраняет `gpu_check` с сервера, а не из присланных настроек: копия
  настроек в интерфейсе может быть старше последней проверки.
- Паспорт (`MainScreen.svelte`): строка `API` показывает реальный бэкенд из актуальной проверки,
  иначе выбранный в настройках; новая строка «УСКОРЕНИЕ» с лампочкой: «ВКЛ» / «ПРОГРАММНОЕ» /
  «ВСТРОЙКА» / «НЕ ПРОВЕРЕНО» (`Unknown` и устаревшая — «НЕ ПРОВЕРЕНО»). Подсказка через `tip`
  объясняет состояние и действие. Ключи i18n RU/EN.
- `notice.softwareRender`: «Игра рисовала без видеокарты. Движок переключён на {to} — запусти
  игру снова. Если не поможет — обнови драйвер видеокарты.»

## 4. Ошибки и крайние случаи
- Нет контекста WebGL или расширения — агент не шлёт `gpu`; отсутствие WebGL ловит ранний `webglLost`.
- IPC отказал — `ask` вернул `null`, тоста нет, причина видна в F10 (`ipcStatus`).
- Программный рендер на всех бэкендах — откат идёт по кругу `d3d11 → d3d11on12 → gl → vulkan`
  по шагу за запуск, как у `webglLost`; текст уведомления советует обновить драйвер.
- `webglLost` и `Software` в одной сессии — откат один.
- Отчёт `gpu` с чужой страницы — отклоняется существующей проверкой `page_trusted`.

## 5. Тесты
- Rust `gpu`: `classify` по таблице строк из раздела 2 на конфигурациях: одна NVIDIA; NVIDIA + Intel
  (рисует Intel → `WrongGpu`, рисует NVIDIA → `Hardware`); одна Intel → `Hardware`; пустой список
  адаптеров; программные строки; пустая строка и мусор → `Unknown`. Разбор бэкенда.
  `best` выбирает наибольшую VRAM.
- Rust `telemetry`: JSON `{"kind":"gpu","renderer":"…"}` парсится; > 512 символов отклоняется;
  `Software` в `Normal` → `angle.next()`, уведомление, `gpu_check` сохранён; в `Baseline` —
  вердикт есть, настройки не тронуты; `GpuCheck::is_current`; старые настройки без `gpu_check` читаются.
- Rust `commands`: второй сигнал за сессию (`webglLost` затем `Software`) отката не даёт.
- TS (vitest): `rendererName` на поддельном `gl` и без расширения; тост только для `software` /
  `wrongGpu`; `ask` при отказе `invoke` возвращает `null`.
- Svelte: паспорт в превью на моках (`src/lib/mock.ts`) — все четыре состояния и устаревшая проверка.
- Вручную (пользователь, на своём сервере): обычный запуск → «ВКЛ» и реальный бэкенд;
  `--use-angle=swiftshader --enable-unsafe-swiftshader` в доп. флагах → тост, уведомление,
  бэкенд в настройках переключён; убрать флаги и запустить снова → «ВКЛ».
  (`--disable-gpu` не годится: в свежем Chromium WebGL без GPU просто пропадает, и срабатывает
  ранний `webglLost`, а не программный рендер.)
