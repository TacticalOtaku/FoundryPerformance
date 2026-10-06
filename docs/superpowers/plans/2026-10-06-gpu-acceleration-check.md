# Проверка GPU-ускорения — план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** лаунчер узнаёт от агента, на чём игра рисует на самом деле, откатывает ANGLE при программном рендере, предупреждает о встроенной видеокарте и показывает статус ускорения в паспорте.

**Architecture:** агент читает `UNMASKED_RENDERER_WEBGL` и шлёт `{kind:"gpu", renderer}` через `report_telemetry`, дожидаясь ответа. Rust классифицирует строку против адаптеров DXGI (`gpu::classify`), сохраняет `Settings.gpu_check` и возвращает вердикт; агент показывает тост Foundry. Паспорт лаунчера показывает реальный бэкенд и лампочку статуса.

**Tech Stack:** Rust (Tauri 2.12, `windows` 0.62 DXGI, serde), TypeScript (агент, vitest + happy-dom), Svelte 5.

**Spec:** `docs/superpowers/specs/2026-10-06-gpu-acceleration-check-design.md`

## Global Constraints

- RU — основной язык, EN — равноправный: каждый новый ключ i18n добавляется в оба файла (`src/lib/i18n/{ru,en}.json`, `agent/src/i18n.ts`).
- Вид значка временный: подпроект 2 (Tactile) его перерисует. Не тратить время на оформление сверх существующих компонентов (`Led`, `.passport`).
- Защита IPC не ослабляется: отчёт `gpu` проходит те же `is_game` / `validate` / `page_trusted`.
- Реальное приложение запускать только на изолированных `APPDATA` и `LOCALAPPDATA`; процессы закрывать только свои и по PID; на сервер пользователя не логиниться.
- Bash в этом окружении без Python; для многострочных правок — Edit или node-скрипты в scratchpad.
- Коммиты на английском, в конце строка `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Релиз и push — только с явного разрешения пользователя.
- Проверки: `npm run check`, `npm test` (из корня `pult`), `cargo test` и `cargo clippy --all-targets` (из `pult/src-tauri`).

---

### Task 1: Типы вердикта и проверки в `Settings`

**Files:**
- Modify: `src-tauri/src/model.rs` (после `impl AngleBackend`; `struct Settings` и `impl Default for Settings`; модуль `tests`)
- Modify: `src-tauri/src/commands.rs` (`save_settings`, модуль `tests`)

**Interfaces:**
- Produces: `model::Verdict` (`Hardware { backend: Option<AngleBackend> }`, `Software`, `WrongGpu { backend: Option<AngleBackend> }`, `Unknown`), `model::GpuCheck { verdict, renderer: String, angle: AngleBackend, adapter: String }`, `GpuCheck::is_current(&self, angle: AngleBackend, adapter: &str) -> bool`, поле `Settings.gpu_check: Option<GpuCheck>`, `commands::merge_settings(stored: &Settings, incoming: Settings) -> Settings`.

- [ ] **Step 1: Написать падающие тесты в `model.rs`** (в конец `mod tests`)

```rust
    #[test]
    fn verdict_json_shape() {
        let hw = Verdict::Hardware { backend: Some(AngleBackend::D3d11) };
        assert_eq!(serde_json::to_string(&hw).unwrap(), r#"{"kind":"hardware","backend":"d3d11"}"#);
        assert_eq!(serde_json::to_string(&Verdict::WrongGpu { backend: None }).unwrap(), r#"{"kind":"wrongGpu","backend":null}"#);
        assert_eq!(serde_json::to_string(&Verdict::Software).unwrap(), r#"{"kind":"software"}"#);
        assert_eq!(serde_json::to_string(&Verdict::Unknown).unwrap(), r#"{"kind":"unknown"}"#);
    }

    #[test]
    fn settings_without_gpu_check_still_load() {
        let s: Settings = serde_json::from_str(r#"{"schema":1,"profile":"potato"}"#).unwrap();
        assert_eq!(s.gpu_check, None);
        assert_eq!(s.profile, ProfileId::Potato);
    }

    #[test]
    fn gpu_check_is_stale_after_backend_or_gpu_change() {
        let c = GpuCheck { verdict: Verdict::Software, renderer: "r".into(), angle: AngleBackend::D3d11, adapter: "RTX".into() };
        assert!(c.is_current(AngleBackend::D3d11, "RTX"));
        assert!(!c.is_current(AngleBackend::Gl, "RTX"));
        assert!(!c.is_current(AngleBackend::D3d11, "GTX"));
    }
```

- [ ] **Step 2: Написать падающий тест в `commands.rs`** (в конец `mod tests`)

```rust
    #[test]
    fn saving_settings_keeps_the_launchers_gpu_check() {
        let check = GpuCheck { verdict: Verdict::Software, renderer: "r".into(), angle: AngleBackend::D3d11, adapter: "RTX".into() };
        let stored = Settings { gpu_check: Some(check.clone()), ..Settings::default() };
        // интерфейс прислал старую копию без проверки и с другим профилем
        let incoming = Settings { profile: ProfileId::Potato, schema: 0, gpu_check: None, ..Settings::default() };
        let merged = merge_settings(&stored, incoming);
        assert_eq!(merged.gpu_check, Some(check));
        assert_eq!(merged.profile, ProfileId::Potato);
        assert_eq!(merged.schema, SCHEMA);
    }
```

- [ ] **Step 3: Убедиться, что не компилируется**

Run: `cd src-tauri && cargo test --lib model:: commands::`
Expected: FAIL — `cannot find type Verdict`, `GpuCheck`, `merge_settings`.

- [ ] **Step 4: Реализация в `model.rs`** — сразу после `impl AngleBackend { … }`:

```rust
/// На чём игра рисует на самом деле — по строке рендерера WebGL (см. `gpu::classify`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Verdict {
    Hardware { backend: Option<AngleBackend> },
    Software,
    WrongGpu { backend: Option<AngleBackend> },
    Unknown,
}

/// Последняя проверка ускорения в обычном запуске.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuCheck {
    pub verdict: Verdict,
    pub renderer: String,
    /// Бэкенд ANGLE из настроек на момент проверки.
    pub angle: AngleBackend,
    /// Имя лучшего адаптера на момент проверки (пусто, если DXGI ничего не нашёл).
    pub adapter: String,
}

impl GpuCheck {
    /// Проверка относится к текущей конфигурации: тот же бэкенд и та же видеокарта.
    pub fn is_current(&self, angle: AngleBackend, adapter: &str) -> bool {
        self.angle == angle && self.adapter == adapter
    }
}
```

В `struct Settings` последним полем:

```rust
    /// Пишет только лаунчер (см. `commands::merge_settings`).
    pub gpu_check: Option<GpuCheck>,
```

В `impl Default for Settings` последним полем: `gpu_check: None,`.

- [ ] **Step 5: Реализация в `commands.rs`** — перед `#[tauri::command] pub fn save_settings`:

```rust
/// Вердикт проверки ускорения пишет только лаунчер: копия настроек в интерфейсе может быть старше.
pub fn merge_settings(stored: &Settings, incoming: Settings) -> Settings {
    Settings { schema: SCHEMA, gpu_check: stored.gpu_check.clone(), ..incoming }
}
```

И в `save_settings` заменить `d.settings = Settings { schema: SCHEMA, ..settings };` на:

```rust
    d.settings = merge_settings(&d.settings, settings);
```

- [ ] **Step 6: Прогнать тесты**

Run: `cd src-tauri && cargo test`
Expected: PASS все, включая 4 новых. Если упал тест `store.rs`, сравнивающий JSON настроек целиком, — дописать в ожидаемый JSON `"gpuCheck":null` (это ожидаемое следствие нового поля).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/model.rs src-tauri/src/commands.rs
git commit -m "feat: GPU check verdict stored in settings, kept on settings save"
```

---

### Task 2: Все адаптеры DXGI и классификация рендерера

**Files:**
- Modify: `src-tauri/src/gpu.rs` (весь файл, кроме `recommend`)
- Modify: `src-tauri/src/state.rs` (`AppState.gpu` → `adapters`)
- Modify: `src-tauri/src/lib.rs:53-60` (инициализация)
- Modify: `src-tauri/src/commands.rs` (`get_state`: поля `gpu`, `recommended`)

**Interfaces:**
- Consumes: `model::{Verdict, AngleBackend}` (Task 1).
- Produces: `gpu::detect_all() -> Vec<GpuInfo>`, `gpu::best(&[GpuInfo]) -> Option<&GpuInfo>`, `gpu::classify(renderer: &str, adapters: &[GpuInfo]) -> Verdict`, `AppState.adapters: Vec<GpuInfo>`. `gpu::detect()` удаляется.

- [ ] **Step 1: Написать падающие тесты** — в `mod tests` файла `gpu.rs` (хелпер `g(vram_mb, vendor_id)` уже есть):

```rust
    const NV_D3D11: &str = "ANGLE (NVIDIA, NVIDIA GeForce GTX 1060 6GB (0x00001C03) Direct3D11 vs_5_0 ps_5_0, D3D11)";
    const NV_ON12: &str = "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 (0x00002F04) Direct3D11on12 vs_5_0 ps_5_0, D3D11on12)";
    const NV_GL: &str = "ANGLE (NVIDIA Corporation, NVIDIA GeForce GTX 1060 6GB/PCIe/SSE2, OpenGL 4.5.0)";
    const NV_VULKAN: &str = "ANGLE (NVIDIA, Vulkan 1.3.277 (NVIDIA GeForce RTX 5070 (0x00002F04)), NVIDIA)";
    const AMD_GL: &str = "ANGLE (ATI Technologies Inc., AMD Radeon RX 580 Series, OpenGL 4.5.0)";
    const INTEL_D3D11: &str = "ANGLE (Intel, Intel(R) UHD Graphics 620 (0x00005917) Direct3D11 vs_5_0 ps_5_0, D3D11)";
    const SWIFTSHADER: &str = "ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver)";
    const BASIC: &str = "ANGLE (Microsoft, Microsoft Basic Render Driver (0x0000008C) Direct3D11 vs_5_0 ps_5_0, D3D11)";

    fn nv() -> GpuInfo {
        g(6144, 0x10DE)
    }
    fn intel() -> GpuInfo {
        g(128, 0x8086)
    }
    fn hw(b: AngleBackend) -> Verdict {
        Verdict::Hardware { backend: Some(b) }
    }

    #[test]
    fn best_prefers_vram_then_the_first() {
        let list = [intel(), nv(), g(6144, 0x1002)];
        assert_eq!(best(&list).unwrap().vendor_id, 0x10DE);
        assert_eq!(best(&[]), None);
    }

    #[test]
    fn hardware_reports_the_real_backend() {
        assert_eq!(classify(NV_D3D11, &[nv()]), hw(AngleBackend::D3d11));
        assert_eq!(classify(NV_ON12, &[nv()]), hw(AngleBackend::D3d11on12));
        assert_eq!(classify(NV_GL, &[nv()]), hw(AngleBackend::Gl));
        assert_eq!(classify(NV_VULKAN, &[nv()]), hw(AngleBackend::Vulkan));
        assert_eq!(classify(AMD_GL, &[g(8192, 0x1002)]), hw(AngleBackend::Gl));
    }

    #[test]
    fn integrated_gpu_on_a_hybrid_laptop_is_the_wrong_gpu() {
        let hybrid = [intel(), nv()];
        assert_eq!(classify(INTEL_D3D11, &hybrid), Verdict::WrongGpu { backend: Some(AngleBackend::D3d11) });
        assert_eq!(classify(NV_D3D11, &hybrid), hw(AngleBackend::D3d11));
    }

    #[test]
    fn a_lone_intel_or_unknown_adapters_are_hardware() {
        assert_eq!(classify(INTEL_D3D11, &[intel()]), hw(AngleBackend::D3d11));
        assert_eq!(classify(INTEL_D3D11, &[]), hw(AngleBackend::D3d11));
    }

    #[test]
    fn software_renderers_are_detected() {
        assert_eq!(classify(SWIFTSHADER, &[nv()]), Verdict::Software);
        assert_eq!(classify(BASIC, &[nv()]), Verdict::Software);
    }

    #[test]
    fn unrecognized_strings_are_unknown() {
        for s in ["", "garbage", "ANGLE (Qualcomm, Adreno 690)", "WebKit WebGL"] {
            assert_eq!(classify(s, &[nv()]), Verdict::Unknown, "{s}");
        }
    }
```

И поменять тело ignored-теста `print_detected_gpu` на `println!("{:?}", detect_all());`.

- [ ] **Step 2: Убедиться, что падает**

Run: `cd src-tauri && cargo test --lib gpu::`
Expected: FAIL — `cannot find function best`, `classify`, `detect_all`.

- [ ] **Step 3: Реализация в `gpu.rs`**

Импорт: `use crate::model::{AngleBackend, ProfileId, Verdict};`

Константы (заменить блок с `VENDOR_INTEL` / `VENDOR_MICROSOFT_BASIC`):

```rust
const VENDOR_NVIDIA: u32 = 0x10DE;
const VENDOR_AMD: u32 = 0x1002;
const VENDOR_INTEL: u32 = 0x8086;
const VENDOR_MICROSOFT_BASIC: u32 = 0x1414;
/// Признаки программного рендера в строке WebGL (в нижнем регистре).
const SOFTWARE_MARKERS: [&str; 3] = ["swiftshader", "basic render driver", "llvmpipe"];
```

`detect()` заменить на:

```rust
/// Все аппаратные адаптеры, без программных.
pub fn detect_all() -> Vec<GpuInfo> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE};

    let mut out = Vec::new();
    unsafe {
        let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() else { return out };
        let mut i = 0u32;
        while let Ok(adapter) = factory.EnumAdapters1(i) {
            i += 1;
            let Ok(desc) = adapter.GetDesc1() else { continue };
            if desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 || desc.VendorId == VENDOR_MICROSOFT_BASIC {
                continue;
            }
            let len = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
            out.push(GpuInfo {
                name: String::from_utf16_lossy(&desc.Description[..len]),
                vram_mb: (desc.DedicatedVideoMemory as u64) / (1024 * 1024),
                vendor_id: desc.VendorId,
            });
        }
    }
    out
}

/// Адаптер с наибольшей выделенной VRAM; при равенстве — первый.
pub fn best(adapters: &[GpuInfo]) -> Option<&GpuInfo> {
    adapters.iter().fold(None, |b: Option<&GpuInfo>, a| if b.is_none_or(|b| a.vram_mb > b.vram_mb) { Some(a) } else { b })
}

/// Производитель из первого поля строки ANGLE: `ANGLE (NVIDIA, …)`.
fn renderer_vendor(renderer: &str) -> Option<u32> {
    let field = renderer.strip_prefix("ANGLE (")?.split(',').next()?.trim().to_ascii_lowercase();
    if field.starts_with("nvidia") {
        Some(VENDOR_NVIDIA)
    } else if field.starts_with("amd") || field.starts_with("ati ") || field.starts_with("advanced micro devices") {
        Some(VENDOR_AMD)
    } else if field.starts_with("intel") {
        Some(VENDOR_INTEL)
    } else {
        None
    }
}

/// Бэкенд, на котором ANGLE рисует на самом деле; `lower` — строка в нижнем регистре.
fn renderer_backend(lower: &str) -> Option<AngleBackend> {
    if lower.contains("on12") {
        Some(AngleBackend::D3d11on12)
    } else if lower.contains("direct3d11") || lower.contains("d3d11") {
        Some(AngleBackend::D3d11)
    } else if lower.contains("vulkan") {
        Some(AngleBackend::Vulkan)
    } else if lower.contains("opengl") {
        Some(AngleBackend::Gl)
    } else {
        None
    }
}

/// На чём игра рисует: по строке `UNMASKED_RENDERER_WEBGL` и адаптерам DXGI.
pub fn classify(renderer: &str, adapters: &[GpuInfo]) -> Verdict {
    let lower = renderer.to_ascii_lowercase();
    if SOFTWARE_MARKERS.iter().any(|m| lower.contains(m)) {
        return Verdict::Software;
    }
    let Some(vendor) = renderer_vendor(renderer) else { return Verdict::Unknown };
    let backend = renderer_backend(&lower);
    // Встройка при живой дискретной: производитель есть среди адаптеров, но это не лучший
    let wrong = best(adapters).is_some_and(|b| b.vendor_id != vendor) && adapters.iter().any(|a| a.vendor_id == vendor);
    if wrong {
        Verdict::WrongGpu { backend }
    } else {
        Verdict::Hardware { backend }
    }
}
```

- [ ] **Step 4: Перевести потребителей на список адаптеров**

`state.rs` — в `AppState` заменить `pub gpu: Option<GpuInfo>,` на:

```rust
    /// Аппаратные адаптеры DXGI; паспорт и рекомендация берут `gpu::best`.
    pub adapters: Vec<GpuInfo>,
```

`lib.rs` — заменить

```rust
            let gpu = gpu::detect();
            let (settings, n1) = store.load_settings(gpu::recommend(gpu.as_ref()));
```

на

```rust
            let adapters = gpu::detect_all();
            let (settings, n1) = store.load_settings(gpu::recommend(gpu::best(&adapters)));
```

и в `AppState { … }` строку `gpu,` на `adapters,`.

`commands.rs` — в `get_state`:

```rust
        gpu: gpu::best(&state.adapters).cloned(),
        recommended: gpu::recommend(gpu::best(&state.adapters)),
```

Run: `cd src-tauri && grep -rn "state.gpu\|gpu::detect()" src` — Expected: пусто.

- [ ] **Step 5: Прогнать тесты и clippy**

Run: `cd src-tauri && cargo test && cargo clippy --all-targets`
Expected: PASS, clippy без предупреждений. Дополнительно: `cargo test print_detected_gpu -- --ignored --nocapture` печатает список с реальной видеокартой.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/gpu.rs src-tauri/src/state.rs src-tauri/src/lib.rs src-tauri/src/commands.rs
git commit -m "feat: list all DXGI adapters and classify the WebGL renderer"
```

---

### Task 3: Отчёт `gpu` в телеметрии

**Files:**
- Modify: `src-tauri/src/telemetry.rs` (`Report`, `Effects`, `validate`, `apply`, новая `apply_gpu`, тесты)

**Interfaces:**
- Consumes: `gpu::{classify, best, GpuInfo}` (Task 2), `model::{Verdict, GpuCheck}` (Task 1).
- Produces: `Report::Gpu { renderer: String }`, `Effects.fallback: bool`, `telemetry::apply_gpu(renderer: &str, adapters: &[GpuInfo], persist: bool, fallback_allowed: bool, settings: &mut Settings) -> (Verdict, Effects)`.

- [ ] **Step 1: Написать падающие тесты** (в `mod tests` файла `telemetry.rs`)

```rust
    use crate::gpu::GpuInfo;

    const NV: &str = "ANGLE (NVIDIA, NVIDIA GeForce GTX 1060 6GB (0x00001C03) Direct3D11 vs_5_0 ps_5_0, D3D11)";
    const SWIFT: &str = "ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver)";

    fn nv() -> GpuInfo {
        GpuInfo { name: "NVIDIA GeForce GTX 1060 6GB".into(), vram_mb: 6144, vendor_id: 0x10DE }
    }

    #[test]
    fn parses_gpu_report_and_limits_its_length() {
        let r: Report = serde_json::from_str(r#"{"kind":"gpu","renderer":"ANGLE (NVIDIA, x)"}"#).unwrap();
        assert_eq!(r, Report::Gpu { renderer: "ANGLE (NVIDIA, x)".into() });
        assert!(validate(&r));
        assert!(!validate(&Report::Gpu { renderer: "x".repeat(513) }));
    }

    #[test]
    fn hardware_check_is_remembered() {
        let mut settings = Settings::default();
        let (v, fx) = apply_gpu(NV, &[nv()], true, true, &mut settings);
        assert_eq!(v, Verdict::Hardware { backend: Some(AngleBackend::D3d11) });
        assert!(fx.settings && !fx.fallback && fx.notice.is_none());
        let c = settings.gpu_check.unwrap();
        assert_eq!((c.renderer.as_str(), c.angle, c.adapter.as_str()), (NV, AngleBackend::D3d11, "NVIDIA GeForce GTX 1060 6GB"));
    }

    #[test]
    fn software_render_switches_the_backend_once_per_session() {
        let mut settings = Settings::default();
        let (v, fx) = apply_gpu(SWIFT, &[nv()], true, true, &mut settings);
        assert_eq!(v, Verdict::Software);
        assert!(fx.fallback && fx.settings);
        assert_eq!(settings.engine.angle, AngleBackend::D3d11on12);
        let n = fx.notice.unwrap();
        assert_eq!((n.key.as_str(), n.params["to"].as_str()), ("notice.softwareRender", "d3d11on12"));
        // проверка записана с бэкендом, на котором её сделали: после отката она уже не текущая
        assert_eq!(settings.gpu_check.as_ref().unwrap().angle, AngleBackend::D3d11);

        let (_, fx) = apply_gpu(SWIFT, &[nv()], true, false, &mut settings);
        assert!(!fx.fallback && fx.notice.is_none());
        assert_eq!(settings.engine.angle, AngleBackend::D3d11on12);
    }

    #[test]
    fn baseline_run_only_answers() {
        let mut settings = Settings::default();
        let (v, fx) = apply_gpu(SWIFT, &[nv()], false, true, &mut settings);
        assert_eq!(v, Verdict::Software);
        assert_eq!(fx, Effects::default());
        assert_eq!(settings, Settings::default());
    }
```

- [ ] **Step 2: Убедиться, что падает**

Run: `cd src-tauri && cargo test --lib telemetry::`
Expected: FAIL — нет `Report::Gpu`, `apply_gpu`, `Effects.fallback`.

- [ ] **Step 3: Реализация**

Импорт в начале файла: `use crate::gpu::{self, GpuInfo};`

В `enum Report` последним вариантом:

```rust
    /// Строка `UNMASKED_RENDERER_WEBGL` игрового канваса — на чём игра рисует на самом деле.
    Gpu { renderer: String },
```

В `struct Effects` последним полем:

```rust
    /// Сделан откат ANGLE — второй за сессию не нужен.
    pub fallback: bool,
```

В `validate` новая ветка:

```rust
        Report::Gpu { renderer } => renderer.len() <= 512,
```

В `apply` (в `match *report`) новая ветка перед `Report::FoundryUrl`:

```rust
        // Нужны адаптеры и режим запуска — отдельный путь `apply_gpu`
        Report::Gpu { .. } => {}
```

После `fn apply`:

```rust
/// Отчёт о рендерере игры. `persist` — обычный запуск (наши флаги), а не базовый замер;
/// `fallback_allowed` — откат ANGLE в этой сессии ещё не делали.
pub fn apply_gpu(renderer: &str, adapters: &[GpuInfo], persist: bool, fallback_allowed: bool, settings: &mut Settings) -> (Verdict, Effects) {
    let verdict = gpu::classify(renderer, adapters);
    let mut fx = Effects::default();
    if !persist {
        return (verdict, fx);
    }
    settings.gpu_check = Some(GpuCheck {
        verdict: verdict.clone(),
        renderer: renderer.to_string(),
        angle: settings.engine.angle,
        adapter: gpu::best(adapters).map(|a| a.name.clone()).unwrap_or_default(),
    });
    fx.settings = true;
    if verdict == Verdict::Software && fallback_allowed {
        let from = settings.engine.angle;
        settings.engine.angle = from.next();
        fx.fallback = true;
        fx.notice = Some(
            Notice::new("notice.softwareRender")
                .with("from", from.flag_value())
                .with("to", settings.engine.angle.flag_value()),
        );
    }
    (verdict, fx)
}
```

- [ ] **Step 4: Прогнать тесты**

Run: `cd src-tauri && cargo test && cargo clippy --all-targets`
Expected: PASS. Команда `report_telemetry` пока не обрабатывает `Gpu` отдельно — общий `apply` его игнорирует, это корректно до Task 4.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/telemetry.rs
git commit -m "feat: gpu telemetry report with software-render fallback"
```

---

### Task 4: Команда возвращает вердикт, паспорт знает актуальность проверки

**Files:**
- Modify: `src-tauri/src/state.rs` (`Data.current_mode`)
- Modify: `src-tauri/src/lib.rs` (инициализация `Data`)
- Modify: `src-tauri/src/commands.rs` (`start_game`, `report_telemetry`, новые `commit` и `gpu_check_current`, `StateDto`, тест)

**Interfaces:**
- Consumes: `telemetry::apply_gpu`, `Effects.fallback` (Task 3), `GpuCheck::is_current` (Task 1), `gpu::best` (Task 2).
- Produces: команда `report_telemetry` → `Result<Option<Verdict>, String>` (JSON вердикта для `gpu`, `null` для остальных); `StateDto.gpu_check_current: bool` (`gpuCheckCurrent` в JSON); `Data.current_mode: LaunchMode`.

- [ ] **Step 1: Написать падающий тест** (в `mod tests` файла `commands.rs`)

```rust
    #[test]
    fn gpu_check_is_current_only_for_the_same_backend_and_gpu() {
        let rtx = gpu::GpuInfo { name: "RTX".into(), vram_mb: 12288, vendor_id: 0x10DE };
        let check = GpuCheck { verdict: Verdict::Unknown, renderer: "r".into(), angle: AngleBackend::D3d11, adapter: "RTX".into() };
        let s = Settings { gpu_check: Some(check), ..Settings::default() };
        assert!(gpu_check_current(&s, std::slice::from_ref(&rtx)));
        let switched = Settings { engine: EngineSettings { angle: AngleBackend::Gl, ..EngineSettings::default() }, ..s.clone() };
        assert!(!gpu_check_current(&switched, std::slice::from_ref(&rtx)));
        assert!(!gpu_check_current(&Settings::default(), &[rtx]));
    }
```

- [ ] **Step 2: Убедиться, что падает**

Run: `cd src-tauri && cargo test --lib commands::`
Expected: FAIL — нет `gpu_check_current`.

- [ ] **Step 3: Режим запуска в состоянии**

`state.rs`: импорт `use crate::commands::LaunchMode;` и в `struct Data` после `current_server`:

```rust
    /// Режим текущего запуска: проверку GPU сохраняем только для обычного.
    pub current_mode: LaunchMode,
```

`lib.rs`, в `Data { … }` после `current_server: None,`: `current_mode: commands::LaunchMode::Normal,`

`commands.rs`, `start_game`: после `d.current_server = Some(server.id.clone());` добавить `d.current_mode = mode;`.

- [ ] **Step 4: `gpu_check_current`, `StateDto`, `commit`, `report_telemetry`**

`commands.rs`, импорт: `use crate::state::{AppState, Data};` (вместо `use crate::state::AppState;`).

Рядом с `merge_settings`:

```rust
/// Вердикт последней проверки относится к текущим бэкенду и видеокарте.
pub fn gpu_check_current(settings: &Settings, adapters: &[gpu::GpuInfo]) -> bool {
    let adapter = gpu::best(adapters).map_or("", |g| g.name.as_str());
    settings.gpu_check.as_ref().is_some_and(|c| c.is_current(settings.engine.angle, adapter))
}
```

`struct StateDto` — после `gpu`: `gpu_check_current: bool,`; в `get_state` после строки `gpu: …`:

```rust
        gpu_check_current: gpu_check_current(&d.settings, &state.adapters),
```

Перед `report_telemetry`:

```rust
/// Сохраняет то, что изменил отчёт агента.
fn commit(state: &AppState, d: &mut Data, fx: telemetry::Effects) {
    if fx.stats {
        let _ = state.store.save_stats(&d.stats);
    }
    if fx.servers {
        let _ = state.store.save_servers(&d.servers);
    }
    if fx.settings {
        let _ = state.store.save_settings(&d.settings);
    }
    if fx.notice.is_some() {
        d.notice = fx.notice;
    }
}
```

`report_telemetry`: сигнатура → `-> Result<Option<Verdict>, String>`. Проверки происхождения (до `let mut guard`) не менять. Всё начиная с `let mut guard` заменить на:

```rust
    let mut guard = state.data.lock().expect("state poisoned");
    let d = &mut *guard;
    let Some(server_id) = d.current_server.clone() else { return Ok(None) };
    if let telemetry::Report::Gpu { renderer } = &report {
        let persist = d.current_mode == LaunchMode::Normal;
        let (verdict, fx) = telemetry::apply_gpu(renderer, &state.adapters, persist, !d.session_fallback_done, &mut d.settings);
        d.session_fallback_done |= fx.fallback;
        commit(&state, d, fx);
        return Ok(Some(verdict));
    }
    if matches!(report, telemetry::Report::WebglLost { early: true }) {
        if d.session_fallback_done {
            return Ok(None);
        }
        d.session_fallback_done = true;
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|t| t.as_secs()).unwrap_or(0);
    let fx = telemetry::apply(&report, &server_id, now, &mut d.stats, &mut d.servers, &mut d.settings);
    commit(&state, d, fx);
    Ok(None)
```

- [ ] **Step 5: Прогнать тесты и clippy**

Run: `cd src-tauri && cargo test && cargo clippy --all-targets`
Expected: PASS, без предупреждений.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/lib.rs src-tauri/src/commands.rs
git commit -m "feat: report_telemetry answers gpu reports with a verdict"
```

---

### Task 5: Агент читает рендерер, спрашивает лаунчер, показывает тост

**Files:**
- Create: `agent/src/gpu.ts`, `agent/src/gpu.test.ts`
- Modify: `agent/src/types.ts` (тип `Verdict`)
- Modify: `agent/src/bridge.ts` (`Report`, `invoke` с результатом, `ask`), `agent/src/bridge.test.ts`
- Modify: `agent/src/i18n.ts` (`gpuSoftware`, `gpuWrong` в `ru` и `en`)
- Modify: `agent/src/diag.ts:53-60` (использовать `rendererName`)
- Modify: `agent/src/main.ts` (`foundryUrl` через `ask`, `checkGpu`)

**Interfaces:**
- Consumes: ответ `report_telemetry` для `gpu` — JSON `Verdict` из Task 4.
- Produces: `rendererName(renderer: any): string | undefined`, `verdictMessage(v: Verdict | null, t: Strings): string | null`, `ask<T>(report: Report): Promise<T | null>`.

- [ ] **Step 1: Тип вердикта** — в конец `agent/src/types.ts`:

```ts
/** Зеркало Rust `model::Verdict` — ответ лаунчера на отчёт `gpu`. */
export type Verdict =
	| { kind: "hardware"; backend: string | null }
	| { kind: "software" }
	| { kind: "wrongGpu"; backend: string | null }
	| { kind: "unknown" };
```

- [ ] **Step 2: Написать падающие тесты** — `agent/src/gpu.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { rendererName, verdictMessage } from "./gpu";
import { strings } from "./i18n";

const UNMASKED = 0x9246;
const fake = (name: unknown, withExt = true) => ({
	gl: {
		getExtension: (n: string) => (withExt && n === "WEBGL_debug_renderer_info" ? { UNMASKED_RENDERER_WEBGL: UNMASKED } : null),
		getParameter: (p: number) => (p === UNMASKED ? name : undefined)
	}
});

describe("rendererName", () => {
	it("reads the unmasked renderer string", () => {
		expect(rendererName(fake("ANGLE (NVIDIA, x)"))).toBe("ANGLE (NVIDIA, x)");
	});

	it("is undefined without a context, the extension or a string", () => {
		expect(rendererName(undefined)).toBeUndefined();
		expect(rendererName(fake("x", false))).toBeUndefined();
		expect(rendererName(fake(""))).toBeUndefined();
		expect(
			rendererName({
				gl: {
					getExtension: () => {
						throw new Error("context lost");
					}
				}
			})
		).toBeUndefined();
	});
});

describe("verdictMessage", () => {
	const t = strings("ru");

	it("warns only about software render and the wrong GPU", () => {
		expect(verdictMessage({ kind: "software" }, t)).toBe(t.gpuSoftware);
		expect(verdictMessage({ kind: "wrongGpu", backend: "d3d11" }, t)).toBe(t.gpuWrong);
		expect(verdictMessage({ kind: "hardware", backend: "d3d11" }, t)).toBeNull();
		expect(verdictMessage({ kind: "unknown" }, t)).toBeNull();
		expect(verdictMessage(null, t)).toBeNull();
	});
});
```

В `agent/src/bridge.test.ts` — импорт `ask` рядом с `send`, и новый блок:

```ts
describe("ask", () => {
	it("returns the host answer", async () => {
		g.__TAURI_INTERNALS__ = { invoke: vi.fn().mockResolvedValue({ kind: "software" }) };
		await expect(ask({ kind: "gpu", renderer: "x" })).resolves.toEqual({ kind: "software" });
	});

	it("returns null without IPC, on rejection and on a throwing invoke", async () => {
		vi.spyOn(console, "warn").mockImplementation(() => {});
		await expect(ask({ kind: "gpu", renderer: "x" })).resolves.toBeNull();
		g.__TAURI_INTERNALS__ = { invoke: vi.fn().mockRejectedValue("rejected") };
		await expect(ask({ kind: "gpu", renderer: "x" })).resolves.toBeNull();
		g.__TAURI_INTERNALS__ = {
			invoke: vi.fn(() => {
				throw new Error("sync");
			})
		};
		await expect(ask({ kind: "gpu", renderer: "x" })).resolves.toBeNull();
	});
});
```

- [ ] **Step 3: Убедиться, что падает**

Run: `npx vitest run agent/src/gpu.test.ts agent/src/bridge.test.ts`
Expected: FAIL — нет модуля `./gpu`, нет экспорта `ask`.

- [ ] **Step 4: Строки i18n** — в `agent/src/i18n.ts` после `diagSaved` в `ru`:

```ts
	gpuSoftware: "Игра рисует без видеокарты. Движок уже переключён — перезапусти игру",
	gpuWrong: "Игра рисует на встроенной видеокарте. Выбери дискретную видеокарту основной в панели NVIDIA или AMD",
```

и в `en`:

```ts
	gpuSoftware: "The game is rendering without the GPU. The engine has been switched — restart the game",
	gpuWrong: "The game is rendering on the integrated GPU. Make the discrete GPU the preferred one in the NVIDIA or AMD control panel",
```

- [ ] **Step 5: `agent/src/gpu.ts`**

```ts
import type { Strings } from "./i18n";
import type { Verdict } from "./types";

/** Настоящее имя рендерера WebGL; undefined — нет контекста, расширения или строки. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function rendererName(renderer: any): string | undefined {
	try {
		const gl = renderer?.gl;
		const ext = gl?.getExtension("WEBGL_debug_renderer_info");
		const name: unknown = ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : undefined;
		return typeof name === "string" && name ? name : undefined;
	} catch {
		return undefined;
	}
}

/** Тост в игре — только когда с ускорением беда. */
export function verdictMessage(v: Verdict | null, t: Strings): string | null {
	if (v?.kind === "software") return t.gpuSoftware;
	if (v?.kind === "wrongGpu") return t.gpuWrong;
	return null;
}
```

- [ ] **Step 6: `bridge.ts`** — в `Report` добавить вариант `| { kind: "gpu"; renderer: string }`. Функцию `invoke` и `send` заменить на:

```ts
/** Единственный канал наружу. Если Tauri не выдал IPC этому origin — ничего не ломаем, только запоминаем. */
function invoke(cmd: string, args: unknown = {}): Promise<unknown> {
	const internals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ as
		| { invoke?: (cmd: string, args: unknown) => Promise<unknown> }
		| undefined;
	if (typeof internals?.invoke !== "function") {
		status = { state: "missing" };
		if (!warned) {
			warned = true;
			console.warn("[foundry-performance] IPC недоступен: отчёты агента не доходят до лаунчера");
		}
		return Promise.resolve(null);
	}
	const fail = (e: unknown) => {
		status = { state: "rejected", error: String(e) };
		return null;
	};
	try {
		return internals.invoke(cmd, args).then((v) => {
			status = { state: "ok" };
			return v ?? null;
		}, fail);
	} catch (e) {
		return Promise.resolve(fail(e));
	}
}

export function send(report: Report): void {
	void invoke("report_telemetry", { report });
}

/** Отчёт с ответом лаунчера; null — канала нет или отчёт отклонён. */
export function ask<T>(report: Report): Promise<T | null> {
	return invoke("report_telemetry", { report }) as Promise<T | null>;
}
```

`toggleFullscreen` оставить, но вызывать как `void invoke("toggle_fullscreen");`.

- [ ] **Step 7: `diag.ts`** — импорт `import { rendererName } from "./gpu";`; блок

```ts
	let gpu: string | undefined;
	try {
		const gl = r?.gl;
		const ext = gl?.getExtension("WEBGL_debug_renderer_info");
		gpu = ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : undefined;
	} catch {
		gpu = undefined;
	}
```

заменить на `const gpu = rendererName(r);`.

- [ ] **Step 8: `main.ts`**

Импорты: добавить `ask` в импорт из `./bridge`, `import { rendererName, verdictMessage } from "./gpu";`, `Verdict` в импорт типов из `./types`.

В `start()` рядом с `let tickerAttached = false;`:

```ts
	// Проверка GPU ждёт ответа на foundryUrl: до него страница игры на хостинге ещё не доверенная
	let urlAck: Promise<unknown> = Promise.resolve(null);
	let gpuChecked = false;
```

После функции `attachTicker()`:

```ts
	/** Один раз за загрузку мира: на чём игра рисует на самом деле. */
	async function checkGpu(): Promise<void> {
		if (gpuChecked) return;
		const renderer = rendererName(g.canvas?.app?.renderer);
		if (!renderer) return;
		gpuChecked = true;
		const msg = verdictMessage(await ask<Verdict>({ kind: "gpu", renderer }), t);
		if (msg) g.ui?.notifications?.warn(msg, { permanent: true });
	}
```

В обработчике `hooks.once("ready", …)` строку `send({ kind: "foundryUrl", url: location.origin + location.pathname });` заменить на

```ts
				urlAck = ask({ kind: "foundryUrl", url: location.origin + location.pathname });
```

и сразу после `attachTicker();` в том же обработчике добавить `void urlAck.then(checkGpu);`.

В `hooks.on("canvasReady", …)` после `attachTicker();` добавить:

```ts
				// canvasReady срабатывает и до ready — проверяем только после него
				if (readyAt) void urlAck.then(checkGpu);
```

- [ ] **Step 9: Прогнать тесты, проверку типов и сборку агента**

Run: `npm test && npm run check && npm run build:agent`
Expected: все тесты PASS (66 старых + новые), 0 ошибок svelte-check, агент собрался.

- [ ] **Step 10: Commit**

```bash
git add agent/src
git commit -m "feat: agent reports the real WebGL renderer and warns in game"
```

---

### Task 6: Статус ускорения в паспорте лаунчера

**Files:**
- Modify: `src/lib/types.ts` (`Verdict`, `GpuCheck`, `Settings.gpuCheck`, `StateDto.gpuCheckCurrent`)
- Create: `src/lib/accel.ts`, `src/lib/accel.test.ts`
- Modify: `src/screens/MainScreen.svelte` (паспорт, стиль `.accel`)
- Modify: `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`
- Modify: `src/lib/mock.ts` (состояния для превью через `?accel=`)

**Interfaces:**
- Consumes: `settings.gpuCheck` и `gpuCheckCurrent` из `get_state` (Tasks 1, 4).
- Produces: `accelView(settings: Settings, current: boolean): { state: AccelState; api: AngleBackend; led: LedState }`, `AccelState = "on" | "software" | "wrongGpu" | "unchecked"`.

- [ ] **Step 1: Типы** — `src/lib/types.ts`, после `export type AngleBackend …`:

```ts
/** Зеркало Rust `model::Verdict`. */
export type Verdict =
	| { kind: "hardware"; backend: AngleBackend | null }
	| { kind: "software" }
	| { kind: "wrongGpu"; backend: AngleBackend | null }
	| { kind: "unknown" };

export interface GpuCheck {
	verdict: Verdict;
	renderer: string;
	angle: AngleBackend;
	adapter: string;
}
```

В `interface Settings` последним полем `gpuCheck: GpuCheck | null;`, в `interface StateDto` после `gpu` — `gpuCheckCurrent: boolean;`.

- [ ] **Step 2: Написать падающий тест** — `src/lib/accel.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { accelView } from "./accel";
import type { Settings, Verdict } from "./types";

const settings = (verdict: Verdict | null): Settings => ({
	schema: 1,
	profile: "balance",
	overrides: {},
	engine: { angle: "d3d11", diskCacheMb: 2048, extraArgs: "" },
	locale: "auto",
	theme: "auto",
	gpuCheck: verdict && { verdict, renderer: "r", angle: "d3d11", adapter: "RTX" }
});

describe("accelView", () => {
	it("shows the real backend when the GPU renders", () => {
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), true)).toEqual({ state: "on", api: "gl", led: "ok" });
	});

	it("flags software render and the wrong GPU", () => {
		expect(accelView(settings({ kind: "software" }), true)).toEqual({ state: "software", api: "d3d11", led: "err" });
		expect(accelView(settings({ kind: "wrongGpu", backend: "d3d11on12" }), true)).toEqual({ state: "wrongGpu", api: "d3d11on12", led: "warn" });
	});

	it("is unchecked without a check, with a stale one or an unknown verdict", () => {
		const unchecked = { state: "unchecked", api: "d3d11", led: "off" };
		expect(accelView(settings(null), true)).toEqual(unchecked);
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), false)).toEqual(unchecked);
		expect(accelView(settings({ kind: "unknown" }), true)).toEqual(unchecked);
	});

	it("falls back to the configured backend when the renderer did not name one", () => {
		expect(accelView(settings({ kind: "hardware", backend: null }), true).api).toBe("d3d11");
	});
});
```

- [ ] **Step 3: Убедиться, что падает**

Run: `npx vitest run src/lib/accel.test.ts`
Expected: FAIL — нет модуля `./accel`.

- [ ] **Step 4: `src/lib/accel.ts`**

```ts
import type { AngleBackend, LedState, Settings } from "./types";

export type AccelState = "on" | "software" | "wrongGpu" | "unchecked";

const LED: Record<AccelState, LedState> = { on: "ok", software: "err", wrongGpu: "warn", unchecked: "off" };

/** Статус ускорения для паспорта: вердикт последней проверки, если она про текущие бэкенд и видеокарту. */
export function accelView(settings: Settings, current: boolean): { state: AccelState; api: AngleBackend; led: LedState } {
	const v = current ? settings.gpuCheck?.verdict : undefined;
	const state: AccelState = v?.kind === "hardware" ? "on" : v?.kind === "software" ? "software" : v?.kind === "wrongGpu" ? "wrongGpu" : "unchecked";
	const backend = v && "backend" in v ? v.backend : null;
	return { state, api: backend ?? settings.engine.angle, led: LED[state] };
}
```

Run: `npx vitest run src/lib/accel.test.ts` — Expected: PASS.

- [ ] **Step 5: Строки i18n** — в `src/lib/i18n/ru.json` (рядом с `"gpu.unknown"`):

```json
  "accel.label": "УСКОР.",
  "accel.on": "ВКЛ",
  "accel.software": "ПРОГРАММНОЕ",
  "accel.wrongGpu": "ВСТРОЙКА",
  "accel.unchecked": "НЕ ПРОВЕРЕНО",
  "accel.tip.on": "Игра рисует на видеокарте. API — движок, на котором она работает на самом деле.",
  "accel.tip.software": "В прошлый раз игра рисовала без видеокарты. Выбери другой движок в «Настройке» или обнови драйвер видеокарты.",
  "accel.tip.wrongGpu": "Игра рисует на встроенной видеокарте, хотя есть дискретная. Выбери дискретную видеокарту основной в панели NVIDIA или AMD.",
  "accel.tip.unchecked": "Статус появится после запуска игры. После смены движка или видеокарты проверка повторяется.",
```

рядом с `"notice.engineFallback"`:

```json
  "notice.softwareRender": "Рендер без видеокарты — движок переключён на {to}. Перезапусти игру",
```

В `src/lib/i18n/en.json` на тех же местах:

```json
  "accel.label": "ACCEL",
  "accel.on": "ON",
  "accel.software": "SOFTWARE",
  "accel.wrongGpu": "iGPU",
  "accel.unchecked": "UNCHECKED",
  "accel.tip.on": "The game renders on the GPU. API is the engine it actually runs on.",
  "accel.tip.software": "Last time the game rendered without the GPU. Pick another engine in Tuning or update the GPU driver.",
  "accel.tip.wrongGpu": "The game renders on the integrated GPU while a discrete one is present. Make the discrete GPU the preferred one in the NVIDIA or AMD control panel.",
  "accel.tip.unchecked": "The status appears after you launch the game. It is checked again after an engine or GPU change.",
```

```json
  "notice.softwareRender": "Rendering without the GPU — engine switched to {to}. Restart the game",
```

- [ ] **Step 6: Паспорт** — `src/screens/MainScreen.svelte`.

Импорты: `import Led from "../components/Led.svelte";` и `import { accelView } from "../lib/accel";`. После `const gpuName = …`:

```ts
	const accel = $derived(accelView(dto.settings, dto.gpuCheckCurrent));
```

В `<dl class="passport mono">` строку `<dd>{dto.settings.engine.angle.toUpperCase()}</dd>` заменить на:

```svelte
				<dd>{accel.api.toUpperCase()}</dd>
				<dt>{t("accel.label")}</dt>
				<dd class="accel" {@attach tip(() => ({ title: t(`accel.${accel.state}`), body: t(`accel.tip.${accel.state}`) }))}>
					<Led state={accel.led} label="" />{t(`accel.${accel.state}`)}
				</dd>
```

В `<style>` после правила `.passport dd { … }`:

```css
	.passport .accel {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
	}
```

- [ ] **Step 7: Моки превью** — `src/lib/mock.ts`: импорт типа `Verdict`; перед `let settings`:

```ts
/** Превью статуса ускорения: `?accel=on|software|wrongGpu|unknown|stale|none`. */
const accelParam = new URLSearchParams(location.search).get("accel") ?? "on";
const ACCEL: Record<string, Verdict> = {
	on: { kind: "hardware", backend: "d3d11" },
	software: { kind: "software" },
	wrongGpu: { kind: "wrongGpu", backend: "d3d11" },
	unknown: { kind: "unknown" },
	stale: { kind: "hardware", backend: "gl" }
};
```

В литерал `settings` последним полем:

```ts
	gpuCheck: ACCEL[accelParam] ? { verdict: ACCEL[accelParam], renderer: "ANGLE (NVIDIA, …)", angle: "d3d11", adapter: "NVIDIA GeForce GTX 1060 6GB" } : null
```

В ответ `get_state` после `gpu: …`: `gpuCheckCurrent: accelParam !== "stale",`.

- [ ] **Step 8: Тесты и проверка типов**

Run: `npm test && npm run check`
Expected: PASS, 0 ошибок. Если svelte-check ругается на другие литералы `Settings` без `gpuCheck` — добавить им `gpuCheck: null`.

- [ ] **Step 9: Проверить в превью**

`preview_start` с `name: "launcher-preview"`, затем по очереди открыть `http://localhost:1420/?accel=on`, `?accel=software`, `?accel=wrongGpu`, `?accel=stale`, `?accel=none`. Через `read_page` / `javascript_tool` убедиться: в паспорте строка «УСКОР.» с текстом «ВКЛ» / «ПРОГРАММНОЕ» / «ВСТРОЙКА» / «НЕ ПРОВЕРЕНО» / «НЕ ПРОВЕРЕНО», у `.led` классы `ok` / `err` / `warn` / нет / нет; при `on` строка API — `D3D11`. Скриншот одного состояния — пользователю. Ошибок в консоли нет. Остановить превью.

- [ ] **Step 10: Commit**

```bash
git add src
git commit -m "feat: acceleration status in the launcher GPU passport"
```

---

### Task 7: Полная проверка и чек-лист для пользователя

**Files:** без изменений кода.

- [ ] **Step 1: Все проверки**

Run (из `pult`): `npm run check && npm test && npm run build:agent`, затем из `pult/src-tauri`: `cargo test && cargo clippy --all-targets`
Expected: всё зелёное.

- [ ] **Step 2: Сборка**

Run: `npm run dist` (лог — в scratchpad)
Expected: `Created dist-release/FoundryPerformance-0.2.2.exe` и `…-portable.zip`.

- [ ] **Step 3: Смоук на изолированных профилях**

Распаковать портативный zip в scratchpad, запустить `FoundryPerformance.exe` с `APPDATA`/`LOCALAPPDATA` из scratchpad (как в подпроекте 1), снять окно через `PrintWindow`: в паспорте «УСКОР. — НЕ ПРОВЕРЕНО», серая лампочка. Закрыть процесс по его PID.

- [ ] **Step 4: Передать пользователю чек-лист ручной проверки** (на его сервере, сам не логинюсь):
  1. Обычный запуск игры → вернуться в лаунчер: «УСКОР. — ВКЛ», в API реальный бэкенд.
  2. «Настройка» → доп. флаги `--use-angle=swiftshader --enable-unsafe-swiftshader`, запуск → в игре тост про рендер без видеокарты; в лаунчере уведомление «движок переключён на d3d11on12».
  3. Убрать флаги, запустить снова → «ВКЛ».
  4. Вернуть движок D3D11 в «Настройке», если откат его сменил.
