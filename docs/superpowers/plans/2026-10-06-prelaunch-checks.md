# Проверки до запуска — план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** статус Sqyre-сервера и доступность GPU видны в лаунчере до запуска игры; отчёты агента с Sqyre снова принимаются.

**Architecture:** Rust выводит адрес Foundry из адреса слота Sqyre, пробует `/api/status`, а если хостинг его закрыл — читает редирект `GET /game`; выведенный адрес сразу доверенный. Лаунчер читает рендерер своего WebGL и классифицирует его командой `classify_renderer`; паспорт берёт проверку из игры, если она актуальна, иначе — лаунчера.

**Tech Stack:** Rust (reqwest 0.13, url), TypeScript, Svelte 5, vitest.

**Spec:** `docs/superpowers/specs/2026-10-06-prelaunch-checks-design.md` (дополняет `2026-10-06-gpu-acceleration-check-design.md`; план той спеки уже выполнен в этой ветке).

## Global Constraints

- Ветка `feat/gpu-check`. Коммиты на английском, в конце `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Без push и релиза.
- Новые i18n-ключи — в `ru.json` и `en.json` одновременно.
- В Sqyre не логиниться и не трогать куки/сессию; разрешены только анонимные GET `/api/status` и `/game`.
- Новая команда регистрируется в трёх местах: `src-tauri/build.rs`, `src-tauri/capabilities/launcher.json` (только окно лаунчера), `generate_handler!` в `src-tauri/src/lib.rs`.
- Проверки: `npm run check`, `npm test` (корень `pult`), `cargo test`, `cargo clippy --all-targets` (`pult/src-tauri`).

---

### Task 1: Адрес Foundry для Sqyre и запасная проба

**Files:** Modify `src-tauri/src/probe.rs`, `src-tauri/src/telemetry.rs` (`session_origins`, тесты)

**Interfaces — Produces:** `probe::hosted_foundry(url: &Url) -> Option<Url>`, `probe::parse_game_redirect(status: u16, location: Option<&str>) -> ProbeResult`.

- [ ] **Step 1: Падающие тесты в `probe.rs`** (в `mod tests`)

```rust
    fn sqyre(u: &str) -> Option<String> {
        hosted_foundry(&normalize_url(u).unwrap()).map(|u| u.to_string())
    }

    #[test]
    fn sqyre_game_page_maps_to_its_foundry_host() {
        let want = Some("https://aldarionv210-3c11b9b6.games.sqyre.app/".to_string());
        assert_eq!(sqyre("https://www.sqyre.app/games/aldarionv210-3c11b9b6/"), want);
        assert_eq!(sqyre("https://www.sqyre.app/games/aldarionv210-3c11b9b6"), want);
        assert_eq!(sqyre("sqyre.app/games/aldarionv210-3c11b9b6"), want);
        assert_eq!(sqyre("https://vtt.example.com/games/x/"), None);
        assert_eq!(sqyre("https://www.sqyre.app/games/"), None);
        assert_eq!(sqyre("https://www.sqyre.app/games/x/extra/"), None);
        assert_eq!(sqyre("https://www.sqyre.app/assets/x/"), None);
        assert_eq!(sqyre("https://www.sqyre.app/games/a.b/"), None);
    }

    #[test]
    fn game_redirect_reveals_foundry_behind_a_closed_status() {
        let active = parse_game_redirect(302, Some("/join"));
        assert!(active.reachable && active.foundry && active.active);
        assert!(parse_game_redirect(302, Some("https://x.games.sqyre.app/join?x=1")).active);
        for to in ["/setup", "/auth", "/license"] {
            let r = parse_game_redirect(302, Some(to));
            assert!(r.foundry && !r.active, "{to}");
        }
        assert_eq!(parse_game_redirect(502, None), ProbeResult::default());
        for (code, to) in [(200, None), (404, None), (302, Some("https://www.sqyre.app/games/x"))] {
            let r = parse_game_redirect(code, to);
            assert!(r.reachable && !r.foundry, "{code}");
        }
    }

    /// Вживую: `cargo test live_sqyre_slot_reports_foundry -- --ignored` (сервер должен работать).
    #[tokio::test]
    #[ignore]
    async fn live_sqyre_slot_reports_foundry() {
        let r = probe("https://www.sqyre.app/games/aldarionv210-3c11b9b6/").await;
        assert!(r.foundry, "{r:?}");
    }
```

В `telemetry.rs` `mod tests` — новый тест, а в `reports_are_trusted_only_from_session_pages` заменить `assert_eq!(origins.len(), 2);` на `assert_eq!(origins.len(), 3); // + выведенный хост Sqyre`:

```rust
    #[test]
    fn sqyre_game_host_is_trusted_from_launch() {
        let s = Server { url: "https://www.sqyre.app/games/aldarionv210-3c11b9b6/".into(), ..srv("a") };
        let origins = session_origins(&s);
        assert!(page_trusted(&"https://aldarionv210-3c11b9b6.games.sqyre.app/game".parse().unwrap(), &origins));
        assert!(!page_trusted(&"https://other.games.sqyre.app/game".parse().unwrap(), &origins));
    }
```

- [ ] **Step 2:** `cd src-tauri && cargo test --lib` — Expected: FAIL (нет `hosted_foundry`, `parse_game_redirect`).

- [ ] **Step 3: Реализация в `probe.rs`** — после `parse_status`:

```rust
/// Хостинги, где адрес слота — страница хостинга, а Foundry живёт по другому адресу.
/// Sqyre: `www.sqyre.app/games/<slug>/` → `<slug>.games.sqyre.app`.
pub fn hosted_foundry(url: &Url) -> Option<Url> {
    if !matches!(url.host_str()?, "www.sqyre.app" | "sqyre.app") {
        return None;
    }
    let segs: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();
    let ["games", slug] = segs.as_slice() else { return None };
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return None;
    }
    Url::parse(&format!("https://{slug}.games.sqyre.app/")).ok()
}

/// Ответ `GET /game` без редиректов — когда хостинг закрыл `/api/status`.
/// Foundry отправляет гостя на `/join`, если мир запущен, и на `/setup` / `/auth`, если нет.
pub fn parse_game_redirect(status: u16, location: Option<&str>) -> ProbeResult {
    if status >= 500 {
        return ProbeResult::default();
    }
    let redirect = (300..400).contains(&status);
    let path = location.and_then(|l| l.split(['?', '#']).next()).map(|p| p.trim_end_matches('/'));
    let foundry = |active| ProbeResult { reachable: true, foundry: true, active, ..ProbeResult::default() };
    match path {
        Some(p) if redirect && p.ends_with("/join") => foundry(true),
        Some(p) if redirect && ["/setup", "/auth", "/license"].iter().any(|r| p.ends_with(r)) => foundry(false),
        _ => ProbeResult { reachable: true, ..ProbeResult::default() },
    }
}
```

После `fn client()`:

```rust
/// Для `/game`: редирект и есть ответ, следовать ему не нужно.
fn bare_client() -> Option<&'static reqwest::Client> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| reqwest::Client::builder().timeout(Duration::from_secs(3)).redirect(reqwest::redirect::Policy::none()).build().ok())
        .as_ref()
}
```

Тело `probe` заменить на:

```rust
pub async fn probe(input: &str) -> ProbeResult {
    let Ok(slot) = normalize_url(input) else { return ProbeResult::default() };
    let base = hosted_foundry(&slot).unwrap_or(slot);
    let (Ok(status_url), Ok(game_url)) = (base.join("api/status"), base.join("game")) else { return ProbeResult::default() };
    let (Some(client), Some(bare)) = (client(), bare_client()) else { return ProbeResult::default() };
    let Ok(resp) = client.get(status_url).send().await else { return ProbeResult::default() };
    if resp.status().is_server_error() {
        return ProbeResult::default();
    }
    if let Some(r) = parse_status(&resp.text().await.unwrap_or_default()) {
        return r;
    }
    // Хостинг закрыл /api/status — Foundry выдаёт себя редиректом с /game
    match bare.get(game_url).send().await {
        Err(_) => ProbeResult { reachable: true, ..ProbeResult::default() },
        Ok(r) => parse_game_redirect(r.status().as_u16(), r.headers().get(reqwest::header::LOCATION).and_then(|v| v.to_str().ok())),
    }
}
```

- [ ] **Step 4: `session_origins` в `telemetry.rs`** — цикл заменить на:

```rust
    for u in [Some(server.url.as_str()), server.game_url.as_deref()].into_iter().flatten() {
        let Ok(u) = crate::probe::normalize_url(u) else { continue };
        // Страница игры хостинга доверенная сразу — проба её /api/status может быть закрыта
        for o in [Some(u.origin()), crate::probe::hosted_foundry(&u).map(|h| h.origin())].into_iter().flatten() {
            if !out.contains(&o) {
                out.push(o);
            }
        }
    }
```

- [ ] **Step 5:** `cargo test && cargo clippy --all-targets` — PASS. Затем `cargo test live_sqyre_slot_reports_foundry -- --ignored` — PASS, если сервер Aldarion сейчас работает (иначе зафиксировать фактический ответ в отчёте пользователю).

- [ ] **Step 6: Commit** `fix: Sqyre slots probe their Foundry host and trust it from launch`

---

### Task 2: Команда `classify_renderer`

**Files:** Modify `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/build.rs`, `src-tauri/capabilities/launcher.json`

**Interfaces — Produces:** команда `classify_renderer(renderer: String) -> Result<Verdict, String>` (JSON `Verdict`), чистая `commands::classify_launcher(renderer: &str, adapters: &[gpu::GpuInfo]) -> Result<Verdict, String>`.

- [ ] **Step 1: Падающий тест** (`commands.rs`, `mod tests`)

```rust
    #[test]
    fn launcher_renderer_is_classified_and_length_limited() {
        let rtx = gpu::GpuInfo { name: "RTX".into(), vram_mb: 12288, vendor_id: 0x10DE };
        let nv = "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 (0x00002F04) Direct3D11 vs_5_0 ps_5_0, D3D11)";
        assert_eq!(classify_launcher(nv, std::slice::from_ref(&rtx)), Ok(Verdict::Hardware { backend: Some(AngleBackend::D3d11) }));
        assert!(classify_launcher(&"x".repeat(513), &[rtx]).is_err());
    }
```

- [ ] **Step 2:** `cargo test --lib commands::` — FAIL.

- [ ] **Step 3: Реализация** (`commands.rs`, рядом с `gpu_check_current`):

```rust
/// Рендерер WebGL самого лаунчера: доступность GPU до запуска игры. Ничего не сохраняет.
pub fn classify_launcher(renderer: &str, adapters: &[gpu::GpuInfo]) -> Result<Verdict, String> {
    if renderer.len() > 512 {
        return Err("rejected".into());
    }
    Ok(gpu::classify(renderer, adapters))
}

#[tauri::command]
pub fn classify_renderer(state: State<'_, AppState>, renderer: String) -> Result<Verdict, String> {
    classify_launcher(&renderer, &state.adapters)
}
```

Регистрация: `"classify_renderer",` в список `build.rs` после `"clear_cache",`; `"allow-classify-renderer"` в `launcher.json` после `"allow-clear-cache"`; `commands::classify_renderer,` в `generate_handler!` после `commands::clear_cache,`.

- [ ] **Step 4:** `cargo test && cargo clippy --all-targets` — PASS.
- [ ] **Step 5: Commit** `feat: classify_renderer command for the launcher's own WebGL`

---

### Task 3: Ускорение до запуска в паспорте

**Files:** Create `src/lib/gpu-probe.ts`, `src/lib/gpu-probe.test.ts`; Modify `src/lib/accel.ts`, `src/lib/accel.test.ts`, `src/lib/api.ts`, `src/lib/store.svelte.ts`, `src/lib/mock.ts`, `src/screens/MainScreen.svelte`, `src/lib/i18n/{ru,en}.json`

**Interfaces — Consumes:** команда `classify_renderer` (Task 2). **Produces:** `launcherRenderer(make?)`, `app.launcherVerdict: Verdict | null`, `accelView(settings, current, launcher)` → `{ state, api, led, source: "game" | "launcher" | null }`.

- [ ] **Step 1: Падающие тесты**

`src/lib/gpu-probe.test.ts`:

```ts
import { describe, expect, it, vi } from "vitest";
import { launcherRenderer } from "./gpu-probe";

const UNMASKED = 0x9246;
function fakeGl(name: unknown) {
	const loseContext = vi.fn();
	const gl = {
		getExtension: (n: string) =>
			n === "WEBGL_debug_renderer_info" ? { UNMASKED_RENDERER_WEBGL: UNMASKED } : n === "WEBGL_lose_context" ? { loseContext } : null,
		getParameter: (p: number) => (p === UNMASKED ? name : undefined)
	};
	return { gl, loseContext };
}
const canvas = (ctx: Record<string, unknown>) => () => ({ getContext: (k: string) => ctx[k] ?? null }) as unknown as HTMLCanvasElement;

describe("launcherRenderer", () => {
	it("reads the renderer and releases the context", () => {
		const { gl, loseContext } = fakeGl("ANGLE (NVIDIA, x)");
		expect(launcherRenderer(canvas({ webgl2: gl }))).toBe("ANGLE (NVIDIA, x)");
		expect(loseContext).toHaveBeenCalled();
	});

	it("falls back to WebGL 1", () => {
		expect(launcherRenderer(canvas({ webgl: fakeGl("r").gl }))).toBe("r");
	});

	it("is undefined without a context or a renderer string, and never throws", () => {
		expect(launcherRenderer(canvas({}))).toBeUndefined();
		expect(launcherRenderer(canvas({ webgl2: fakeGl("").gl }))).toBeUndefined();
		expect(
			launcherRenderer(() => {
				throw new Error("no canvas");
			})
		).toBeUndefined();
	});
});
```

`src/lib/accel.test.ts` — во всех существующих вызовах добавить третий аргумент `null`, в ожидаемые объекты — `source: "game"` (для `on` / `software` / `wrongGpu`) и `source: null` (в `unchecked`). Новый блок:

```ts
describe("accelView before launch", () => {
	it("uses the launcher check when the game has none", () => {
		expect(accelView(settings(null), true, { kind: "hardware", backend: "d3d11" })).toEqual({ state: "on", api: "d3d11", led: "ok", source: "launcher" });
		expect(accelView(settings(null), true, { kind: "software" })).toEqual({ state: "software", api: "d3d11", led: "err", source: "launcher" });
	});

	it("treats the launcher on an integrated GPU as available", () => {
		expect(accelView(settings(null), true, { kind: "wrongGpu", backend: "d3d11" }).state).toBe("on");
	});

	it("prefers a current game check and ignores unknown launcher results", () => {
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), true, { kind: "software" }).source).toBe("game");
		expect(accelView(settings({ kind: "hardware", backend: "gl" }), false, { kind: "hardware", backend: "gl" })).toEqual({ state: "on", api: "d3d11", led: "ok", source: "launcher" });
		expect(accelView(settings(null), true, { kind: "unknown" }).state).toBe("unchecked");
	});
});
```

- [ ] **Step 2:** `npx vitest run src/lib/gpu-probe.test.ts src/lib/accel.test.ts` — FAIL.

- [ ] **Step 3: `src/lib/gpu-probe.ts`**

```ts
/** Рендерер WebGL самого лаунчера: доступен ли GPU для WebView2 ещё до запуска игры. */
export function launcherRenderer(make: () => HTMLCanvasElement = () => document.createElement("canvas")): string | undefined {
	try {
		const c = make();
		const gl = (c.getContext("webgl2") ?? c.getContext("webgl")) as WebGLRenderingContext | null;
		if (!gl) return undefined;
		const ext = gl.getExtension("WEBGL_debug_renderer_info");
		const name: unknown = ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : undefined;
		gl.getExtension("WEBGL_lose_context")?.loseContext();
		return typeof name === "string" && name ? name : undefined;
	} catch {
		return undefined;
	}
}
```

- [ ] **Step 4: `src/lib/accel.ts`** — заменить `accelView`:

```ts
export type AccelSource = "game" | "launcher" | null;

const STATE: Record<Verdict["kind"], AccelState> = { hardware: "on", software: "software", wrongGpu: "wrongGpu", unknown: "unchecked" };

/**
 * Статус ускорения для паспорта. Проверка из игры точнее (флаги игры) — она главная, если актуальна;
 * иначе — WebGL лаунчера: он рисует без --force-high-performance-gpu, поэтому встройка у него законна.
 */
export function accelView(
	settings: Settings,
	current: boolean,
	launcher: Verdict | null
): { state: AccelState; api: AngleBackend; led: LedState; source: AccelSource } {
	const game = current ? settings.gpuCheck?.verdict : undefined;
	if (game && game.kind !== "unknown") {
		const state = STATE[game.kind];
		const backend = "backend" in game ? game.backend : null;
		return { state, api: backend ?? settings.engine.angle, led: LED[state], source: "game" };
	}
	if (launcher && launcher.kind !== "unknown") {
		const state: AccelState = launcher.kind === "software" ? "software" : "on";
		return { state, api: settings.engine.angle, led: LED[state], source: "launcher" };
	}
	return { state: "unchecked", api: settings.engine.angle, led: LED.unchecked, source: null };
}
```

Импорт типов: `import type { AngleBackend, LedState, Settings, Verdict } from "./types";`.

Run: `npx vitest run src/lib/gpu-probe.test.ts src/lib/accel.test.ts` — PASS.

- [ ] **Step 5: API, стор, моки**

`api.ts`: импорт `Verdict`; в `api` после `probe`: `classifyRenderer: (renderer: string) => call<Verdict>("classify_renderer", { renderer }),`.

`store.svelte.ts`: импорт `launcherRenderer` из `./gpu-probe` и типа `Verdict`; поле рядом с `probes`:

```ts
	/** Доступность GPU до запуска — по WebGL самого лаунчера (null — не проверено). */
	launcherVerdict = $state<Verdict | null>(null);
```

в `load()` после `this.applyLocale();` — `void this.checkLauncherGpu();`, и метод после `load`:

```ts
	private async checkLauncherGpu(): Promise<void> {
		const renderer = launcherRenderer();
		if (!renderer) return;
		try {
			this.launcherVerdict = await api.classifyRenderer(renderer);
		} catch {
			this.launcherVerdict = null;
		}
	}
```

`mock.ts`: в `switch` новый `case`:

```ts
		case "classify_renderer": {
			const l = new URLSearchParams(location.search).get("launcher") ?? "on";
			const v: Record<string, Verdict> = { on: { kind: "hardware", backend: "d3d11" }, software: { kind: "software" }, wrongGpu: { kind: "wrongGpu", backend: "d3d11" } };
			return v[l] ?? { kind: "unknown" };
		}
```

и ветку Sqyre в `probe_server` — на работающий Foundry (Rust теперь сам находит хост игры):

```ts
			if (String(args.url).includes("sqyre.app/games")) return { reachable: true, foundry: true, active: true, version: null, world: null, system: null, users: null };
```

- [ ] **Step 6: Паспорт и i18n**

`MainScreen.svelte`: `const accel = $derived(accelView(dto.settings, dto.gpuCheckCurrent, app.launcherVerdict));` и в подсказке

```svelte
{@attach tip(() => ({ title: t(`accel.${accel.state}`), body: t(accel.source === "launcher" ? `accel.tip.launcher.${accel.state}` : `accel.tip.${accel.state}`) }))}
```

`ru.json` после `"accel.tip.unchecked"`:

```json
  "accel.tip.launcher.on": "Видеокарта доступна. Точный движок и видеокарта игры станут известны после её запуска.",
  "accel.tip.launcher.software": "Лаунчер не получил доступа к видеокарте — скорее всего, и игра не получит. Обнови драйвер видеокарты.",
```

`en.json`:

```json
  "accel.tip.launcher.on": "The GPU is available. The game's exact engine and GPU will be known after it launches.",
  "accel.tip.launcher.software": "The launcher got no access to the GPU — the game likely won't either. Update the GPU driver.",
```

- [ ] **Step 7:** `npm test && npm run check` — PASS.

- [ ] **Step 8: Превью** (`preview_start` `launcher-preview`): `?accel=none&launcher=on` → «ВКЛ», подсказка про лаунчер; `?accel=none&launcher=software` → «ПРОГРАММНОЕ», красная; `?accel=none&launcher=none` → «НЕ ПРОВЕРЕНО»; `?accel=on&launcher=software` → «ВКЛ» из игры. Слот Aldarion — зелёная лампочка. Остановить превью.

- [ ] **Step 9: Commit** `feat: GPU availability before launch from the launcher's own WebGL`

---

### Task 4: Проверка и сборка

- [ ] `npm run check && npm test && npm run build:agent`; `cargo test && cargo clippy --all-targets`.
- [ ] `npm run dist`.
- [ ] Смоук портативной сборки на изолированных APPDATA/LOCALAPPDATA: паспорт сразу «УСКОР. — ВКЛ» (RTX 5070). Закрыть по PID.
- [ ] Отчёт пользователю: что видно до запуска, как проверить Sqyre-статус у себя (слот Aldarion после запуска лаунчера), обновлённый чек-лист проверки в игре.
