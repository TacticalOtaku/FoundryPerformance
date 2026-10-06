# «Намерение» на Tactile AIM 1.8.0 — план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** перевести все пять экранов лаунчера Foundry Performance на Tactile из AIM 1.8.0 с главным экраном «Намерение» и заменить определение видеокарты через DXGI на значок в духе FLC.

**Architecture:** токены — дословная копия `tokens.css` из AIM; примитивы — Svelte-компоненты со scoped-стилями в `src/components/tactile/`; движение — Svelte actions на Web Animations в `src/lib/motion.ts`. Rust теряет `gpu.rs` и всё, что на нём висело, и получает `accent` и `last_server` в настройках. Логика экранов (`store.svelte.ts`, `levers.ts`, `tooltip.svelte.ts`) не переписывается, меняется разметка.

**Tech Stack:** Tauri 2 (Rust), Svelte 5 (runes, snippets, attachments), TypeScript 6, Vite 8, Vitest 5, `@fontsource-variable/{onest,jetbrains-mono,unbounded}`.

**Spec:** `docs/superpowers/specs/2026-10-07-intent-redesign-design.md` — читать вместе с планом.

## Global Constraints

- Ветка `feat/intent` (от `main` `ca04965`). Никаких push, тегов и релизов.
- Токены живут на `.tc-root`, не на `:root`. Цвета, тени, радиусы и шрифты в компонентах — только через `var(--tc-*)`.
- Сбросы стилей — только через `:where()`.
- Анимируются только `transform`, `opacity`, цвета и ширина шкал. `backdrop-filter` и `filter: blur` запрещены. Бесконечных анимаций нет.
- Уменьшенное движение — только системный `prefers-reduced-motion: reduce`; настройки «Движение» нет.
- Акцент по умолчанию — `peach`. Профиль по умолчанию — `balance`.
- Шрифты только из бандла, подмножества `latin` и `cyrillic`. Запросов к CDN нет.
- Каждая новая или изменённая строка — сразу в `src/lib/i18n/ru.json` и `src/lib/i18n/en.json`. Тест `parity.test.ts` должен оставаться зелёным.
- Примитив не принимает чужой scoped-класс. Раскладку задаёт родитель обёрткой `<div class="…">`. Атрибуты (`style`, `data-part`, `role`, обработчики) передаются в примитив через `...rest`.
- Перед `cargo test` / `cargo clippy` нужен собранный агент: `npm run build:agent` (`agent.rs` подключает `agent/dist/agent.js` через `include_str!`).
- Гейты каждой задачи: `npm test` и `npm run check`; если задача трогает Rust — ещё `cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings`.
- Каждый коммит заканчивается строкой `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Отличия от спеки, принятые при планировании:
  - статусов сервера пять, как в `status.ts` (`running`, `idle`, `stopped`, `unknown`, `checking`), а не три;
  - на экране удаления нет шкалы: Rust не шлёт прогресс удаления, остаётся строка статуса;
  - пик кривой `SETTLE` — 1.02 (1.04 — контрольная точка Безье, а не значение кривой).

## Карта файлов

| Файл | Что делает |
|---|---|
| `src-tauri/src/gpu.rs` | удаляется |
| `src-tauri/src/{model,commands,telemetry,state,store,lib}.rs`, `build.rs`, `Cargo.toml`, `capabilities/launcher.json` | без GPU; `Accent`, `last_server`, `toggle_maximize` |
| `agent/src/{gpu.ts,gpu.test.ts}` | удаляются |
| `agent/src/{bridge,bridge.test,main,types,i18n}.ts` | без отчёта `gpu` и `ask` |
| `src/lib/gpu.ts` (+test) | проба WebGL2 / аппаратного ускорения |
| `src/lib/palette.ts` (+test) | 10 акцентов Tactile |
| `src/lib/radio.ts` (+test) | клавиатура радиогрупп |
| `src/lib/profile-tip.ts` | порядок и подсказки профилей |
| `src/lib/intent.ts` (+test) | причина выбора, стартовый выбор, тон статуса, последний замер, хост |
| `src/lib/motion.ts` (+test) | `SETTLE`, `openWindow`, `press`, `rise`, `countTo` |
| `src/lib/{accel,gpu-probe}.ts` (+tests) | удаляются |
| `src/styles/tactile/tokens.css` | копия AIM |
| `src/styles/base.css` | основа поверх токенов |
| `src/styles/tokens.css` | удаляется |
| `src/components/tactile/*.svelte` | примитивы и `Icon` |
| `src/components/{TitleBar,GpuBadge,VersionChip,UpdatePill,CacheControl,Tooltip}.svelte` | шапка, значок GPU, версия, обновление, кэш, подсказка |
| `src/components/main/{Hero,ServerList,LastRun}.svelte` | части главного экрана |
| `src/screens/*.svelte` | пять экранов на примитивах |
| `src/components/{Slot,LaunchButton,Knob,Led,Lcd,Fader,Segmented,Toggle,VersionTag}.svelte` | удаляются по ходу |
| `src/dev/Kit.svelte` | витрина примитивов для превью (`?kit`, только dev) |
| `src/lib/i18n/usage.test.ts` | нет обращений к несуществующим ключам и мёртвых ключей |

---

### Task 1: Rust без определения видеокарты

**Files:**
- Delete: `src-tauri/src/gpu.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/state.rs`, `src-tauri/src/store.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/telemetry.rs`, `src-tauri/src/model.rs`, `src-tauri/build.rs`, `src-tauri/Cargo.toml`, `src-tauri/capabilities/launcher.json`

**Interfaces:**
- Produces: `Store::load_settings(&self) -> (Settings, Option<Notice>)` без аргумента; `report_telemetry(...) -> Result<(), String>`; `StateDto` без `gpu`, `gpu_check_current`, `recommended`; `Settings` без `gpu_check`; `Data` без `current_mode`; `AppState` без `adapters`.

- [ ] **Step 1: Тест на старые настройки (model.rs)**

В `mod tests` файла `src-tauri/src/model.rs` удалить тесты `verdict_json_shape`, `settings_without_gpu_check_still_load`, `gpu_check_is_stale_after_backend_or_gpu_change` и добавить:

```rust
    #[test]
    fn settings_from_older_versions_still_load() {
        let json = r#"{"schema":1,"profile":"potato","gpuCheck":{"verdict":{"kind":"software"},"renderer":"r","angle":"d3d11","adapter":"RTX"},"motion":"reduced"}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.profile, ProfileId::Potato);
    }
```

- [ ] **Step 2: Удалить типы GPU из model.rs**

Удалить целиком `enum Verdict` (с комментарием над ним), `struct GpuCheck` и `impl GpuCheck`. В `struct Settings` удалить поле `gpu_check` вместе с комментарием, в `impl Default for Settings` — строку `gpu_check: None,`.

- [ ] **Step 3: Удалить gpu.rs и его подключение**

```bash
git rm src-tauri/src/gpu.rs
```

В `src-tauri/src/lib.rs` удалить строку `pub mod gpu;`. В блоке `.setup` заменить загрузку и `AppState`:

```rust
            let store = store::Store::new(store::Store::default_dir());
            let (settings, n1) = store.load_settings();
            let (servers, n2) = store.load_servers();
            let (stats, n3) = store.load_stats();
            let lang = locale::effective(settings.locale);
            app.manage(AppState {
                store,
                data: Mutex::new(Data {
                    settings,
                    servers,
                    stats,
                    notice: n1.or(n2).or(n3),
                    current_server: None,
                    session_fallback_done: false,
                    session_origins: Vec::new(),
                }),
            });
```

В `invoke_handler` удалить `commands::classify_renderer,`. В `src-tauri/build.rs` удалить `"classify_renderer",`. В `src-tauri/capabilities/launcher.json` удалить `"allow-classify-renderer"` и запятую перед ним. В `src-tauri/Cargo.toml` удалить строку `"Win32_Graphics_Dxgi",`.

- [ ] **Step 4: state.rs**

```rust
use crate::model::{Notice, Server, ServerStats, Settings};
use crate::store::Store;
use std::collections::BTreeMap;
use std::sync::Mutex;

pub struct Data {
    pub settings: Settings,
    pub servers: Vec<Server>,
    pub stats: BTreeMap<String, ServerStats>,
    pub notice: Option<Notice>,
    /// Сервер, открытый в окне `game` (для атрибуции телеметрии).
    pub current_server: Option<String>,
    /// Откат ANGLE выполняется не больше одного раза за сессию.
    pub session_fallback_done: bool,
    /// Источники страниц, от которых в этой сессии принимаются отчёты агента:
    /// адрес входа, известный адрес игры и адрес, подтверждённый проверкой Foundry.
    pub session_origins: Vec<url::Origin>,
}

pub struct AppState {
    pub store: Store,
    pub data: Mutex<Data>,
}
```

- [ ] **Step 5: store.rs — профиль по умолчанию «Баланс»**

```rust
    pub fn load_settings(&self) -> (Settings, Option<Notice>) {
        match self.read::<Settings>("settings.json") {
            Ok(Some(mut s)) => {
                s.schema = SCHEMA;
                (s, None)
            }
            Ok(None) => (Settings::default(), None),
            Err(n) => (Settings::default(), Some(n)),
        }
    }
```

В тестах `store.rs` заменить `missing_settings_use_fallback_profile` на:

```rust
    #[test]
    fn missing_settings_start_on_balance() {
        let (_d, s) = tmp();
        let (settings, notice) = s.load_settings();
        assert_eq!(settings.profile, ProfileId::Balance);
        assert!(notice.is_none());
    }
```

и в `settings_roundtrip` заменить `s.load_settings(ProfileId::Potato).0` на `s.load_settings().0`. Остальные вызовы `load_settings(` в файле, если есть, — без аргумента.

- [ ] **Step 6: commands.rs**

- В `use crate::{...}` убрать `gpu`.
- `StateDto`: удалить поля `gpu`, `gpu_check_current`, `recommended`; в `get_state` — их заполнение.
- `start_game`: удалить строку `d.current_mode = mode;`.
- Удалить `gpu_check_current`, `classify_launcher`, `classify_renderer`.
- `merge_settings` временно (Task 2 её расширит):

```rust
pub fn merge_settings(_stored: &Settings, incoming: Settings) -> Settings {
    Settings { schema: SCHEMA, ..incoming }
}
```

- `report_telemetry` — новая сигнатура и без ветки `Gpu`:

```rust
#[tauri::command]
pub async fn report_telemetry(webview: Webview, state: State<'_, AppState>, report: telemetry::Report) -> Result<(), String> {
    if !windows::is_game(webview.label()) || !telemetry::validate(&report) {
        return Err("rejected".into());
    }
    let page = webview.url().map_err(|_| "rejected".to_string())?;
    if let telemetry::Report::FoundryUrl { url } = &report {
        if !telemetry::describes_page(url, &page) || !probe::probe(url).await.foundry {
            return Err("rejected".into());
        }
        let mut d = state.data.lock().expect("state poisoned");
        if !d.session_origins.contains(&page.origin()) {
            d.session_origins.push(page.origin());
        }
    } else if !telemetry::page_trusted(&page, &state.data.lock().expect("state poisoned").session_origins) {
        return Err("rejected".into());
    }
    let mut guard = state.data.lock().expect("state poisoned");
    let d = &mut *guard;
    let Some(server_id) = d.current_server.clone() else { return Ok(()) };
    if matches!(report, telemetry::Report::WebglLost { early: true }) {
        if d.session_fallback_done {
            return Ok(());
        }
        d.session_fallback_done = true;
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|t| t.as_secs()).unwrap_or(0);
    let fx = telemetry::apply(&report, &server_id, now, &mut d.stats, &mut d.servers, &mut d.settings);
    commit(&state, d, fx);
    Ok(())
}
```

- В тестах удалить `saving_settings_keeps_the_launchers_gpu_check`, `gpu_check_is_current_only_for_the_same_backend_and_gpu`, `launcher_renderer_is_classified_and_length_limited` и импорты, нужные только им.

- [ ] **Step 7: telemetry.rs**

- Первая строка становится `use crate::model::*;` (без `gpu`).
- Удалить вариант `Gpu { renderer: String }` с комментарием, поле `fallback` в `Effects` с комментарием, ветку `Report::Gpu { renderer } => ...` в `validate`, ветку `Report::Gpu { .. } => {}` с комментарием в `apply`, функцию `apply_gpu` с комментарием.
- В тестах удалить `fn nv()`, `parses_gpu_report_and_limits_its_length`, `hardware_check_is_remembered`, `software_render_switches_the_backend_once_per_session`, `baseline_run_only_answers` и константы, нужные только им. `early_webgl_loss_switches_angle_backend` остаётся без изменений.

- [ ] **Step 8: Прогнать Rust**

```bash
npm run build:agent
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

Expected: PASS, включая `settings_from_older_versions_still_load`, `missing_settings_start_on_balance`, `early_webgl_loss_switches_angle_backend`. Если clippy ругается на неиспользуемый импорт или `LaunchMode` — убрать ровно его.

- [ ] **Step 9: Коммит**

```bash
git add -A src-tauri
git commit -m "refactor: drop DXGI adapters and renderer classification" -m "Profile defaults to Balance. Early webglLost fallback stays." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Акцент и последний запущенный сервер в Rust

**Files:**
- Modify: `src-tauri/src/model.rs`, `src-tauri/src/commands.rs`

**Interfaces:**
- Consumes: Task 1 (`merge_settings` без `gpu_check`).
- Produces: `enum Accent` (camelCase, умолчание `Peach`); `Settings.accent: Accent`; `Settings.last_server: Option<String>` (JSON `lastServer`); `commands::forget_server(&mut Settings, &str) -> bool`; `merge_settings` сохраняет `last_server` из хранилища.

- [ ] **Step 1: Падающие тесты**

В `model.rs`, `mod tests`:

```rust
    #[test]
    fn accent_and_last_server_default_and_roundtrip() {
        let s: Settings = serde_json::from_str(r#"{"schema":1}"#).unwrap();
        assert_eq!(s.accent, Accent::Peach);
        assert_eq!(s.last_server, None);
        let s = Settings { accent: Accent::Steel, last_server: Some("a1".into()), ..Settings::default() };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains(r#""accent":"steel""#), "{json}");
        assert!(json.contains(r#""lastServer":"a1""#), "{json}");
        assert_eq!(serde_json::from_str::<Settings>(&json).unwrap(), s);
    }
```

В `commands.rs`, `mod tests`:

```rust
    #[test]
    fn saving_settings_keeps_the_last_launched_server() {
        let stored = Settings { last_server: Some("a1".into()), ..Settings::default() };
        let incoming = Settings { profile: ProfileId::Potato, last_server: None, ..Settings::default() };
        let merged = merge_settings(&stored, incoming);
        assert_eq!(merged.last_server.as_deref(), Some("a1"));
        assert_eq!(merged.profile, ProfileId::Potato);
    }

    #[test]
    fn deleting_the_last_launched_server_forgets_it() {
        let mut s = Settings { last_server: Some("a1".into()), ..Settings::default() };
        assert!(!forget_server(&mut s, "a2"));
        assert_eq!(s.last_server.as_deref(), Some("a1"));
        assert!(forget_server(&mut s, "a1"));
        assert_eq!(s.last_server, None);
    }
```

- [ ] **Step 2: Убедиться, что не компилируется**

Run: `cd src-tauri && cargo test`
Expected: FAIL — `Accent`, `accent`, `last_server`, `forget_server` не найдены.

- [ ] **Step 3: Реализация в model.rs**

Над `pub const SCHEMA`:

```rust
/// Акцент интерфейса; оттенки — в `src/lib/palette.ts` (палитра Tactile из AIM).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Accent {
    #[default]
    Peach,
    Amber,
    Sage,
    Mint,
    Azure,
    Periwinkle,
    Lavender,
    Orchid,
    Rose,
    Steel,
}
```

В `Settings` после `theme`:

```rust
    pub accent: Accent,
    /// Последний запущенный сервер; пишет только лаунчер (см. `commands::merge_settings`).
    pub last_server: Option<String>,
```

В `Default` после `theme: Theme::Auto,`:

```rust
            accent: Accent::Peach,
            last_server: None,
```

- [ ] **Step 4: Реализация в commands.rs**

```rust
/// Последний запущенный сервер пишет только лаунчер: копия настроек в интерфейсе может быть старше.
pub fn merge_settings(stored: &Settings, incoming: Settings) -> Settings {
    Settings { schema: SCHEMA, last_server: stored.last_server.clone(), ..incoming }
}

/// Удалённый сервер больше не «последний запущенный». `true` — настройки изменились.
pub fn forget_server(settings: &mut Settings, id: &str) -> bool {
    if settings.last_server.as_deref() != Some(id) {
        return false;
    }
    settings.last_server = None;
    true
}
```

В `delete_server` после `d.stats.remove(&id);`:

```rust
    if forget_server(&mut d.settings, &id) {
        let _ = state.store.save_settings(&d.settings);
    }
```

Команда `launch` целиком:

```rust
/// async: на Windows создание окна из синхронной команды может взаимно заблокироваться.
#[tauri::command]
pub async fn launch(app: AppHandle, state: State<'_, AppState>, server_id: String, safe_mode: bool) -> Result<(), String> {
    let server = {
        let d = state.data.lock().expect("state poisoned");
        d.servers.iter().find(|s| s.id == server_id).cloned().ok_or_else(|| "err.serverMissing".to_string())?
    };
    let mode = if safe_mode { LaunchMode::Safe } else { LaunchMode::Normal };
    start_game(&app, &state, &server, mode)?;
    // При следующем открытии лаунчер предложит продолжить с этого сервера
    let mut d = state.data.lock().expect("state poisoned");
    if d.settings.last_server.as_deref() != Some(server.id.as_str()) {
        d.settings.last_server = Some(server.id);
        let _ = state.store.save_settings(&d.settings);
    }
    Ok(())
}
```

- [ ] **Step 5: Прогнать**

Run: `cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: PASS.

- [ ] **Step 6: Коммит**

```bash
git add src-tauri/src/model.rs src-tauri/src/commands.rs
git commit -m "feat: accent setting and last launched server" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Агент без отчёта о видеокарте

**Files:**
- Delete: `agent/src/gpu.ts`, `agent/src/gpu.test.ts`
- Modify: `agent/src/bridge.ts`, `agent/src/bridge.test.ts`, `agent/src/main.ts`, `agent/src/types.ts`, `agent/src/i18n.ts`

**Interfaces:**
- Produces: `Report` без варианта `gpu`; функции `ask` больше нет (единственный канал — `send`).

- [ ] **Step 1: Удалить файлы пробы**

```bash
git rm agent/src/gpu.ts agent/src/gpu.test.ts
```

- [ ] **Step 2: bridge.ts**

Удалить строку `| { kind: "gpu"; renderer: string };`, а предыдущую строку `| { kind: "foundryUrl"; url: string }` закончить `;`. Удалить функцию `ask` вместе с комментарием над ней.

- [ ] **Step 3: bridge.test.ts**

Импорт сократить до `import { ipcStatus, send, toggleFullscreen } from "./bridge";`. Удалить весь блок `describe("ask", …)`.

- [ ] **Step 4: main.ts**

- Удалить `import { rendererName, verdictMessage } from "./gpu";`.
- В `import { ask, ipcStatus, send, toggleFullscreen } from "./bridge";` убрать `ask`.
- В импорте типов убрать `Verdict`.
- Удалить `let gpuChecked = false;` и `let urlAck: Promise<unknown> = Promise.resolve(null);`.
- Удалить функцию `checkGpu` вместе с комментарием.
- Строку `urlAck = ask({ kind: "foundryUrl", url: location.origin + location.pathname });` заменить на `send({ kind: "foundryUrl", url: location.origin + location.pathname });`.
- Удалить строку `void urlAck.then(checkGpu);` в обработчике `ready`.
- В обработчике `canvasReady` удалить комментарий `// canvasReady срабатывает и до ready — проверяем только после него` и строку `if (readyAt) void urlAck.then(checkGpu);`.

- [ ] **Step 5: types.ts и i18n.ts**

В `agent/src/types.ts` удалить `Verdict` с комментарием. В `agent/src/i18n.ts` удалить ключи `gpuSoftware` и `gpuWrong` в обоих языках; если в файле есть тип или интерфейс со списком ключей — удалить их и оттуда.

- [ ] **Step 6: Проверить**

```bash
npx vitest run agent
npm run build:agent
npm run check
```

Expected: тесты агента PASS, сборка без ошибок. `grep -rn "\"gpu\"\|Verdict\|ask(" agent/src` — пусто.

- [ ] **Step 7: Коммит**

```bash
git add -A agent
git commit -m "refactor(agent): drop the renderer report and in-game GPU warning" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Проба видеокарты во фронте и новые типы

**Files:**
- Create: `src/lib/gpu.ts`, `src/lib/gpu.test.ts`
- Delete: `src/lib/accel.ts`, `src/lib/accel.test.ts`, `src/lib/gpu-probe.ts`, `src/lib/gpu-probe.test.ts`
- Modify: `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/store.svelte.ts`, `src/lib/mock.ts`, `src/screens/MainScreen.svelte`

**Interfaces:**
- Produces:
  - `src/lib/gpu.ts`: `interface GpuProbe { webgl2: boolean; hardware: boolean }`, `type GpuLevel = "ok" | "software" | "none"`, `probeGpu(make?: () => CanvasLike): GpuProbe`, `gpuLevel(p: GpuProbe): GpuLevel`.
  - `types.ts`: `type AccentId = "peach" | "amber" | "sage" | "mint" | "azure" | "periwinkle" | "lavender" | "orchid" | "rose" | "steel"`; `Settings.accent: AccentId`; `Settings.lastServer: string | null`; `StateDto` без `gpu`, `gpuCheckCurrent`, `recommended`; нет `Verdict`, `GpuCheck`, `GpuInfo`.
  - `api.ts`: `gpuProbe(): Promise<GpuProbe>`.
  - `store`: `app.gpu: GpuProbe | null`.
  - `mock.ts`: `mockGpu(): GpuProbe | null` (`?gpu=ok|software|none`).

- [ ] **Step 1: Падающий тест `src/lib/gpu.test.ts`**

```ts
import { describe, expect, it, vi } from "vitest";
import { gpuLevel, probeGpu } from "./gpu";

/** Холст, который открывает обычный и/или строгий (`failIfMajorPerformanceCaveat`) контекст. */
function fakeCanvas(plain: boolean, strict: boolean) {
	const lose = vi.fn();
	const make = vi.fn(() => ({
		getContext: (_id: "webgl2", attrs?: WebGLContextAttributes) =>
			(attrs?.failIfMajorPerformanceCaveat ? strict : plain) ? { getExtension: () => ({ loseContext: lose }) } : null
	}));
	return { make, lose };
}

describe("probeGpu", () => {
	it("hardware WebGL2: both contexts open and are released", () => {
		const { make, lose } = fakeCanvas(true, true);
		expect(probeGpu(make)).toEqual({ webgl2: true, hardware: true });
		expect(make).toHaveBeenCalledTimes(2);
		expect(lose).toHaveBeenCalledTimes(2);
	});

	it("software WebGL2: the strict context is refused", () => {
		expect(probeGpu(fakeCanvas(true, false).make)).toEqual({ webgl2: true, hardware: false });
	});

	it("no WebGL2: the strict probe is skipped", () => {
		const { make } = fakeCanvas(false, true);
		expect(probeGpu(make)).toEqual({ webgl2: false, hardware: false });
		expect(make).toHaveBeenCalledTimes(1);
	});

	it("a throwing getContext counts as missing", () => {
		const make = () => ({
			getContext: () => {
				throw new Error("blocked");
			}
		});
		expect(probeGpu(make)).toEqual({ webgl2: false, hardware: false });
	});
});

describe("gpuLevel", () => {
	it("maps the probe onto ok / software / none", () => {
		expect(gpuLevel({ webgl2: true, hardware: true })).toBe("ok");
		expect(gpuLevel({ webgl2: true, hardware: false })).toBe("software");
		expect(gpuLevel({ webgl2: false, hardware: false })).toBe("none");
	});
});
```

Run: `npx vitest run src/lib/gpu.test.ts` — Expected: FAIL, модуль `./gpu` не найден.

- [ ] **Step 2: `src/lib/gpu.ts`**

```ts
/** Что лаунчер знает о видеокарте — как в FLC, но честно: есть ли WebGL2 и не программный ли он. */
export interface GpuProbe {
	webgl2: boolean;
	hardware: boolean;
}

export type GpuLevel = "ok" | "software" | "none";

interface CanvasLike {
	getContext(id: "webgl2", attrs?: WebGLContextAttributes): unknown;
}

type Releasable = { getExtension(name: string): { loseContext(): void } | null };

function opens(make: () => CanvasLike, attrs?: WebGLContextAttributes): boolean {
	try {
		const gl = make().getContext("webgl2", attrs) as Releasable | null;
		if (!gl) return false;
		gl.getExtension("WEBGL_lose_context")?.loseContext();
		return true;
	} catch {
		return false;
	}
}

/**
 * Проба WebView самого лаунчера: флаги ANGLE и драйвер у него те же, что у игрового окна,
 * но холст Foundry здесь не проверяется. С `failIfMajorPerformanceCaveat` Chromium отказывает,
 * если WebGL2 есть только программный (SwiftShader, WARP).
 */
export function probeGpu(make: () => CanvasLike = () => document.createElement("canvas") as CanvasLike): GpuProbe {
	const webgl2 = opens(make);
	return { webgl2, hardware: webgl2 && opens(make, { failIfMajorPerformanceCaveat: true }) };
}

export const gpuLevel = (p: GpuProbe): GpuLevel => (!p.webgl2 ? "none" : p.hardware ? "ok" : "software");
```

Run: `npx vitest run src/lib/gpu.test.ts` — Expected: PASS.

- [ ] **Step 3: Удалить старую пробу и вердикты**

```bash
git rm src/lib/accel.ts src/lib/accel.test.ts src/lib/gpu-probe.ts src/lib/gpu-probe.test.ts
```

- [ ] **Step 4: types.ts**

- Удалить `Verdict` (с комментарием), `GpuCheck`, `GpuInfo`.
- После `ThemePref` добавить:

```ts
export type AccentId = "peach" | "amber" | "sage" | "mint" | "azure" | "periwinkle" | "lavender" | "orchid" | "rose" | "steel";
```

- В `Settings` заменить `gpuCheck: GpuCheck | null;` на:

```ts
	accent: AccentId;
	/** Последний запущенный сервер; пишет только Rust. */
	lastServer: string | null;
```

- В `StateDto` удалить `gpu`, `gpuCheckCurrent`, `recommended`.

- [ ] **Step 5: api.ts**

Импорт типов: убрать `Verdict`. Удалить `classifyRenderer`. После объекта `api` добавить:

```ts
/** Проба видеокарты; в браузерном превью её можно подменить: `?gpu=ok|software|none`. */
export async function gpuProbe(): Promise<GpuProbe> {
	if (!inTauri) {
		const { mockGpu } = await import("./mock");
		const forced = mockGpu();
		if (forced) return forced;
	}
	return probeGpu();
}
```

и импорт сверху: `import { type GpuProbe, probeGpu } from "./gpu";`.

- [ ] **Step 6: store.svelte.ts**

- Удалить `import { launcherRenderer } from "./gpu-probe";`, из импорта `./types` убрать `Verdict`, к импорту из `./api` добавить `gpuProbe`, добавить `import type { GpuProbe } from "./gpu";`.
- Поле `launcherVerdict` (с комментарием) заменить на:

```ts
	/** WebGL2 и аппаратное ускорение в окне лаунчера (null — проба ещё не прошла). */
	gpu = $state<GpuProbe | null>(null);
```

- В `load()` строку `void this.checkLauncherGpu();` заменить на `void gpuProbe().then((p) => (this.gpu = p));`.
- Удалить метод `checkLauncherGpu`.

- [ ] **Step 7: mock.ts**

- В импорте типов убрать `Verdict`, добавить `import type { GpuProbe } from "./gpu";`.
- Удалить `accelParam`, `ACCEL` и комментарий над ними, ветку `case "classify_renderer"`.
- В `settings` заменить строку `gpuCheck: …` на `accent: "peach",` и `lastServer: "a1"`.
- В `get_state` удалить `gpu`, `gpuCheckCurrent`, `recommended`.
- В конец файла:

```ts
/** Превью значка видеокарты: `?gpu=ok|software|none`; без параметра — настоящая проба браузера. */
export function mockGpu(): GpuProbe | null {
	switch (new URLSearchParams(location.search).get("gpu")) {
		case "ok":
			return { webgl2: true, hardware: true };
		case "software":
			return { webgl2: true, hardware: false };
		case "none":
			return { webgl2: false, hardware: false };
		default:
			return null;
	}
}
```

- [ ] **Step 8: MainScreen.svelte — убрать паспорт**

Удалить импорты `Led` и `accelView`, константы `gpuName` и `accel`, весь блок `{#if dto.gpu} … {:else} … {/if}` с паспортом и стили `.passport`, `.accel` (и вложенные). Экран целиком переписывается в Task 11; здесь нужно только, чтобы он компилировался.

- [ ] **Step 9: Гейты и коммит**

```bash
npm test && npm run check
git add -A src
git commit -m "feat: FLC-style WebGL2 and hardware acceleration probe" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: всё зелёное; `grep -rn "Verdict\|gpuCheck\|launcherVerdict\|accelView" src` — пусто.

---

### Task 5: Логика «Намерения»: палитра, радиогруппы, профили, причина выбора

**Files:**
- Create: `src/lib/palette.ts`, `src/lib/palette.test.ts`, `src/lib/radio.ts`, `src/lib/radio.test.ts`, `src/lib/profile-tip.ts`, `src/lib/intent.ts`, `src/lib/intent.test.ts`
- Modify: `src/lib/store.svelte.ts`, `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `AccentId`, `Settings.lastServer` (Task 4); `SlotStatus` из `status.ts`; `KnobPosition` из `levers.ts`.
- Produces:
  - `palette.ts`: `interface Accent { id: AccentId; h: number; c: number }`, `ACCENTS: readonly Accent[]`, `DEFAULT_ACCENT: AccentId`, `accentById(id): Accent`, `accentStyle(id): string` (строка `--tc-acc-h: 45; --tc-acc-c: 0.13`).
  - `radio.ts`: `nextIndex(current: number, key: string, enabled: boolean[]): number | null`.
  - `profile-tip.ts`: `PROFILE_POSITIONS: readonly KnobPosition[]`, `profileNameKey(p)`, `profileTipKey(p)`.
  - `intent.ts`: `type IntentReason = "last" | "manual"`, `intentReason(selectedId, lastServer)`, `initialSelection(servers, lastServer)`, `type Tone = "good" | "warn" | "danger" | "plain"`, `STATUS_TONE: Record<SlotStatus, Tone>`, `STATUS_DETAIL: Record<SlotStatus, string>` (i18n-ключ подробностей), `interface LastRun { fps: number; profile: ProfileId; kind: "bench" | "session" }`, `lastRun(stats)`, `hostOf(url)`.

- [ ] **Step 1: Тесты**

`src/lib/palette.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import en from "./i18n/en.json";
import ru from "./i18n/ru.json";
import { ACCENTS, accentById, accentStyle, DEFAULT_ACCENT } from "./palette";

describe("palette", () => {
	it("is the Tactile palette from AIM, in picker order", () => {
		expect(ACCENTS.map((a) => a.id)).toEqual(["peach", "amber", "sage", "mint", "azure", "periwinkle", "lavender", "orchid", "rose", "steel"]);
		expect(accentById("peach")).toEqual({ id: "peach", h: 45, c: 0.13 });
		expect(accentById("steel")).toEqual({ id: "steel", h: 250, c: 0.035 });
	});

	it("defaults to peach and falls back to it", () => {
		expect(DEFAULT_ACCENT).toBe("peach");
		expect(accentById("ultraviolet" as never).id).toBe("peach");
	});

	it("renders hue and chroma as root custom properties", () => {
		expect(accentStyle("sage")).toBe("--tc-acc-h: 145; --tc-acc-c: 0.08");
	});

	it("every accent is named in both languages", () => {
		for (const a of ACCENTS) {
			expect(ru).toHaveProperty([`accent.${a.id}`]);
			expect(en).toHaveProperty([`accent.${a.id}`]);
		}
	});
});
```

`src/lib/radio.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { nextIndex } from "./radio";

const all = [true, true, true, true];
const noManual = [true, true, true, false];

describe("nextIndex", () => {
	it("moves with arrows and wraps", () => {
		expect(nextIndex(0, "ArrowRight", all)).toBe(1);
		expect(nextIndex(3, "ArrowRight", all)).toBe(0);
		expect(nextIndex(0, "ArrowLeft", all)).toBe(3);
		expect(nextIndex(1, "ArrowDown", all)).toBe(2);
		expect(nextIndex(1, "ArrowUp", all)).toBe(0);
	});

	it("skips disabled items", () => {
		expect(nextIndex(2, "ArrowRight", noManual)).toBe(0);
		expect(nextIndex(0, "ArrowLeft", noManual)).toBe(2);
		expect(nextIndex(3, "ArrowRight", noManual)).toBe(0);
	});

	it("Home and End go to the first and last enabled item", () => {
		expect(nextIndex(2, "Home", noManual)).toBe(0);
		expect(nextIndex(0, "End", noManual)).toBe(2);
	});

	it("ignores other keys and empty groups", () => {
		expect(nextIndex(0, "Enter", all)).toBeNull();
		expect(nextIndex(0, "ArrowRight", [false, false])).toBeNull();
	});

	it("starts from the edge when nothing is selected", () => {
		expect(nextIndex(-1, "ArrowRight", all)).toBe(0);
		expect(nextIndex(-1, "ArrowLeft", all)).toBe(3);
	});
});
```

`src/lib/intent.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { hostOf, initialSelection, intentReason, lastRun, STATUS_DETAIL, STATUS_TONE } from "./intent";
import ru from "./i18n/ru.json";

const servers = [{ id: "a1" }, { id: "a2" }];

describe("intentReason", () => {
	it("last launched vs picked by hand", () => {
		expect(intentReason("a1", "a1")).toBe("last");
		expect(intentReason("a2", "a1")).toBe("manual");
	});

	it("no reason without a selection or without launches", () => {
		expect(intentReason(null, "a1")).toBeNull();
		expect(intentReason("a1", null)).toBeNull();
	});
});

describe("initialSelection", () => {
	it("prefers the last launched server", () => {
		expect(initialSelection(servers, "a2")).toBe("a2");
	});

	it("falls back to the first one", () => {
		expect(initialSelection(servers, "gone")).toBe("a1");
		expect(initialSelection(servers, null)).toBe("a1");
		expect(initialSelection([], "a1")).toBeNull();
	});
});

describe("lastRun", () => {
	const session = { avg: 52.4, low1: 31, profile: "balance" as const, at: 0 };

	it("benchmark wins over a session", () => {
		expect(lastRun({ lastSession: session, lastBench: { ...session, avg: 60.6, min: 20, profile: "potato" } })).toEqual({ fps: 61, profile: "potato", kind: "bench" });
	});

	it("session, rounded", () => {
		expect(lastRun({ lastSession: session, lastBench: null })).toEqual({ fps: 52, profile: "balance", kind: "session" });
	});

	it("nothing measured", () => {
		expect(lastRun(undefined)).toBeNull();
		expect(lastRun({ lastSession: null, lastBench: null })).toBeNull();
	});
});

describe("status", () => {
	it("every status has a tone and a detail string", () => {
		expect(STATUS_TONE).toEqual({ running: "good", idle: "warn", stopped: "danger", unknown: "plain", checking: "plain" });
		for (const k of Object.values(STATUS_DETAIL)) expect(ru).toHaveProperty([k]);
	});
});

describe("hostOf", () => {
	it("keeps host and port only", () => {
		expect(hostOf("https://vtt.example.com:30000/game")).toBe("vtt.example.com:30000");
		expect(hostOf("192.168.1.40:30000")).toBe("192.168.1.40:30000");
		expect(hostOf("https://www.sqyre.app/games/x/")).toBe("www.sqyre.app");
	});

	it("returns garbage as is", () => {
		expect(hostOf("::")).toBe("::");
	});
});
```

Run: `npx vitest run src/lib/palette.test.ts src/lib/radio.test.ts src/lib/intent.test.ts` — Expected: FAIL, модулей нет.

- [ ] **Step 2: `src/lib/palette.ts`**

```ts
import type { AccentId } from "./types";

export interface Accent {
	id: AccentId;
	h: number;
	c: number;
}

/** Палитра Tactile — копия `scripts/tactile/palette.js` из AIM 1.8.0; порядок — как в выборе акцента. */
export const ACCENTS: readonly Accent[] = [
	{ id: "peach", h: 45, c: 0.13 },
	{ id: "amber", h: 78, c: 0.12 },
	{ id: "sage", h: 145, c: 0.08 },
	{ id: "mint", h: 178, c: 0.09 },
	{ id: "azure", h: 235, c: 0.1 },
	{ id: "periwinkle", h: 275, c: 0.1 },
	{ id: "lavender", h: 300, c: 0.1 },
	{ id: "orchid", h: 330, c: 0.11 },
	{ id: "rose", h: 10, c: 0.11 },
	{ id: "steel", h: 250, c: 0.035 }
];

export const DEFAULT_ACCENT: AccentId = "peach";

export const accentById = (id: AccentId): Accent => ACCENTS.find((a) => a.id === id) ?? ACCENTS[0];

/** Оттенок и насыщенность акцента для `style` корня; светлоту даёт тема. */
export function accentStyle(id: AccentId): string {
	const a = accentById(id);
	return `--tc-acc-h: ${a.h}; --tc-acc-c: ${a.c}`;
}
```

- [ ] **Step 3: `src/lib/radio.ts`**

```ts
/**
 * Следующий выбор в радиогруппе по клавише. `enabled[i] === false` — пункт пропускается
 * стрелками (например, «Ручной»: его выбирают только мышью). `null` — клавиша не наша.
 */
export function nextIndex(current: number, key: string, enabled: boolean[]): number | null {
	const n = enabled.length;
	if (!enabled.some(Boolean)) return null;
	if (key === "Home") return enabled.indexOf(true);
	if (key === "End") return enabled.lastIndexOf(true);
	const step = key === "ArrowRight" || key === "ArrowDown" ? 1 : key === "ArrowLeft" || key === "ArrowUp" ? -1 : 0;
	if (step === 0) return null;
	let i = current < 0 ? (step > 0 ? -1 : n) : current;
	for (let k = 0; k < n; k++) {
		i = (i + step + n) % n;
		if (enabled[i]) return i;
	}
	return null;
}
```

- [ ] **Step 4: `src/lib/profile-tip.ts`**

```ts
import type { KnobPosition } from "./levers";

/** Порядок сегментов профиля на главном экране. */
export const PROFILE_POSITIONS: readonly KnobPosition[] = ["quality", "balance", "potato", "manual"];

export const profileNameKey = (p: KnobPosition): string => `profile.${p}.long`;
export const profileTipKey = (p: KnobPosition): string => `tip.${p}`;
```

- [ ] **Step 5: `src/lib/intent.ts`**

```ts
import type { SlotStatus } from "./status";
import type { ProfileId, StateDto } from "./types";

export type IntentReason = "last" | "manual";

/** Почему выбран этот сервер: его запускали последним или его выбрали руками. Без запусков — без причины. */
export function intentReason(selectedId: string | null, lastServer: string | null): IntentReason | null {
	if (!selectedId || !lastServer) return null;
	return selectedId === lastServer ? "last" : "manual";
}

/** При старте выбран последний запущенный сервер, если он ещё в списке, иначе первый. */
export function initialSelection(servers: readonly { id: string }[], lastServer: string | null): string | null {
	return servers.find((s) => s.id === lastServer)?.id ?? servers[0]?.id ?? null;
}

export type Tone = "good" | "warn" | "danger" | "plain";

export const STATUS_TONE: Record<SlotStatus, Tone> = {
	running: "good",
	idle: "warn",
	stopped: "danger",
	unknown: "plain",
	checking: "plain"
};

/** Подробности статуса для подсказки. */
export const STATUS_DETAIL: Record<SlotStatus, string> = {
	running: "led.ok",
	idle: "led.idle",
	stopped: "led.err",
	unknown: "led.unknown",
	checking: "led.pending"
};

export interface LastRun {
	fps: number;
	profile: ProfileId;
	kind: "bench" | "session";
}

/** Последний замер важнее последнего сеанса: он снят в одинаковых условиях. */
export function lastRun(stats: StateDto["stats"][string] | undefined): LastRun | null {
	const b = stats?.lastBench;
	if (b) return { fps: Math.round(b.avg), profile: b.profile, kind: "bench" };
	const s = stats?.lastSession;
	return s ? { fps: Math.round(s.avg), profile: s.profile, kind: "session" } : null;
}

/** «https://vtt.example.com:30000/game» → «vtt.example.com:30000». */
export function hostOf(url: string): string {
	try {
		return new URL(/^https?:\/\//i.test(url) ? url : `http://${url}`).host || url;
	} catch {
		return url;
	}
}
```

- [ ] **Step 6: Названия акцентов в словарях**

`ru.json`: `"accent.peach": "Персик"`, `"accent.amber": "Янтарь"`, `"accent.sage": "Шалфей"`, `"accent.mint": "Мята"`, `"accent.azure": "Лазурь"`, `"accent.periwinkle": "Барвинок"`, `"accent.lavender": "Лаванда"`, `"accent.orchid": "Орхидея"`, `"accent.rose": "Роза"`, `"accent.steel": "Сталь"`.

`en.json`: `"accent.peach": "Peach"`, `"accent.amber": "Amber"`, `"accent.sage": "Sage"`, `"accent.mint": "Mint"`, `"accent.azure": "Azure"`, `"accent.periwinkle": "Periwinkle"`, `"accent.lavender": "Lavender"`, `"accent.orchid": "Orchid"`, `"accent.rose": "Rose"`, `"accent.steel": "Steel"`.

- [ ] **Step 7: Стартовый выбор в сторе**

В `store.svelte.ts` добавить `import { initialSelection } from "./intent";`, а в `load()` заменить `this.selectedId = dto.servers[0]?.id ?? null;` на:

```ts
		this.selectedId = initialSelection(dto.servers, dto.settings.lastServer);
```

- [ ] **Step 8: Гейты и коммит**

```bash
npm test && npm run check
git add src/lib
git commit -m "feat: palette, radio keys and intent logic for the main screen" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Фундамент — шрифты, токены AIM, корень темы

**Files:**
- Create: `src/styles/tactile/tokens.css`
- Delete: `src/styles/tokens.css`
- Modify: `package.json`, `package-lock.json`, `src/main.ts`, `src/styles/base.css`, `src/App.svelte`

**Interfaces:**
- Consumes: `accentStyle`, `DEFAULT_ACCENT` (Task 5).
- Produces: корень `<div class="tc-root fp-app" data-theme="light|dark" style="--tc-acc-h…">` в `App.svelte`, внутри него — `Tooltip`; класс `fp-no-transitions` на два кадра при смене темы.

> Старые компоненты пультового дизайна после этой задачи выглядят сломанными: их переменные (`--panel`, `--face`…) исчезают. Экраны чинятся в Tasks 10–14.

- [ ] **Step 1: Шрифты**

```bash
npm uninstall @fontsource/geologica @fontsource/martian-mono
npm install @fontsource-variable/onest @fontsource-variable/jetbrains-mono @fontsource-variable/unbounded
ls node_modules/@fontsource-variable/onest node_modules/@fontsource-variable/jetbrains-mono node_modules/@fontsource-variable/unbounded | grep -E "^(latin|cyrillic)-wght\.css$"
```

Expected: по `latin-wght.css` и `cyrillic-wght.css` у каждого пакета (шесть строк). Если у какого-то пакета нет `cyrillic-wght.css`, в Step 3 для него импортировать `wght.css` целиком.

- [ ] **Step 2: Токены — копия AIM**

```bash
mkdir -p src/styles/tactile
git -C "/d/MyAiProjects/Foundry Modules/actor-inventory-manager" show 988fc96:styles/tactile/tokens.css > src/styles/tactile/tokens.css
node -e "const fs=require('fs');const f='src/styles/tactile/tokens.css';fs.writeFileSync(f,'/* Копия Tactile из AIM 1.8.0 (988fc96, styles/tactile/tokens.css). Правки — сначала в AIM, потом сюда. */\n'+fs.readFileSync(f,'utf8'))"
git rm src/styles/tokens.css
```

Проверить: файл начинается с комментария-шапки, внутри есть `@property --tc-acc-h` и `.tc-root[data-theme="dark"]`.

- [ ] **Step 3: `src/main.ts`**

```ts
// только латиница и кириллица: лаунчер работает без сети, лишние наборы не нужны
import "@fontsource-variable/onest/latin-wght.css";
import "@fontsource-variable/onest/cyrillic-wght.css";
import "@fontsource-variable/jetbrains-mono/latin-wght.css";
import "@fontsource-variable/jetbrains-mono/cyrillic-wght.css";
import "@fontsource-variable/unbounded/latin-wght.css";
import "@fontsource-variable/unbounded/cyrillic-wght.css";
import "./styles/tactile/tokens.css";
import "./styles/base.css";
import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
```

- [ ] **Step 4: `src/styles/base.css`**

```css
/* Основа лаунчера поверх токенов Tactile (tactile/tokens.css).
   Сбросы — только через :where(): у них нулевая специфичность, классы компонентов всегда побеждают. */
html,
body,
#app {
	height: 100%;
	margin: 0;
}
body {
	overflow: hidden;
	-webkit-font-smoothing: antialiased;
}
:where(*, *::before, *::after) {
	box-sizing: border-box;
}
:where(button, input, select, textarea) {
	font: inherit;
	color: inherit;
	margin: 0;
}
:where(button) {
	background: none;
	border: 0;
	padding: 0;
	cursor: pointer;
}
:where(h1, h2, h3, p, ol, ul, dl, dd) {
	margin: 0;
	padding: 0;
}
.tc-root {
	height: 100%;
	background: var(--tc-bg);
	color: var(--tc-ink);
	font: 400 14px/1.45 var(--tc-font-ui);
	font-variant-numeric: tabular-nums;
}
.tc-root :focus-visible {
	outline: 2px solid var(--tc-accent);
	outline-offset: 2px;
}
.tc-root ::selection {
	background: var(--tc-accent-soft);
}
/* смена темы: два кадра без переходов, чтобы все токены легли разом */
.fp-no-transitions,
.fp-no-transitions * {
	transition: none !important;
}
@media (prefers-reduced-motion: reduce) {
	.tc-root * {
		transition-duration: 0.01ms !important;
	}
}
```

- [ ] **Step 5: `src/App.svelte` — корень темы и акцента**

Скрипт: удалить старый `$effect` с `document.documentElement.dataset.theme`, добавить импорт `import { accentStyle, DEFAULT_ACCENT } from "./lib/palette";` и:

```ts
	const DARK = "(prefers-color-scheme: dark)";
	let systemDark = $state(matchMedia(DARK).matches);
	$effect(() => {
		const mq = matchMedia(DARK);
		const sync = () => (systemDark = mq.matches);
		mq.addEventListener("change", sync);
		return () => mq.removeEventListener("change", sync);
	});
	const themePref = $derived(app.dto?.settings.theme ?? "auto");
	const dark = $derived(themePref === "night" || (themePref === "auto" && systemDark));
	const accent = $derived(accentStyle(app.dto?.settings.accent ?? DEFAULT_ACCENT));

	// Смена темы гасит переходы на два кадра — иначе токены доезжают вразнобой
	let root = $state<HTMLDivElement>();
	let shownDark: boolean | undefined;
	$effect(() => {
		const next = dark;
		if (!root || shownDark === undefined || shownDark === next) {
			shownDark = next;
			return;
		}
		shownDark = next;
		const el = root;
		el.classList.add("fp-no-transitions");
		requestAnimationFrame(() => requestAnimationFrame(() => el.classList.remove("fp-no-transitions")));
	});
```

Разметка — внешний `<div class="frame">` заменить корнем и перенести `<Tooltip />` внутрь:

```svelte
<div class="tc-root fp-app" bind:this={root} data-theme={dark ? "dark" : "light"} style={accent}>
	<!-- шапка и <main class="screen"> — как были -->
	<Tooltip />
</div>
```

Стили компонента:

```css
	.fp-app {
		display: grid;
		grid-template-rows: 44px minmax(0, 1fr);
	}
	.screen {
		min-height: 0;
	}
	.boot {
		display: grid;
		place-items: center;
		height: 100%;
		color: var(--tc-muted);
		font: 500 12px var(--tc-font-mono);
	}
```

У элемента `.boot` убрать класс `mono`.

- [ ] **Step 6: Гейты и коммит**

```bash
npm test && npm run check && npm run build
git add -A package.json package-lock.json src
git commit -m "feat: Tactile tokens from AIM 1.8.0, variable fonts and theme root" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: сборка проходит; в `dist/assets` есть только woff2 подмножеств latin и cyrillic.

---

### Task 7: Движение на Web Animations

**Files:**
- Create: `src/lib/motion.ts`, `src/lib/motion.test.ts`

**Interfaces:**
- Produces: `TACTILE: string`, `SETTLE: string` (CSS `linear()`), `settlePoints(perSegment?: number): [number, number][]`, `DUR = { micro: 160, short: 240, base: 380, long: 700 }`, `reduceMotion(): boolean`, Svelte actions `openWindow: Action<HTMLElement>`, `press: Action<HTMLElement>`, `rise: Action<HTMLElement, unknown>`, `countTo: Action<HTMLElement, number>`.

- [ ] **Step 1: Падающий тест `src/lib/motion.test.ts`**

```ts
import { describe, expect, it } from "vitest";
import { SETTLE, settlePoints } from "./motion";

describe("SETTLE", () => {
	it("starts at 0, ends at 1 and settles from a small overshoot", () => {
		const pts = settlePoints();
		expect(pts[0]).toEqual([0, 0]);
		const [x1, y1] = pts[pts.length - 1];
		expect(x1).toBeCloseTo(1);
		expect(y1).toBeCloseTo(1);
		const peak = Math.max(...pts.map(([, y]) => y));
		expect(peak).toBeCloseTo(1.02, 2);
	});

	it("time only moves forward", () => {
		const xs = settlePoints().map(([x]) => x);
		for (let i = 1; i < xs.length; i++) expect(xs[i]).toBeGreaterThan(xs[i - 1]);
	});

	it("is a CSS linear() easing", () => {
		expect(SETTLE).toMatch(/^linear\(0 0%, .+, 1 100%\)$/);
	});
});
```

Run: `npx vitest run src/lib/motion.test.ts` — Expected: FAIL, модуля нет.

- [ ] **Step 2: `src/lib/motion.ts`**

```ts
import type { Action } from "svelte/action";

/** Кривая Tactile по умолчанию (AIM: CustomEase "tactile"). */
export const TACTILE = "cubic-bezier(0.32, 0.72, 0, 1)";
export const DUR = { micro: 160, short: 240, base: 380, long: 700 } as const;
const SAFETY_MS = 2200;

type Pt = [number, number];
/** Кривая AIM "settle": M0,0 C0.18,0.9 0.3,1.04 0.52,1.02 0.7,1 0.84,1 1,1 — перелёт и осадка. */
const SETTLE_PATH: [Pt, Pt, Pt, Pt][] = [
	[[0, 0], [0.18, 0.9], [0.3, 1.04], [0.52, 1.02]],
	[[0.52, 1.02], [0.7, 1], [0.84, 1], [1, 1]]
];
const bez = (a: number, b: number, c: number, d: number, t: number) => {
	const u = 1 - t;
	return u * u * u * a + 3 * u * u * t * b + 3 * u * t * t * c + t * t * t * d;
};

/** Точки кривой settle для CSS `linear()`: Web Animations не умеют SVG-пути. */
export function settlePoints(perSegment = 12): Pt[] {
	const out: Pt[] = [];
	for (const [p0, p1, p2, p3] of SETTLE_PATH) {
		for (let i = out.length ? 1 : 0; i <= perSegment; i++) {
			const t = i / perSegment;
			out.push([bez(p0[0], p1[0], p2[0], p3[0], t), bez(p0[1], p1[1], p2[1], p3[1], t)]);
		}
	}
	return out;
}

export const SETTLE = `linear(${settlePoints()
	.map(([x, y]) => `${+y.toFixed(4)} ${+(x * 100).toFixed(2)}%`)
	.join(", ")})`;

export const reduceMotion = (): boolean => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

const fadeIn = (el: HTMLElement) => el.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 150, easing: "linear" });

/**
 * Появление экрана: корень оседает из 0.965 и 10 px снизу, зоны `[data-part]` всплывают с шагом 40 ms.
 * Страховка 2,2 s: в скрытом окне кадры не идут, и без неё экран остался бы прозрачным.
 */
export const openWindow: Action<HTMLElement> = (root) => {
	const anims: Animation[] = [];
	if (reduceMotion()) anims.push(fadeIn(root));
	else {
		anims.push(
			root.animate([{ opacity: 0, transform: "translateY(10px) scale(0.965)" }, { opacity: 1, transform: "none" }], { duration: DUR.long, easing: SETTLE })
		);
		root.querySelectorAll<HTMLElement>("[data-part]").forEach((el, i) => {
			anims.push(
				el.animate([{ opacity: 0, transform: "translateY(10px)" }, { opacity: 1, transform: "none" }], {
					duration: 450,
					delay: 120 + i * 40,
					easing: TACTILE,
					fill: "backwards"
				})
			);
		});
	}
	const safety = setTimeout(() => anims.forEach((a) => a.finish()), SAFETY_MS);
	return {
		destroy() {
			clearTimeout(safety);
			anims.forEach((a) => a.cancel());
		}
	};
};

/** Нажатие: кнопка уходит на 1 px и 0.96, отпускание возвращает её с осадкой. Клик не ждёт анимацию. */
export const press: Action<HTMLElement> = (el) => {
	let anim: Animation | undefined;
	const pressed = "translateY(1px) scale(0.96)";
	const down = () => {
		if (reduceMotion() || (el as HTMLButtonElement).disabled) return;
		anim?.cancel();
		anim = el.animate([{ transform: "none" }, { transform: pressed }], { duration: 120, easing: "ease-out", fill: "forwards" });
	};
	const up = () => {
		if (!anim) return;
		anim.cancel();
		anim = el.animate([{ transform: pressed }, { transform: "none" }], { duration: 450, easing: SETTLE });
		anim.onfinish = () => (anim = undefined);
	};
	el.addEventListener("pointerdown", down);
	for (const e of ["pointerup", "pointerleave", "pointercancel"]) el.addEventListener(e, up);
	return {
		destroy() {
			anim?.cancel();
			el.removeEventListener("pointerdown", down);
			for (const e of ["pointerup", "pointerleave", "pointercancel"]) el.removeEventListener(e, up);
		}
	};
};

/** Смена ключа (например, выбранного сервера): содержимое всплывает на 6 px. */
export const rise: Action<HTMLElement, unknown> = (el, key) => {
	let last = key;
	let anim: Animation | undefined;
	return {
		update(next) {
			if (next === last) return;
			last = next;
			anim?.cancel();
			anim = reduceMotion()
				? fadeIn(el)
				: el.animate([{ opacity: 0, transform: "translateY(6px)" }, { opacity: 1, transform: "none" }], { duration: DUR.base, easing: TACTILE });
		},
		destroy() {
			anim?.cancel();
		}
	};
};

/** Число досчитывается от старого значения за 700 ms; элемент без детей — текст пишет сам action. */
export const countTo: Action<HTMLElement, number> = (el, value) => {
	let shown = value;
	let raf = 0;
	let safety: ReturnType<typeof setTimeout> | undefined;
	const set = (v: number) => (el.textContent = String(Math.round(v)));
	set(value);
	const stop = () => {
		cancelAnimationFrame(raf);
		clearTimeout(safety);
	};
	return {
		update(next) {
			stop();
			const from = shown;
			shown = next;
			if (from === next || reduceMotion()) {
				set(next);
				return;
			}
			const t0 = performance.now();
			const step = (now: number) => {
				const k = Math.min(1, (now - t0) / DUR.long);
				set(from + (next - from) * (1 - (1 - k) ** 3));
				if (k < 1) raf = requestAnimationFrame(step);
			};
			raf = requestAnimationFrame(step);
			// в скрытом окне кадров нет — число всё равно встаёт на место
			safety = setTimeout(() => {
				cancelAnimationFrame(raf);
				set(next);
			}, DUR.long + 100);
		},
		destroy: stop
	};
};
```

- [ ] **Step 3: Гейты и коммит**

```bash
npx vitest run src/lib/motion.test.ts && npm test && npm run check
git add src/lib/motion.ts src/lib/motion.test.ts
git commit -m "feat: Tactile motion on Web Animations (settle, open, press, rise, count)" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Статичные примитивы, значки и витрина

**Files:**
- Create: `src/components/tactile/{Icon,Shell,Core,Tray,Label,Section,Button,IconButton,Primary,Pill,Chip,Field}.svelte`, `src/dev/Kit.svelte`
- Modify: `src/App.svelte`

**Interfaces:**
- Consumes: `press` (Task 7), `tip`/`TipData` (`tooltip.svelte.ts`, `tips.ts`).
- Produces (свойства; везде `...rest` уходит на корневой элемент):
  - `Icon { name: IconName; size?: number = 16 }`; `IconName = "app" | "chip" | "sliders" | "minimize" | "maximize" | "close" | "edit" | "plus" | "clock" | "enter" | "back" | "refresh" | "folder" | "trash"`.
  - `Shell { children; pad?: string = "16px"; fill?: boolean }` — кант + ядро. `Core { children; pad?: string = "16px"; fill?: boolean; scroll?: boolean }`. `Tray { children; pad?: string = "6px"; fill?: boolean; scroll?: boolean }`.
  - `Label { children; modified?: boolean; id?: string }`. `Section { title: string; children? }`.
  - `Button { children; danger?: boolean; armed?: boolean; tip?: TipData; type? }`. `IconButton { label: string; children; size?: 28 | 30; on?: boolean; danger?: boolean; tip?: TipData }`.
  - `Primary { children; size?: "sm" | "lg"; tone?: "accent" | "danger"; busy?: boolean; icon?: IconName = "enter"; type? }`.
  - `Pill { children; tone?: "accent" | "plain" | "warn" | "good" | "danger"; button?: boolean; tip?: TipData }`. `Chip { children; button?: boolean; on?: boolean; tip?: TipData }`.
  - `Field { label: string; value?: string (bindable); error?: string | null; hint?: string; mono?: boolean; end?: Snippet }` + атрибуты `<input>`.

- [ ] **Step 1: `Icon.svelte`**

```svelte
<script lang="ts" module>
	export type IconName = "app" | "chip" | "sliders" | "minimize" | "maximize" | "close" | "edit" | "plus" | "clock" | "enter" | "back" | "refresh" | "folder" | "trash";

	/** Тонкие линейные значки 16×16 в духе Phosphor Light — вне Foundry значков Font Awesome нет. */
	const PATHS: Record<IconName, string> = {
		app: "M8 2l5.2 3v6L8 14l-5.2-3V5zM5 10.5h6L8 5z",
		chip: "M5.5 4h5A1.5 1.5 0 0 1 12 5.5v5a1.5 1.5 0 0 1-1.5 1.5h-5A1.5 1.5 0 0 1 4 10.5v-5A1.5 1.5 0 0 1 5.5 4zM6 1.5v2M10 1.5v2M6 12.5v2M10 12.5v2M1.5 6h2M1.5 10h2M12.5 6h2M12.5 10h2",
		sliders: "M2.5 4.5h6M12 4.5h1.5M2.5 11.5h1.5M7.5 11.5h6M11.8 4.5a1.6 1.6 0 1 1-3.2 0a1.6 1.6 0 1 1 3.2 0M7.4 11.5a1.6 1.6 0 1 1-3.2 0a1.6 1.6 0 1 1 3.2 0",
		minimize: "M4 8h8",
		maximize: "M5.5 4.5h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1z",
		close: "M4.5 4.5l7 7M11.5 4.5l-7 7",
		edit: "M10.5 2.5l3 3L6 13H3v-3zM9 4l3 3",
		plus: "M8 3v10M3 8h10",
		clock: "M13.5 8a5.5 5.5 0 1 1-11 0a5.5 5.5 0 1 1 11 0M8 5v3.2l2 1.3",
		enter: "M12.5 3.5v4a2 2 0 0 1-2 2h-7M6 6.5l-3 3 3 3",
		back: "M10 3.5L5.5 8l4.5 4.5",
		refresh: "M12.8 8.6A4.9 4.9 0 1 1 11.3 4.2M12.2 1.8v3h-3",
		folder: "M2 4.5A1.5 1.5 0 0 1 3.5 3h2.8l1.5 1.5h4.7A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z",
		trash: "M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.7 8.5h5.6l.7-8.5"
	};
</script>

<script lang="ts">
	let { name, size = 16 }: { name: IconName; size?: number } = $props();
</script>

<svg class="ic" viewBox="0 0 16 16" width={size} height={size} aria-hidden="true"><path d={PATHS[name]} /></svg>

<style>
	.ic {
		flex: none;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
```

- [ ] **Step 2: `Shell.svelte`, `Core.svelte`, `Tray.svelte`**

`Shell.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = HTMLAttributes<HTMLDivElement> & { children: Snippet; pad?: string; fill?: boolean };
	let { children, pad = "16px", fill = false, ...rest }: Props = $props();
</script>

<!-- двойной кант: утопленный лоток вокруг поднятой пластины; радиусы концентричны -->
<div class="shell" class:fill {...rest}>
	<div class="core" style:padding={pad}>{@render children()}</div>
</div>

<style>
	.shell {
		min-width: 0;
		min-height: 0;
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		grid-template-rows: minmax(0, 1fr);
		padding: 6px;
		border-radius: calc(var(--tc-r-block) + 6px);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
	}
	.fill {
		height: 100%;
	}
	.core {
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		border-radius: var(--tc-r-block);
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
	}
</style>
```

`Core.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = HTMLAttributes<HTMLDivElement> & { children: Snippet; pad?: string; fill?: boolean; scroll?: boolean };
	let { children, pad = "16px", fill = false, scroll = false, ...rest }: Props = $props();
</script>

<div class="core" class:fill class:scroll style:padding={pad} {...rest}>{@render children()}</div>

<style>
	.core {
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		border-radius: var(--tc-r-block);
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
	}
	.fill {
		height: 100%;
	}
	.scroll {
		overflow-y: auto;
	}
</style>
```

`Tray.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";

	type Props = HTMLAttributes<HTMLDivElement> & { children: Snippet; pad?: string; fill?: boolean; scroll?: boolean };
	let { children, pad = "6px", fill = false, scroll = false, ...rest }: Props = $props();
</script>

<!-- лоток для списков: внутри плоские строки, выступает только выбранная -->
<div class="tray" class:fill class:scroll style:padding={pad} {...rest}>{@render children()}</div>

<style>
	.tray {
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		border-radius: var(--tc-r-block);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
	}
	.fill {
		height: 100%;
	}
	.scroll {
		overflow-y: auto;
	}
</style>
```

- [ ] **Step 3: `Label.svelte`, `Section.svelte`**

`Label.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";

	let { children, modified = false, id }: { children: Snippet; modified?: boolean; id?: string } = $props();
</script>

<span class="lbl" {id}>{@render children()}{#if modified}<i class="mod" aria-hidden="true"></i>{/if}</span>

<style>
	.lbl {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		white-space: nowrap;
		font: 500 10.5px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.mod {
		width: 6px;
		height: 6px;
		border-radius: 99px;
		background: var(--tc-accent);
		box-shadow: 0 0 6px var(--tc-glow);
	}
</style>
```

`Section.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import Label from "./Label.svelte";

	let { title, children }: { title: string; children?: Snippet } = $props();
</script>

<section class="sec">
	<div class="head"><Label>{title}</Label></div>
	{#if children}<div class="body">{@render children()}</div>{/if}
</section>

<style>
	.sec {
		display: grid;
		gap: 12px;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.head::after {
		content: "";
		flex: 1;
		height: 1px;
		background: var(--tc-line);
	}
	.body {
		display: grid;
		gap: 12px;
	}
</style>
```

- [ ] **Step 4: `Button.svelte`, `IconButton.svelte`**

`Button.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { press } from "../../lib/motion";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = HTMLButtonAttributes & { children: Snippet; danger?: boolean; armed?: boolean; tip?: TipData };
	let { children, danger = false, armed = false, tip, type = "button", ...rest }: Props = $props();
</script>

<button {type} class="btn" class:danger class:armed use:press {@attach tooltip(() => tip)} {...rest}>{@render children()}</button>

<style>
	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		height: 34px;
		padding: 0 14px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
		font-size: 12.5px;
		font-weight: 600;
		white-space: nowrap;
		transition: color 160ms var(--tc-ease);
	}
	.btn:hover:not(:disabled) {
		color: var(--tc-accent-text);
	}
	.danger:hover:not(:disabled),
	.armed {
		color: var(--tc-danger);
	}
	.btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.btn[aria-busy="true"] {
		cursor: progress;
	}
</style>
```

`IconButton.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { press } from "../../lib/motion";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = HTMLButtonAttributes & { label: string; children: Snippet; size?: 28 | 30; on?: boolean; danger?: boolean; tip?: TipData };
	let { label, children, size = 28, on = false, danger = false, tip, ...rest }: Props = $props();
</script>

<button type="button" class="ib" class:on class:danger style:--s={`${size}px`} aria-label={label} use:press {@attach tooltip(() => tip)} {...rest}>
	{@render children()}
</button>

<style>
	.ib {
		width: var(--s);
		height: var(--s);
		display: grid;
		place-items: center;
		flex: none;
		border-radius: var(--tc-r-ctl);
		color: var(--tc-muted);
		transition:
			color 160ms var(--tc-ease),
			box-shadow 160ms var(--tc-ease);
	}
	.ib:hover {
		color: var(--tc-ink);
		box-shadow: var(--tc-raise);
	}
	.on {
		color: var(--tc-accent-text);
	}
	.danger:hover {
		color: var(--tc-danger);
	}
</style>
```

- [ ] **Step 5: `Primary.svelte`**

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { press } from "../../lib/motion";
	import Icon, { type IconName } from "./Icon.svelte";

	type Props = HTMLButtonAttributes & { children: Snippet; size?: "sm" | "lg"; tone?: "accent" | "danger"; busy?: boolean; icon?: IconName };
	let { children, size = "sm", tone = "accent", busy = false, icon = "enter", type = "button", disabled, ...rest }: Props = $props();
</script>

<!-- главное действие: акцентная капсула со вложенной лункой справа (кнопка в кнопке) -->
<button {type} class="primary {size} {tone}" aria-busy={busy} disabled={disabled || busy} use:press {...rest}>
	<span class="text">{@render children()}</span>
	<span class="nest" aria-hidden="true"><Icon name={icon} size={size === "lg" ? 20 : 14} /></span>
</button>

<style>
	.primary {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-accent);
		color: var(--tc-on-accent);
		font-weight: 700;
		white-space: nowrap;
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 40%),
			0 8px 18px -8px var(--tc-glow);
		transition:
			box-shadow 240ms var(--tc-ease),
			opacity 160ms var(--tc-ease);
	}
	.sm {
		height: 34px;
		padding: 0 3px 0 16px;
		font-size: 12.5px;
	}
	.lg {
		width: 100%;
		height: 58px;
		padding: 0 5px 0 26px;
		font-size: 17px;
		letter-spacing: 0.08em;
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 40%),
			0 14px 28px -12px var(--tc-glow);
	}
	.text {
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.nest {
		display: grid;
		place-items: center;
		flex: none;
		border-radius: var(--tc-r-ctl);
		background: rgb(255 255 255 / 28%);
	}
	.sm .nest {
		width: 28px;
		height: 28px;
	}
	.lg .nest {
		width: 48px;
		height: 48px;
	}
	.danger {
		background: var(--tc-danger);
		color: #fff;
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 30%),
			0 8px 18px -8px color-mix(in oklab, var(--tc-danger) 60%, transparent);
	}
	:global(.tc-root[data-theme="dark"]) .danger {
		color: var(--tc-sunken);
	}
	.primary:disabled {
		opacity: 0.45;
		box-shadow: none;
		cursor: not-allowed;
	}
	.primary[aria-busy="true"] {
		opacity: 0.75;
		cursor: progress;
	}
</style>
```

- [ ] **Step 6: `Pill.svelte`, `Chip.svelte`**

`Pill.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = HTMLAttributes<HTMLElement> & { children: Snippet; tone?: "accent" | "plain" | "warn" | "good" | "danger"; button?: boolean; tip?: TipData };
	let { children, tone = "accent", button = false, tip, ...rest }: Props = $props();
</script>

<svelte:element this={button ? "button" : "span"} type={button ? "button" : undefined} class="pill {tone}" {@attach tooltip(() => tip)} {...rest}>
	{@render children()}
</svelte:element>

<style>
	.pill {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		height: 18px;
		padding: 0 7px;
		flex: none;
		border-radius: 99px;
		font: 600 9.5px/1 var(--tc-font-mono);
		letter-spacing: 0.05em;
		text-transform: uppercase;
		white-space: nowrap;
	}
	.accent {
		background: var(--tc-accent-soft);
		color: var(--tc-accent-text);
	}
	.plain {
		color: var(--tc-muted);
		box-shadow: inset 0 0 0 1px var(--tc-line);
	}
	.warn {
		background: color-mix(in oklab, var(--tc-warning) 14%, transparent);
		color: var(--tc-warning);
	}
	.good {
		background: color-mix(in oklab, var(--tc-good) 14%, transparent);
		color: var(--tc-good);
	}
	.danger {
		background: color-mix(in oklab, var(--tc-danger) 14%, transparent);
		color: var(--tc-danger);
	}
	button.pill {
		height: 22px;
		padding: 0 9px;
		transition: box-shadow 160ms var(--tc-ease);
	}
	button.pill:hover {
		box-shadow: var(--tc-raise);
	}
</style>
```

`Chip.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = HTMLAttributes<HTMLElement> & { children: Snippet; button?: boolean; on?: boolean; tip?: TipData };
	let { children, button = false, on = false, tip, ...rest }: Props = $props();
</script>

<svelte:element this={button ? "button" : "span"} type={button ? "button" : undefined} class="chip" class:on {@attach tooltip(() => tip)} {...rest}>
	{@render children()}
</svelte:element>

<style>
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		height: 20px;
		padding: 0 7px;
		flex: none;
		border-radius: 99px;
		background: var(--tc-sunken);
		color: var(--tc-muted);
		font: 500 10px/1 var(--tc-font-mono);
		white-space: nowrap;
		transition: color 160ms var(--tc-ease);
	}
	button.chip:hover {
		color: var(--tc-ink);
	}
	.on {
		background: var(--tc-accent-soft);
		color: var(--tc-accent-text);
	}
</style>
```

- [ ] **Step 7: `Field.svelte`**

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import type { HTMLInputAttributes } from "svelte/elements";

	type Props = Omit<HTMLInputAttributes, "value"> & { label: string; value?: string; error?: string | null; hint?: string; mono?: boolean; end?: Snippet };
	let { label, value = $bindable(""), error = null, hint, mono = false, end, ...rest }: Props = $props();
	const id = $props.id();
	const note = $derived(error || hint ? `${id}-note` : undefined);
</script>

<div class="field">
	<label class="lbl" for={id}>{label}</label>
	<div class="row">
		<span class="cap" class:bad={Boolean(error)}>
			<input {id} class:mono bind:value aria-invalid={Boolean(error)} aria-describedby={note} {...rest} />
		</span>
		{#if end}{@render end()}{/if}
	</div>
	{#if error}<span class="note bad" id={note} role="alert">{error}</span>{:else if hint}<span class="note" id={note}>{hint}</span>{/if}
</div>

<style>
	.field {
		display: grid;
		gap: 6px;
		min-width: 0;
	}
	.lbl {
		font: 500 10.5px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}
	.cap {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: center;
		height: 34px;
		padding: 0 14px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		transition: box-shadow 160ms var(--tc-ease);
	}
	.cap:focus-within {
		box-shadow:
			var(--tc-press),
			0 0 0 2px var(--tc-accent);
	}
	.cap.bad {
		box-shadow:
			var(--tc-press),
			0 0 0 1.5px var(--tc-danger);
	}
	input {
		flex: 1;
		min-width: 0;
		padding: 0;
		border: 0;
		background: none;
		outline: none;
		font-size: 13px;
		color: var(--tc-ink);
	}
	input:focus-visible {
		outline: none;
	}
	input::placeholder {
		color: var(--tc-muted);
	}
	input:disabled {
		opacity: 0.6;
	}
	.mono {
		font-family: var(--tc-font-mono);
		font-size: 12.5px;
	}
	.note {
		font: 500 11px/1.35 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.note.bad {
		color: var(--tc-danger);
	}
</style>
```

- [ ] **Step 8: Витрина `src/dev/Kit.svelte` и подключение**

```svelte
<script lang="ts">
	import Button from "../components/tactile/Button.svelte";
	import Chip from "../components/tactile/Chip.svelte";
	import Core from "../components/tactile/Core.svelte";
	import Field from "../components/tactile/Field.svelte";
	import Icon from "../components/tactile/Icon.svelte";
	import IconButton from "../components/tactile/IconButton.svelte";
	import Label from "../components/tactile/Label.svelte";
	import Pill from "../components/tactile/Pill.svelte";
	import Primary from "../components/tactile/Primary.svelte";
	import Section from "../components/tactile/Section.svelte";
	import Shell from "../components/tactile/Shell.svelte";
	import Tray from "../components/tactile/Tray.svelte";

	let text = $state("Проклятие Страда");
</script>

<!-- Витрина примитивов для превью: http://localhost:1420/?kit (только dev) -->
<div class="kit">
	<Shell pad="18px">
		<Section title="Кнопки">
			<div class="row">
				<Button>Обычная</Button>
				<Button danger>Удалить</Button>
				<Button disabled>Отключена</Button>
				<IconButton label="Изменить"><Icon name="edit" /></IconButton>
				<IconButton label="Настройка" size={30} on><Icon name="sliders" /></IconButton>
				<Primary>Сохранить</Primary>
				<Primary tone="danger" icon="trash">Удалить</Primary>
				<Primary busy>Занято</Primary>
			</div>
			<Primary size="lg">ЗАПУСК</Primary>
		</Section>
		<Section title="Метки">
			<div class="row">
				<Label>Профиль</Label>
				<Label modified>Изменено</Label>
				<Pill>accent</Pill><Pill tone="plain">plain</Pill><Pill tone="warn">warn</Pill><Pill tone="good">онлайн</Pill><Pill tone="danger">не отвечает</Pill>
				<Chip>v0.2.2</Chip><Chip button on>60</Chip>
			</div>
		</Section>
		<Section title="Поля">
			<Field label="Название" bind:value={text} />
			<Field label="Адрес" mono value="vtt.example.com" error="Не похоже на адрес" />
		</Section>
	</Shell>
	<Tray pad="10px"><Label>Лоток</Label></Tray>
	<Core><Label>Пластина</Label></Core>
</div>

<style>
	.kit {
		height: 100%;
		overflow-y: auto;
		display: grid;
		gap: 14px;
		align-content: start;
		padding: 2px 14px 14px;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		align-items: center;
	}
</style>
```

В `App.svelte` импорт `import Kit from "./dev/Kit.svelte";`, константа `const kit = import.meta.env.DEV && new URLSearchParams(location.search).has("kit");`, а в `<main class="screen">` первой веткой — `{#if kit}<Kit />{:else if …}` (остальные ветки без изменений).

- [ ] **Step 9: Гейты и коммит**

```bash
npm test && npm run check && npm run build
git add src/components/tactile src/dev src/App.svelte
git commit -m "feat: Tactile primitives — shell, tray, buttons, primary, pills, field" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Интерактивные примитивы и подсказка

**Files:**
- Create: `src/components/tactile/{Segments,Toggle,Slider,Band,Ticks,Select,Swatches}.svelte`
- Modify: `src/components/Tooltip.svelte`, `src/lib/tips.ts`, `src/lib/tooltip.svelte.ts`, `src/dev/Kit.svelte`

**Interfaces:**
- Consumes: `nextIndex` (Task 5), `ACCENTS` (Task 5), `TACTILE`/`reduceMotion`/`press` (Task 7), `Label`/`Chip` (Task 8).
- Produces:
  - `Segments<T extends string | number> { label: string; options: SegOption<T>[]; value: T; onchange: (v: T) => void; showLabel?: boolean = true; modified?: boolean; tip?: TipData }`; `SegOption<V> { value: V; label: string; short?: string; arrowSkip?: boolean; disabled?: boolean }` (экспорт из module-скрипта).
  - `Toggle { label: string; checked: boolean; onchange: (v: boolean) => void; modified?; tip?; danger?; disabled? }`.
  - `Slider { label; value; min; max; step; format: (v: number) => string; onchange; modified?; tip?; ticks?: number[]; presets?: number[]; editable?; inputLabel? }`.
  - `Band { value: number (0..1); ghost?: number | null; stops?: number[]; label?: string; animate?: boolean = true }`.
  - `Ticks { items: { at: number; label: string }[] }`.
  - `Select<T> { label; options: { value: T; label: string }[]; value: T; onchange; tip? }`.
  - `Swatches { label: string; value: AccentId; onchange: (v: AccentId) => void }`.
  - `TipData.checks?: { ok: boolean; text: string }[]`.

- [ ] **Step 1: `Segments.svelte`**

```svelte
<script lang="ts" module>
	export interface SegOption<V> {
		value: V;
		label: string;
		/** Короткая подпись неактивного сегмента; у активного — полная. */
		short?: string;
		/** Стрелки пропускают пункт: его выбирают только мышью. */
		arrowSkip?: boolean;
		disabled?: boolean;
	}
</script>

<script lang="ts" generics="T extends string | number">
	import { nextIndex } from "../../lib/radio";
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";
	import Label from "./Label.svelte";

	type Props = { label: string; options: SegOption<T>[]; value: T; onchange: (v: T) => void; showLabel?: boolean; modified?: boolean; tip?: TipData };
	let { label, options, value, onchange, showLabel = true, modified = false, tip }: Props = $props();

	const btns: HTMLButtonElement[] = [];
	const current = $derived(options.findIndex((o) => o.value === value));
	const focusable = $derived(current >= 0 ? current : options.findIndex((o) => !o.disabled));
	const wide = $derived(options.some((o) => o.short));

	function onkey(e: KeyboardEvent, i: number) {
		const n = nextIndex(i, e.key, options.map((o) => !o.disabled && !o.arrowSkip));
		if (n === null) return;
		e.preventDefault();
		onchange(options[n].value);
		btns[n]?.focus();
	}
</script>

<div class="wrap" {@attach tooltip(() => tip)}>
	{#if showLabel}<Label {modified}>{label}</Label>{/if}
	<div class="segs" class:wide role="radiogroup" aria-label={label}>
		{#each options as o, i (o.value)}
			<button
				type="button"
				class="seg"
				class:on={i === current}
				role="radio"
				aria-checked={i === current}
				tabindex={i === focusable ? 0 : -1}
				disabled={o.disabled}
				bind:this={btns[i]}
				onclick={() => onchange(o.value)}
				onkeydown={(e) => onkey(e, i)}
			>
				{i === current || !o.short ? o.label : o.short}
			</button>
		{/each}
	</div>
</div>

<style>
	.wrap {
		display: grid;
		gap: 8px;
		min-width: 0;
	}
	.segs {
		display: flex;
		gap: 2px;
		min-width: 0;
		padding: 3px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
	}
	.seg {
		flex: 1 1 0;
		min-width: 0;
		height: 30px;
		padding: 0 10px;
		overflow: hidden;
		border-radius: var(--tc-r-ctl);
		color: var(--tc-muted);
		font-size: 12.5px;
		font-weight: 600;
		white-space: nowrap;
		text-overflow: ellipsis;
		transition:
			color 160ms var(--tc-ease),
			box-shadow 240ms var(--tc-ease),
			background-color 240ms var(--tc-ease);
	}
	.seg:hover:not(:disabled) {
		color: var(--tc-ink);
	}
	.seg.on {
		background: var(--tc-surface);
		color: var(--tc-ink);
		box-shadow: var(--tc-raise);
	}
	.seg:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.wide .seg {
		font: 600 11.5px/1 var(--tc-font-mono);
		letter-spacing: 0.05em;
	}
	.wide .seg.on {
		flex: 1.7 1 0;
		font: 600 13px/1 var(--tc-font-ui);
		letter-spacing: 0;
	}
</style>
```

- [ ] **Step 2: `Toggle.svelte`**

```svelte
<script lang="ts">
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = { label: string; checked: boolean; onchange: (v: boolean) => void; modified?: boolean; tip?: TipData; danger?: boolean; disabled?: boolean };
	let { label, checked, onchange, modified = false, tip, danger = false, disabled = false }: Props = $props();
</script>

<!-- тумблер: утопленный желоб, поднятая шайба; включён — желоб заливается акцентом -->
<button type="button" class="tg" class:danger role="switch" aria-checked={checked} {disabled} {@attach tooltip(() => tip)} onclick={() => onchange(!checked)}>
	<span class="text">{label}{#if modified}<i class="mod" aria-hidden="true"></i>{/if}</span>
	<span class="track" class:on={checked} aria-hidden="true"><span class="puck"></span></span>
</button>

<style>
	.tg {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		width: 100%;
		min-height: 34px;
		padding: 4px 4px 4px 0;
		text-align: left;
		font-size: 13px;
	}
	.text {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}
	.mod {
		width: 6px;
		height: 6px;
		flex: none;
		border-radius: 99px;
		background: var(--tc-accent);
		box-shadow: 0 0 6px var(--tc-glow);
	}
	.track {
		position: relative;
		flex: none;
		width: 36px;
		height: 20px;
		border-radius: 99px;
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		transition:
			background-color 240ms var(--tc-ease),
			box-shadow 240ms var(--tc-ease);
	}
	.puck {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 16px;
		height: 16px;
		border-radius: 99px;
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
		transition: transform 240ms var(--tc-ease);
	}
	.on {
		background: var(--tc-accent);
		box-shadow:
			inset 0 1px 2px rgb(0 0 0 / 15%),
			0 0 10px -2px var(--tc-glow);
	}
	.on .puck {
		transform: translateX(16px);
	}
	.danger .on {
		background: var(--tc-danger);
		box-shadow: inset 0 1px 2px rgb(0 0 0 / 15%);
	}
	.tg:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
</style>
```

- [ ] **Step 3: `Band.svelte`, `Ticks.svelte`**

`Band.svelte`:

```svelte
<script lang="ts">
	import { reduceMotion, TACTILE } from "../../lib/motion";

	type Props = { value: number; ghost?: number | null; stops?: number[]; label?: string; animate?: boolean };
	let { value, ghost = null, stops = [], label, animate = true }: Props = $props();

	const pct = (v: number) => `${Math.max(0, Math.min(1, v)) * 100}%`;

	/** Шкала доливается по ширине за 350 ms, как в AIM. */
	function fill(el: HTMLElement, v: number) {
		let cur = v;
		return {
			update(next: number) {
				if (next === cur) return;
				const from = pct(cur);
				cur = next;
				if (animate && !reduceMotion()) el.animate([{ width: from }, { width: pct(next) }], { duration: 350, easing: TACTILE });
			}
		};
	}
</script>

<div
	class="band"
	role={label ? "meter" : undefined}
	aria-label={label}
	aria-valuemin={label ? 0 : undefined}
	aria-valuemax={label ? 100 : undefined}
	aria-valuenow={label ? Math.round(Math.max(0, Math.min(1, value)) * 100) : undefined}
>
	{#if ghost !== null}<span class="ghost" style:width={pct(ghost)}></span>{/if}
	<span class="fill" style:width={pct(value)} use:fill={value}></span>
	{#each stops as s (s)}<span class="stop" style:left={pct(s)}></span>{/each}
</div>

<style>
	.band {
		position: relative;
		height: 6px;
		overflow: hidden;
		border-radius: 99px;
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
	}
	.fill,
	.ghost {
		position: absolute;
		left: 0;
		top: 0;
		height: 100%;
		border-radius: 99px;
		background: var(--tc-accent);
		box-shadow: 0 0 10px var(--tc-glow);
	}
	.ghost {
		background: var(--tc-muted);
		box-shadow: none;
		opacity: 0.35;
	}
	.stop {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		margin-left: -1px;
		background: var(--tc-surface);
	}
</style>
```

`Ticks.svelte`:

```svelte
<script lang="ts">
	let { items }: { items: { at: number; label: string }[] } = $props();
	const shift = (at: number) => (at <= 0 ? "none" : at >= 1 ? "translateX(-100%)" : "translateX(-50%)");
</script>

<div class="ticks" aria-hidden="true">
	{#each items as it (it.label)}<span style:left={`${it.at * 100}%`} style:transform={shift(it.at)}>{it.label}</span>{/each}
</div>

<style>
	.ticks {
		position: relative;
		height: 11px;
		font: 500 9.5px/1 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.ticks span {
		position: absolute;
		top: 0;
	}
</style>
```

- [ ] **Step 4: `Slider.svelte`**

```svelte
<script lang="ts">
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";
	import Band from "./Band.svelte";
	import Chip from "./Chip.svelte";
	import Label from "./Label.svelte";
	import Ticks from "./Ticks.svelte";

	type Props = {
		label: string;
		value: number;
		min: number;
		max: number;
		step: number;
		format: (v: number) => string;
		onchange: (v: number) => void;
		modified?: boolean;
		tip?: TipData;
		/** Подписи под шкалой. */
		ticks?: number[];
		/** Быстрые значения под шкалой. */
		presets?: number[];
		/** Точный ввод числа вместо подписи значения. */
		editable?: boolean;
		inputLabel?: string;
	};
	let { label, value, min, max, step, format, onchange, modified = false, tip, ticks = [], presets = [], editable = false, inputLabel = "" }: Props = $props();

	const id = $props.id();
	// Пока ручку тянут, значение только показывается; сохраняется на `change`.
	let live = $derived(value);
	const frac = (v: number) => (v - min) / (max - min);
	const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v / step) * step));

	function commitTyped(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		const n = Number(input.value);
		if (!Number.isFinite(n) || input.value.trim() === "") {
			input.value = String(live);
			return;
		}
		live = Math.min(max, Math.max(min, Math.round(n)));
		input.value = String(live);
		onchange(live);
	}
</script>

<div class="slider" {@attach tooltip(() => tip)}>
	<div class="head">
		<Label {modified} id={`${id}-l`}>{label}</Label>
		{#if editable}
			<input
				class="readout"
				type="number"
				inputmode="numeric"
				{min}
				{max}
				value={live}
				aria-label={inputLabel || label}
				onchange={commitTyped}
				onkeydown={(e) => e.key === "Enter" && e.currentTarget.blur()}
			/>
		{:else}
			<span class="val">{format(live)}</span>
		{/if}
	</div>
	<div class="track">
		<Band value={frac(live)} animate={false} />
		<span class="thumb" style:left={`${frac(live) * 100}%`} aria-hidden="true"></span>
		<input
			type="range"
			{min}
			{max}
			{step}
			value={live}
			aria-labelledby={`${id}-l`}
			aria-valuetext={format(live)}
			oninput={(e) => (live = Number(e.currentTarget.value))}
			onchange={(e) => onchange(clamp(Number(e.currentTarget.value)))}
		/>
	</div>
	{#if ticks.length}<Ticks items={ticks.map((v) => ({ at: frac(v), label: String(v) }))} />{/if}
	{#if presets.length}
		<div class="presets">
			{#each presets as p (p)}
				<Chip
					button
					on={live === p}
					onclick={() => {
						live = p;
						onchange(p);
					}}>{p}</Chip
				>
			{/each}
		</div>
	{/if}
</div>

<style>
	.slider {
		display: grid;
		gap: 8px;
		min-width: 0;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		min-height: 24px;
	}
	.val,
	.readout {
		font: 600 12.5px/1 var(--tc-font-mono);
		color: var(--tc-ink);
	}
	.readout {
		width: 60px;
		height: 24px;
		padding: 0 10px;
		border: 0;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		text-align: right;
	}
	.track {
		position: relative;
		display: grid;
		align-items: center;
		height: 16px;
	}
	.thumb {
		position: absolute;
		top: 0;
		width: 16px;
		height: 16px;
		margin-left: -8px;
		border-radius: 99px;
		background: var(--tc-surface);
		box-shadow: var(--tc-raise);
		pointer-events: none;
	}
	input[type="range"] {
		position: absolute;
		inset: 0;
		width: 100%;
		margin: 0;
		opacity: 0;
		cursor: pointer;
	}
	.track:has(input:focus-visible) .thumb {
		outline: 2px solid var(--tc-accent);
		outline-offset: 2px;
	}
	.presets {
		display: flex;
		gap: 4px;
	}
</style>
```

- [ ] **Step 5: `Select.svelte`**

```svelte
<script lang="ts" generics="T extends string | number">
	import type { TipData } from "../../lib/tips";
	import { tip as tooltip } from "../../lib/tooltip.svelte";

	type Props = { label: string; options: { value: T; label: string }[]; value: T; onchange: (v: T) => void; tip?: TipData };
	let { label, options, value, onchange, tip }: Props = $props();
	const id = $props.id();
</script>

<div class="sel" {@attach tooltip(() => tip)}>
	<label class="lbl" for={id}>{label}</label>
	<select {id} onchange={(e) => onchange(options[e.currentTarget.selectedIndex].value)}>
		{#each options as o (o.value)}<option value={String(o.value)} selected={o.value === value}>{o.label}</option>{/each}
	</select>
</div>

<style>
	.sel {
		display: grid;
		gap: 8px;
		min-width: 0;
	}
	.lbl {
		font: 500 10.5px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	select {
		height: 34px;
		padding: 0 30px 0 14px;
		border: 0;
		border-radius: var(--tc-r-ctl);
		appearance: none;
		cursor: pointer;
		font-size: 12.5px;
		font-weight: 600;
		background-color: var(--tc-surface);
		box-shadow: var(--tc-raise);
		background-image:
			linear-gradient(45deg, transparent 50%, var(--tc-muted) 50%),
			linear-gradient(135deg, var(--tc-muted) 50%, transparent 50%);
		background-position:
			right 16px center,
			right 11px center;
		background-size: 5px 5px;
		background-repeat: no-repeat;
	}
</style>
```

- [ ] **Step 6: `Swatches.svelte`**

```svelte
<script lang="ts">
	import { t } from "../../lib/i18n.svelte";
	import { ACCENTS } from "../../lib/palette";
	import { nextIndex } from "../../lib/radio";
	import { tip as tooltip } from "../../lib/tooltip.svelte";
	import type { AccentId } from "../../lib/types";
	import Label from "./Label.svelte";

	let { label, value, onchange }: { label: string; value: AccentId; onchange: (v: AccentId) => void } = $props();
	const btns: HTMLButtonElement[] = [];

	function onkey(e: KeyboardEvent, i: number) {
		const n = nextIndex(i, e.key, ACCENTS.map(() => true));
		if (n === null) return;
		e.preventDefault();
		onchange(ACCENTS[n].id);
		btns[n]?.focus();
	}
</script>

<div class="wrap">
	<Label>{label}</Label>
	<div class="sw" role="radiogroup" aria-label={label}>
		{#each ACCENTS as a, i (a.id)}
			<button
				type="button"
				class="dot"
				class:on={a.id === value}
				role="radio"
				aria-checked={a.id === value}
				aria-label={t(`accent.${a.id}`)}
				tabindex={a.id === value ? 0 : -1}
				style={`--h: ${a.h}; --c: ${a.c}`}
				bind:this={btns[i]}
				{@attach tooltip(() => ({ title: t(`accent.${a.id}`), body: "" }))}
				onclick={() => onchange(a.id)}
				onkeydown={(e) => onkey(e, i)}
			></button>
		{/each}
	</div>
</div>

<style>
	.wrap {
		display: grid;
		gap: 8px;
	}
	.sw {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
	}
	.dot {
		width: 24px;
		height: 24px;
		border-radius: 99px;
		background: oklch(0.73 var(--c) var(--h));
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 40%),
			var(--tc-raise);
		transition:
			box-shadow 160ms var(--tc-ease),
			transform 160ms var(--tc-ease);
	}
	:global(.tc-root[data-theme="dark"]) .dot {
		background: oklch(0.78 var(--c) var(--h));
	}
	.dot:hover {
		transform: translateY(-1px);
	}
	.dot.on {
		box-shadow:
			0 0 0 2px var(--tc-surface),
			0 0 0 4px var(--tc-ink);
	}
</style>
```

- [ ] **Step 7: Строки проверок в подсказке**

`src/lib/tips.ts`, в `interface TipData` после `lines?`:

```ts
	/** Проверки с точкой статуса: зелёная — да, красная — нет (значок видеокарты). */
	checks?: { ok: boolean; text: string }[];
```

`src/lib/tooltip.svelte.ts`, в `sameTip` добавить сравнение:

```ts
	a.lines?.join("\n") === b.lines?.join("\n") &&
	JSON.stringify(a.checks) === JSON.stringify(b.checks);
```

(заменить им прежний хвост выражения `a.lines?.join("\n") === b.lines?.join("\n");`).

- [ ] **Step 8: `Tooltip.svelte` — плашка Tactile**

Скрипт не меняется. В разметке после блока `{#if d.lines?.length}…{/if}` добавить:

```svelte
		{#if d.checks?.length}
			<ul class="checks">
				{#each d.checks as c, i (i)}<li><i class:ok={c.ok} aria-hidden="true"></i>{c.text}</li>{/each}
			</ul>
		{/if}
```

Блок `<style>` заменить целиком:

```css
	.tip {
		position: fixed;
		z-index: 100;
		display: grid;
		gap: 6px;
		max-width: 300px;
		padding: 10px 12px;
		border-radius: var(--tc-r-block);
		background: var(--tc-surface);
		box-shadow:
			var(--tc-raise),
			0 18px 40px -16px rgb(0 0 0 / 40%);
		color: var(--tc-ink);
		font-size: 12.5px;
		line-height: 1.4;
		pointer-events: none;
		animation: tip-in 160ms var(--tc-ease);
	}
	b {
		font-weight: 600;
	}
	p {
		color: var(--tc-muted);
	}
	ul {
		display: grid;
		gap: 3px;
		list-style: none;
		color: var(--tc-muted);
	}
	ul:not(.checks) li::before {
		content: "– ";
	}
	.checks {
		color: var(--tc-ink);
	}
	.checks li {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.checks i {
		width: 7px;
		height: 7px;
		flex: none;
		border-radius: 99px;
		background: var(--tc-danger);
	}
	.checks i.ok {
		background: var(--tc-good);
	}
	.meters {
		display: flex;
		gap: 14px;
		margin-top: 2px;
		font: 500 9.5px/1 var(--tc-font-mono);
		letter-spacing: 0.06em;
		color: var(--tc-muted);
	}
	.meter {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.bars {
		display: inline-flex;
		gap: 2px;
	}
	.bars i {
		width: 5px;
		height: 10px;
		border-radius: 2px;
		background: var(--tc-sunken);
	}
	.bars.fps i.on {
		background: var(--tc-accent);
	}
	.bars.look i.on {
		background: var(--tc-muted);
	}
	@keyframes tip-in {
		from {
			opacity: 0;
			transform: translateY(2px);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.tip {
			animation: none;
		}
	}
```

Если в старой разметке у `.tip` были классы сторон (`top`, `bottom`) и переменная `--arrow` — оставить как есть: стрелку больше не рисуем, стили под них просто не нужны.

- [ ] **Step 9: Дополнить витрину**

В `src/dev/Kit.svelte` импортировать `Segments`, `Toggle`, `Slider`, `Band`, `Select`, `Swatches` и `type AccentId`. Добавить состояние и секцию:

```ts
	let seg = $state<"quality" | "balance" | "potato" | "manual">("balance");
	let on = $state(true);
	let fps = $state(60);
	let accent = $state<AccentId>("peach");
	let angle = $state("d3d11");
```

```svelte
		<Section title="Выбор">
			<Segments
				label="Профиль"
				options={[
					{ value: "quality", label: "Качество", short: "КАЧ" },
					{ value: "balance", label: "Баланс", short: "БАЛ" },
					{ value: "potato", label: "Картошка", short: "КРТ" },
					{ value: "manual", label: "Ручной", short: "РУЧ", arrowSkip: true }
				]}
				value={seg}
				onchange={(v) => (seg = v)}
			/>
			<Toggle label="Анимация света" checked={on} modified onchange={(v) => (on = v)} />
			<Toggle label="Удалить данные" danger checked={!on} onchange={(v) => (on = !v)} />
			<Slider label="Макс. FPS" value={fps} min={20} max={240} step={1} ticks={[60, 144, 240]} presets={[60, 144, 240]} editable format={String} onchange={(v) => (fps = v)} />
			<Band value={0.42} ghost={0.6} stops={[0.25, 0.5, 0.75]} label="Кэш" />
			<Select label="ANGLE" options={[{ value: "d3d11", label: "D3D11" }, { value: "gl", label: "GL" }]} value={angle} onchange={(v) => (angle = v)} />
			<Swatches label="Акцент" value={accent} onchange={(v) => (accent = v)} />
		</Section>
```

- [ ] **Step 10: Гейты и коммит**

```bash
npm test && npm run check && npm run build
git add src/components src/lib/tips.ts src/lib/tooltip.svelte.ts src/dev
git commit -m "feat: Tactile segments, toggle, slider, band, select, swatches and tooltip plate" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 10: Шапка — версия, обновление, видеокарта, кнопки окна

**Files:**
- Create: `src/components/GpuBadge.svelte`, `src/components/VersionChip.svelte`, `src/components/UpdatePill.svelte`
- Rewrite: `src/components/TitleBar.svelte`
- Delete: `src/components/VersionTag.svelte`
- Modify: `src/App.svelte`, `src/lib/api.ts`, `src-tauri/capabilities/launcher.json`, `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `app.gpu`, `gpuLevel` (Task 4); `IconButton`, `Icon`, `Chip`, `Pill` (Task 8); `TipData.checks` (Task 9).
- Produces: `TitleBar { start?: Snippet; end?: Snippet }`; `windowControls.toggleMaximize()`.

- [ ] **Step 1: Развернуть окно**

`src/lib/api.ts`, в `windowControls`:

```ts
	toggleMaximize: async () => void (await currentWindow())?.toggleMaximize(),
```

`src-tauri/capabilities/launcher.json`: после `"core:window:allow-minimize",` добавить `"core:window:allow-toggle-maximize",`.

- [ ] **Step 2: `TitleBar.svelte`**

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import { windowControls } from "../lib/api";
	import { t } from "../lib/i18n.svelte";
	import Icon from "./tactile/Icon.svelte";
	import IconButton from "./tactile/IconButton.svelte";

	/** `start` — после названия (версия, обновление); `end` — перед кнопками окна (видеокарта, «Настройка»). */
	let { start, end }: { start?: Snippet; end?: Snippet } = $props();
</script>

<header class="bar" data-tauri-drag-region>
	<span class="app" aria-hidden="true"><Icon name="app" /></span>
	<b class="ttl" data-tauri-drag-region>Foundry Performance</b>
	{#if start}{@render start()}{/if}
	<span class="sp" data-tauri-drag-region></span>
	{#if end}{@render end()}{/if}
	<span class="wb">
		<IconButton label={t("window.minimize")} size={30} onclick={() => windowControls.minimize()}><Icon name="minimize" /></IconButton>
		<IconButton label={t("window.maximize")} size={30} onclick={() => windowControls.toggleMaximize()}><Icon name="maximize" /></IconButton>
		<IconButton label={t("window.close")} size={30} danger onclick={() => windowControls.close()}><Icon name="close" /></IconButton>
	</span>
</header>

<style>
	.bar {
		height: 44px;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 0 8px 0 10px;
		user-select: none;
	}
	.app {
		width: 28px;
		height: 28px;
		display: grid;
		place-items: center;
		flex: none;
		border-radius: 99px;
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		color: var(--tc-accent-text);
	}
	.ttl {
		font-weight: 600;
		font-size: 15px;
		line-height: 1.15;
		white-space: nowrap;
	}
	.sp {
		flex: 1;
		align-self: stretch;
	}
	.wb {
		display: flex;
		gap: 2px;
		margin-left: 4px;
	}
</style>
```

- [ ] **Step 3: `GpuBadge.svelte`**

```svelte
<script lang="ts">
	import { gpuLevel } from "../lib/gpu";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import Icon from "./tactile/Icon.svelte";
	import IconButton from "./tactile/IconButton.svelte";

	const probe = $derived(app.gpu);
	const level = $derived(probe ? gpuLevel(probe) : null);
	const checks = $derived(
		probe
			? [
					{ ok: probe.webgl2, text: t(probe.webgl2 ? "gpu.webgl2.yes" : "gpu.webgl2.no") },
					{ ok: probe.hardware, text: t(probe.hardware ? "gpu.hw.yes" : "gpu.hw.no") }
				]
			: []
	);
	const color = $derived(level === "ok" ? "var(--tc-good)" : level === "software" ? "var(--tc-warning)" : "var(--tc-danger)");
	const advice = $derived(level === "software" ? t("gpu.advice.software") : level === "none" ? t("gpu.advice.none") : "");
</script>

{#if probe}
	<IconButton
		size={30}
		label={`${t("gpu.title")}: ${checks.map((c) => c.text).join(", ")}`}
		style={`color: ${color}`}
		tip={{ title: t("gpu.title"), body: advice, checks }}
	>
		<Icon name="chip" />
	</IconButton>
{/if}
```

- [ ] **Step 4: `VersionChip.svelte` и `UpdatePill.svelte`**

`VersionChip.svelte`:

```svelte
<script lang="ts">
	import changelog from "../../CHANGELOG.md?raw";
	import { notesFor } from "../lib/changelog";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import Chip from "./tactile/Chip.svelte";
	import Pill from "./tactile/Pill.svelte";

	let { version }: { version: string } = $props();

	const notes = $derived(notesFor(changelog, version));
	const check = $derived(app.updateCheck);
	const status = $derived(
		check === "idle"
			? ""
			: check === "available" && app.update
				? t("update.state.available", { version: app.update.version })
				: t(`update.state.${check}`)
	);
</script>

<!-- версия — кнопка ручной проверки обновлений; в подсказке — что нового в этой версии -->
<Chip
	button
	aria-label={t("update.check")}
	aria-busy={check === "checking"}
	onclick={() => app.checkUpdate(true)}
	tip={{ title: t("version.notesTitle", { version }), body: notes ? t("update.checkHint") : `${t("version.noNotes")} ${t("update.checkHint")}`, lines: notes ?? undefined }}
>
	v{version}
</Chip>
{#if status}<Pill tone={check === "failed" ? "warn" : "plain"} role="status">{status}</Pill>{/if}
```

`UpdatePill.svelte`:

```svelte
<script lang="ts">
	import { noteLines } from "../lib/changelog";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import Pill from "./tactile/Pill.svelte";
</script>

{#if app.updating !== null}
	<Pill tone="accent" role="status">{t("update.progress", { pct: app.updating })}</Pill>
{:else if app.update}
	{@const u = app.update}
	<Pill
		tone="accent"
		button
		onclick={() => app.applyUpdate()}
		tip={{ title: t("update.notesTitle", { version: u.version }), body: t("update.noNotes"), lines: noteLines(u.notes) }}
	>
		{t("update.available", { version: u.version })}
	</Pill>
{/if}
```

Проверить, что `noteLines` экспортируется из `src/lib/changelog.ts` (его уже импортировал старый `MainScreen`). Удалить `VersionTag`:

```bash
git rm src/components/VersionTag.svelte
```

- [ ] **Step 5: Шапка в `App.svelte`**

Импорты: убрать `VersionTag`; добавить `GpuBadge`, `UpdatePill`, `VersionChip`, `Chip`, `Icon`, `IconButton` (из `./components/tactile/…`). Блок шапки:

```svelte
	{#if mode?.mode === "launcher" && app.dto}
		<TitleBar>
			{#snippet start()}
				<VersionChip version={app.dto!.version} />
				<UpdatePill />
			{/snippet}
			{#snippet end()}
				<GpuBadge />
				<IconButton
					label={t("main.tune")}
					size={30}
					on={app.screen === "tuning"}
					onclick={() => app.openTuning(app.selected ? { kind: "server", id: app.selected.id } : { kind: "global" })}
				>
					<Icon name="sliders" />
				</IconButton>
			{/snippet}
		</TitleBar>
	{:else}
		<TitleBar>
			{#snippet start()}{#if channel}<Chip>{channel}</Chip>{/if}{/snippet}
		</TitleBar>
	{/if}
```

- [ ] **Step 6: Строки**

`ru.json` — добавить или заменить:

```json
  "window.maximize": "Развернуть",
  "main.tune": "Настройка",
  "gpu.title": "Видеокарта",
  "gpu.webgl2.yes": "WebGL2 есть",
  "gpu.webgl2.no": "WebGL2 нет",
  "gpu.hw.yes": "Аппаратное ускорение есть",
  "gpu.hw.no": "Аппаратного ускорения нет",
  "gpu.advice.software": "Обновите драйвер видеокарты и проверьте, что аппаратное ускорение не выключено в Windows.",
  "gpu.advice.none": "Foundry не запустится без WebGL2: обновите драйвер видеокарты.",
  "update.available": "Обновить до {version}",
  "update.progress": "Загрузка {pct} %",
  "update.checkHint": "Нажмите, чтобы проверить обновления.",
  "install.channel": "Установка",
  "uninstall.channel": "Удаление"
```

`en.json`:

```json
  "window.maximize": "Maximize",
  "main.tune": "Tuning",
  "gpu.title": "Graphics card",
  "gpu.webgl2.yes": "WebGL2 available",
  "gpu.webgl2.no": "No WebGL2",
  "gpu.hw.yes": "Hardware acceleration on",
  "gpu.hw.no": "No hardware acceleration",
  "gpu.advice.software": "Update the GPU driver and make sure hardware acceleration isn't turned off in Windows.",
  "gpu.advice.none": "Foundry won't run without WebGL2: update the GPU driver.",
  "update.available": "Update to {version}",
  "update.progress": "Downloading {pct}%",
  "update.checkHint": "Click to check for updates.",
  "install.channel": "Setup",
  "uninstall.channel": "Uninstall"
```

Удалить из обоих словарей `gpu.unknown`, `update.downloading`, `update.checkBody`.

- [ ] **Step 7: Гейты и коммит**

```bash
npm test && npm run check && npm run build
git add -A src src-tauri/capabilities
git commit -m "feat: Tactile title bar with version chip, update pill and FLC-style GPU badge" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 11: Главный экран «Намерение»

**Files:**
- Create: `src/components/main/Hero.svelte`, `src/components/main/ServerList.svelte`, `src/components/main/LastRun.svelte`
- Rewrite: `src/screens/MainScreen.svelte`
- Delete: `src/components/Slot.svelte`, `src/components/LaunchButton.svelte`, `src/components/Knob.svelte`, `src/components/Led.svelte`
- Modify: `src/lib/status.ts`, `src/lib/types.ts`, `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `intentReason`, `lastRun`, `hostOf`, `STATUS_TONE`, `STATUS_DETAIL` (Task 5); `PROFILE_POSITIONS`, `profileTipKey` (Task 5); `knobPosition`, `KnobPosition` (`levers.ts`); `slotStatus` (`status.ts`); `openWindow`, `rise`, `countTo` (Task 7); примитивы (Tasks 8–9).

- [ ] **Step 1: `Hero.svelte`**

```svelte
<script lang="ts">
	import { t } from "../../lib/i18n.svelte";
	import { hostOf, intentReason, lastRun, STATUS_DETAIL, STATUS_TONE } from "../../lib/intent";
	import { type KnobPosition, knobPosition } from "../../lib/levers";
	import { rise } from "../../lib/motion";
	import { PROFILE_POSITIONS, profileTipKey } from "../../lib/profile-tip";
	import { slotStatus } from "../../lib/status";
	import { app } from "../../lib/store.svelte";
	import Icon from "../tactile/Icon.svelte";
	import Label from "../tactile/Label.svelte";
	import Pill from "../tactile/Pill.svelte";
	import Primary from "../tactile/Primary.svelte";
	import Segments from "../tactile/Segments.svelte";
	import Shell from "../tactile/Shell.svelte";

	const dto = $derived(app.dto!);
	const sel = $derived(app.selected);
	const knob = $derived(knobPosition(dto, sel?.id ?? null));
	const probe = $derived(sel ? app.probes[sel.id] : undefined);
	const status = $derived(sel ? slotStatus(sel, probe) : null);
	const run = $derived(sel ? lastRun(dto.stats[sel.id]) : null);
	const reason = $derived(intentReason(sel?.id ?? null, dto.settings.lastServer));
	const world = $derived(probe && probe !== "pending" ? (probe.world ?? "") : "");
	const statusTip = $derived(status ? { title: t(`status.${status}`), body: t(STATUS_DETAIL[status], { world }) } : undefined);
	const profiles = $derived(
		PROFILE_POSITIONS.map((p) => ({ value: p, label: t(`profile.${p}.long`), short: t(`profile.${p}`), arrowSkip: p === "manual" }))
	);

	// Shift при нажатии — безопасный запуск; подпись кнопки меняется, пока Shift зажат
	let shift = $state(false);
	const track = (e: KeyboardEvent) => (shift = e.shiftKey);
	const goLabel = $derived(app.launching ? t("main.launching") : shift ? t("main.launchSafe") : t("main.launch"));
</script>

<svelte:window onkeydown={track} onkeyup={track} onblur={() => (shift = false)} />

<Shell fill pad="22px 24px 20px">
	{#if sel}
		<div class="top">
			<Label>{reason === "last" ? t("main.continue") : t("main.selected")}</Label>
			{#if status}<Pill tone={STATUS_TONE[status]} tip={statusTip}>{t(`status.${status}`)}</Pill>{/if}
		</div>
		<div class="grow"></div>
		<div class="intent" use:rise={sel.id}>
			<h1 class="name">{sel.name}</h1>
			<p class="meta">
				<span>{hostOf(sel.url)}</span>
				{#if run}
					<span>{t(run.kind === "bench" ? "main.lastBench" : "main.lastSession")} <b>{run.fps}</b> FPS</span>
				{:else}
					<span>{t("main.noData")}</span>
				{/if}
			</p>
			{#if reason}<span class="why"><Icon name="clock" size={14} />{t(`main.reason.${reason}`)}</span>{/if}
		</div>
		<div class="grow"></div>
		<Segments label={t("main.profile")} options={profiles} value={knob} onchange={(p: KnobPosition) => app.setKnob(p)} />
		<p class="ptip">{t(profileTipKey(knob))}</p>
		<Primary size="lg" busy={app.launching} onclick={(e: MouseEvent) => app.launch(e.shiftKey)}>{goLabel}</Primary>
		<p class="hint">{t("main.safeHint")}</p>
	{:else}
		<div class="grow"></div>
		<h1 class="name empty">{t("main.noServer")}</h1>
		<div class="grow"></div>
		<Primary size="lg" icon="plus" onclick={() => app.newSlot()}>{t("main.addServer")}</Primary>
	{/if}
</Shell>

<style>
	.top {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 18px;
	}
	.grow {
		flex: 1;
		min-height: 12px;
	}
	.intent {
		display: grid;
		justify-items: start;
	}
	.name {
		display: -webkit-box;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		overflow: hidden;
		overflow-wrap: anywhere;
		font: 700 50px/1.02 var(--tc-font-display);
		letter-spacing: -0.035em;
		text-wrap: balance;
	}
	.name.empty {
		font-size: 34px;
		color: var(--tc-muted);
	}
	.meta {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 14px;
		margin-top: 14px;
		font: 500 12.5px/1.4 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.meta b {
		color: var(--tc-ink);
		font-weight: 600;
	}
	.why {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		margin-top: 16px;
		padding: 6px 12px 6px 9px;
		border-radius: var(--tc-r-ctl);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		font-size: 12.5px;
		color: var(--tc-muted);
	}
	.why :global(.ic) {
		color: var(--tc-accent-text);
	}
	.ptip {
		min-height: 35px;
		margin: 8px 2px 16px;
		font-size: 12.5px;
		line-height: 1.4;
		color: var(--tc-muted);
	}
	.hint {
		margin-top: 9px;
		text-align: center;
		font: 500 10.5px/1.3 var(--tc-font-mono);
		letter-spacing: 0.06em;
		color: var(--tc-muted);
	}
</style>
```

- [ ] **Step 2: `ServerList.svelte`**

```svelte
<script lang="ts">
	import { t } from "../../lib/i18n.svelte";
	import { hostOf, lastRun, STATUS_TONE } from "../../lib/intent";
	import { slotStatus } from "../../lib/status";
	import { app } from "../../lib/store.svelte";
	import Icon from "../tactile/Icon.svelte";
	import IconButton from "../tactile/IconButton.svelte";
	import Label from "../tactile/Label.svelte";
	import Tray from "../tactile/Tray.svelte";

	const dto = $derived(app.dto!);

	/** Клик выбирает; Enter на уже выбранной строке и двойной клик — запускают. */
	function pick(e: MouseEvent, id: string, selected: boolean) {
		if (selected && e.detail === 0) void app.launch(e.shiftKey);
		else app.selectedId = id;
	}
</script>

<Tray fill scroll pad="6px">
	<div class="lbl"><Label>{t("main.slots")}</Label></div>
	<div class="rows">
		{#each dto.servers as s (s.id)}
			{@const status = slotStatus(s, app.probes[s.id])}
			{@const run = lastRun(dto.stats[s.id])}
			{@const selected = s.id === app.selectedId}
			<div class="row" class:on={selected}>
				<button type="button" class="pick" aria-current={selected} onclick={(e) => pick(e, s.id, selected)} ondblclick={() => app.launch(false)}>
					<span class="dot {STATUS_TONE[status]}" class:hollow={status !== "running" && status !== "idle"} aria-hidden="true"></span>
					<span class="text">
						<span class="name">{s.name}</span>
						<span class="meta" class:bad={status === "stopped"}>
							{hostOf(s.url)} · {status === "stopped" || !run ? t(`status.${status}`) : `${run.fps} FPS`}
						</span>
					</span>
				</button>
				<span class="edit">
					<IconButton label={t("slot.editAria", { name: s.name })} onclick={() => app.editSlot(s)}><Icon name="edit" size={14} /></IconButton>
				</span>
			</div>
		{/each}
		<button type="button" class="add" onclick={() => app.newSlot()}><Icon name="plus" />{t("main.addServer")}</button>
	</div>
</Tray>

<style>
	.lbl {
		padding: 8px 10px 6px;
	}
	.rows {
		display: grid;
		gap: 2px;
	}
	.row {
		position: relative;
		border-radius: var(--tc-r-row);
		transition:
			background-color 240ms var(--tc-ease),
			box-shadow 240ms var(--tc-ease);
	}
	.row:hover {
		background: color-mix(in oklab, var(--tc-surface) 55%, transparent);
	}
	.row.on {
		background: var(--tc-surface);
		box-shadow:
			var(--tc-raise),
			0 0 0 1.5px var(--tc-accent),
			0 0 16px -4px var(--tc-glow);
	}
	.pick {
		display: flex;
		align-items: center;
		gap: 11px;
		width: 100%;
		height: 56px;
		padding: 0 44px 0 12px;
		border-radius: inherit;
		text-align: left;
	}
	.dot {
		--tone: var(--tc-muted);
		width: 7px;
		height: 7px;
		flex: none;
		border-radius: 99px;
		background: var(--tone);
	}
	.dot.good {
		--tone: var(--tc-good);
	}
	.dot.warn {
		--tone: var(--tc-warning);
	}
	.dot.danger {
		--tone: var(--tc-danger);
	}
	.dot.hollow {
		background: transparent;
		box-shadow: inset 0 0 0 1.5px var(--tone);
	}
	.text {
		flex: 1;
		min-width: 0;
	}
	.name,
	.meta {
		display: block;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}
	.name {
		font-weight: 600;
		font-size: 14px;
		line-height: 1.2;
	}
	.meta {
		margin-top: 3px;
		font: 500 11px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.meta.bad {
		color: var(--tc-danger);
	}
	.edit {
		position: absolute;
		top: 50%;
		right: 8px;
		transform: translateY(-50%);
		opacity: 0;
		transition: opacity 160ms var(--tc-ease);
	}
	.row:hover .edit,
	.row.on .edit,
	.edit:focus-within {
		opacity: 1;
	}
	.add {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		height: 44px;
		margin-top: 4px;
		border-radius: var(--tc-r-row);
		color: var(--tc-muted);
		font-size: 13px;
		transition: color 160ms var(--tc-ease);
	}
	.add:hover {
		color: var(--tc-ink);
	}
</style>
```

- [ ] **Step 3: `LastRun.svelte`**

```svelte
<script lang="ts">
	import { t } from "../../lib/i18n.svelte";
	import { lastRun } from "../../lib/intent";
	import { countTo } from "../../lib/motion";
	import { app } from "../../lib/store.svelte";
	import Core from "../tactile/Core.svelte";
	import Label from "../tactile/Label.svelte";

	const run = $derived(app.selected ? lastRun(app.dto!.stats[app.selected.id]) : null);
</script>

<Core pad="14px 16px 15px">
	<Label>{t(run?.kind === "bench" ? "main.lastBench" : "main.lastSession")}</Label>
	<div class="sr">
		{#if run}
			<span><span class="num" use:countTo={run.fps}></span><span class="unit">FPS</span></span>
			<Label>{t(`profile.${run.profile}.long`)}</Label>
		{:else}
			<span class="nodata">{t("main.noData")}</span>
		{/if}
	</div>
</Core>

<style>
	.sr {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		min-height: 30px;
		margin-top: 8px;
	}
	.num {
		font: 700 30px/1 var(--tc-font-display);
		letter-spacing: -0.04em;
	}
	.unit {
		margin-left: 5px;
		font: 600 11px/1 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.nodata {
		align-self: center;
		font: 500 12.5px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
	}
</style>
```

- [ ] **Step 4: `MainScreen.svelte` целиком**

```svelte
<script lang="ts">
	import Hero from "../components/main/Hero.svelte";
	import LastRun from "../components/main/LastRun.svelte";
	import ServerList from "../components/main/ServerList.svelte";
	import Icon from "../components/tactile/Icon.svelte";
	import IconButton from "../components/tactile/IconButton.svelte";
	import { t } from "../lib/i18n.svelte";
	import { openWindow } from "../lib/motion";
	import { app } from "../lib/store.svelte";
</script>

<div class="main" use:openWindow>
	<div class="hero" data-part><Hero /></div>
	<div class="side">
		<div class="list" data-part><ServerList /></div>
		{#if app.message}
			<div class="notice" role="status">
				<span>{app.message}</span>
				<IconButton label={t("window.close")} onclick={() => app.dismiss()}><Icon name="close" size={14} /></IconButton>
			</div>
		{/if}
		<div data-part><LastRun /></div>
	</div>
</div>

<style>
	.main {
		height: 100%;
		display: grid;
		grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
		gap: 14px;
		padding: 2px 14px 14px;
	}
	.hero,
	.list {
		min-height: 0;
	}
	.side {
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.list {
		flex: 1;
	}
	.notice {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 8px 6px 8px 14px;
		border-radius: var(--tc-r-block);
		background: var(--tc-sunken);
		box-shadow: var(--tc-press);
		font-size: 12.5px;
		line-height: 1.4;
	}
	.notice span {
		flex: 1;
		padding-top: 5px;
	}
</style>
```

- [ ] **Step 5: Удалить старые компоненты и LED**

```bash
git rm src/components/Slot.svelte src/components/LaunchButton.svelte src/components/Knob.svelte src/components/Led.svelte
```

В `src/lib/status.ts` удалить `LED_FOR` и импорт `LedState`; в `src/lib/types.ts` удалить `LedState`. Если `src/lib/status.test.ts` проверяет `LED_FOR` — удалить эти проверки. `grep -rn "LED_FOR\|LedState\|Knob.svelte\|Slot.svelte\|Led.svelte\|LaunchButton" src` — пусто.

- [ ] **Step 6: Строки**

`ru.json` — добавить или заменить:

```json
  "main.slots": "Серверы",
  "main.profile": "Профиль",
  "main.continue": "Продолжить",
  "main.selected": "Выбран",
  "main.reason.last": "Вы запускали его последним",
  "main.reason.manual": "Выбран вручную",
  "main.addServer": "Добавить сервер",
  "main.launchSafe": "БЕЗОПАСНЫЙ ЗАПУСК",
  "status.running": "онлайн",
  "status.idle": "мир не загружен",
  "status.stopped": "не отвечает",
  "status.unknown": "нет данных",
  "status.checking": "проверяю",
  "slot.editAria": "Изменить сервер {name}",
  "tip.manual": "Свои положения рычагов из «Настройки». Выбор пресета сбрасывает правки этого сервера — их можно вернуть, снова выбрав «Ручной»."
```

`en.json`:

```json
  "main.slots": "Servers",
  "main.profile": "Profile",
  "main.continue": "Continue",
  "main.selected": "Selected",
  "main.reason.last": "You launched it last",
  "main.reason.manual": "Picked by hand",
  "main.addServer": "Add server",
  "main.launchSafe": "SAFE LAUNCH",
  "status.running": "online",
  "status.idle": "no world",
  "status.stopped": "not responding",
  "status.unknown": "no data",
  "status.checking": "checking",
  "slot.editAria": "Edit server {name}",
  "tip.manual": "Your own lever positions from Tuning. Picking a preset resets this server's changes — pick Manual again to bring them back."
```

Удалить из обоих словарей `main.emptySlot`, `main.channels`, `knob.label`, `slot.editShort`.

- [ ] **Step 7: Гейты и коммит**

```bash
npm test && npm run check && npm run build
git add -A src
git commit -m "feat: Intent main screen — server hero, profile segments, launch and server tray" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 12: «Настройка» на Tactile

**Files:**
- Rewrite: `src/screens/TuningScreen.svelte`, `src/components/CacheControl.svelte`
- Delete: `src/components/Fader.svelte`
- Modify: `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `Segments`, `Slider`, `Toggle`, `Select`, `Swatches`, `Band`, `Field`, `Section`, `Core`, `Button`, `IconButton`, `Pill`, `Label`, `Icon`; `openWindow`; `levers.ts` и стор — без изменений.

- [ ] **Step 1: `CacheControl.svelte`**

Скрипт сохраняется: фазы `idle → armed → busy → done`, `fmt`, `later`, `onMount`. Изменения: функцию `press` переименовать в `onPress`, импорт `tip, tipFor` оставить и добавить:

```ts
	import { app } from "../lib/store.svelte";
	import Band from "./tactile/Band.svelte";
	import Button from "./tactile/Button.svelte";
	import Label from "./tactile/Label.svelte";

	const limit = $derived((app.dto?.settings.engine.diskCacheMb ?? 2048) * 1024 * 1024);
```

Разметка и стили целиком:

```svelte
<div class="cache" {@attach tip(() => tipFor("cacheClear", t("cache.title")))}>
	<div class="head">
		<Label>{t("cache.title")}</Label>
		<span class="used">{bytes === null ? "…" : `${fmt(bytes)} / ${fmt(limit)}`}</span>
	</div>
	<Band value={bytes === null ? 0 : bytes / limit} label={t("cache.title")} />
	<div>
		<Button
			danger
			armed={phase.kind === "armed" || (phase.kind === "done" && phase.warn)}
			aria-live="polite"
			disabled={phase.kind === "busy"}
			onclick={onPress}>{label}</Button
		>
	</div>
</div>

<style>
	.cache {
		display: grid;
		gap: 8px;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.used {
		font: 600 12.5px/1 var(--tc-font-mono);
	}
</style>
```

- [ ] **Step 2: `TuningScreen.svelte` — скрипт**

Импорты:

```ts
	import CacheControl from "../components/CacheControl.svelte";
	import Button from "../components/tactile/Button.svelte";
	import Core from "../components/tactile/Core.svelte";
	import Field from "../components/tactile/Field.svelte";
	import Icon from "../components/tactile/Icon.svelte";
	import IconButton from "../components/tactile/IconButton.svelte";
	import Pill from "../components/tactile/Pill.svelte";
	import Section from "../components/tactile/Section.svelte";
	import Segments from "../components/tactile/Segments.svelte";
	import Select from "../components/tactile/Select.svelte";
	import Slider from "../components/tactile/Slider.svelte";
	import Swatches from "../components/tactile/Swatches.svelte";
	import Toggle from "../components/tactile/Toggle.svelte";
	import { t } from "../lib/i18n.svelte";
	import { countOverrides, overridesFor, resolved } from "../lib/levers";
	import { openWindow } from "../lib/motion";
	import { app } from "../lib/store.svelte";
	import { tipFor } from "../lib/tooltip.svelte";
	import type { AngleBackend, Levers, LocalePref, PrimeLevel, ProfileId, ThemePref, VideoMode } from "../lib/types";
```

Константы `dto`, `scope`, `l`, `over`, `n`, `mod`, `pct`, `profiles`, `profileOptions`, `profileValue`, `videoOptions`, `primeOptions`, `perfOptions`, `angleOptions`, `cacheOptions`, `localeOptions`, `themeOptions` — как сейчас. Заменить `serverIndex`, `server`, `title`, `selIndex`, `scopeOptions`:

```ts
	const server = $derived(scope.kind === "server" ? (dto.servers.find((s) => s.id === scope.id) ?? null) : null);
	const title = $derived(server ? server.name : t("tuning.global"));
	// Переключатель уровня: общие настройки (движок, вид) и правки выбранного сервера
	const selected = $derived(dto.servers.find((s) => s.id === app.selectedId) ?? null);
	const scopeOptions = $derived([{ value: "global", label: t("tuning.global") }, ...(selected ? [{ value: selected.id, label: selected.name }] : [])]);
	const toggles = $derived<{ key: keyof Levers; label: string; tipKey: Parameters<typeof tipFor>[0] }[]>([
		{ key: "adaptive", label: t("tuning.adaptive"), tipKey: "adaptive" },
		{ key: "lightAnimation", label: t("tuning.lightAnimation"), tipKey: "lightAnimation" },
		{ key: "visionAnimation", label: t("tuning.visionAnimation"), tipKey: "visionAnimation" },
		{ key: "mipmap", label: t("tuning.mipmap"), tipKey: "mipmap" },
		{ key: "pixelRatioScaling", label: t("tuning.pixelRatio"), tipKey: "pixelRatio" },
		{ key: "uiBlur", label: t("tuning.uiBlur"), tipKey: "uiBlur" },
		{ key: "sequencer", label: "Sequencer", tipKey: "sequencer" },
		{ key: "fxmaster", label: "FXMaster", tipKey: "fxmaster" }
	]);
```

Если `tipFor` в `tooltip.svelte.ts` типизирован ключом `TipKey`, вместо `Parameters<typeof tipFor>[0]` импортировать `type TipKey` из `../lib/tips`.

- [ ] **Step 3: `TuningScreen.svelte` — разметка**

```svelte
<div class="tuning" use:openWindow>
	<header class="head" data-part>
		<IconButton label={t("tuning.back")} onclick={() => app.closeScreen()}><Icon name="back" /></IconButton>
		<h1>{t("tuning.title")}</h1>
		<span class="for">{title}</span>
		{#if n > 0}<Pill>{t("tuning.changed", { n })}</Pill>{/if}
		<span class="sp"></span>
		{#if scopeOptions.length > 1}
			<div class="scope">
				<Segments
					showLabel={false}
					label={t("tuning.scope")}
					tip={tipFor("scope", t("tuning.scope"))}
					options={scopeOptions}
					value={scope.kind === "global" ? "global" : scope.id}
					onchange={(v) => app.openTuning(v === "global" ? { kind: "global" } : { kind: "server", id: v })}
				/>
			</div>
		{/if}
	</header>

	<div class="body" data-part>
		<Core fill scroll pad="18px 20px 22px">
			<div class="sections">
				<Section title={t("tuning.sec.levers")}>
					<Segments
						label={t("tuning.profile")}
						tip={tipFor("profile", t("tuning.profile"))}
						options={profileOptions}
						value={profileValue}
						onchange={(v) => app.setScopeProfile(v === "inherit" ? null : (v as ProfileId))}
					/>
					<div class="grid">
						<Slider
							label={t("tuning.resolution")}
							tip={tipFor("resolution", t("tuning.resolution"))}
							value={l.resMin}
							min={0.4}
							max={1}
							step={0.05}
							format={(v) => (l.adaptive ? `${pct(v)}–${pct(l.resMax)}` : pct(v))}
							modified={mod("resMin", "resMax")}
							onchange={(v) => app.setLevers(l.adaptive ? { resMin: Math.min(v, l.resMax) } : { resMin: v, resMax: v })}
						/>
						<Slider
							label={t("tuning.maxFps")}
							tip={tipFor("maxFps", t("tuning.maxFps"))}
							value={l.maxFps}
							min={20}
							max={240}
							step={1}
							ticks={[60, 144, 240]}
							presets={[60, 144, 240]}
							editable
							inputLabel={t("tuning.fpsInput")}
							format={(v) => String(v)}
							modified={mod("maxFps")}
							onchange={(v) => app.setLevers({ maxFps: v })}
						/>
						<Slider
							label={t("tuning.unfocused")}
							tip={tipFor("unfocused", t("tuning.unfocused"))}
							value={l.unfocusedFps}
							min={5}
							max={60}
							step={5}
							format={(v) => String(v)}
							modified={mod("unfocusedFps")}
							onchange={(v) => app.setLevers({ unfocusedFps: v })}
						/>
						<Segments
							label={t("tuning.perfMode")}
							tip={tipFor("perfMode", t("tuning.perfMode"))}
							options={perfOptions}
							value={l.perfMode}
							modified={mod("perfMode")}
							onchange={(v) => app.setLevers({ perfMode: v as Levers["perfMode"] })}
						/>
					</div>
					<div class="grid toggles">
						{#each toggles as tg (tg.key)}
							<Toggle
								label={tg.label}
								tip={tipFor(tg.tipKey, tg.label)}
								checked={Boolean(l[tg.key])}
								modified={mod(tg.key)}
								onchange={(v) => app.setLevers({ [tg.key]: v } as Partial<Levers>)}
							/>
						{/each}
					</div>
					<div class="grid">
						<Segments label={t("tuning.video")} tip={tipFor("video", t("tuning.video"))} options={videoOptions} value={l.video} modified={mod("video")} onchange={(v) => app.setLevers({ video: v as VideoMode })} />
						<Segments label={t("tuning.prime")} tip={tipFor("prime", "Prime Performance")} options={primeOptions} value={l.prime} modified={mod("prime")} onchange={(v) => app.setLevers({ prime: v as PrimeLevel })} />
					</div>
				</Section>

				{#if !server}
					<Section title={t("tuning.sec.engine")}>
						<div class="grid">
							<Select
								label={t("tuning.angle")}
								tip={tipFor("angle", t("tuning.angle"))}
								options={angleOptions}
								value={dto.settings.engine.angle}
								onchange={(v) => app.saveSettings({ engine: { ...dto.settings.engine, angle: v as AngleBackend } })}
							/>
							<Segments
								label={t("tuning.cache")}
								tip={tipFor("cache", t("tuning.cache"))}
								options={cacheOptions}
								value={dto.settings.engine.diskCacheMb}
								onchange={(v) => app.saveSettings({ engine: { ...dto.settings.engine, diskCacheMb: v } })}
							/>
						</div>
						<CacheControl />
						<Field
							label={t("tuning.extraArgs")}
							mono
							value={dto.settings.engine.extraArgs}
							placeholder="--flag=value"
							spellcheck={false}
							hint={t("tuning.extraArgsHint")}
							onchange={(e) => app.saveSettings({ engine: { ...dto.settings.engine, extraArgs: e.currentTarget.value } })}
						/>
					</Section>

					<Section title={t("tuning.sec.look")}>
						<div class="grid">
							<Segments label={t("tuning.theme")} options={themeOptions} value={dto.settings.theme} onchange={(v) => app.saveSettings({ theme: v as ThemePref })} />
							<Segments label={t("tuning.language")} options={localeOptions} value={dto.settings.locale} onchange={(v) => app.saveSettings({ locale: v as LocalePref })} />
						</div>
						<Swatches label={t("tuning.accent")} value={dto.settings.accent} onchange={(v) => app.saveSettings({ accent: v })} />
					</Section>
				{/if}
			</div>
		</Core>
	</div>

	<footer class="foot" data-part>
		<span>{t("tuning.engineLine", { angle: dto.settings.engine.angle.toUpperCase(), cache: dto.settings.engine.diskCacheMb / 1024 })}</span>
		<span class="sp"></span>
		<span class="legend"><i aria-hidden="true"></i>{t("tuning.modifiedLegend")}</span>
		<Button disabled={n === 0} onclick={() => app.resetOverrides()}>{t("tuning.reset")}</Button>
	</footer>
</div>

<style>
	.tuning {
		height: 100%;
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto;
		gap: 10px;
		padding: 2px 14px 14px;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
	}
	h1 {
		font: 700 18px/1.1 var(--tc-font-display);
		letter-spacing: -0.02em;
	}
	.for {
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		font: 500 12.5px/1 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.sp {
		flex: 1;
	}
	.scope {
		width: 300px;
		flex: none;
	}
	.body {
		min-height: 0;
	}
	.sections {
		display: grid;
		gap: 26px;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 16px 24px;
		align-items: start;
	}
	.toggles {
		gap: 0 24px;
	}
	.foot {
		display: flex;
		align-items: center;
		gap: 12px;
		font: 500 11px/1 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.legend {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.legend i {
		width: 6px;
		height: 6px;
		border-radius: 99px;
		background: var(--tc-accent);
		box-shadow: 0 0 6px var(--tc-glow);
	}
</style>
```

Удалить `Fader`:

```bash
git rm src/components/Fader.svelte
```

- [ ] **Step 4: Строки**

`ru.json` — заменить или добавить:

```json
  "tuning.title": "Настройка",
  "tuning.global": "Все серверы",
  "tuning.scope": "Уровень",
  "tuning.back": "Назад",
  "tuning.profile": "Профиль",
  "tuning.changed": "+{n} изм.",
  "tuning.reset": "Сбросить к профилю",
  "tuning.modifiedLegend": "изменено вручную",
  "tuning.resolution": "Разрешение карты",
  "tuning.maxFps": "Макс. FPS",
  "tuning.unfocused": "FPS без фокуса",
  "tuning.perfMode": "Режим качества Foundry",
  "tuning.video": "Видеофоны",
  "tuning.prime": "Prime Performance",
  "tuning.angle": "Движок ANGLE",
  "tuning.cache": "Кэш на диске",
  "tuning.language": "Язык",
  "tuning.theme": "Тема",
  "tuning.accent": "Акцент",
  "tuning.sec.levers": "Профиль и рычаги",
  "tuning.sec.engine": "Движок",
  "tuning.sec.look": "Вид",
  "tuning.engineLine": "Движок: ANGLE {angle} · кэш {cache} ГБ",
  "video.play": "Играют",
  "video.pauseUnfocused": "Пауза без фокуса",
  "video.static": "Статика",
  "prime.soft": "Мягко",
  "prime.medium": "Средне",
  "prime.aggressive": "Макс.",
  "theme.auto": "Как в системе",
  "theme.day": "Светлая",
  "theme.night": "Тёмная",
  "locale.auto": "Как в системе",
  "profile.inherit": "Как у всех",
  "cache.title": "Кэш игры",
  "cache.clear": "Очистить",
  "cache.confirm": "Точно? Ещё раз",
  "cache.clearing": "Чищу…",
  "cache.freed": "Освобождено {size}",
  "cache.pending": "Дочищу при запуске",
  "cache.err.gameOpen": "Закройте игру",
  "cache.err.failed": "Не вышло"
```

`en.json`:

```json
  "tuning.title": "Tuning",
  "tuning.global": "All servers",
  "tuning.scope": "Scope",
  "tuning.back": "Back",
  "tuning.profile": "Profile",
  "tuning.changed": "+{n} changed",
  "tuning.reset": "Reset to profile",
  "tuning.modifiedLegend": "changed manually",
  "tuning.resolution": "Map resolution",
  "tuning.maxFps": "Max FPS",
  "tuning.unfocused": "FPS unfocused",
  "tuning.perfMode": "Foundry performance mode",
  "tuning.video": "Video backgrounds",
  "tuning.prime": "Prime Performance",
  "tuning.angle": "ANGLE backend",
  "tuning.cache": "Disk cache",
  "tuning.language": "Language",
  "tuning.theme": "Theme",
  "tuning.accent": "Accent",
  "tuning.sec.levers": "Profile and levers",
  "tuning.sec.engine": "Engine",
  "tuning.sec.look": "Appearance",
  "tuning.engineLine": "Engine: ANGLE {angle} · cache {cache} GB",
  "video.play": "Play",
  "video.pauseUnfocused": "Pause unfocused",
  "video.static": "Static",
  "prime.soft": "Soft",
  "prime.medium": "Medium",
  "prime.aggressive": "Max",
  "theme.auto": "System",
  "theme.day": "Light",
  "theme.night": "Dark",
  "locale.auto": "System",
  "profile.inherit": "Same as all",
  "cache.title": "Game cache",
  "cache.clear": "Clear",
  "cache.confirm": "Sure? Press again",
  "cache.clearing": "Clearing…",
  "cache.freed": "Freed {size}",
  "cache.pending": "Will finish on start",
  "cache.err.gameOpen": "Close the game",
  "cache.err.failed": "Failed"
```

- [ ] **Step 5: Гейты и коммит**

```bash
npm test && npm run check && npm run build
git add -A src
git commit -m "feat: Tuning on Tactile — sections, sliders, toggles, theme and accent" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 13: Редактор сервера

**Files:**
- Rewrite: `src/screens/SlotEditor.svelte`
- Delete: `src/components/Segmented.svelte`
- Modify: `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `Shell`, `Field`, `Segments`, `Button`, `Primary` (Tasks 8–9), `openWindow`; стор — без изменений.

- [ ] **Step 1: `SlotEditor.svelte` целиком**

```svelte
<script lang="ts">
	import Button from "../components/tactile/Button.svelte";
	import Field from "../components/tactile/Field.svelte";
	import Primary from "../components/tactile/Primary.svelte";
	import Segments from "../components/tactile/Segments.svelte";
	import Shell from "../components/tactile/Shell.svelte";
	import { t } from "../lib/i18n.svelte";
	import { openWindow } from "../lib/motion";
	import { app } from "../lib/store.svelte";
	import type { ProfileId } from "../lib/types";

	const draft = $derived(app.editing!);
	const stored = $derived(app.dto!.servers.find((s) => s.id === draft.id) ?? null);
	const isNew = $derived(stored === null);
	let errors = $state<{ name?: string; url?: string }>({});
	let confirmDelete = $state(false);
	let saving = $state(false);

	const profileOptions = $derived([
		{ value: "inherit" as ProfileId | "inherit", label: t("profile.inherit") },
		...(["quality", "balance", "potato"] as ProfileId[]).map((p) => ({ value: p as ProfileId | "inherit", label: t(`profile.${p}.long`) }))
	]);

	async function save(e: SubmitEvent) {
		e.preventDefault();
		errors = {};
		if (!draft.name.trim()) errors.name = t("err.nameRequired");
		if (!draft.url.trim()) errors.url = t("err.urlInvalid");
		if (errors.name || errors.url) return;
		saving = true;
		const err = await app.saveServer(draft);
		saving = false;
		if (err === "err.urlInvalid") errors.url = t(err);
		else if (err === "err.nameRequired") errors.name = t(err);
		else if (err) app.error = err;
	}
</script>

<div class="wrap" use:openWindow>
	<form class="editor" onsubmit={save} novalidate data-part>
		<Shell pad="22px 24px 20px">
			<h1>{isNew ? t("slot.new") : stored!.name}</h1>
			<div class="fields">
				<Field
					label={t("slot.name")}
					bind:value={draft.name}
					placeholder={t("slot.namePlaceholder")}
					maxlength={60}
					error={errors.name}
					oninput={() => (errors.name = undefined)}
				/>
				<Field
					label={t("slot.url")}
					mono
					bind:value={draft.url}
					placeholder={t("slot.urlPlaceholder")}
					spellcheck={false}
					error={errors.url}
					oninput={() => (errors.url = undefined)}
				/>
				<Segments label={t("slot.profile")} options={profileOptions} value={draft.profile ?? "inherit"} onchange={(v) => (draft.profile = v === "inherit" ? null : v)} />
				{#if !isNew}
					<button type="button" class="link" onclick={() => app.openTuning({ kind: "server", id: draft.id })}>{t("slot.tune")}</button>
				{/if}
			</div>
			<footer class="actions">
				{#if !isNew}
					<Button
						danger
						armed={confirmDelete}
						onclick={() => (confirmDelete ? app.deleteServer(draft.id) : (confirmDelete = true))}
						onblur={() => (confirmDelete = false)}
					>
						{confirmDelete ? t("slot.deleteConfirm") : t("slot.delete")}
					</Button>
				{/if}
				<span class="sp"></span>
				<Button onclick={() => app.closeScreen()}>{t("slot.cancel")}</Button>
				<Primary type="submit" busy={saving}>{t("slot.save")}</Primary>
			</footer>
		</Shell>
	</form>
</div>

<style>
	.wrap {
		height: 100%;
		display: grid;
		place-items: center;
		padding: 2px 14px 14px;
	}
	.editor {
		width: min(480px, 100%);
	}
	h1 {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		font: 700 18px/1.2 var(--tc-font-display);
		letter-spacing: -0.02em;
	}
	.fields {
		display: grid;
		gap: 16px;
		margin: 20px 0 22px;
	}
	.link {
		justify-self: start;
		font: 500 12px/1 var(--tc-font-mono);
		color: var(--tc-accent-text);
	}
	.link:hover {
		text-decoration: underline;
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.sp {
		flex: 1;
	}
</style>
```

Удалить `Segmented`:

```bash
git rm src/components/Segmented.svelte
```

- [ ] **Step 2: Строки**

`ru.json`:

```json
  "slot.new": "Новый сервер",
  "slot.save": "Сохранить",
  "slot.cancel": "Отмена",
  "slot.delete": "Удалить",
  "slot.deleteConfirm": "Точно удалить?",
  "slot.tune": "Тонкая настройка →",
  "err.serverMissing": "Сервер не найден — обновите список"
```

`en.json`:

```json
  "slot.new": "New server",
  "slot.save": "Save",
  "slot.cancel": "Cancel",
  "slot.delete": "Delete",
  "slot.deleteConfirm": "Delete for sure?",
  "slot.tune": "Fine-tune →",
  "err.serverMissing": "Server not found — refresh the list"
```

Удалить из обоих словарей `slot.edit`.

- [ ] **Step 3: Гейты и коммит**

```bash
npm test && npm run check && npm run build
git add -A src
git commit -m "feat: server editor on Tactile" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 14: Установка и удаление на Tactile

**Files:**
- Rewrite: `src/screens/InstallScreen.svelte`, `src/screens/UninstallScreen.svelte`
- Delete: `src/components/Lcd.svelte`, `src/components/Toggle.svelte`
- Modify: `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `Shell`, `Tray`, `Field`, `Toggle` (tactile), `Band`, `Button`, `Primary`, `Icon`, `openWindow`; `setupApi`, `windowControls`.

- [ ] **Step 1: `InstallScreen.svelte` — скрипт**

Импорты:

```ts
	import Band from "../components/tactile/Band.svelte";
	import Button from "../components/tactile/Button.svelte";
	import Field from "../components/tactile/Field.svelte";
	import Icon from "../components/tactile/Icon.svelte";
	import Primary from "../components/tactile/Primary.svelte";
	import Shell from "../components/tactile/Shell.svelte";
	import Toggle from "../components/tactile/Toggle.svelte";
	import Tray from "../components/tactile/Tray.svelte";
	import { setupApi } from "../lib/api";
	import { t } from "../lib/i18n.svelte";
	import { openWindow } from "../lib/motion";
	import type { InstallInfo, InstallStep } from "../lib/types";
```

Остальное без изменений (`STEPS`, состояние, `lead`, `goLabel`, `stepState`, `validate`, `browse`, `run`), кроме:
- удалить константу `angle` и комментарий над ней;
- `title`:

```ts
	const title = $derived(
		info.state === "upgrade"
			? t("install.titleUpgrade", { version: info.currentVersion })
			: info.state === "current"
				? t("install.titleCurrent")
				: t("install.title")
	);
```

- [ ] **Step 2: `InstallScreen.svelte` — разметка и стили**

```svelte
<div class="install" use:openWindow>
	<Shell fill pad="26px 28px 24px">
		<div class="cols">
			<section class="left" data-part>
				<h1>{title}</h1>
				<p class="lead">{lead}</p>

				{#if phase === "done"}
					<p class="done" role="status">{t("install.done")}</p>
					<div class="grow"></div>
					{#if !launch}<Primary size="lg" onclick={() => setupApi.openInstalled()}>{t("install.openInstalled")}</Primary>{/if}
				{:else}
					<div class="form">
						<Field
							label={t("install.dir")}
							mono
							bind:value={dir}
							spellcheck={false}
							disabled={phase === "running"}
							error={dirError}
							hint={t("install.dirHint")}
							onchange={validate}
						>
							{#snippet end()}
								<Button disabled={phase === "running"} onclick={browse}><Icon name="folder" size={14} />{t("install.browse")}</Button>
							{/snippet}
						</Field>
						<div>
							<Toggle label={t("install.desktop")} checked={desktop} disabled={phase === "running"} onchange={(v) => (desktop = v)} />
							<Toggle label={t("install.launch")} checked={launch} disabled={phase === "running"} onchange={(v) => (launch = v)} />
						</div>
						{#if error}<p class="err" role="alert">{error}</p>{/if}
					</div>
					<div class="grow"></div>
					{#if info.state === "current" && phase !== "running"}
						<div class="pair">
							<Button onclick={run}>{goLabel}</Button>
							<Primary size="lg" onclick={() => setupApi.openInstalled()}>{t("install.openInstalled")}</Primary>
						</div>
					{:else}
						<Primary size="lg" busy={phase === "running"} onclick={run}>{phase === "error" ? t("install.retry") : goLabel}</Primary>
					{/if}
				{/if}
			</section>

			<aside class="right" data-part>
				<Tray pad="10px 12px">
					<ol class="steps">
						{#each STEPS as s (s)}
							{@const st = stepState(s)}
							<li class={st}>
								<span class="dot" aria-hidden="true"></span>
								<span>{t(`install.step.${s}`)}</span>
							</li>
						{/each}
					</ol>
				</Tray>
				<Band value={pct / 100} label={t("install.progress")} />
				<span class="pct">{pct} %</span>
			</aside>
		</div>
	</Shell>
</div>

<style>
	.install {
		height: 100%;
		padding: 2px 14px 14px;
	}
	.cols {
		height: 100%;
		display: grid;
		grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr);
		gap: 28px;
	}
	.left {
		min-width: 0;
		display: flex;
		flex-direction: column;
	}
	h1 {
		font: 700 28px/1.1 var(--tc-font-display);
		letter-spacing: -0.03em;
		text-wrap: balance;
	}
	.lead {
		margin-top: 10px;
		color: var(--tc-muted);
	}
	.form {
		display: grid;
		gap: 16px;
		margin-top: 24px;
	}
	.done {
		margin-top: 24px;
		font-weight: 600;
		color: var(--tc-good);
	}
	.err {
		font: 500 12px/1.4 var(--tc-font-mono);
		color: var(--tc-danger);
	}
	.grow {
		flex: 1;
		min-height: 16px;
	}
	.pair {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.right {
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 12px;
	}
	.steps {
		display: grid;
		gap: 10px;
		list-style: none;
		font: 500 12px/1.3 var(--tc-font-mono);
		color: var(--tc-muted);
	}
	.steps li {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.dot {
		width: 8px;
		height: 8px;
		flex: none;
		border-radius: 99px;
		box-shadow: inset 0 0 0 1.5px var(--tc-muted);
	}
	.now {
		color: var(--tc-ink);
	}
	.now .dot {
		background: var(--tc-accent);
		box-shadow: 0 0 8px var(--tc-glow);
	}
	.done .dot {
		background: var(--tc-good);
		box-shadow: none;
	}
	.fail {
		color: var(--tc-danger);
	}
	.fail .dot {
		background: var(--tc-danger);
		box-shadow: none;
	}
	.pct {
		align-self: flex-end;
		font: 600 12.5px/1 var(--tc-font-mono);
	}
</style>
```

> `.done` используется дважды: у абзаца «Готово» и у пункта шага. Оба зелёные — так и задумано. Точка шага красится правилом `.done .dot`.

- [ ] **Step 3: `UninstallScreen.svelte` целиком**

```svelte
<script lang="ts">
	import Button from "../components/tactile/Button.svelte";
	import Primary from "../components/tactile/Primary.svelte";
	import Shell from "../components/tactile/Shell.svelte";
	import Toggle from "../components/tactile/Toggle.svelte";
	import { setupApi, windowControls } from "../lib/api";
	import { t } from "../lib/i18n.svelte";
	import { openWindow } from "../lib/motion";

	let wipe = $state(false);
	let phase = $state<"idle" | "running" | "done" | "error">("idle");
	let error = $state<string | null>(null);

	async function run() {
		if (phase === "running") return;
		phase = "running";
		error = null;
		try {
			await setupApi.uninstall(wipe);
			phase = "done";
		} catch (e) {
			phase = "error";
			error = t(String(e));
		}
	}
</script>

<div class="wrap" use:openWindow>
	<div class="card" data-part>
		<Shell pad="22px 24px 20px">
			<h1>{t("uninstall.title")}</h1>
			<p class="lead">{t("uninstall.lead")}</p>
			{#if phase === "done"}
				<p class="done" role="status">{t("uninstall.done")}</p>
				<div class="actions"><Button onclick={() => windowControls.close()}>{t("uninstall.close")}</Button></div>
			{:else}
				<div class="opt">
					<Toggle danger label={t("uninstall.wipe")} checked={wipe} disabled={phase === "running"} onchange={(v) => (wipe = v)} />
				</div>
				{#if phase === "running"}<p class="status" role="status">{t("uninstall.running")}</p>{/if}
				{#if error}<p class="err" role="alert">{error}</p>{/if}
				<div class="actions">
					<Button disabled={phase === "running"} onclick={() => windowControls.close()}>{t("uninstall.cancel")}</Button>
					<Primary tone="danger" icon="trash" busy={phase === "running"} onclick={run}>{t("uninstall.go")}</Primary>
				</div>
			{/if}
		</Shell>
	</div>
</div>

<style>
	.wrap {
		height: 100%;
		display: grid;
		place-items: center;
		padding: 2px 14px 14px;
	}
	.card {
		width: min(480px, 100%);
	}
	h1 {
		font: 700 22px/1.15 var(--tc-font-display);
		letter-spacing: -0.02em;
	}
	.lead {
		margin-top: 10px;
		color: var(--tc-muted);
	}
	.opt {
		margin-top: 18px;
	}
	.status,
	.err {
		margin-top: 10px;
		font: 500 12px/1.4 var(--tc-font-mono);
	}
	.status {
		color: var(--tc-muted);
	}
	.err {
		color: var(--tc-danger);
	}
	.done {
		margin-top: 18px;
		font-weight: 600;
		color: var(--tc-good);
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 22px;
	}
</style>
```

- [ ] **Step 4: Удалить старые компоненты**

```bash
git rm src/components/Lcd.svelte src/components/Toggle.svelte
```

`ls src/components` — только `tactile/`, `main/`, `CacheControl`, `GpuBadge`, `TitleBar`, `Tooltip`, `UpdatePill`, `VersionChip`.

- [ ] **Step 5: Строки**

`ru.json`:

```json
  "install.title": "Установка Foundry Performance",
  "install.titleUpgrade": "Обновление до {version}",
  "install.titleCurrent": "Уже установлено",
  "install.dir": "Папка установки",
  "install.browse": "Обзор…",
  "install.go": "Установить",
  "install.upgrade": "Обновить {from} → {to}",
  "install.reinstall": "Переустановить",
  "install.openInstalled": "Открыть Foundry Performance",
  "install.running": "Устанавливаем…",
  "install.retry": "Повторить",
  "install.done": "Готово. Можно играть.",
  "install.progress": "Ход установки",
  "install.step.check": "Проверка папки",
  "install.step.copy": "Копирование",
  "install.step.shortcuts": "Ярлыки",
  "install.step.register": "Регистрация",
  "install.step.done": "Готово",
  "uninstall.title": "Удалить Foundry Performance?",
  "uninstall.wipe": "Удалить также мои серверы, настройки и кэш игры — это нельзя отменить",
  "uninstall.go": "Удалить",
  "uninstall.cancel": "Отмена",
  "uninstall.running": "Удаляем…",
  "uninstall.close": "Закрыть"
```

`en.json`:

```json
  "install.title": "Install Foundry Performance",
  "install.titleUpgrade": "Update to {version}",
  "install.titleCurrent": "Already installed",
  "install.dir": "Install folder",
  "install.browse": "Browse…",
  "install.go": "Install",
  "install.upgrade": "Update {from} → {to}",
  "install.reinstall": "Reinstall",
  "install.openInstalled": "Open Foundry Performance",
  "install.running": "Installing…",
  "install.retry": "Retry",
  "install.done": "Done. Time to play.",
  "install.progress": "Installation progress",
  "install.step.check": "Checking folder",
  "install.step.copy": "Copying",
  "install.step.shortcuts": "Shortcuts",
  "install.step.register": "Registering",
  "install.step.done": "Done",
  "uninstall.title": "Uninstall Foundry Performance?",
  "uninstall.wipe": "Also remove my servers, settings and game cache — this can't be undone",
  "uninstall.go": "Uninstall",
  "uninstall.cancel": "Cancel",
  "uninstall.running": "Removing…",
  "uninstall.close": "Close"
```

- [ ] **Step 6: Гейты и коммит**

```bash
npm test && npm run check && npm run build
git add -A src
git commit -m "feat: setup and uninstall screens on Tactile" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 15: Чистка словарей

**Files:**
- Create: `src/lib/i18n/usage.test.ts`
- Modify: `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: все экраны (Tasks 10–14).

- [ ] **Step 1: Тест `src/lib/i18n/usage.test.ts`**

```ts
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import ru from "./ru.json";

function sources(dir: string): string[] {
	return readdirSync(dir).flatMap((f) => {
		const p = join(dir, f);
		if (statSync(p).isDirectory()) return sources(p);
		return /\.(svelte|ts)$/.test(f) && !f.endsWith(".test.ts") ? [p] : [];
	});
}

const code = sources("src").map((f) => readFileSync(f, "utf8")).join("\n");
const keys = Object.keys(ru);
/** Ключи, которые код собирает из частей (`t(\`profile.${p}.long\`)`) или присылает Rust (`notice.*`, `err.*`). */
const DYNAMIC = [
	/^profile\./,
	/^status\./,
	/^led\./,
	/^tip\./,
	/^accent\./,
	/^video\./,
	/^prime\./,
	/^theme\./,
	/^main\.reason\./,
	/^install\.step\./,
	/^update\.state\./,
	/^err\./,
	/^notice\./,
	/^cache\.err\./,
	/^install\.err\./,
	/^uninstall\.err\./,
	/^update\.err\./
];

describe("i18n usage", () => {
	it("every literal t() key exists", () => {
		const used = [...code.matchAll(/\bt\(\s*"([\w.]+)"/g)].map((m) => m[1]);
		expect(used.filter((k) => !keys.includes(k))).toEqual([]);
	});

	it("every static key is used", () => {
		const unused = keys.filter((k) => !DYNAMIC.some((r) => r.test(k)) && !code.includes(`"${k}"`));
		expect(unused).toEqual([]);
	});
});
```

- [ ] **Step 2: Прогнать и вычистить**

Run: `npx vitest run src/lib/i18n/usage.test.ts`
Expected: первый тест PASS. Второй FAIL со списком мёртвых ключей; среди них ожидаются `accel.*`.

Удалить из **обоих** словарей каждый ключ из списка. Если ключ в списке на самом деле нужен (используется через переменную, а не литералом) — переписать вызов на литерал или добавить его префикс в `DYNAMIC` с комментарием, откуда он берётся. Отдельно удалить `notice.softwareRender`: его прячет шаблон `^notice\.`, но Rust его больше не присылает (`grep -rn softwareRender src-tauri/src` — пусто).

Повторять, пока оба теста не PASS.

- [ ] **Step 3: Гейты и коммит**

```bash
npm test && npm run check
git add src/lib/i18n
git commit -m "test: i18n keys are all used and all exist" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 16: Проверка целиком (делает контролёр, не субагент)

**Files:** нет (только проверка; найденные дефекты чинятся отдельными коммитами `fix: …`).

- [ ] **Step 1: Гейты**

```bash
npm run build:agent && npm test && npm run check && npm run build
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

Expected: всё зелёное.

- [ ] **Step 2: Превью во встроенном браузере**

`preview_start` с `{name: "launcher-preview"}` (порт 1420, `D:\MyAiProjects\Apps\.claude\launch.json`). Пройти и снять скриншоты:

| Адрес / действие | Что проверить |
|---|---|
| `/` | «Намерение»: имя 50 px, чип «Вы запускали его последним», статус, профиль, «ЗАПУСК» |
| клик по второму серверу | чип «Выбран вручную», имя всплывает, FPS в карточке досчитывается |
| Shift зажат | подпись «БЕЗОПАСНЫЙ ЗАПУСК» |
| `?gpu=ok`, `?gpu=software`, `?gpu=none` | цвет значка и строки подсказки |
| `?update=1` | «Обновить до 0.2.3», подсказка со списком |
| кнопка «Настройка» | секции, ползунки, тумблеры, тема, акцент, язык |
| смена темы и акцентов персик / шалфей / сталь | обе темы, акцент перетекает |
| карандаш у сервера, «Добавить сервер» | редактор, ошибки полей |
| `?mode=install`, `&state=upgrade`, `&state=current` | установка, шаги, шкала |
| `?mode=uninstall` | удаление, красный тумблер и кнопка |
| `?kit` | все примитивы в обеих темах |
| `resize_window` 820 × 580 | длинное имя, нет горизонтальной прокрутки |
| `colorScheme: dark` | тёмная тема по умолчанию («Как в системе») |

`read_console_messages` — без ошибок. Найденное — чинить, коммитить, переснимать.

- [ ] **Step 3: Реальная сборка**

```powershell
npm run build:agent
npm run tauri build -- --no-bundle
$root = "$env:TEMP\fp-smoke"; New-Item -ItemType Directory -Force "$root\Roaming","$root\Local" | Out-Null
$env:APPDATA = "$root\Roaming"; $env:LOCALAPPDATA = "$root\Local"
$p = Start-Process -PassThru "src-tauri\target\release\foundry-performance.exe"
$p.Id
```

Проверить:
- окно открывается в «Намерении», значок GPU зелёный, в подсказке оба «есть»;
- через минуту простоя загрузка GPU по дереву процессов (`$p.Id` и дочерние `msedgewebview2`) ≈ 0 %, например через `Get-Counter "\GPU Engine(pid_*)\Utilization Percentage"` с фильтром по PID;
- «Развернуть» и «Свернуть» работают.

Закрыть только свой процесс (`Stop-Process -Id $p.Id`), вернуть переменные `$env:APPDATA = $null; $env:LOCALAPPDATA = $null` (в новом окне PowerShell они и так исходные). Имя exe сверить с `src-tauri\target\release\` — если отличается, взять фактическое.

---

### Task 17: Уборка (делает контролёр, по подтверждению пользователя)

- [ ] **Step 1: Пометить устаревшие документы**

Первой строкой под заголовком в `docs/superpowers/specs/2026-10-06-tactile-redesign-design.md`, `docs/superpowers/plans/2026-10-06-tactile-redesign.md` и `docs/superpowers/specs/2026-10-06-gpu-acceleration-check-design.md` добавить:

```markdown
> **Заменено** `2026-10-07-intent-redesign-design.md` (дизайн «Намерение», значок видеокарты в духе FLC). Документ оставлен для истории.
```

Если каких-то из этих файлов нет в `feat/intent` (их добавляли в другой ветке) — пропустить их. Затем:

```bash
git add docs/superpowers
git commit -m "docs: mark the glass redesign and DXGI GPU check as superseded" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

- [ ] **Step 2: Дополнить доку Tactile**

В доке https://claude.ai/code/artifact/b90e9f28-8187-4a4e-98fe-2c8dc1e0abfe, раздел «Основной стиль: компоненты и движение», добавить под таблицу примитивов строки `Toggle`, `Slider`, `Swatches` с анатомией из спеки §3.3. Пометка: «появились в Foundry Performance, в AIM их нет». Работать через инструменты Docs: `guide topic.index`, затем `read … sinceRev:25`.

- [ ] **Step 3: Ветки — только после «да» пользователя**

```bash
git branch -vv
for b in feat/tactile-redesign feat/portal feat/v1; do echo "$b: $(git rev-list --count main..$b) коммит(ов) не в main"; done
git stash list
```

Показать пользователю список: ветки, число коммитов не в `main`, stash «focus-fix for fog drift». Удаление необратимо для незапушенных коммитов. После явного «да» и только для подтверждённых пунктов:

```bash
git branch -D <ветка>
git stash drop <stash@{N}>
```

