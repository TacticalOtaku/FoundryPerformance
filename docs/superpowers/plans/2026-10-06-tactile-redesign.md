# Редизайн лаунчера на Tactile — план реализации

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Перевести лаунчер Foundry Performance с ретро-пульта на Tactile «Монитор под стеклом»:
туман-статус, три яруса стекла, перетекающая капля, тихая линза FPS, раздел «Вид» в «Настройке».

**Architecture:** Слой Tactile живёт внутри приложения: токены и классы материалов в CSS,
поведение — Svelte-компоненты (`Field`, `Groove`, `DropList`, `Lens`, `Ticks`, `Swatches`) и
`lib/motion.ts` на GSAP. Решения, которые можно проверить без DOM, вынесены в чистые функции
(`palette`, `aura`, `motion-pref`, `profile-tip`, `radio`, `lens`) и покрыты vitest. В Rust
добавляются только два поля настроек.

**Tech Stack:** Svelte 5 (runes, `{@attach}`), TypeScript 6, Vite 8, Vitest 5, GSAP 3.13+ с `CustomEase`,
@fontsource (Onest, DotGothic16, JetBrains Mono), Tauri 2 / Rust (serde).

**Spec:** `docs/superpowers/specs/2026-10-06-tactile-redesign-design.md`

## Global Constraints

- Никакого `backdrop-filter` и `filter: blur` на крупных слоях.
- Анимировать только `transform`, `opacity` и `@property`-числа (`--tc-acc-h`, `--tc-acc-c`, `--aura-h`, `--aura-c`).
- Режим «меньше» = настройка `motion: "reduced"` **или** системный `prefers-reduced-motion`; в нём только прозрачность за 150 ms.
- Фоновое движение (дыхание ауры, дрейф тумана) — только пока окно лаунчера в фокусе и видно.
- Каждая новая строка интерфейса — в `src/lib/i18n/ru.json` **и** `src/lib/i18n/en.json` (тест `parity.test.ts`).
- Акценты (h / c): персик 45/0.13, янтарь 78/0.12, шалфей 145/0.08, мята 178/0.09, лазурь 235/0.10, барвинок 275/0.10, лаванда 300/0.10, орхидея 330/0.11, роза 10/0.11, сталь 250/0.035. По умолчанию мята.
- Кривые: `tactile` 0.32, 0.72, 0, 1; `settle` `M0,0 C0.18,0.9 0.3,1.04 0.52,1.02 0.7,1 0.84,1 1,1`; `breath` 0.37, 0, 0.63, 1 (только циклы ауры и тумана).
- Установщик и удаление (`InstallScreen`, `UninstallScreen`, `Lcd.svelte`) не перерисовываются, но должны работать: старые имена токенов остаются алиасами.
- Окно Tauri непрозрачное и без рамки: скругление окна даёт система, CSS-радиус у корня не ставим.
- Коммиты — conventional (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`), в конце каждого:
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Push и релиз — только по явному разрешению пользователя.
- Команды: `npm test` (vitest), `npm run check` (svelte-check), `cargo test` и `cargo clippy --all-targets -- -D warnings` из `src-tauri/`.

## Карта файлов

| Файл | Что делает |
|---|---|
| `src-tauri/src/model.rs` | + `Accent`, `Motion`, поля `Settings.accent`, `Settings.motion` |
| `src/lib/types.ts`, `src/lib/mock.ts` | зеркало новых полей |
| `src/lib/palette.ts` (+test) | 10 акцентов, `accentById`, `accentVars` |
| `src/lib/aura.ts` (+test) | `auraFor(state, accent)` → цвет и «живость» тумана |
| `src/lib/motion-pref.ts` (+test) | `reducedMotion`, `ambientOn` |
| `src/lib/profile-tip.ts` (+test) | позиции профиля и ключи их имён и подсказок |
| `src/lib/radio.ts` (+test) | `nextIndex` для клавиатуры радиогрупп |
| `src/lib/lens.ts` (+test) | `lensView`, `tickBars` |
| `src/lib/motion.ts` | GSAP: `flow`, `glide`, `breathe`, `drift`, `inhale`, `count`, `openScreen`, пауза фона |
| `src/styles/tokens.css` | токены Tactile, день/ночь, алиасы старых имён |
| `src/styles/materials.css` | `.pane`, `.well`, `.drop`, `.lens`, `.glide`, `.chip`, кант |
| `src/styles/base.css` | шрифты, сброс, `.label`, `.mono`, `.silk` |
| `src/main.ts` | импорт шрифтов и стилей |
| `src/App.svelte` | акцент и `data-motion` на `<html>`, `--aura-*` и `Field` |
| `src/components/Field.svelte` | туман |
| `src/components/Groove.svelte` | паз с каплей (замена `Segmented`) |
| `src/components/DropList.svelte` | список с каплей и подсветкой |
| `src/components/Lens.svelte`, `Ticks.svelte` | линза FPS и линейка |
| `src/components/Swatches.svelte` | выбор акцента |
| `src/components/{TitleBar,VersionTag,Slot,LaunchButton,Toggle,Fader,CacheControl,Tooltip}.svelte` | перерисованы |
| `src/components/{Knob,Segmented}.svelte` | удаляются |
| `src/screens/{MainScreen,TuningScreen,SlotEditor}.svelte` | новая раскладка и материал |

---

### Task 1: Настройки акцента и движения

**Files:**
- Modify: `src-tauri/src/model.rs` (enum `Theme` ~строка 168, `Settings` ~181–204, тесты в конце файла)
- Modify: `src/lib/types.ts:20-37`
- Modify: `src/lib/mock.ts:73-81`

**Interfaces:**
- Produces: Rust `model::Accent` (`Peach…Steel`, default `Mint`), `model::Motion` (`Full`, `Reduced`, default `Full`),
  `Settings.accent`, `Settings.motion`. TS: `export type AccentId = "peach" | "amber" | "sage" | "mint" | "azure" | "periwinkle" | "lavender" | "orchid" | "rose" | "steel"`,
  `export type MotionPref = "full" | "reduced"`, `Settings.accent: AccentId`, `Settings.motion: MotionPref`.

- [ ] **Step 1: Падающие тесты в `model.rs`** (в модуль `tests` рядом с `settings_tolerate_missing_fields`)

```rust
    #[test]
    fn settings_default_accent_and_motion() {
        let s: Settings = serde_json::from_str(r#"{"schema":1,"profile":"balance"}"#).unwrap();
        assert_eq!(s.accent, Accent::Mint);
        assert_eq!(s.motion, Motion::Full);
    }

    #[test]
    fn settings_accent_and_motion_roundtrip() {
        let s = Settings { accent: Accent::Periwinkle, motion: Motion::Reduced, ..Settings::default() };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains(r#""accent":"periwinkle""#), "{json}");
        assert!(json.contains(r#""motion":"reduced""#), "{json}");
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }
```

И в `commands.rs` рядом с существующим тестом `merge_settings` (~строка 321):

```rust
    #[test]
    fn merge_settings_keeps_accent_and_motion() {
        let stored = Settings::default();
        let incoming = Settings { accent: Accent::Rose, motion: Motion::Reduced, ..Settings::default() };
        let merged = merge_settings(&stored, incoming);
        assert_eq!(merged.accent, Accent::Rose);
        assert_eq!(merged.motion, Motion::Reduced);
    }
```

(Если в тестовом модуле `commands.rs` нет `use crate::model::{Accent, Motion};` — добавить.)

- [ ] **Step 2: Убедиться, что тесты не компилируются**

Run: `cd src-tauri && cargo test accent`
Expected: ошибка компиляции `cannot find type Accent` / `no field accent`.

- [ ] **Step 3: Типы и поля в `model.rs`** — после `enum Theme`:

```rust
/// Акцент интерфейса лаунчера; оттенки — в `src/lib/palette.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Accent {
    Peach,
    Amber,
    Sage,
    #[default]
    Mint,
    Azure,
    Periwinkle,
    Lavender,
    Orchid,
    Rose,
    Steel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Motion {
    #[default]
    Full,
    Reduced,
}
```

В `Settings` после `pub theme: Theme,`:

```rust
    pub accent: Accent,
    pub motion: Motion,
```

В `impl Default for Settings` после `theme: Theme::Auto,`:

```rust
            accent: Accent::Mint,
            motion: Motion::Full,
```

`merge_settings` менять не нужно: `..incoming` уже переносит новые поля. `SCHEMA` не меняется.

- [ ] **Step 4: Тесты Rust зелёные**

Run: `cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: все тесты PASS (113 + 3 новых), clippy без предупреждений.

- [ ] **Step 5: TS-типы** — в `src/lib/types.ts` после `export type ThemePref = …`:

```ts
export type AccentId = "peach" | "amber" | "sage" | "mint" | "azure" | "periwinkle" | "lavender" | "orchid" | "rose" | "steel";
export type MotionPref = "full" | "reduced";
```

В `interface Settings` после `theme: ThemePref;`:

```ts
	accent: AccentId;
	motion: MotionPref;
```

В `src/lib/mock.ts` в объект `settings` после `theme: "auto",`:

```ts
	accent: "mint",
	motion: "full",
```

- [ ] **Step 6: Проверка типов и тестов**

Run: `npm run check && npm test`
Expected: 0 ошибок svelte-check, vitest 81 PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/model.rs src-tauri/src/commands.rs src/lib/types.ts src/lib/mock.ts
git commit -m "feat: accent and motion settings" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Палитра, аура и предпочтение движения

**Files:**
- Create: `src/lib/palette.ts`, `src/lib/palette.test.ts`
- Create: `src/lib/aura.ts`, `src/lib/aura.test.ts`
- Create: `src/lib/motion-pref.ts`, `src/lib/motion-pref.test.ts`

**Interfaces:**
- Consumes: `AccentId`, `MotionPref` (Task 1); `AccelState` из `src/lib/accel.ts` (`"on" | "software" | "wrongGpu" | "unchecked"`).
- Produces:
  - `ACCENTS: readonly Accent[]`, `interface Accent { id: AccentId; h: number; c: number }`, `accentById(id: AccentId): Accent`, `accentVars(id: AccentId): Record<"--tc-acc-h" | "--tc-acc-c", string>`
  - `interface Aura { h: number; c: number; alive: boolean }`, `auraFor(state: AccelState | null, accent: AccentId): Aura`
  - `reducedMotion(pref: MotionPref, systemReduced: boolean): boolean`, `interface AmbientInput { alive: boolean; reduced: boolean; focused: boolean; visible: boolean }`, `ambientOn(i: AmbientInput): boolean`

- [ ] **Step 1: Падающие тесты**

`src/lib/palette.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { ACCENTS, accentById, accentVars } from "./palette";

describe("palette", () => {
	it("has ten accents in picker order", () => {
		expect(ACCENTS.map((a) => a.id)).toEqual(["peach", "amber", "sage", "mint", "azure", "periwinkle", "lavender", "orchid", "rose", "steel"]);
	});

	it("mint is 178 / 0.09", () => {
		expect(accentById("mint")).toEqual({ id: "mint", h: 178, c: 0.09 });
	});

	it("unknown id falls back to mint", () => {
		expect(accentById("ultraviolet" as never).id).toBe("mint");
	});

	it("css vars carry hue and chroma", () => {
		expect(accentVars("steel")).toEqual({ "--tc-acc-h": "250", "--tc-acc-c": "0.035" });
	});
});
```

`src/lib/aura.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { auraFor } from "./aura";

describe("auraFor", () => {
	it("working GPU: accent colour, alive", () => {
		expect(auraFor("on", "azure")).toEqual({ h: 235, c: 0.1, alive: true });
	});

	it("game on integrated GPU: amber, still", () => {
		expect(auraFor("wrongGpu", "mint")).toEqual({ h: 78, c: 0.12, alive: false });
	});

	it("software render: red, still", () => {
		expect(auraFor("software", "mint")).toEqual({ h: 24, c: 0.15, alive: false });
	});

	it("unchecked and missing GPU: almost grey accent", () => {
		expect(auraFor("unchecked", "rose")).toEqual({ h: 10, c: 0.012, alive: false });
		expect(auraFor(null, "rose")).toEqual({ h: 10, c: 0.012, alive: false });
	});
});
```

`src/lib/motion-pref.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { ambientOn, reducedMotion } from "./motion-pref";

describe("reducedMotion", () => {
	it("setting or system preference reduces motion", () => {
		expect(reducedMotion("full", false)).toBe(false);
		expect(reducedMotion("reduced", false)).toBe(true);
		expect(reducedMotion("full", true)).toBe(true);
	});
});

describe("ambientOn", () => {
	const base = { alive: true, reduced: false, focused: true, visible: true };
	it("runs only when alive, full motion, focused and visible", () => {
		expect(ambientOn(base)).toBe(true);
		expect(ambientOn({ ...base, alive: false })).toBe(false);
		expect(ambientOn({ ...base, reduced: true })).toBe(false);
		expect(ambientOn({ ...base, focused: false })).toBe(false);
		expect(ambientOn({ ...base, visible: false })).toBe(false);
	});
});
```

- [ ] **Step 2: Тесты падают**

Run: `npx vitest run src/lib/palette.test.ts src/lib/aura.test.ts src/lib/motion-pref.test.ts`
Expected: FAIL — `Failed to resolve import "./palette"` и т. п.

- [ ] **Step 3: Реализация**

`src/lib/palette.ts`:

```ts
import type { AccentId } from "./types";

export interface Accent {
	id: AccentId;
	h: number;
	c: number;
}

/** Порядок — как в выборе акцента в «Настройке». */
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

const MINT = ACCENTS[3];

export const accentById = (id: AccentId): Accent => ACCENTS.find((a) => a.id === id) ?? MINT;

export function accentVars(id: AccentId): Record<"--tc-acc-h" | "--tc-acc-c", string> {
	const a = accentById(id);
	return { "--tc-acc-h": String(a.h), "--tc-acc-c": String(a.c) };
}
```

`src/lib/aura.ts`:

```ts
import type { AccelState } from "./accel";
import { accentById } from "./palette";
import type { AccentId } from "./types";

/** Цвет тумана под окном и можно ли ему плыть и дышать. */
export interface Aura {
	h: number;
	c: number;
	alive: boolean;
}

/** `state === null` — видеокарта не найдена. */
export function auraFor(state: AccelState | null, accent: AccentId): Aura {
	const a = accentById(accent);
	switch (state) {
		case "on":
			return { h: a.h, c: a.c, alive: true };
		case "wrongGpu":
			return { h: 78, c: 0.12, alive: false };
		case "software":
			return { h: 24, c: 0.15, alive: false };
		default:
			return { h: a.h, c: 0.012, alive: false };
	}
}
```

`src/lib/motion-pref.ts`:

```ts
import type { MotionPref } from "./types";

export const reducedMotion = (pref: MotionPref, systemReduced: boolean): boolean => pref === "reduced" || systemReduced;

export interface AmbientInput {
	alive: boolean;
	reduced: boolean;
	focused: boolean;
	visible: boolean;
}

/** Фоновые циклы не отнимают кадры у игры: только в фокусе, на виду и при полном движении. */
export const ambientOn = (i: AmbientInput): boolean => i.alive && !i.reduced && i.focused && i.visible;
```

- [ ] **Step 4: Тесты зелёные**

Run: `npm test`
Expected: PASS, прибавилось 8 тестов.

- [ ] **Step 5: Commit**

```bash
git add src/lib/palette.ts src/lib/palette.test.ts src/lib/aura.ts src/lib/aura.test.ts src/lib/motion-pref.ts src/lib/motion-pref.test.ts
git commit -m "feat: accent palette, aura colour and motion preference" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Клавиатура радиогрупп, линза и подсказки профиля

**Files:**
- Create: `src/lib/radio.ts`, `src/lib/radio.test.ts`
- Create: `src/lib/lens.ts`, `src/lib/lens.test.ts`
- Create: `src/lib/profile-tip.ts`, `src/lib/profile-tip.test.ts`
- Modify: `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `KnobPosition` из `src/lib/levers.ts` (`ProfileId | "manual"`), `StateDto` из `types.ts`.
- Produces:
  - `nextIndex(current: number, key: string, enabled: boolean[]): number | null`
  - `interface LensView { value: number | null; unit: "FPS" | "%"; captionKey: string; low: number | null; history: number[] }`, `lensView(stats: StateDto["stats"][string] | undefined, updating: number | null): LensView`
  - `interface Bar { x: number; y: number; w: number; h: number; last: boolean }`, `tickBars(values: number[], width: number, height: number, gap?: number): Bar[]`
  - `PROFILE_POSITIONS: readonly KnobPosition[]`, `profileNameKey(p: KnobPosition): string`, `profileTipKey(p: KnobPosition): string`
  - i18n: `tip.manual`, `main.profile`, `main.addServer`

- [ ] **Step 1: Падающие тесты**

`src/lib/radio.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { nextIndex } from "./radio";

const all = [true, true, true, true];

describe("nextIndex", () => {
	it("arrows move and wrap", () => {
		expect(nextIndex(1, "ArrowRight", all)).toBe(2);
		expect(nextIndex(3, "ArrowDown", all)).toBe(0);
		expect(nextIndex(0, "ArrowLeft", all)).toBe(3);
		expect(nextIndex(2, "ArrowUp", all)).toBe(1);
	});

	it("Home and End jump to the ends", () => {
		expect(nextIndex(2, "Home", all)).toBe(0);
		expect(nextIndex(0, "End", all)).toBe(3);
	});

	it("skips disabled options (manual profile)", () => {
		const presets = [true, true, true, false];
		expect(nextIndex(2, "ArrowRight", presets)).toBe(0);
		expect(nextIndex(0, "ArrowLeft", presets)).toBe(2);
		expect(nextIndex(3, "ArrowLeft", presets)).toBe(2);
		expect(nextIndex(1, "End", presets)).toBe(2);
	});

	it("no selection starts from the edge", () => {
		expect(nextIndex(-1, "ArrowRight", all)).toBe(0);
		expect(nextIndex(-1, "ArrowLeft", all)).toBe(3);
	});

	it("other keys and empty groups do nothing", () => {
		expect(nextIndex(1, "Enter", all)).toBeNull();
		expect(nextIndex(0, "ArrowRight", [false, false])).toBeNull();
	});
});
```

`src/lib/lens.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { lensView, tickBars } from "./lens";

const session = { avg: 52.4, low1: 31, profile: "balance" as const, at: 1 };
const bench = { avg: 58, low1: 40, min: 22, profile: "balance" as const, at: 2 };

describe("lensView", () => {
	it("no data", () => {
		expect(lensView(undefined, null)).toEqual({ value: null, unit: "FPS", captionKey: "main.noData", low: null, history: [] });
	});

	it("bench wins over session", () => {
		const v = lensView({ lastSession: session, lastBench: bench, history: [40, 50] }, null);
		expect(v).toEqual({ value: 58, unit: "FPS", captionKey: "main.lastBench", low: 40, history: [40, 50] });
	});

	it("session without history", () => {
		const v = lensView({ lastSession: session, lastBench: null }, null);
		expect(v).toEqual({ value: 52.4, unit: "FPS", captionKey: "main.lastSession", low: 31, history: [] });
	});

	it("update download shows percent", () => {
		const v = lensView({ lastSession: session, lastBench: null, history: [1] }, 37);
		expect(v).toEqual({ value: 37, unit: "%", captionKey: "update.downloading", low: null, history: [] });
	});
});

describe("tickBars", () => {
	it("empty history draws nothing", () => {
		expect(tickBars([], 100, 30)).toEqual([]);
	});

	it("bars share the width and the last one is marked", () => {
		const bars = tickBars([10, 20], 100, 30, 4);
		expect(bars).toHaveLength(2);
		expect(bars[0]).toMatchObject({ x: 0, w: 48, h: 10, y: 20, last: false });
		expect(bars[1].x).toBe(52);
		expect(bars[1].h).toBeCloseTo(26.667, 2);
		expect(bars[1].last).toBe(true);
	});

	it("keeps only the last 32 values", () => {
		expect(tickBars(Array.from({ length: 50 }, (_, i) => i), 320, 30)).toHaveLength(32);
	});
});
```

`src/lib/profile-tip.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import en from "./i18n/en.json";
import ru from "./i18n/ru.json";
import { PROFILE_POSITIONS, profileNameKey, profileTipKey } from "./profile-tip";

describe("profile tips", () => {
	it("positions in groove order", () => {
		expect(PROFILE_POSITIONS).toEqual(["quality", "balance", "potato", "manual"]);
	});

	it("every position has a name and a tip in both languages", () => {
		for (const p of PROFILE_POSITIONS) {
			for (const dict of [ru, en]) {
				expect(dict, profileNameKey(p)).toHaveProperty([profileNameKey(p)]);
				expect(dict, profileTipKey(p)).toHaveProperty([profileTipKey(p)]);
			}
		}
	});
});
```

- [ ] **Step 2: Тесты падают**

Run: `npx vitest run src/lib/radio.test.ts src/lib/lens.test.ts src/lib/profile-tip.test.ts`
Expected: FAIL — модулей нет.

- [ ] **Step 3: Реализация**

`src/lib/radio.ts`:

```ts
/**
 * Следующий выбор в радиогруппе по клавише. `enabled[i] === false` — пункт пропускается
 * стрелками (например, «РУЧ»: его выбирают только мышью). `null` — клавиша не наша.
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

`src/lib/lens.ts`:

```ts
import type { StateDto } from "./types";

type ServerStats = StateDto["stats"][string];

export interface LensView {
	value: number | null;
	unit: "FPS" | "%";
	captionKey: string;
	low: number | null;
	history: number[];
}

/** Что показывает линза: проценты загрузки обновления, иначе последний замер или сеанс. */
export function lensView(stats: ServerStats | undefined, updating: number | null): LensView {
	if (updating !== null) return { value: updating, unit: "%", captionKey: "update.downloading", low: null, history: [] };
	const last = stats?.lastBench ?? stats?.lastSession ?? null;
	if (!stats || !last) return { value: null, unit: "FPS", captionKey: "main.noData", low: null, history: [] };
	return {
		value: last.avg,
		unit: "FPS",
		captionKey: stats.lastBench ? "main.lastBench" : "main.lastSession",
		low: last.low1,
		history: stats.history ?? []
	};
}

export interface Bar {
	x: number;
	y: number;
	w: number;
	h: number;
	last: boolean;
}

const MAX_BARS = 32;

/** Линейка тонких баров: низ шкалы на 6 FPS ниже минимума, чтобы короткий бар не исчезал. */
export function tickBars(values: number[], width: number, height: number, gap = 3): Bar[] {
	const v = values.slice(-MAX_BARS);
	if (!v.length) return [];
	const min = Math.min(...v) - 6;
	const max = Math.max(...v) + 2;
	const w = (width - gap * (v.length - 1)) / v.length;
	return v.map((x, i) => {
		const h = Math.max(2, ((x - min) / (max - min)) * height);
		return { x: i * (w + gap), y: height - h, w, h, last: i === v.length - 1 };
	});
}
```

`src/lib/profile-tip.ts`:

```ts
import type { KnobPosition } from "./levers";

/** Порядок в пазу профиля на главном экране. */
export const PROFILE_POSITIONS: readonly KnobPosition[] = ["quality", "balance", "potato", "manual"];

export const profileNameKey = (p: KnobPosition): string => `profile.${p}.long`;
export const profileTipKey = (p: KnobPosition): string => `tip.${p}`;
```

- [ ] **Step 4: Строки** — добавить в `src/lib/i18n/ru.json` (в конец объекта, сохраняя валидный JSON):

```json
  "tip.manual": "Свои положения рычагов из «Настройки» для этого сервера.",
  "main.profile": "ПРОФИЛЬ",
  "main.addServer": "Добавить сервер"
```

и в `src/lib/i18n/en.json`:

```json
  "tip.manual": "Your own lever positions from Settings for this server.",
  "main.profile": "PROFILE",
  "main.addServer": "Add server"
```

- [ ] **Step 5: Тесты зелёные**

Run: `npm test`
Expected: PASS (radio 5, lens 7, profile-tip 2, parity зелёный).

- [ ] **Step 6: Commit**

```bash
git add src/lib/radio.ts src/lib/radio.test.ts src/lib/lens.ts src/lib/lens.test.ts src/lib/profile-tip.ts src/lib/profile-tip.test.ts src/lib/i18n/ru.json src/lib/i18n/en.json
git commit -m "feat: radio keyboard, lens view and profile tips" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Шрифты, токены и материалы Tactile

**Files:**
- Modify: `package.json` (через npm)
- Modify: `src/main.ts`
- Rewrite: `src/styles/tokens.css`, `src/styles/base.css`
- Create: `src/styles/materials.css`

**Interfaces:**
- Produces (CSS, используют все следующие задачи):
  - токены `--tc-acc-h`, `--tc-acc-c`, `--tc-bg`, `--tc-ink`, `--tc-muted`, `--tc-faint`, `--tc-line`, `--tc-danger`, `--tc-warning`, `--tc-good`, `--tc-off`, `--tc-accent`, `--tc-accent-hi`, `--tc-accent-text`, `--tc-on-accent`, `--tc-glow`, `--g-pane-solid`, `--g-rim`, `--g-shadow`, `--g-well`, `--g-well-shadow`, `--g-drop`, `--g-drop-shadow`, `--g-spec`, `--g-lens`, `--g-lens-top`, `--g-hover`, `--field-base`, `--blob-l1..3`, `--blob-a`, `--tc-tip-bg`, `--tc-tip-fg`, `--tc-tip-muted`, `--tc-font-ui`, `--tc-font-mono`, `--tc-font-display`, `--tc-r-block` (16px), `--tc-r-row` (12px), `--tc-r-ctl` (999px), `--tc-ease`; `@property --aura-h`, `--aura-c`;
  - классы `.pane`, `.well`, `.drop`, `.lens`, `.glide`, `.chip`, `.label`, `.dot`, `.num`;
  - старые имена (`--panel`, `--face`, `--signal`, `--led-*`, `--grain`, `--bevel`, `--recess`, `--lift`, `--font-ui`, `--font-mono`, …) — алиасы.

- [ ] **Step 1: Зависимости**

Run:

```bash
npm install gsap @fontsource/onest @fontsource/dotgothic16 @fontsource/jetbrains-mono
npm uninstall @fontsource/geologica @fontsource/martian-mono
```

Expected: `package.json` → `dependencies` содержит `gsap` (^3.13 или новее), три `@fontsource/*`, без geologica и martian-mono.

- [ ] **Step 2: `src/main.ts`** — заменить импорты шрифтов и стилей:

```ts
import "@fontsource/onest/400.css";
import "@fontsource/onest/500.css";
import "@fontsource/onest/600.css";
import "@fontsource/onest/700.css";
import "@fontsource/dotgothic16/400.css";
import "@fontsource/jetbrains-mono/400.css";
import "@fontsource/jetbrains-mono/500.css";
import "@fontsource/jetbrains-mono/600.css";
import "./styles/tokens.css";
import "./styles/materials.css";
import "./styles/base.css";
import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
```

- [ ] **Step 3: `src/styles/tokens.css`** — полностью заменить:

```css
/* Tactile · Soft Clinical Glow. День — :root, ночь — [data-theme="night"] (ставит App.svelte).
   Акцент задаётся числами --tc-acc-h / --tc-acc-c (App.svelte из настроек), цвет тумана — --aura-h / --aura-c. */
@property --tc-acc-h { syntax: "<number>"; inherits: true; initial-value: 178; }
@property --tc-acc-c { syntax: "<number>"; inherits: true; initial-value: 0.09; }
@property --aura-h { syntax: "<number>"; inherits: true; initial-value: 178; }
@property --aura-c { syntax: "<number>"; inherits: true; initial-value: 0.09; }

:root {
	--tc-acc-h: 178;
	--tc-acc-c: 0.09;
	--tc-bg: #e4e8ee;
	--tc-ink: #1e2330;
	--tc-muted: #5b6376;
	--tc-faint: #8a90a0;
	--tc-line: rgb(30 35 48 / 9%);
	--tc-danger: #c2363f;
	--tc-warning: #9a6a12;
	--tc-good: #2f7d57;
	--tc-off: #a6abb7;
	--tc-accent: oklch(0.72 var(--tc-acc-c) var(--tc-acc-h));
	--tc-accent-hi: oklch(0.84 calc(var(--tc-acc-c) * 0.85) var(--tc-acc-h));
	--tc-accent-text: oklch(0.49 calc(var(--tc-acc-c) * 1.05) var(--tc-acc-h));
	--tc-on-accent: oklch(0.25 0.045 var(--tc-acc-h));
	--tc-glow: oklch(0.74 var(--tc-acc-c) var(--tc-acc-h) / 0.55);

	/* ярус 1 — лист (режим «На героях»: молочный, почти непрозрачный) */
	--g-pane-solid: linear-gradient(180deg, rgb(251 252 254 / 95%), rgb(244 246 250 / 92%));
	--g-rim: linear-gradient(180deg, rgb(255 255 255 / 100%), rgb(255 255 255 / 35%) 42%, rgb(255 255 255 / 70%));
	--g-shadow: 0 22px 44px -26px rgb(28 38 66 / 48%), 0 3px 8px -4px rgb(28 38 66 / 16%);
	/* паз */
	--g-well: rgb(28 38 66 / 6%);
	--g-well-shadow: inset 0 1px 3px rgb(28 38 66 / 14%), inset 0 -1px 0 rgb(255 255 255 / 75%);
	/* ярус 2 — капля */
	--g-drop: linear-gradient(180deg, rgb(255 255 255 / 95%), rgb(255 255 255 / 66%));
	--g-drop-shadow: inset 0 1px 0 #fff, inset 0 0 0 1px rgb(255 255 255 / 60%), inset 0 -6px 10px -7px var(--tc-glow),
		0 8px 16px -10px rgb(28 38 66 / 50%), 0 1px 2px rgb(28 38 66 / 14%);
	--g-spec: 0.9;
	/* ярус 3 — линза */
	--g-lens: linear-gradient(180deg, rgb(255 255 255 / 34%), rgb(255 255 255 / 10%));
	--g-lens-top: rgb(255 255 255 / 85%);
	--g-hover: rgb(255 255 255 / 50%);
	/* туман */
	--field-base: #e1e5ec;
	--blob-l1: 0.83;
	--blob-l2: 0.87;
	--blob-l3: 0.9;
	--blob-a: 0.9;
	/* подсказки */
	--tc-tip-bg: rgb(30 35 48 / 95%);
	--tc-tip-fg: #eef1f6;
	--tc-tip-muted: #a9b0c0;

	--tc-font-ui: "Onest", "Segoe UI", system-ui, sans-serif;
	--tc-font-mono: "JetBrains Mono", ui-monospace, Consolas, monospace;
	--tc-font-display: "DotGothic16", "JetBrains Mono", ui-monospace, monospace;
	--tc-r-block: 16px;
	--tc-r-row: 12px;
	--tc-r-ctl: 999px;
	--tc-ease: cubic-bezier(0.32, 0.72, 0, 1);

	/* алиасы старого пульта — для установщика и удаления до подпроекта 4 */
	--panel: var(--field-base);
	--face: #f5f6f9;
	--face-2: #fbfcfd;
	--well: #e2e5eb;
	--ink: var(--tc-ink);
	--ink-2: var(--tc-muted);
	--ink-3: var(--tc-faint);
	--line: #cdd2db;
	--inverse-bg: var(--tc-tip-bg);
	--inverse-fg: var(--tc-tip-fg);
	--signal: var(--tc-accent);
	--signal-hi: var(--tc-accent-hi);
	--signal-deep: oklch(0.55 var(--tc-acc-c) var(--tc-acc-h));
	--signal-ink: var(--tc-on-accent);
	--signal-text: var(--tc-accent-text);
	--lcd-bg: #16221a;
	--lcd-fg: #b8f28a;
	--lcd-dim: #7fa866;
	--lcd-ghost: #22332a;
	--led-ok: var(--tc-good);
	--led-warn: var(--tc-warning);
	--led-err: var(--tc-danger);
	--led-off: var(--tc-off);
	--grain: linear-gradient(transparent, transparent);
	--bevel: inset 0 1px 0 rgb(255 255 255 / 70%);
	--recess: var(--g-well-shadow);
	--lift: var(--g-drop-shadow);
	--glow-lcd: 0 0 6px rgb(184 242 138 / 0.35);
	--glow-signal: 0 0 8px var(--tc-glow);
	--led-halo: 0.18;
	--engrave: none;
	--knurl-a: #9a978f;
	--knurl-b: #85827a;
	--cap-hi: #ffffff;
	--cap-lo: #dde2ea;
	--font-ui: var(--tc-font-ui);
	--font-mono: var(--tc-font-mono);
	--ease-detent: cubic-bezier(0.3, 1.35, 0.5, 1);
	--ease-out: var(--tc-ease);
	color-scheme: light;
	transition:
		--tc-acc-h 700ms var(--tc-ease),
		--tc-acc-c 700ms var(--tc-ease);
}

:root[data-theme="night"] {
	--tc-bg: #141925;
	--tc-ink: #e8ebf4;
	--tc-muted: #9aa3b9;
	--tc-faint: #6a7288;
	--tc-line: rgb(220 228 255 / 8%);
	--tc-danger: #ff7a82;
	--tc-warning: #f2c46b;
	--tc-good: #7fd3a6;
	--tc-off: #4a5268;
	--tc-accent: oklch(0.78 var(--tc-acc-c) var(--tc-acc-h));
	--tc-accent-hi: oklch(0.87 calc(var(--tc-acc-c) * 0.8) var(--tc-acc-h));
	--tc-accent-text: oklch(0.85 calc(var(--tc-acc-c) * 0.9) var(--tc-acc-h));
	--tc-glow: oklch(0.78 var(--tc-acc-c) var(--tc-acc-h) / 0.6);
	--g-pane-solid: linear-gradient(180deg, rgb(37 44 64 / 97%), rgb(29 35 52 / 95%));
	--g-rim: linear-gradient(180deg, rgb(255 255 255 / 28%), rgb(255 255 255 / 5%) 42%, rgb(255 255 255 / 11%));
	--g-shadow: 0 22px 44px -24px rgb(0 0 0 / 75%), 0 3px 8px -4px rgb(0 0 0 / 45%);
	--g-well: rgb(0 0 0 / 24%);
	--g-well-shadow: inset 0 1px 3px rgb(0 0 0 / 42%), inset 0 -1px 0 rgb(255 255 255 / 5%);
	--g-drop: linear-gradient(180deg, rgb(255 255 255 / 22%), rgb(255 255 255 / 8%));
	--g-drop-shadow: inset 0 1px 0 rgb(255 255 255 / 38%), inset 0 0 0 1px rgb(255 255 255 / 10%), inset 0 -6px 10px -7px var(--tc-glow),
		0 8px 16px -10px rgb(0 0 0 / 70%), 0 1px 2px rgb(0 0 0 / 40%);
	--g-spec: 0.35;
	--g-lens: linear-gradient(180deg, rgb(255 255 255 / 9%), rgb(255 255 255 / 2%));
	--g-lens-top: rgb(255 255 255 / 22%);
	--g-hover: rgb(255 255 255 / 6%);
	--field-base: #121722;
	--blob-l1: 0.5;
	--blob-l2: 0.42;
	--blob-l3: 0.36;
	--blob-a: 0.8;
	--tc-tip-bg: rgb(232 236 244 / 96%);
	--tc-tip-fg: #1e2330;
	--tc-tip-muted: #5b6376;

	--face: #252b3d;
	--face-2: #2c3348;
	--well: #1a1f2d;
	--line: #343c52;
	--lcd-bg: #0e170f;
	--lcd-ghost: #1a2a1c;
	--bevel: inset 0 1px 0 rgb(255 255 255 / 6%);
	--led-halo: 0.2;
	--knurl-a: #3a3a36;
	--knurl-b: #282826;
	--cap-hi: #4a5470;
	--cap-lo: #323a52;
	color-scheme: dark;
}

:root[data-motion="reduced"] {
	transition: none;
}
```

- [ ] **Step 4: `src/styles/materials.css`** — создать:

```css
/* Материалы Tactile: лист, паз, капля, линза. Кант — один градиент --g-rim для всех стеклянных деталей. */
.pane {
	position: relative;
	border-radius: var(--tc-r-block);
	background: var(--g-pane-solid);
	box-shadow: var(--g-shadow);
	isolation: isolate;
}
.pane::before,
.lens::before,
.drop::before {
	content: "";
	position: absolute;
	inset: 0;
	border-radius: inherit;
	padding: 1px;
	background: var(--g-rim);
	pointer-events: none;
	-webkit-mask:
		linear-gradient(#000 0 0) content-box,
		linear-gradient(#000 0 0);
	-webkit-mask-composite: xor;
	mask:
		linear-gradient(#000 0 0) content-box exclude,
		linear-gradient(#000 0 0);
}
.well {
	position: relative;
	border-radius: var(--tc-r-row);
	background: var(--g-well);
	box-shadow: var(--g-well-shadow);
}

/* капля: индикатор выбора, едет трансформацией (lib/motion.ts → flow) */
.drop {
	position: absolute;
	z-index: 1;
	top: 0;
	left: 0;
	border-radius: var(--tc-r-ctl);
	background: var(--g-drop);
	box-shadow: var(--g-drop-shadow);
	pointer-events: none;
	will-change: transform;
}
.drop::after {
	content: "";
	position: absolute;
	inset: 1px 12% 48% 9%;
	border-radius: inherit;
	background: radial-gradient(120% 90% at 22% 0%, rgb(255 255 255 / 95%), rgb(255 255 255 / 0) 62%);
	opacity: var(--g-spec);
	pointer-events: none;
}

/* скользящая подсветка наведения (lib/motion.ts → glide) */
.glide {
	position: absolute;
	z-index: 0;
	top: 0;
	left: 0;
	border-radius: var(--glide-r, var(--tc-r-row));
	background: var(--g-hover);
	opacity: 0;
	pointer-events: none;
}

/* линза: окно в туман, внутри аура */
.lens {
	position: relative;
	border-radius: 18px;
	background: var(--g-lens);
	isolation: isolate;
	overflow: hidden;
	box-shadow:
		inset 0 14px 22px -18px var(--g-lens-top),
		inset 0 -22px 34px -22px oklch(0.72 calc(var(--aura-c) * 1.4) var(--aura-h) / 0.75),
		var(--g-shadow);
}
.lens::before {
	padding: 1.5px;
}

.chip {
	padding: 5px 9px;
	border-radius: var(--tc-r-ctl);
	background: var(--g-well);
	box-shadow: var(--g-well-shadow);
	color: var(--tc-muted);
	font: 500 10.5px/1 var(--tc-font-mono);
}

.num {
	font-family: var(--tc-font-display);
	font-weight: 400;
	font-variant-numeric: tabular-nums;
	letter-spacing: -0.02em;
}
```

- [ ] **Step 5: `src/styles/base.css`** — полностью заменить:

```css
*,
*::before,
*::after {
	box-sizing: border-box;
}

html,
body {
	margin: 0;
	height: 100%;
	overflow: hidden;
	background: var(--field-base);
	color: var(--tc-ink);
	font: 400 14px/1.45 var(--tc-font-ui);
	-webkit-font-smoothing: antialiased;
	user-select: none;
}

button {
	font: inherit;
	color: inherit;
	background: none;
	border: 0;
	padding: 0;
	cursor: pointer;
}

input {
	font: inherit;
	color: inherit;
	user-select: text;
}

:focus-visible {
	outline: 2px solid var(--tc-accent);
	outline-offset: 2px;
}

.mono {
	font-family: var(--tc-font-mono);
	font-size: 11px;
	letter-spacing: 0.02em;
}

/* подписи групп; .silk — старое имя для установщика */
.label,
.silk {
	font: 500 10.5px/1.3 var(--tc-font-mono);
	letter-spacing: 0.08em;
	text-transform: uppercase;
	color: var(--tc-muted);
}

/* старая лицевая пластина установщика */
.plate {
	background: var(--face);
	box-shadow: var(--bevel);
}

/* отметка «изменено вручную» */
.dot {
	display: inline-block;
	width: 6px;
	height: 6px;
	margin-left: 6px;
	border-radius: 50%;
	background: var(--tc-accent);
	box-shadow: 0 0 6px var(--tc-glow);
	vertical-align: middle;
}

@media (prefers-reduced-motion: reduce) {
	*,
	*::before,
	*::after {
		transition: none !important;
		animation: none !important;
	}
}
```

- [ ] **Step 6: Сборка и тесты**

Run: `npm run check && npm test && npx vite build`
Expected: 0 ошибок; vitest PASS; сборка проходит (предупреждения о размере чанка допустимы).

- [ ] **Step 7: Commit**

```bash
git add package.json package-lock.json src/main.ts src/styles/tokens.css src/styles/materials.css src/styles/base.css
git commit -m "feat: Tactile tokens, glass materials and fonts" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Движение и туман

**Files:**
- Create: `src/lib/motion.ts`
- Create: `src/components/Field.svelte`
- Modify: `src/App.svelte`

**Interfaces:**
- Consumes: `reducedMotion`, `ambientOn` (Task 2); `accentVars`, `auraFor` (Task 2); `accelView` из `src/lib/accel.ts` — `accelView(settings, gpuCheckCurrent, launcherVerdict)` → `{ state: AccelState, … }`.
- Produces:
  - `reduced(): boolean`
  - `flow(drop: HTMLElement, target: HTMLElement, axis: "x" | "y", instant: boolean): void`
  - `glide(selector: string): (group: HTMLElement) => () => void` — Svelte-attachment; подсвечивает прямых детей `group`, подходящих под `selector`
  - `breathe(el: Element): () => void`, `drift(blobs: Element[]): () => void`
  - `inhale(): void` — анимирует все `[data-inhale]`
  - `count(el: HTMLElement, to: number, decimals?: number): void`
  - `openScreen(root: HTMLElement): void` — оболочка + все `[data-part]` внутри
  - `Field.svelte` props: `{ alive: boolean; spots: [number, number, number][] }` (x %, y %, размер px)
  - На `.frame` в `App.svelte`: `--aura-h`, `--aura-c`; на `<html>`: `--tc-acc-h`, `--tc-acc-c`, `data-motion`

- [ ] **Step 1: `src/lib/motion.ts`**

```ts
import { gsap } from "gsap";
import { CustomEase } from "gsap/CustomEase";
import { ambientOn, reducedMotion } from "./motion-pref";
import type { MotionPref } from "./types";

gsap.registerPlugin(CustomEase);
CustomEase.create("tactile", "0.32, 0.72, 0, 1");
CustomEase.create("settle", "M0,0 C0.18,0.9 0.3,1.04 0.52,1.02 0.7,1 0.84,1 1,1");
CustomEase.create("breath", "0.37, 0, 0.63, 1");

const systemReduced = matchMedia("(prefers-reduced-motion: reduce)");

export function reduced(): boolean {
	const pref = (document.documentElement.dataset.motion ?? "full") as MotionPref;
	return reducedMotion(pref, systemReduced.matches);
}

/* ── фоновые циклы: только в фокусе и на виду, чтобы не отнимать кадры у игры ── */
const ambient = new Set<gsap.core.Animation>();

function syncAmbient(): void {
	const on = ambientOn({ alive: true, reduced: reduced(), focused: document.hasFocus(), visible: document.visibilityState === "visible" });
	for (const a of ambient) {
		if (on) a.resume();
		else a.pause();
	}
}
addEventListener("focus", syncAmbient);
addEventListener("blur", syncAmbient);
document.addEventListener("visibilitychange", syncAmbient);

function keep(a: gsap.core.Animation): () => void {
	ambient.add(a);
	syncAmbient();
	return () => {
		ambient.delete(a);
		a.kill();
	};
}

/** Капля едет к выбранному пункту, вытягиваясь по ходу и садясь с осадкой. */
export function flow(drop: HTMLElement, target: HTMLElement, axis: "x" | "y", instant: boolean): void {
	const to = axis === "x" ? target.offsetLeft : target.offsetTop;
	gsap.killTweensOf(drop);
	gsap.set(drop, { width: target.offsetWidth, height: target.offsetHeight });
	if (instant || reduced()) {
		gsap.set(drop, { [axis]: to, scaleX: 1, scaleY: 1 });
		if (!instant) gsap.fromTo(drop, { opacity: 0.35 }, { opacity: 1, duration: 0.15 });
		return;
	}
	const from = Number(gsap.getProperty(drop, axis));
	const size = axis === "x" ? target.offsetWidth : target.offsetHeight;
	const stretch = 1 + Math.min(Math.abs(to - from) / Math.max(size, 1), 2.2) * 0.26;
	const along = axis === "x" ? "scaleX" : "scaleY";
	const across = axis === "x" ? "scaleY" : "scaleX";
	gsap
		.timeline()
		.to(drop, { [axis]: to, duration: 0.46, ease: "tactile" }, 0)
		.to(drop, { [along]: stretch, [across]: 0.86, duration: 0.15, ease: "power2.out" }, 0)
		.to(drop, { [along]: 1, [across]: 1, duration: 0.55, ease: "settle" }, 0.15);
}

/** Одно пятно наведения на группу: переезжает между соседями, а не гаснет. */
export function glide(selector: string): (group: HTMLElement) => () => void {
	return (group) => {
		const hl = document.createElement("span");
		hl.className = "glide";
		hl.setAttribute("aria-hidden", "true");
		group.prepend(hl);
		const over = (e: PointerEvent) => {
			const t = (e.target as HTMLElement).closest<HTMLElement>(selector);
			if (!t || t.parentElement !== group) return;
			gsap.set(hl, { width: t.offsetWidth, height: t.offsetHeight });
			const box = { x: t.offsetLeft, y: t.offsetTop };
			const shown = Number(gsap.getProperty(hl, "opacity")) > 0.05;
			if (!shown || reduced()) {
				gsap.set(hl, box);
				gsap.to(hl, { opacity: 1, duration: 0.15, ease: "tactile" });
			} else gsap.to(hl, { ...box, opacity: 1, duration: 0.3, ease: "tactile", overwrite: "auto" });
		};
		const leave = () => gsap.to(hl, { opacity: 0, duration: 0.2, ease: "power2.in" });
		group.addEventListener("pointerover", over);
		group.addEventListener("pointerleave", leave);
		return () => {
			group.removeEventListener("pointerover", over);
			group.removeEventListener("pointerleave", leave);
			gsap.killTweensOf(hl);
			hl.remove();
		};
	};
}

/** Дыхание ауры: работает только при живом GPU (решает вызывающий). */
export function breathe(el: Element): () => void {
	if (reduced()) return () => {};
	return keep(gsap.fromTo(el, { scale: 1, opacity: 0.95 }, { scale: 1.12, opacity: 0.7, duration: 2.6, ease: "breath", yoyo: true, repeat: -1 }));
}

const DRIFT = { x: [26, -34, 22], y: [-18, 24, -26], scale: [1.08, 0.94, 1.06], duration: [9, 12, 15] };

/** Медленный дрейф пятен тумана. */
export function drift(blobs: Element[]): () => void {
	if (reduced() || !blobs.length) return () => {};
	const tl = gsap.timeline();
	blobs.forEach((b, i) => {
		const k = i % 3;
		tl.to(b, { x: DRIFT.x[k], y: DRIFT.y[k], scale: DRIFT.scale[k], duration: DRIFT.duration[k], ease: "breath", yoyo: true, repeat: -1 }, 0);
	});
	const stop = keep(tl);
	return () => {
		stop();
		gsap.to(blobs, { x: 0, y: 0, scale: 1, duration: 0.6, ease: "tactile" });
	};
}

/** Геройский момент «Запуска»: аура и главное пятно тумана делают вдох. */
export function inhale(): void {
	if (reduced()) return;
	const els = document.querySelectorAll("[data-inhale]");
	if (!els.length) return;
	gsap.killTweensOf(els);
	gsap.timeline().to(els, { scale: 1.3, duration: 0.38, ease: "tactile" }).to(els, { scale: 1, duration: 0.8, ease: "settle" });
}

/** Число докручивается до нового значения. */
export function count(el: HTMLElement, to: number, decimals = 0): void {
	if (reduced()) {
		el.textContent = to.toFixed(decimals);
		return;
	}
	const o = { v: Number(el.textContent) || 0 };
	gsap.to(o, { v: to, duration: 0.6, ease: "tactile", onUpdate: () => (el.textContent = o.v.toFixed(decimals)) });
}

/** Открытие экрана: оболочка садится, зоны [data-part] идут следом через 40 мс. */
export function openScreen(root: HTMLElement): void {
	const parts = root.querySelectorAll("[data-part]");
	gsap.killTweensOf([root, ...parts]);
	if (reduced()) {
		gsap.fromTo(root, { autoAlpha: 0 }, { autoAlpha: 1, duration: 0.15, clearProps: "opacity,visibility" });
		return;
	}
	const tl = gsap
		.timeline()
		.fromTo(root, { autoAlpha: 0, scale: 0.965, y: 10 }, { autoAlpha: 1, scale: 1, y: 0, duration: 0.7, ease: "settle", clearProps: "transform,opacity,visibility" })
		.fromTo(parts, { autoAlpha: 0, y: 6 }, { autoAlpha: 1, y: 0, duration: 0.38, ease: "tactile", stagger: 0.04, clearProps: "transform,opacity,visibility" }, 0.12);
	// без фокуса кадры тормозят — доводим до конца, чтобы экран не остался прозрачным
	setTimeout(() => tl.progress() < 1 && tl.progress(1), 2200);
}
```

- [ ] **Step 2: `src/components/Field.svelte`**

```svelte
<script lang="ts">
	import { drift } from "../lib/motion";

	/** spots: [x %, y %, размер px] — центр пятна относительно окна. Первое пятно — главное. */
	let { alive, spots }: { alive: boolean; spots: [number, number, number][] } = $props();

	let blobs = $state<HTMLSpanElement[]>([]);

	$effect(() => {
		if (!alive) return;
		return drift(blobs.filter(Boolean));
	});
</script>

<div class="field" aria-hidden="true">
	{#each spots as [x, y, s], i (i)}
		<span
			class="spot"
			data-inhale={i === 0 ? "" : undefined}
			style:left="{x}%"
			style:top="{y}%"
			style:width="{s}px"
			style:height="{s}px"
			style:margin="{-s / 2}px 0 0 {-s / 2}px"
		>
			<span class="blob b{i + 1}" bind:this={blobs[i]}></span>
		</span>
	{/each}
	<span class="grain"></span>
</div>

<style>
	.field {
		position: absolute;
		inset: 0;
		overflow: hidden;
		background: var(--field-base);
		pointer-events: none;
		z-index: 0;
	}
	.spot {
		position: absolute;
	}
	.blob {
		position: absolute;
		inset: 0;
		border-radius: 50%;
		will-change: transform;
	}
	.b1 {
		background: radial-gradient(closest-side, oklch(var(--blob-l1) calc(var(--aura-c) * 1.35) var(--aura-h) / var(--blob-a)), transparent);
	}
	.b2 {
		background: radial-gradient(closest-side, oklch(var(--blob-l2) calc(var(--aura-c) * 0.95) calc(var(--aura-h) + 48) / calc(var(--blob-a) * 0.8)), transparent);
	}
	.b3 {
		background: radial-gradient(closest-side, oklch(var(--blob-l3) calc(0.02 + var(--aura-c) * 0.45) calc(var(--aura-h) - 130) / calc(var(--blob-a) * 0.7)), transparent);
	}
	/* микрошум против полос в градиентах */
	.grain {
		position: absolute;
		inset: 0;
		opacity: 0.07;
		mix-blend-mode: soft-light;
		background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='160' height='160'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
	}
</style>
```

- [ ] **Step 3: `src/App.svelte`** — в `<script>` добавить импорты и производные:

```ts
	import Field from "./components/Field.svelte";
	import { accelView } from "./lib/accel";
	import { auraFor } from "./lib/aura";
	import { accentVars } from "./lib/palette";
```

```ts
	// Туман: цвет главного пятна = статус ускорения
	const aura = $derived.by(() => {
		const dto = app.dto;
		if (!dto) return auraFor(null, "mint");
		const state = dto.gpu ? accelView(dto.settings, dto.gpuCheckCurrent, app.launcherVerdict).state : null;
		return auraFor(state, dto.settings.accent);
	});
	const alive = $derived(aura.alive && app.dto?.settings.motion !== "reduced");
	const SPOTS: Record<"main" | "tuning" | "slot", [number, number, number][]> = {
		main: [[80, 30, 520], [98, 96, 440], [8, 98, 460]],
		tuning: [[88, 14, 480], [6, 62, 400], [62, 108, 440]],
		slot: [[72, 40, 460], [14, 18, 360], [92, 100, 400]]
	};

	// Акцент и режим движения — на корень документа
	$effect(() => {
		const s = app.dto?.settings;
		const root = document.documentElement;
		for (const [k, v] of Object.entries(accentVars(s?.accent ?? "mint"))) root.style.setProperty(k, v);
		root.dataset.motion = s?.motion ?? "full";
	});
```

Разметку `.frame` заменить на:

```svelte
<div class="frame" style:--aura-h={aura.h} style:--aura-c={aura.c}>
	{#if mode?.mode === "launcher" && app.dto}
		<Field {alive} spots={SPOTS[app.screen]} />
		<TitleBar {channel}><VersionTag version={app.dto.version} /></TitleBar>
	{:else}
		<TitleBar {channel} />
	{/if}
	<main class="screen">
		<!-- содержимое без изменений -->
	</main>
</div>
```

(блок `{#if mode?.mode === "install" …}` внутри `<main>` оставить как есть.) Стили заменить на:

```css
	.frame {
		position: relative;
		display: grid;
		grid-template-rows: 48px minmax(0, 1fr);
		height: 100vh;
		background: var(--field-base);
		isolation: isolate;
		transition:
			--aura-h 900ms var(--tc-ease),
			--aura-c 900ms var(--tc-ease);
	}
	:global(:root[data-motion="reduced"]) .frame {
		transition: none;
	}
	.screen {
		position: relative;
		z-index: 1;
		min-height: 0;
	}
	.boot {
		display: grid;
		place-items: center;
		height: 100%;
		color: var(--tc-muted);
	}
```

- [ ] **Step 4: Проверка**

Run: `npm run check && npm test`
Expected: 0 ошибок, тесты PASS.

Run: `npm run dev` (порт 1420; в браузере без Tauri включается `mock.ts`). Открыть `http://localhost:1420/` и `http://localhost:1420/?accel=software`.
Expected: под окном виден мятный туман из трёх пятен (с `?accel=software` — красный); в консоли нет ошибок. Старая раскладка экранов пока на месте — это нормально.

- [ ] **Step 5: Commit**

```bash
git add src/lib/motion.ts src/components/Field.svelte src/App.svelte
git commit -m "feat: Tactile motion and status fog behind the launcher" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Паз с каплей вместо `Segmented`

**Files:**
- Create: `src/components/Groove.svelte`
- Delete: `src/components/Segmented.svelte`
- Modify: `src/screens/TuningScreen.svelte` (импорт и все `<Segmented` → `<Groove`, убрать `vertical`)
- Modify: `src/screens/SlotEditor.svelte` (импорт и `<Segmented` → `<Groove`)

**Interfaces:**
- Consumes: `nextIndex` (Task 3), `flow`, `glide` (Task 5), `TipData` из `src/lib/tips.ts`, `tip` из `src/lib/tooltip.svelte.ts`.
- Produces: `Groove.svelte<T extends string | number>` props:
  `{ label: string; options: { value: T; label: string; title?: string; keyless?: boolean }[]; value: T; modified?: boolean; showLabel?: boolean; tip?: TipData; onchange: (v: T) => void }`.
  `keyless: true` — пункт не выбирается стрелками.

- [ ] **Step 1: `src/components/Groove.svelte`**

```svelte
<script lang="ts" generics="T extends string | number">
	import { flow, glide } from "../lib/motion";
	import { nextIndex } from "../lib/radio";
	import type { TipData } from "../lib/tips";
	import { tip as tooltip } from "../lib/tooltip.svelte";

	type Option = { value: T; label: string; title?: string; keyless?: boolean };
	let {
		label,
		options,
		value,
		modified = false,
		showLabel = true,
		tip,
		onchange
	}: {
		label: string;
		options: Option[];
		value: T;
		modified?: boolean;
		showLabel?: boolean;
		tip?: TipData;
		onchange: (v: T) => void;
	} = $props();

	let track = $state<HTMLDivElement>();
	let drop = $state<HTMLSpanElement>();
	let placed = false;
	const index = $derived(options.findIndex((o) => o.value === value));
	const buttons = () => [...(track?.querySelectorAll<HTMLButtonElement>(':scope > [role="radio"]') ?? [])];

	function place(instant: boolean) {
		if (!drop) return;
		const btn = buttons()[index];
		drop.style.visibility = btn ? "" : "hidden";
		if (btn) flow(drop, btn, "x", instant);
	}

	$effect(() => {
		void index;
		void options.length;
		place(!placed);
		placed = true;
	});

	// шрифты догружаются и окно меняет размер — капля встаёт на место без анимации
	$effect(() => {
		if (!track) return;
		const ro = new ResizeObserver(() => place(true));
		ro.observe(track);
		return () => ro.disconnect();
	});

	function onkeydown(e: KeyboardEvent) {
		const i = nextIndex(index, e.key, options.map((o) => !o.keyless));
		if (i === null) return;
		e.preventDefault();
		onchange(options[i].value);
		buttons()[i]?.focus();
	}
</script>

<div class="groove-field" {@attach tooltip(() => tip)}>
	{#if showLabel}<span class="label">{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>{/if}
	<div class="groove" role="radiogroup" aria-label={label} tabindex="-1" bind:this={track} {onkeydown} {@attach glide('[role="radio"]')}>
		<span class="drop" bind:this={drop} aria-hidden="true"></span>
		{#each options as o, i (o.value)}
			<button
				type="button"
				role="radio"
				aria-checked={i === index}
				tabindex={i === index || (index < 0 && i === 0) ? 0 : -1}
				title={o.title}
				onclick={() => onchange(o.value)}>{o.label}</button
			>
		{/each}
	</div>
</div>

<style>
	.groove-field {
		display: grid;
		gap: 7px;
		min-width: 0;
	}
	.groove {
		--glide-r: var(--tc-r-ctl);
		position: relative;
		display: grid;
		grid-auto-flow: column;
		grid-auto-columns: minmax(max-content, 1fr);
		padding: 4px;
		border-radius: var(--tc-r-ctl);
		background: var(--g-well);
		box-shadow: var(--g-well-shadow);
		isolation: isolate;
	}
	.groove:focus {
		outline: none;
	}
	button {
		position: relative;
		z-index: 2;
		height: 30px;
		padding: 0 12px;
		border-radius: var(--tc-r-ctl);
		font: 600 11px/1 var(--tc-font-mono);
		letter-spacing: 0.06em;
		color: var(--tc-muted);
		white-space: nowrap;
		transition: color 240ms var(--tc-ease);
	}
	button[aria-checked="true"] {
		color: var(--tc-ink);
	}
	button:hover {
		color: var(--tc-ink);
	}
</style>
```

- [ ] **Step 2: Заменить использования**
  - `src/screens/TuningScreen.svelte`: `import Segmented from "../components/Segmented.svelte";` → `import Groove from "../components/Groove.svelte";`; все `<Segmented` → `<Groove`; у «Режима производительности» (perfMode) удалить строку `vertical`.
  - `src/screens/SlotEditor.svelte`: тот же импорт и `<Segmented` → `<Groove`.
  - Удалить файл: `git rm src/components/Segmented.svelte`.

- [ ] **Step 3: Проверка**

Run: `npm run check && npm test`
Expected: 0 ошибок (нет ссылок на `Segmented`), тесты PASS.

Run: `npm run dev`, открыть «Настройка» (старая кнопка «НАСТРОЙКА» справа на главном экране).
Expected: профиль, видео, Prime, ANGLE, кэш, язык, тема — пазы; клик перегоняет стеклянную каплю с растяжкой; стрелки ← → меняют выбор; наведение — одно пятно, едущее между пунктами.

- [ ] **Step 4: Commit**

```bash
git add src/components/Groove.svelte src/screens/TuningScreen.svelte src/screens/SlotEditor.svelte
git commit -m "feat: Groove radio with flowing glass drop replaces Segmented" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Заголовок окна и версия

**Files:**
- Rewrite: `src/components/TitleBar.svelte`
- Rewrite: `src/components/VersionTag.svelte`
- Modify: `src/App.svelte` (проп `ontune`)

**Interfaces:**
- Consumes: `glide` (Task 5); `app.update`, `app.updating`, `app.updateCheck`, `app.applyUpdate()`, `app.checkUpdate(true)`, `app.openTuning(scope)`, `app.selected` из `store.svelte.ts`; `noteLines` из `lib/changelog.ts`.
- Produces: `TitleBar` props `{ channel: string; ontune?: () => void; children?: Snippet }`.

- [ ] **Step 1: `src/components/TitleBar.svelte`**

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import { windowControls } from "../lib/api";
	import { t } from "../lib/i18n.svelte";
	import { glide } from "../lib/motion";

	/** `children` — блок версии в лаунчере; `ontune` — кнопка «Настройка» (только лаунчер). */
	let { channel, ontune, children }: { channel: string; ontune?: () => void; children?: Snippet } = $props();
</script>

<header class="bar" data-tauri-drag-region>
	<span class="mark" aria-hidden="true">
		<svg viewBox="0 0 16 16"><path d="M1.5 8.5h3l1.5-4 3 8 1.8-5 1.2 1h2.5" /></svg>
	</span>
	<b class="brand" data-tauri-drag-region>Foundry Performance</b>
	{#if children}{@render children()}{:else if channel}<span class="chip" data-tauri-drag-region>{channel}</span>{/if}
	<div class="ctl" {@attach glide("button")}>
		{#if ontune}
			<button aria-label={t("main.tune")} title={t("main.tune")} onclick={ontune}>
				<svg viewBox="0 0 16 16"><path d="M3 4.5h6M12 4.5h1M3 11.5h1M7 11.5h6" /><circle cx="10.5" cy="4.5" r="1.5" /><circle cx="5.5" cy="11.5" r="1.5" /></svg>
			</button>
		{/if}
		<button aria-label={t("window.minimize")} onclick={() => windowControls.minimize()}>
			<svg viewBox="0 0 16 16"><path d="M3.5 8h9" /></svg>
		</button>
		<button aria-label={t("window.close")} onclick={() => windowControls.close()}>
			<svg viewBox="0 0 16 16"><path d="M4 4l8 8M12 4l-8 8" /></svg>
		</button>
	</div>
</header>

<style>
	.bar {
		position: relative;
		z-index: 2;
		display: flex;
		align-items: center;
		gap: 10px;
		height: 48px;
		padding: 0 10px 0 14px;
	}
	.mark {
		position: relative;
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		border-radius: 50%;
		color: var(--tc-accent-text);
		background: var(--g-drop);
		box-shadow: var(--g-drop-shadow);
	}
	svg {
		width: 15px;
		height: 15px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.brand {
		font: 600 14px/1 var(--tc-font-ui);
	}
	.ctl {
		--glide-r: 50%;
		position: relative;
		display: flex;
		gap: 2px;
		margin-left: auto;
	}
	.ctl button {
		position: relative;
		z-index: 1;
		display: grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border-radius: 50%;
		color: var(--tc-muted);
	}
	.ctl button:hover {
		color: var(--tc-ink);
	}
</style>
```

- [ ] **Step 2: `src/components/VersionTag.svelte`** — `<script>`: к существующим импортам добавить `import { noteLines } from "../lib/changelog";` (оставить `notesFor`). Разметку и стили заменить:

```svelte
<div class="version">
	{#if app.update && app.updating === null}
		<button
			class="chip update"
			{@attach tip(() =>
				app.update ? { title: t("update.notesTitle", { version: app.update.version }), body: t("update.noNotes"), lines: noteLines(app.update.notes) } : undefined
			)}
			onclick={() => app.applyUpdate()}>{t("update.available", { version: app.update.version })}</button
		>
	{:else}
		<!-- tabindex: список изменений доступен и с клавиатуры -->
		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<span class="chip" tabindex="0" {@attach tip(() => notesTip)}>v{version}</span>
	{/if}
	<button
		class="check"
		class:busy={check === "checking"}
		aria-label={t("update.check")}
		aria-busy={check === "checking"}
		{@attach tip(() => ({ title: t("update.check"), body: t("update.checkBody") }))}
		onclick={() => app.checkUpdate(true)}
	>
		<svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true">
			<path d="M10.2 6.4A4.2 4.2 0 1 1 8.9 3" />
			<path d="M9.8 0.9v2.8H7" />
		</svg>
	</button>
	{#if check === "current" || check === "failed"}<span class="status mono {check}" role="status">{status}</span>{/if}
</div>

<style>
	.version {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.chip {
		cursor: help;
	}
	.update {
		position: relative;
		cursor: pointer;
		color: var(--tc-on-accent);
		background: linear-gradient(180deg, var(--tc-accent-hi), var(--tc-accent));
		box-shadow: inset 0 1px 0 rgb(255 255 255 / 70%), 0 6px 14px -8px var(--tc-glow);
	}
	.check {
		display: grid;
		place-items: center;
		width: 26px;
		height: 26px;
		border-radius: 50%;
		color: var(--tc-muted);
	}
	.check:hover {
		color: var(--tc-ink);
	}
	.check svg {
		fill: none;
		stroke: currentColor;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.busy svg {
		animation: spin 0.8s linear infinite;
	}
	.status {
		color: var(--tc-muted);
	}
	.status.current {
		color: var(--tc-good);
	}
	.status.failed {
		color: var(--tc-danger);
	}
	@keyframes spin {
		to {
			rotate: 360deg;
		}
	}
</style>
```

- [ ] **Step 3: `src/App.svelte`** — в строке лаунчера передать `ontune`:

```svelte
		<TitleBar {channel} ontune={() => app.openTuning(app.selected ? { kind: "server", id: app.selected.id } : { kind: "global" })}>
			<VersionTag version={app.dto.version} />
		</TitleBar>
```

- [ ] **Step 4: Проверка**

Run: `npm run check && npm test`
Expected: 0 ошибок, PASS.

Run: `npm run dev`. Expected: заголовок «Foundry Performance» с каплей-значком, чип `v0.2.2`, кнопка проверки обновлений; справа ползунки (открывают «Настройку»), свернуть, закрыть; подсветка наведения переезжает между тремя кнопками. Окно таскается за заголовок (в `npm run tauri dev`).

- [ ] **Step 5: Commit**

```bash
git add src/components/TitleBar.svelte src/components/VersionTag.svelte src/App.svelte
git commit -m "feat: glass title bar with settings button and update chip" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Линза FPS и линейка

**Files:**
- Create: `src/components/Ticks.svelte`
- Create: `src/components/Lens.svelte`

**Interfaces:**
- Consumes: `LensView`, `tickBars` (Task 3); `breathe`, `count` (Task 5); `t`.
- Produces: `Lens.svelte` props `{ view: LensView; alive: boolean }` — корень `.lens` с `data-part`, аура `[data-inhale]`; `Ticks.svelte` props `{ values: number[]; width?: number; height?: number }`.

- [ ] **Step 1: `src/components/Ticks.svelte`**

```svelte
<script lang="ts">
	import { tickBars } from "../lib/lens";

	let { values, width = 150, height = 30 }: { values: number[]; width?: number; height?: number } = $props();
	const bars = $derived(tickBars(values, width, height));
</script>

<svg class="ticks" viewBox="0 0 {width} {height}" preserveAspectRatio="none" aria-hidden="true">
	{#each bars as b, i (i)}
		<rect class:last={b.last} x={b.x} y={b.y} width={b.w} height={b.h} rx={Math.min(b.w / 2, 1.5)} />
	{/each}
</svg>

<style>
	.ticks {
		display: block;
		flex: 1;
		min-width: 0;
		height: 30px;
		overflow: visible;
	}
	rect {
		fill: color-mix(in srgb, var(--tc-ink) 22%, transparent);
	}
	rect.last {
		fill: var(--tc-accent-text);
	}
</style>
```

- [ ] **Step 2: `src/components/Lens.svelte`**

```svelte
<script lang="ts">
	import { t } from "../lib/i18n.svelte";
	import type { LensView } from "../lib/lens";
	import { breathe, count } from "../lib/motion";
	import Ticks from "./Ticks.svelte";

	/** FPS — справка, а не герой: узкая полоса, число 40 px, линейка сбоку. */
	let { view, alive }: { view: LensView; alive: boolean } = $props();

	let num = $state<HTMLSpanElement>();
	let aura = $state<HTMLSpanElement>();
	let shown = false;

	$effect(() => {
		const v = view.value;
		if (!num) return;
		if (v === null) num.textContent = "—";
		else if (!shown) num.textContent = String(Math.round(v));
		else count(num, v);
		shown = v !== null;
	});

	$effect(() => {
		if (!alive || !aura) return;
		return breathe(aura);
	});
</script>

<div class="lens" data-part role="img" aria-label={view.value === null ? `— ${view.unit}` : `${Math.round(view.value)} ${view.unit}`}>
	<span class="aura-wrap" data-inhale aria-hidden="true"><span class="aura" bind:this={aura}></span></span>
	<div class="head">
		<span class="label">{t(view.captionKey)}</span>
		{#if view.low !== null}<span class="low mono">1% low {Math.round(view.low)}</span>{/if}
	</div>
	<div class="body">
		<span class="num" bind:this={num}></span>
		<span class="unit">{view.unit}</span>
		{#if view.history.length}<Ticks values={view.history} />{/if}
	</div>
</div>

<style>
	.lens {
		display: grid;
		gap: 6px;
		padding: 11px 14px 12px;
	}
	.aura-wrap {
		position: absolute;
		z-index: -1;
		left: -20px;
		top: 12px;
		width: 180px;
		height: 116px;
	}
	.aura {
		position: absolute;
		inset: 0;
		border-radius: 50%;
		background: radial-gradient(
			closest-side,
			oklch(0.78 calc(var(--aura-c) * 1.6) var(--aura-h) / 0.85),
			oklch(0.78 calc(var(--aura-c) * 1.2) var(--aura-h) / 0.25) 60%,
			transparent
		);
	}
	.head {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: 8px;
	}
	.low {
		color: var(--tc-muted);
	}
	.body {
		display: flex;
		align-items: flex-end;
		gap: 7px;
	}
	.num {
		font-size: 40px;
		line-height: 0.9;
	}
	.unit {
		padding-bottom: 3px;
		margin-right: 10px;
		font: 600 11px/1 var(--tc-font-mono);
		letter-spacing: 0.1em;
		color: var(--tc-muted);
	}
</style>
```

(Класс `.num` и общий вид `.lens` — из `materials.css`; здесь только раскладка.)

- [ ] **Step 3: Проверка**

Run: `npm run check`
Expected: 0 ошибок. (Компонент подключается в Task 10.)

- [ ] **Step 4: Commit**

```bash
git add src/components/Ticks.svelte src/components/Lens.svelte
git commit -m "feat: quiet FPS lens with session ticks" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Список серверов с каплей

**Files:**
- Create: `src/components/DropList.svelte`
- Rewrite: `src/components/Slot.svelte`

**Interfaces:**
- Consumes: `flow`, `glide` (Task 5); `slotStatus`, `LED_FOR` из `lib/status.ts`; `Led.svelte` (`{ state: LedState; label: string }`).
- Produces:
  - `DropList.svelte` props `{ selected: number; label: string; children: Snippet }` — капля встаёт на `selected`-й прямой дочерний `[data-row]`.
  - `Slot.svelte` props `{ server: Server; selected: boolean; probe: ProbeResult | "pending" | undefined; fps: number | null; onselect: () => void; onlaunch: () => void; onedit: () => void }` (проп `code` удалён); корень — `[data-row]`.

- [ ] **Step 1: `src/components/DropList.svelte`**

```svelte
<script lang="ts">
	import type { Snippet } from "svelte";
	import { flow, glide } from "../lib/motion";

	let { selected, label, children }: { selected: number; label: string; children: Snippet } = $props();

	let list = $state<HTMLDivElement>();
	let drop = $state<HTMLSpanElement>();
	let placed = false;

	function place(instant: boolean) {
		if (!drop || !list) return;
		const row = list.querySelectorAll<HTMLElement>(":scope > [data-row]")[selected];
		drop.style.visibility = row && selected >= 0 ? "" : "hidden";
		if (row) flow(drop, row, "y", instant);
	}

	$effect(() => {
		void selected;
		place(!placed);
		placed = true;
	});

	$effect(() => {
		if (!list) return;
		const ro = new ResizeObserver(() => place(true));
		ro.observe(list);
		return () => ro.disconnect();
	});
</script>

<div class="drop-list" role="listbox" aria-label={label} bind:this={list} {@attach glide("[data-row]")}>
	<span class="drop" bind:this={drop} aria-hidden="true"></span>
	{@render children()}
</div>

<style>
	.drop-list {
		position: relative;
		display: grid;
		align-content: start;
		gap: 2px;
		isolation: isolate;
	}
	.drop-list > :global(.drop) {
		border-radius: var(--tc-r-row);
	}
</style>
```

- [ ] **Step 2: `src/components/Slot.svelte`** — `<script>`: проп `code` заменить на `fps: number | null` (в деструктуризации и в типе); остальной скрипт без изменений. Разметку и стили заменить:

```svelte
<div
	class="slot"
	class:selected
	data-row
	role="option"
	aria-selected={selected}
	tabindex="0"
	onclick={onselect}
	ondblclick={onlaunch}
	onkeydown={(e) => {
		if (e.key === "Enter") onlaunch();
		else if (e.key === " ") {
			e.preventDefault();
			onselect();
		}
	}}
>
	<Led state={LED_FOR[status]} label={statusText} />
	<span class="body">
		<span class="name">{server.name}</span>
		<span class="meta mono">{host}{#if world}&nbsp;· {world}{:else}&nbsp;· {statusText}{/if}</span>
	</span>
	<button
		class="edit chip"
		aria-label={t("slot.editAria", { name: server.name })}
		onclick={(e) => {
			e.stopPropagation();
			onedit();
		}}>{t("slot.editShort")}</button
	>
	<span class="fps num">{fps === null ? "—" : Math.round(fps)}</span>
</div>

<style>
	.slot {
		position: relative;
		z-index: 1;
		display: grid;
		grid-template-columns: 12px minmax(0, 1fr) auto 34px;
		align-items: center;
		gap: 11px;
		height: 54px;
		padding: 0 14px;
		border-radius: var(--tc-r-row);
		cursor: pointer;
	}
	.body {
		display: grid;
		min-width: 0;
	}
	.name {
		font: 600 14px/1.25 var(--tc-font-ui);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.meta {
		color: var(--tc-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.fps {
		font-size: 19px;
		text-align: right;
		color: var(--tc-muted);
	}
	.selected .fps {
		color: var(--tc-accent-text);
	}
	.edit {
		cursor: pointer;
		opacity: 0;
		transition: opacity 0.15s var(--tc-ease);
	}
	.edit:hover {
		color: var(--tc-ink);
	}
	.slot:hover .edit,
	.slot:focus-within .edit,
	.selected .edit {
		opacity: 1;
	}
</style>
```

- [ ] **Step 3: Старый главный экран — на новый проп.** В текущем `src/screens/MainScreen.svelte` у `<Slot` заменить `code={code(i)}` на

```svelte
					fps={dto.stats[s.id]?.lastBench?.avg ?? dto.stats[s.id]?.lastSession?.avg ?? null}
```

(`const code` оставить — им ещё подписан пустой слот; экран целиком переписывает Task 10.)

- [ ] **Step 4: Проверка**

Run: `npm run check && npm test`
Expected: 0 ошибок, PASS. В `npm run dev` строки серверов — с лампочкой слева, FPS справа и кнопкой «ИЗМ»; капли пока нет (DropList подключает Task 10).

- [ ] **Step 5: Commit**

```bash
git add src/components/DropList.svelte src/components/Slot.svelte src/screens/MainScreen.svelte
git commit -m "feat: drop list and glass server rows" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 10: Главный экран «Монитор» и «Запуск»

**Files:**
- Rewrite: `src/screens/MainScreen.svelte`
- Rewrite: `src/components/LaunchButton.svelte`
- Delete: `src/components/Knob.svelte`
- Modify: `src/lib/i18n/ru.json`, `src/lib/i18n/en.json` (удалить ставшие ненужными ключи)

**Interfaces:**
- Consumes: `DropList`, `Slot` (Task 9); `Lens` (Task 8); `Groove` (Task 6); `lensView` (Task 3); `PROFILE_POSITIONS`, `profileNameKey`, `profileTipKey` (Task 3); `auraFor` (Task 2); `inhale`, `openScreen` (Task 5); `knobPosition` из `lib/levers.ts`; `accelView`; `app.setKnob(p: KnobPosition)`, `app.launch(safe)`, `app.newSlot()`, `app.editSlot(s)`, `app.dismiss()`, `app.message`.
- Produces: `LaunchButton` props без изменений: `{ busy: boolean; onlaunch: (safe: boolean) => void }`.

- [ ] **Step 1: `src/components/LaunchButton.svelte`**

```svelte
<script lang="ts">
	import { t } from "../lib/i18n.svelte";
	import { inhale } from "../lib/motion";

	let { busy, onlaunch }: { busy: boolean; onlaunch: (safe: boolean) => void } = $props();
	let shift = $state(false);
	const track = (e: KeyboardEvent) => (shift = e.shiftKey);

	function press(e: MouseEvent) {
		if (busy) return;
		inhale();
		onlaunch(e.shiftKey);
	}
</script>

<svelte:window onkeydown={track} onkeyup={track} onblur={() => (shift = false)} />

<div class="group" data-part>
	<button class="launch" class:safe={shift} class:busy aria-busy={busy} onclick={press}>
		<span>{busy ? t("main.launching") : shift ? t("main.launchSafe") : t("main.launch")}</span>
		<span class="knob" aria-hidden="true">
			{#if busy}<span class="spin"></span>{:else}<svg viewBox="0 0 16 16"><path d="M5 3.5v9l7.5-4.5z" /></svg>{/if}
		</span>
	</button>
	<span class="hint">{t("main.safeHint")}</span>
</div>

<style>
	.group {
		display: grid;
		gap: 7px;
	}
	/* тонированное стекло: акцент внутри, кант и каустика снизу */
	.launch {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		height: 52px;
		padding: 0 8px 0 22px;
		border-radius: var(--tc-r-ctl);
		isolation: isolate;
		background: linear-gradient(180deg, oklch(0.86 calc(var(--tc-acc-c) * 0.8) var(--tc-acc-h) / 0.95), oklch(0.72 var(--tc-acc-c) var(--tc-acc-h) / 0.88));
		color: var(--tc-on-accent);
		font: 700 15px/1 var(--tc-font-ui);
		letter-spacing: 0.04em;
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 75%),
			inset 0 -10px 16px -10px oklch(0.55 calc(var(--tc-acc-c) * 1.3) var(--tc-acc-h) / 0.6),
			0 18px 30px -16px var(--tc-glow),
			0 3px 6px -2px rgb(0 0 0 / 20%);
		transition: transform 380ms cubic-bezier(0.3, 1.35, 0.5, 1);
	}
	.launch::before {
		content: "";
		position: absolute;
		inset: 0;
		border-radius: inherit;
		padding: 1.5px;
		background: linear-gradient(180deg, rgb(255 255 255 / 90%), rgb(255 255 255 / 20%) 50%, rgb(255 255 255 / 55%));
		pointer-events: none;
		-webkit-mask:
			linear-gradient(#000 0 0) content-box,
			linear-gradient(#000 0 0);
		-webkit-mask-composite: xor;
		mask:
			linear-gradient(#000 0 0) content-box exclude,
			linear-gradient(#000 0 0);
	}
	.launch::after {
		content: "";
		position: absolute;
		inset: 2px 32% 52% 14px;
		border-radius: inherit;
		background: radial-gradient(120% 100% at 20% 0%, rgb(255 255 255 / 80%), rgb(255 255 255 / 0) 60%);
		pointer-events: none;
	}
	.launch:active {
		transform: translateY(1px) scale(0.98);
		transition-duration: 90ms;
	}
	/* безопасный запуск — прозрачная капля без акцента */
	.launch.safe {
		color: var(--tc-ink);
		background: var(--g-drop);
		box-shadow: var(--g-drop-shadow);
	}
	.launch.busy {
		cursor: progress;
	}
	.knob {
		display: grid;
		place-items: center;
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: rgb(255 255 255 / 34%);
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 70%),
			inset 0 -3px 6px -3px rgb(0 0 0 / 18%),
			0 2px 5px -2px rgb(0 0 0 / 25%);
	}
	.knob svg {
		width: 14px;
		height: 14px;
		fill: currentColor;
	}
	.spin {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 2px solid currentColor;
		border-right-color: transparent;
		animation: spin 0.8s linear infinite;
	}
	.hint {
		font-size: 11.5px;
		color: var(--tc-muted);
	}
	@keyframes spin {
		to {
			rotate: 360deg;
		}
	}
</style>
```

- [ ] **Step 2: `src/screens/MainScreen.svelte`** — полностью заменить:

```svelte
<script lang="ts">
	import DropList from "../components/DropList.svelte";
	import Groove from "../components/Groove.svelte";
	import LaunchButton from "../components/LaunchButton.svelte";
	import Led from "../components/Led.svelte";
	import Lens from "../components/Lens.svelte";
	import Slot from "../components/Slot.svelte";
	import { accelView } from "../lib/accel";
	import { auraFor } from "../lib/aura";
	import { t } from "../lib/i18n.svelte";
	import { lensView } from "../lib/lens";
	import { knobPosition } from "../lib/levers";
	import { openScreen } from "../lib/motion";
	import { PROFILE_POSITIONS, profileNameKey, profileTipKey } from "../lib/profile-tip";
	import { app } from "../lib/store.svelte";
	import { tip, tipFor } from "../lib/tooltip.svelte";

	const dto = $derived(app.dto!);
	const sel = $derived(app.selected);
	const selectedIndex = $derived(dto.servers.findIndex((s) => s.id === app.selectedId));
	const knob = $derived(knobPosition(dto, sel?.id ?? null));
	const view = $derived(lensView(sel ? dto.stats[sel.id] : undefined, app.updating));
	// «NVIDIA GeForce RTX 5070» → «RTX 5070»: марка на табличке не нужна
	const gpuName = $derived(dto.gpu?.name.replace(/^(NVIDIA|AMD|Intel\(R\))\s+(GeForce\s+|Radeon\s+(?=RX))?/i, "") ?? "");
	const accel = $derived(accelView(dto.settings, dto.gpuCheckCurrent, app.launcherVerdict));
	const alive = $derived(auraFor(dto.gpu ? accel.state : null, dto.settings.accent).alive && dto.settings.motion !== "reduced");
	const profileOptions = $derived(
		PROFILE_POSITIONS.map((p) => ({ value: p, label: t(`profile.${p}`), title: t(profileNameKey(p)), keyless: p === "manual" }))
	);
	const fpsOf = (id: string) => {
		const st = dto.stats[id];
		return st?.lastBench?.avg ?? st?.lastSession?.avg ?? null;
	};

	let root = $state<HTMLDivElement>();
	$effect(() => {
		if (root) openScreen(root);
	});
</script>

<div class="main" bind:this={root}>
	<section class="pane servers" data-part>
		<span class="label head">{t("main.slots")}</span>
		<div class="scroll">
			<DropList selected={selectedIndex} label={t("main.slots")}>
				{#each dto.servers as s (s.id)}
					<Slot
						server={s}
						selected={s.id === app.selectedId}
						probe={app.probes[s.id]}
						fps={fpsOf(s.id)}
						onselect={() => (app.selectedId = s.id)}
						onlaunch={() => {
							app.selectedId = s.id;
							void app.launch(false);
						}}
						onedit={() => app.editSlot(s)}
					/>
				{/each}
				<button class="add" data-row onclick={() => app.newSlot()}>
					<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 3v10M3 8h10" /></svg>
					{dto.servers.length ? t("main.addServer") : t("main.noServer")}
				</button>
			</DropList>
		</div>
		{#if app.message}
			<div class="notice" role="status">
				<span>{app.message}</span>
				<button aria-label={t("window.close")} onclick={() => app.dismiss()}>
					<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg>
				</button>
			</div>
		{/if}
	</section>

	<aside class="right">
		<Lens {view} {alive} />

		<div class="pane profile" data-part {@attach tip(() => tipFor("profile", t("knob.label")))}>
			<div class="prof-head">
				<span class="label">{t("main.profile")}</span>
				<b>{t(profileNameKey(knob))}</b>
			</div>
			<Groove label={t("knob.label")} showLabel={false} options={profileOptions} value={knob} onchange={(p) => app.setKnob(p)} />
			<p class="prof-tip">{t(profileTipKey(knob))}</p>
		</div>

		{#if dto.gpu}
			<dl class="pane passport mono" data-part>
				<dt>GPU</dt>
				<dd title="{dto.gpu.name} · {Math.round(dto.gpu.vramMb / 1024)} {t('unit.gb')}">{gpuName}</dd>
				<dt>API</dt>
				<dd>{accel.api.toUpperCase()}</dd>
				<dt>{t("accel.label")}</dt>
				<dd
					class="accel"
					{@attach tip(() => ({
						title: t(`accel.${accel.state}`),
						body: t(accel.source === "launcher" ? `accel.tip.launcher.${accel.state}` : `accel.tip.${accel.state}`)
					}))}
				>
					<Led state={accel.led} label="" />{t(`accel.${accel.state}`)}
				</dd>
			</dl>
		{:else}
			<div class="pane passport mono" data-part>{t("gpu.unknown")}</div>
		{/if}

		<LaunchButton busy={app.launching} onlaunch={(safe) => (sel ? app.launch(safe) : app.newSlot())} />
	</aside>
</div>

<style>
	.main {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 320px;
		gap: 14px;
		height: 100%;
		min-height: 0;
		padding: 2px 14px 14px;
	}
	.servers {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto;
		min-height: 0;
		padding: 6px;
	}
	.head {
		padding: 9px 12px 6px;
	}
	.scroll {
		min-height: 0;
		overflow-y: auto;
	}
	.add {
		position: relative;
		z-index: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		height: 44px;
		border-radius: var(--tc-r-row);
		color: var(--tc-muted);
		font-size: 13px;
	}
	.add:hover {
		color: var(--tc-ink);
	}
	svg {
		width: 14px;
		height: 14px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.6;
		stroke-linecap: round;
	}
	.notice {
		display: flex;
		align-items: center;
		gap: 10px;
		margin: 6px;
		padding: 10px 12px;
		border-radius: var(--tc-r-row);
		background: var(--g-well);
		box-shadow: var(--g-well-shadow);
		font-size: 12.5px;
	}
	.notice span {
		flex: 1;
	}
	.notice button {
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		color: var(--tc-muted);
	}
	.right {
		display: flex;
		flex-direction: column;
		gap: 11px;
		min-height: 0;
	}
	.profile {
		display: grid;
		gap: 9px;
		padding: 12px 12px 11px;
	}
	.prof-head {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		padding: 0 4px;
	}
	.prof-head b {
		font: 600 13px/1 var(--tc-font-ui);
	}
	.prof-tip {
		min-height: 2.9em;
		margin: 0;
		padding: 0 4px;
		font-size: 12px;
		line-height: 1.45;
		color: var(--tc-muted);
	}
	/* паспорт прижат к «Запуску»: сверху выбор, снизу готовность машины */
	.passport {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 5px 12px;
		margin: auto 0 0;
		padding: 12px 15px;
		font-size: 11.5px;
	}
	dt {
		color: var(--tc-muted);
		letter-spacing: 0.06em;
	}
	dd {
		margin: 0;
		text-align: right;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.accel {
		display: flex;
		justify-content: flex-end;
		align-items: center;
		gap: 8px;
	}
</style>
```

- [ ] **Step 3: Удалить ручку и лишние строки**

```bash
git rm src/components/Knob.svelte
```

Затем для каждого ключа `main.channels`, `main.emptySlot` и всех `knob.*`, кроме `knob.label`, выполнить `grep -rn "<ключ>" src agent/src` и удалить ключ из обоих словарей, только если вне `i18n/*.json` совпадений нет.

- [ ] **Step 4: Проверка**

Run: `npm run check && npm test`
Expected: 0 ошибок, все тесты PASS (parity зелёный).

Run: `npm run dev`, открыть `http://localhost:1420/`, затем `?accel=software`, затем переключить тему ОС.
Expected:
- слева лист «СЕРВЕРЫ»: три строки, у выбранной стеклянная капля, клик перегоняет её с растяжкой, наведение — скользящее пятно; двойной клик запускает (в моке — «ОТКРЫВАЕМ…»); «ИЗМ» открывает редактор;
- справа узкая линза (подпись, «1% low», число 40 px, бары), лист «ПРОФИЛЬ» с каплей и подсказкой, паспорт прижат к «Запуску»;
- «Запуск» — мятное стекло; с зажатым Shift — прозрачная капля «БЕЗОПАСНО»; клик даёт вдох ауры и тумана;
- с `?accel=software` туман и аура красные и неподвижные.

- [ ] **Step 5: Окно без фокуса**

В `npm run tauri dev` открыть лаунчер, кликнуть в другое окно.
Expected: туман и аура замирают; при возврате фокуса продолжают.

- [ ] **Step 6: Commit**

```bash
git add src/components/LaunchButton.svelte src/screens/MainScreen.svelte src/lib/i18n/ru.json src/lib/i18n/en.json
git commit -m "feat: Monitor main screen with drop list, quiet lens and profile pane" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 11: Переключатель, фейдер, кэш и подсказки

**Files:**
- Rewrite: `src/components/Toggle.svelte`
- Modify: `src/components/Fader.svelte` (только `<style>`)
- Modify: `src/components/CacheControl.svelte` (только `<style>`)
- Modify: `src/components/Tooltip.svelte` (только `<style>`)

**Interfaces:**
- Props всех четырёх компонентов не меняются.

- [ ] **Step 1: `src/components/Toggle.svelte`** — разметку и стили заменить (скрипт без изменений):

```svelte
<button class="toggle" {@attach tooltip(() => tip)} role="switch" aria-checked={checked} onclick={() => onchange(!checked)}>
	<span>{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
	<span class="sw" class:on={checked} aria-hidden="true"><i></i></span>
</button>

<style>
	.toggle {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		width: 100%;
		padding: 6px 0;
		text-align: left;
		font-size: 13px;
	}
	/* капсула-паз; включённый заливается акцентом, бегунок — стеклянная капля */
	.sw {
		position: relative;
		flex: none;
		width: 40px;
		height: 24px;
		border-radius: var(--tc-r-ctl);
		background: var(--g-well);
		box-shadow: var(--g-well-shadow);
		transition: background-color 240ms var(--tc-ease);
	}
	.sw.on {
		background: oklch(0.74 var(--tc-acc-c) var(--tc-acc-h) / 0.55);
	}
	.sw i {
		position: absolute;
		top: 3px;
		left: 3px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: var(--g-drop);
		box-shadow: var(--g-drop-shadow);
		transition: transform 380ms cubic-bezier(0.3, 1.35, 0.5, 1);
	}
	.sw.on i {
		transform: translateX(16px);
	}
	:global(:root[data-motion="reduced"]) .sw i {
		transition: none;
	}
</style>
```

- [ ] **Step 2: `src/components/Fader.svelte`** — заменить `<style>` целиком:

```svelte
<style>
	.fader {
		display: grid;
		justify-items: center;
		align-content: start;
		gap: 10px;
		padding: 14px 8px 12px;
	}
	.lbl,
	.val {
		white-space: nowrap;
	}
	.row {
		display: flex;
		gap: 6px;
	}
	.scale {
		position: relative;
		width: 26px;
	}
	.scale span {
		position: absolute;
		right: 0;
		translate: 0 -50%;
		font: 400 10px var(--tc-font-mono);
		color: var(--tc-faint);
	}
	/* паз-дорожка и бегунок-капля */
	input[type="range"] {
		appearance: none;
		writing-mode: vertical-lr;
		direction: rtl;
		width: 30px;
		margin: 0;
		background: transparent;
		cursor: ns-resize;
	}
	input[type="range"]::-webkit-slider-runnable-track {
		width: 8px;
		margin: 0 auto;
		border-radius: var(--tc-r-ctl);
		background: var(--g-well);
		box-shadow: var(--g-well-shadow);
	}
	input[type="range"]::-webkit-slider-thumb {
		appearance: none;
		width: 26px;
		height: 18px;
		margin-left: -9px;
		border-radius: var(--tc-r-ctl);
		background: var(--g-drop);
		box-shadow: var(--g-drop-shadow);
	}
	.val {
		font-size: 12px;
	}
	.readout {
		width: 62px;
		padding: 5px 9px;
		text-align: right;
		border: 0;
		border-radius: var(--tc-r-row);
		color: var(--tc-ink);
		background: var(--g-well);
		box-shadow: var(--g-well-shadow);
		appearance: textfield;
	}
	.readout::-webkit-inner-spin-button,
	.readout::-webkit-outer-spin-button {
		appearance: none;
		margin: 0;
	}
	.readout:focus-visible {
		outline-offset: 1px;
	}
	.presets {
		display: flex;
		gap: 3px;
	}
	.presets button {
		padding: 3px 7px;
		border-radius: var(--tc-r-ctl);
		font-size: 10px;
		color: var(--tc-muted);
	}
	.presets button:hover {
		color: var(--tc-ink);
	}
	.presets button.on {
		color: var(--tc-ink);
		background: var(--g-drop);
		box-shadow: var(--g-drop-shadow);
	}
</style>
```

- [ ] **Step 3: `src/components/CacheControl.svelte`** — заменить `<style>` целиком:

```svelte
<style>
	.cache {
		display: grid;
		gap: 7px;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.used {
		min-width: 64px;
		padding: 5px 9px;
		text-align: right;
		border-radius: var(--tc-r-row);
		background: var(--g-well);
		box-shadow: var(--g-well-shadow);
	}
	.key {
		padding: 6px 12px;
		border-radius: var(--tc-r-ctl);
		color: var(--tc-ink);
		background: var(--g-drop);
		box-shadow: var(--g-drop-shadow);
	}
	/* второе нажатие удалит файлы — клавиша краснеет */
	.key.armed {
		color: #fff;
		background: var(--tc-danger);
		box-shadow: 0 6px 14px -8px var(--tc-danger);
	}
	.key.warn {
		color: var(--tc-danger);
	}
</style>
```

- [ ] **Step 4: `src/components/Tooltip.svelte`** — в `<style>` заменить правила `.tip`, `.top::after`, `.bottom::after`, `p`, `ul`, `li::before`, `.meters`, `.bars i`, `.fps i.on`, `.look i.on` на:

```css
	.tip {
		position: fixed;
		z-index: 100;
		width: max-content;
		max-width: 320px;
		padding: 11px 13px;
		border-radius: var(--tc-r-row);
		background: var(--tc-tip-bg);
		color: var(--tc-tip-fg);
		box-shadow: var(--g-shadow);
		pointer-events: none;
		animation: appear 0.15s var(--tc-ease);
	}
	.tip::after {
		content: "";
		position: absolute;
		left: calc(var(--arrow) - 5px);
		border: 5px solid transparent;
	}
	.top::after {
		top: 100%;
		border-top-color: var(--tc-tip-bg);
	}
	.bottom::after {
		bottom: 100%;
		border-bottom-color: var(--tc-tip-bg);
	}
	p {
		margin: 0;
		font-size: 12px;
		line-height: 1.45;
		color: var(--tc-tip-muted);
	}
	ul {
		display: grid;
		gap: 5px;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: 12px;
		line-height: 1.45;
		color: var(--tc-tip-muted);
	}
	li::before {
		content: "";
		position: absolute;
		left: 0;
		top: 0.62em;
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--tc-accent);
	}
	.meters {
		display: grid;
		grid-template-columns: 1fr auto;
		align-items: center;
		gap: 5px 16px;
		margin-top: 8px;
		padding-top: 8px;
		border-top: 1px solid color-mix(in srgb, var(--tc-tip-muted) 35%, transparent);
		white-space: nowrap;
	}
	.bars i {
		width: 5px;
		height: 10px;
		border-radius: 2px;
		background: color-mix(in srgb, var(--tc-tip-muted) 40%, transparent);
	}
	.fps i.on {
		background: var(--tc-good);
	}
	.look i.on {
		background: var(--tc-accent);
	}
```

(правила `b`, `b.solo`, `li`, `.meter`, `.bars`, `@keyframes appear` оставить.)

- [ ] **Step 5: Проверка**

Run: `npm run check && npm test`
Expected: 0 ошибок, PASS.

Run: `npm run dev` → «Настройка». Expected: переключатели — капсулы со стеклянным бегунком, включённые залиты акцентом; фейдеры — пазы с каплей; число FPS вводится в паз; «Очистить кэш» — капля, при втором нажатии красная; подсказки — тёмная пластина (в ночной теме — светлая) с акцентными маркерами.

- [ ] **Step 6: Commit**

```bash
git add src/components/Toggle.svelte src/components/Fader.svelte src/components/CacheControl.svelte src/components/Tooltip.svelte
git commit -m "feat: glass toggle, fader, cache key and tooltip" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 12: «Настройка» и раздел «Вид»

**Files:**
- Create: `src/components/Swatches.svelte`
- Modify: `src/screens/TuningScreen.svelte`
- Modify: `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`

**Interfaces:**
- Consumes: `ACCENTS` (Task 2), `nextIndex` (Task 3), `Groove` (Task 6), `openScreen` (Task 5), `app.saveSettings({ accent } | { motion } | { theme } | { locale })`.
- Produces: `Swatches.svelte` props `{ value: AccentId; label: string; onchange: (v: AccentId) => void }`; i18n `tuning.appearance`, `tuning.accent`, `tuning.motion`, `motion.full`, `motion.reduced`, `accent.<id>` ×10; `theme.day` / `theme.night` переименованы.

- [ ] **Step 1: Строки.** В `ru.json` заменить значения `"theme.day": "СВЕТЛАЯ"`, `"theme.night": "ТЁМНАЯ"` и добавить:

```json
  "tuning.appearance": "ВИД",
  "tuning.accent": "АКЦЕНТ",
  "tuning.motion": "ДВИЖЕНИЕ",
  "motion.full": "ПОЛНОЕ",
  "motion.reduced": "МЕНЬШЕ",
  "accent.peach": "Персик",
  "accent.amber": "Янтарь",
  "accent.sage": "Шалфей",
  "accent.mint": "Мята",
  "accent.azure": "Лазурь",
  "accent.periwinkle": "Барвинок",
  "accent.lavender": "Лаванда",
  "accent.orchid": "Орхидея",
  "accent.rose": "Роза",
  "accent.steel": "Сталь"
```

В `en.json`: `"theme.day": "LIGHT"`, `"theme.night": "DARK"` и:

```json
  "tuning.appearance": "APPEARANCE",
  "tuning.accent": "ACCENT",
  "tuning.motion": "MOTION",
  "motion.full": "FULL",
  "motion.reduced": "LESS",
  "accent.peach": "Peach",
  "accent.amber": "Amber",
  "accent.sage": "Sage",
  "accent.mint": "Mint",
  "accent.azure": "Azure",
  "accent.periwinkle": "Periwinkle",
  "accent.lavender": "Lavender",
  "accent.orchid": "Orchid",
  "accent.rose": "Rose",
  "accent.steel": "Steel"
```

- [ ] **Step 2: Тест на имена акцентов** — в начало `src/lib/palette.test.ts` к импортам добавить

```ts
import en from "./i18n/en.json";
import ru from "./i18n/ru.json";
```

и в конец файла:

```ts
describe("accent names", () => {
	it("every accent is named in both languages", () => {
		for (const a of ACCENTS) {
			expect(ru).toHaveProperty([`accent.${a.id}`]);
			expect(en).toHaveProperty([`accent.${a.id}`]);
		}
	});
});
```

Run: `npm test` — Expected: PASS (если Step 1 сделан), иначе FAIL с именем недостающего ключа.

- [ ] **Step 3: `src/components/Swatches.svelte`**

```svelte
<script lang="ts">
	import { t } from "../lib/i18n.svelte";
	import { ACCENTS } from "../lib/palette";
	import { nextIndex } from "../lib/radio";
	import type { AccentId } from "../lib/types";

	let { value, label, onchange }: { value: AccentId; label: string; onchange: (v: AccentId) => void } = $props();

	let box = $state<HTMLDivElement>();
	const index = $derived(ACCENTS.findIndex((a) => a.id === value));

	function onkeydown(e: KeyboardEvent) {
		const i = nextIndex(index, e.key, ACCENTS.map(() => true));
		if (i === null) return;
		e.preventDefault();
		onchange(ACCENTS[i].id);
		box?.querySelectorAll<HTMLButtonElement>("button")[i]?.focus();
	}
</script>

<div class="field">
	<span class="label">{label}</span>
	<div class="swatches" role="radiogroup" aria-label={label} tabindex="-1" bind:this={box} {onkeydown}>
		{#each ACCENTS as a (a.id)}
			<button
				type="button"
				role="radio"
				aria-checked={a.id === value}
				aria-label={t(`accent.${a.id}`)}
				title={t(`accent.${a.id}`)}
				tabindex={a.id === value ? 0 : -1}
				style:--c={`oklch(0.73 ${a.c} ${a.h})`}
				onclick={() => onchange(a.id)}
			></button>
		{/each}
	</div>
</div>

<style>
	.field {
		display: grid;
		gap: 7px;
	}
	.swatches {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.swatches:focus {
		outline: none;
	}
	button {
		width: 26px;
		height: 26px;
		border-radius: 50%;
		background: var(--c);
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 45%),
			0 2px 4px -1px rgb(0 0 0 / 30%);
		transition: transform 240ms var(--tc-ease);
	}
	button:hover {
		transform: translateY(-1px);
	}
	button[aria-checked="true"] {
		box-shadow:
			0 0 0 2px var(--field-base),
			0 0 0 4px var(--c);
	}
</style>
```

- [ ] **Step 4: `src/screens/TuningScreen.svelte` — скрипт.** Импорты: добавить `import Swatches from "../components/Swatches.svelte";`, `import { openScreen } from "../lib/motion";`, в импорт типов добавить `MotionPref`. Заменить:

```ts
	const title = $derived(server ? server.name : t("tuning.global"));
```

```ts
	const scopeOptions = $derived([
		{ value: "global", label: t("tuning.global") },
		...(selIndex >= 0 ? [{ value: dto.servers[selIndex].id, label: dto.servers[selIndex].name }] : [])
	]);
```

Добавить:

```ts
	const motionOptions = $derived((["full", "reduced"] as MotionPref[]).map((v) => ({ value: v, label: t(`motion.${v}`) })));

	let root = $state<HTMLDivElement>();
	$effect(() => {
		if (root) openScreen(root);
	});
```

- [ ] **Step 5: Разметка.** Корень: `<div class="tuning" bind:this={root}>`. Шапка:

```svelte
	<header class="head" data-part>
		<button class="back" onclick={() => app.closeScreen()}>
			<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M10 3.5 5.5 8l4.5 4.5" /></svg>{t("tuning.back")}
		</button>
		<b>{t("tuning.title")} · {title}</b>
		<span class="label">
			{t("tuning.profile")}
			{t(`profile.${profileFor(dto, scope)}`)}
			{#if n > 0}<span class="badge">{t("tuning.changed", { n })}</span>{/if}
		</span>
	</header>
```

Блокам `.profile`, `.toggles`, `.rows` добавить классы `pane` и атрибут `data-part`; каждому `<Fader …/>` и `<Groove label={t("tuning.perfMode")} …/>` внутри `.faders` — обернуть в `<div class="pane cell" data-part>…</div>`. В блоке `{#if !server}` удалить две строки `<Groove label={t("tuning.language")} …/>` и `<Groove label={t("tuning.theme")} …/>`, а сразу после закрывающего `</div>` блока `.rows.engine` (внутри того же `{#if !server}`) вставить:

```svelte
			<section class="pane rows look" data-part>
				<span class="label sec">{t("tuning.appearance")}</span>
				<Groove label={t("tuning.theme")} options={themeOptions} value={dto.settings.theme} onchange={(v) => app.saveSettings({ theme: v })} />
				<Groove label={t("tuning.motion")} options={motionOptions} value={dto.settings.motion} onchange={(v) => app.saveSettings({ motion: v })} />
				<Swatches label={t("tuning.accent")} value={dto.settings.accent} onchange={(v) => app.saveSettings({ accent: v })} />
				<Groove label={t("tuning.language")} options={localeOptions} value={dto.settings.locale} onchange={(v) => app.saveSettings({ locale: v })} />
			</section>
```

Подвал: `<footer class="foot mono" data-part>` — содержимое без изменений.

- [ ] **Step 6: Стили** — заменить `<style>` целиком:

```svelte
<style>
	.tuning {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto;
		gap: 12px;
		height: 100%;
		min-height: 0;
		padding: 2px 14px 14px;
	}
	.head,
	.foot {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 0 4px;
	}
	.head b {
		margin-right: auto;
		font: 600 16px/1.2 var(--tc-font-ui);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.back {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 32px;
		padding: 0 14px 0 10px;
		border-radius: var(--tc-r-ctl);
		background: var(--g-drop);
		box-shadow: var(--g-drop-shadow);
		font: 600 11px/1 var(--tc-font-mono);
		letter-spacing: 0.06em;
	}
	.back svg {
		width: 14px;
		height: 14px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.badge {
		margin-left: 6px;
		padding: 2px 7px;
		border-radius: var(--tc-r-ctl);
		color: var(--tc-on-accent);
		background: var(--tc-accent);
	}
	.body {
		display: grid;
		align-content: start;
		gap: 12px;
		overflow-y: auto;
		min-height: 0;
		padding: 2px;
	}
	.profile,
	.toggles,
	.rows {
		padding: 14px 18px;
	}
	.profile {
		display: flex;
		flex-wrap: wrap;
		gap: 14px 28px;
	}
	.faders {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 12px;
	}
	.cell {
		display: grid;
		align-content: start;
		padding: 2px;
	}
	.toggles {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 2px 28px;
	}
	.rows {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 14px 28px;
	}
	.sec {
		grid-column: 1 / -1;
		color: var(--tc-ink);
	}
	.extra {
		grid-column: 1 / -1;
		display: grid;
		gap: 6px;
	}
	.extra input {
		height: 36px;
		padding: 0 12px;
		border: 0;
		border-radius: var(--tc-r-row);
		background: var(--g-well);
		box-shadow: var(--g-well-shadow);
	}
	.hint {
		color: var(--tc-muted);
	}
	.foot {
		justify-content: space-between;
		color: var(--tc-muted);
	}
	.reset {
		color: var(--tc-accent-text);
		text-decoration: underline;
		text-underline-offset: 3px;
	}
</style>
```

- [ ] **Step 7: Проверка**

Run: `npm run check && npm test`
Expected: 0 ошибок, PASS.

Run: `npm run dev` → «Настройка» (уровень «Все серверы»). Expected: шапка с каплей «← НАЗАД»; листы над туманом; раздел «ВИД»: тема Авто/Светлая/Тёмная, движение Полное/Меньше, 10 образцов акцента, язык. Клик по «Лаванда» за 0.7 s перекрашивает акцент и мятный туман в лавандовый; «Меньше» останавливает дрейф и капли перестают тянуться; после перезапуска мока (F5) выбор сохраняется в пределах сессии мока.

- [ ] **Step 8: Commit**

```bash
git add src/components/Swatches.svelte src/screens/TuningScreen.svelte src/lib/i18n/ru.json src/lib/i18n/en.json src/lib/palette.test.ts
git commit -m "feat: Settings on Tactile with Appearance: theme, motion, accent" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 13: Редактор слота

**Files:**
- Modify: `src/screens/SlotEditor.svelte`
- Modify: `src/lib/i18n/ru.json`, `src/lib/i18n/en.json` (`slot.edit`)

**Interfaces:**
- Consumes: `Groove` (Task 6), `openScreen` (Task 5).

- [ ] **Step 1: Строки.** `ru.json`: `"slot.edit": "Сервер «{name}»"`; `en.json`: `"slot.edit": "Server “{name}”"`.

- [ ] **Step 2: Скрипт.** Добавить `import { openScreen } from "../lib/motion";` и:

```ts
	const savedName = $derived(index >= 0 ? app.dto!.servers[index].name : "");

	let root = $state<HTMLFormElement>();
	$effect(() => {
		if (root) openScreen(root);
	});
```

- [ ] **Step 3: Разметка.** `<form class="editor" bind:this={root} onsubmit={save} novalidate>`; заголовок:

```svelte
	<header class="head" data-part>
		<b>{isNew ? t("slot.new") : t("slot.edit", { name: savedName })}</b>
	</header>
```

`<div class="fields">` → `<div class="fields pane" data-part>`; `<footer class="actions">` → `<footer class="actions" data-part>`. Остальное без изменений.

- [ ] **Step 4: Стили** — заменить `<style>` целиком:

```svelte
<style>
	.editor {
		display: grid;
		grid-template-rows: auto auto 1fr auto;
		gap: 12px;
		height: 100%;
		padding: 2px 14px 14px;
	}
	.head {
		padding: 4px 4px 0;
	}
	.head b {
		font: 600 18px/1.2 var(--tc-font-ui);
	}
	.fields {
		display: grid;
		gap: 16px;
		max-width: 560px;
		padding: 18px;
	}
	.field {
		display: grid;
		gap: 6px;
	}
	.field > span {
		color: var(--tc-muted);
		text-transform: uppercase;
		letter-spacing: 0.08em;
		font-size: 10.5px;
	}
	input {
		height: 40px;
		padding: 0 12px;
		border: 0;
		border-radius: var(--tc-r-row);
		background: var(--g-well);
		box-shadow: var(--g-well-shadow);
	}
	input[aria-invalid="true"] {
		box-shadow:
			var(--g-well-shadow),
			0 0 0 2px var(--tc-danger);
	}
	.err {
		color: var(--tc-danger);
	}
	.link {
		justify-self: start;
		color: var(--tc-accent-text);
		text-decoration: underline;
		text-underline-offset: 3px;
	}
	.actions {
		grid-row: 4;
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.primary {
		height: 40px;
		padding: 0 22px;
		border-radius: var(--tc-r-ctl);
		color: var(--tc-on-accent);
		font-weight: 700;
		background: linear-gradient(180deg, oklch(0.86 calc(var(--tc-acc-c) * 0.8) var(--tc-acc-h) / 0.95), oklch(0.72 var(--tc-acc-c) var(--tc-acc-h) / 0.88));
		box-shadow:
			inset 0 1px 0 rgb(255 255 255 / 75%),
			0 12px 22px -14px var(--tc-glow);
	}
	.primary[aria-busy="true"] {
		cursor: progress;
	}
	.secondary {
		height: 40px;
		padding: 0 18px;
		border-radius: var(--tc-r-ctl);
		background: var(--g-drop);
		box-shadow: var(--g-drop-shadow);
	}
	.danger {
		margin-left: auto;
		color: var(--tc-danger);
	}
	.danger.armed {
		padding: 0 14px;
		height: 36px;
		border-radius: var(--tc-r-ctl);
		color: #fff;
		background: var(--tc-danger);
	}
</style>
```

- [ ] **Step 5: Проверка**

Run: `npm run check && npm test`
Expected: 0 ошибок, PASS.

Run: `npm run dev` → «ИЗМ» у сервера. Expected: заголовок «Сервер «Aldarion»», лист с полями-пазами, паз профиля с каплей, «Сохранить» — мятное стекло, «Отмена» — капля, «Удалить» → «точно удалить?» красной капсулой; пустое имя подсвечивается красным кольцом.

- [ ] **Step 6: Commit**

```bash
git add src/screens/SlotEditor.svelte src/lib/i18n/ru.json src/lib/i18n/en.json
git commit -m "feat: slot editor on Tactile" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 14: Приёмка

**Files:** без изменений кода, кроме найденных дефектов.

- [ ] **Step 1: Автоматика**

Run:

```bash
npm test
npm run check
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings && cd ..
npm run dist
```

Expected: всё зелёное; `npm run dist` собирает портативную сборку и установщик.

- [ ] **Step 2: Остатки старого пульта**

Run: `grep -rn "Knob\|Segmented\|main.channels\|--grain\|--signal" src --include=*.svelte | grep -v "InstallScreen\|UninstallScreen\|Lcd.svelte"`
Expected: пусто (старые токены остаются только у установщика, удаления и ЖК).

- [ ] **Step 3: Ручной смоук портативной сборки** — запускать с изолированными `APPDATA` / `LOCALAPPDATA` и копией `servers.json`, закрывать только свой процесс по PID:
  - светлая и тёмная тема, переключение «Авто» вслед за ОС;
  - все 10 акцентов (контраст подписей в линзе и на листах читается);
  - «Движение: меньше» и системное «Уменьшить анимацию» — капли прыгают, туман и аура стоят;
  - выбор сервера, двойной клик — запуск, Shift — «БЕЗОПАСНО», «ИЗМ» — редактор, «Добавить сервер»;
  - профиль: КАЧ / БАЛ / КРТ стрелками и мышью, «РУЧ» мышью (возвращает правки или открывает «Настройку»);
  - уведомление `app.message` всплывает и закрывается;
  - проверка обновлений в заголовке; при доступном обновлении — акцентный чип, загрузка показывает проценты в линзе;
  - статусы ускорения: в dev-превью `?accel=software`, `?accel=wrongGpu`, без проверки — туман красный, янтарный, почти серый;
  - установщик (`--install`-режим сборки) открывается и проходит экраны без поломок.

- [ ] **Step 4: Нагрузка на GPU** — портативная сборка на GTX 1060, диспетчер задач → «Графический процессор» у процесса лаунчера:
  - в фокусе, простой 60 s — не выше 3 %;
  - без фокуса (кликнуть в другое окно) — 0 %.
  Записать цифры в отчёт задачи. Если порог превышен — сначала проверить, что `syncAmbient` ставит циклы на паузу, затем уменьшить число пятен до двух.

- [ ] **Step 5: Commit** — только если на шагах 1–4 правился код:

```bash
git add -A src src-tauri
git commit -m "fix: Tactile redesign acceptance fixes" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 15: Обновить документ Tactile

**Files:** Claude Docs, документ id `b90e9f28-8187-4a4e-98fe-2c8dc1e0abfe` (через MCP-коннектор Claude Docs; в репозитории ничего не меняется).

- [ ] **Step 1: Прочитать документ** через коннектор (`read`). Ответ около 106 тыс. символов: сохранить и конвертировать из `data.xml` скриптом в scratchpad, найти разделы «Материалы», «Движение», «Источники».

- [ ] **Step 2: Точечно дописать** (`update`, без перезаписи всего документа):
  - раздел «Стекло»: ярусы `--g-pane` / `--g-pane-solid` (лист), `--g-drop` (капля), `--g-lens` (линза); режимы «Везде» и «На героях»; единый кант `--g-rim`; правило «текст только на листе»; микрошум;
  - «Туман как индикатор статуса»: таблица `on` / `wrongGpu` / `software` / `unchecked` → `--aura-h`, `--aura-c`, дрейф;
  - «Перетекающая капля» и «Скользящая подсветка» с параметрами из таблицы §4 спеки;
  - кривая `breath` (0.37, 0, 0.63, 1 — только для циклов ауры и тумана);
  - «Источники»: раунд 3 https://claude.ai/artifact/WseGvp4xuAsyrzyKUHw1Z8, раунд 4/4.1 https://claude.ai/artifact/CFgaFBe9uq7Q8JUkvyiBzj, референсы Pinterest из раунда 4.

- [ ] **Step 3: Перечитать изменённые разделы** и убедиться, что остальное содержимое документа не тронуто.
