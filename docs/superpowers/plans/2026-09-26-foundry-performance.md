# Foundry Performance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Windows-клиент для Foundry VTT v14 (список серверов → игра в окне), который поднимает FPS на слабых GPU за счёт тюнинга движка WebView2 и агента производительности внутри страницы Foundry.

**Architecture:** Tauri 2 (Rust) управляет данными, GPU-детектом, флагами Chromium и окнами. Лаунчер — Svelte 5 SPA в окне `main`. Игра открывается в окне `game` с отдельной папкой данных WebView2 и своими флагами; в страницу внедряется агент (TypeScript → IIFE), который применяет профиль, рисует HUD и шлёт телеметрию через одну разрешённую команду.

**Tech Stack:** Rust (stable-msvc), Tauri 2.11, serde, reqwest (rustls), windows-rs 0.61 (DXGI); Svelte 5, TypeScript, Vite 8, Vitest 5, happy-dom; @fontsource/geologica, @fontsource/martian-mono; @resvg/resvg-js (иконка).

**Spec:** `docs/superpowers/specs/2026-09-26-foundry-performance-design.md`

## Global Constraints

- Только Windows 10/11 x64, WebView2 Evergreen. Никакого кода под macOS/Linux.
- Продукт: `Foundry Performance`; идентификатор `com.nikif.foundryperformance`; Cargo-пакет `foundry-performance`, lib-крейт `foundry_performance_lib`.
- Данные: `%APPDATA%\FoundryPerformance\` (`settings.json`, `servers.json`, `stats.json`); движок игры: `%LOCALAPPDATA%\FoundryPerformance\engine\`.
- Метки окон: лаунчер `main`, игра `game`.
- Все JSON между Rust и TS — camelCase (`#[serde(rename_all = "camelCase")]`).
- Профили: `quality` / `balance` / `potato`; ANGLE: `d3d11` / `d3d11on12` / `vulkan` / `gl`.
- Код оригинального FLC не копируется.
- UI без UI-китов и CSS-фреймворков; шрифты Geologica (UI) и Martian Mono (индикаторы) — локально через @fontsource.
- Цвета «Пульта»: панель `#D8D6D0`, лицевая `#E6E4DE`, графит `#2A2A28`, сигнал `#FF5A1F`, ЖК `#B8F28A` на `#1E2A1E`.
- Все строки интерфейса — через i18n (RU/EN), в коде компонентов нет захардкоженного текста (кроме имён собственных: Sequencer, FXMaster, Prime Performance, ANGLE, PERF MODE, FPS).
- Анимации только CSS, выключаются при `prefers-reduced-motion`.
- **Перед любой `cargo`-командой** должен существовать `agent/dist/agent.js` (Rust включает его через `include_str!`): при необходимости выполнить `npm run build:agent`.
- Команды выполняются из корня репозитория `D:\MyAiProjects\Apps\pult` в Git Bash, если не указано иное; `cargo`-команды — с флагом `--manifest-path src-tauri/Cargo.toml`.
- Коммиты заканчиваются строкой `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## File Map

```
package.json, vite.config.ts, vitest.config.ts, tsconfig.json, svelte.config.js, index.html, .gitignore
scripts/make-icon.mjs            — рендер app-icon.svg → app-icon.png
scripts/package-portable.ps1     — zip с exe
app-icon.svg

agent/vite.config.ts             — сборка агента в IIFE agent/dist/agent.js
agent/src/types.ts               — Levers, Boot, ProfileId …
agent/src/foundry.ts             — безопасный доступ к глобалам Foundry
agent/src/fps-meter.ts (+test)   — кольцевой буфер кадров, avg, 1% low, summarize
agent/src/adaptive.ts (+test)    — контроллер адаптивного разрешения (чистая функция)
agent/src/client-settings.ts (+test) — core.* ключи, preboot в localStorage, reconcile
agent/src/modules.ts (+test)     — клиентские настройки модулей (Sequencer, FXMaster, Prime)
agent/src/levers/blur.ts (+test)
agent/src/levers/video.ts (+test)
agent/src/levers/focus.ts (+test)
agent/src/levers/resolution.ts
agent/src/bench.ts               — «Замер»
agent/src/bridge.ts              — телеметрия в Rust
agent/src/diag.ts                — диагностический дамп (F10)
agent/src/i18n.ts                — строки HUD
agent/src/hud.ts                 — HUD + меню Shift+F9
agent/src/main.ts                — точка входа, связка

src-tauri/Cargo.toml, build.rs, tauri.conf.json
src-tauri/capabilities/launcher.json, game.json
src-tauri/icons/*                — генерируются `tauri icon`
src-tauri/src/main.rs
src-tauri/src/lib.rs             — Builder, setup, события окон, RunEvent
src-tauri/src/model.rs           — типы данных
src-tauri/src/profile.rs         — пресеты, наложение правок, sanitize
src-tauri/src/engine_flags.rs    — строка флагов Chromium
src-tauri/src/store.rs           — JSON-хранилище
src-tauri/src/gpu.rs             — DXGI + рекомендация профиля
src-tauri/src/locale.rs          — язык системы
src-tauri/src/probe.rs           — нормализация URL, /api/status
src-tauri/src/agent.rs           — сборка initialization_script
src-tauri/src/telemetry.rs       — валидация и применение отчётов агента
src-tauri/src/windows.rs         — окна лаунчера и игры
src-tauri/src/tray.rs            — трей
src-tauri/src/state.rs           — AppState
src-tauri/src/commands.rs        — команды Tauri

src/main.ts, src/App.svelte
src/lib/types.ts                 — TS-зеркало моделей
src/lib/levers.ts (+test)        — наложение правок для отображения
src/lib/api.ts, src/lib/mock.ts  — вызовы Tauri / моки для превью в браузере
src/lib/i18n/ru.json, en.json, src/lib/i18n.svelte.ts (+test)
src/lib/store.svelte.ts          — состояние приложения (runes)
src/styles/tokens.css, src/styles/base.css
src/components/TitleBar.svelte, Lcd.svelte, Led.svelte, Knob.svelte, Slot.svelte,
  LaunchButton.svelte, Fader.svelte, Toggle.svelte, Segmented.svelte
src/screens/MainScreen.svelte, TuningScreen.svelte, SlotEditor.svelte
```

---

### Task 0: Тулчейн и каркас проекта

**Files:**
- Create: `package.json`, `vite.config.ts`, `vitest.config.ts`, `tsconfig.json`, `svelte.config.js`, `index.html`, `.gitignore`, `src/main.ts`, `src/App.svelte`, `agent/vite.config.ts`, `agent/src/main.ts`, `app-icon.svg`, `scripts/make-icon.mjs`
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/launcher.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: npm-скрипты `dev`, `build`, `build:agent`, `test`, `check`, `tauri`, `icon`; lib-крейт `foundry_performance_lib::run()`.

- [ ] **Step 1: Проверить/установить MSVC Build Tools**

Run:
```bash
"/c/Program Files (x86)/Microsoft Visual Studio/Installer/vswhere.exe" -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
```
Если вывод пустой или файла нет:
```bash
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --accept-source-agreements --accept-package-agreements --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```
Expected: путь к установке Build Tools.

- [ ] **Step 2: Установить Rust**

```bash
winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements
"$USERPROFILE/.cargo/bin/rustup" default stable-msvc
"$USERPROFILE/.cargo/bin/cargo" --version
```
Expected: `cargo 1.x`. В следующих командах, если `cargo` не в PATH текущей оболочки, использовать `export PATH="$USERPROFILE/.cargo/bin:$PATH"`.

- [ ] **Step 3: Создать `package.json`**

```json
{
  "name": "foundry-performance",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "build:agent": "vite build --config agent/vite.config.ts",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "test": "vitest run",
    "icon": "node scripts/make-icon.mjs && tauri icon app-icon.png",
    "portable": "powershell -ExecutionPolicy Bypass -File scripts/package-portable.ps1",
    "tauri": "tauri"
  },
  "dependencies": {
    "@fontsource/geologica": "^5.3.0",
    "@fontsource/martian-mono": "^5.3.0",
    "@tauri-apps/api": "^2.11.0"
  },
  "devDependencies": {
    "@resvg/resvg-js": "^2.6.2",
    "@sveltejs/vite-plugin-svelte": "^7.3.1",
    "@tauri-apps/cli": "^2.11.5",
    "@tsconfig/svelte": "^5.0.4",
    "happy-dom": "^20.14.5",
    "svelte": "^5.57.1",
    "svelte-check": "^4.3.0",
    "typescript": "^5.9.0",
    "vite": "^8.3.1",
    "vitest": "^5.0.2"
  }
}
```

- [ ] **Step 4: Конфиги сборки**

`vite.config.ts`:
```ts
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
	plugins: [svelte()],
	clearScreen: false,
	server: { port: 1420, strictPort: true },
	build: { target: "es2022", outDir: "dist", emptyOutDir: true }
});
```

`vitest.config.ts`:
```ts
import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		include: ["agent/src/**/*.test.ts", "src/**/*.test.ts"],
		environment: "node"
	}
});
```

`svelte.config.js`:
```js
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default { preprocess: vitePreprocess() };
```

`tsconfig.json`:
```json
{
  "extends": "@tsconfig/svelte/tsconfig.json",
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "skipLibCheck": true,
    "types": ["vite/client"]
  },
  "include": ["src/**/*.ts", "src/**/*.svelte", "agent/src/**/*.ts"]
}
```

`agent/vite.config.ts`:
```ts
import { defineConfig } from "vite";

export default defineConfig({
	build: {
		lib: {
			entry: "agent/src/main.ts",
			formats: ["iife"],
			name: "FoundryPerformanceAgent",
			fileName: () => "agent.js"
		},
		outDir: "agent/dist",
		emptyOutDir: true,
		target: "es2022",
		minify: true
	}
});
```

`.gitignore`:
```
node_modules/
dist/
agent/dist/
dist-portable/
src-tauri/target/
src-tauri/gen/
app-icon.png
```

- [ ] **Step 5: Минимальные входные точки фронтенда и агента**

`index.html`:
```html
<!doctype html>
<html lang="ru">
	<head>
		<meta charset="UTF-8" />
		<meta name="viewport" content="width=device-width, initial-scale=1.0" />
		<title>Foundry Performance</title>
	</head>
	<body>
		<div id="app"></div>
		<script type="module" src="/src/main.ts"></script>
	</body>
</html>
```

`src/main.ts`:
```ts
import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
```

`src/App.svelte` (заменяется в Task 14):
```svelte
<main>Foundry Performance</main>
```

`agent/src/main.ts` (заменяется в Task 8):
```ts
(globalThis as Record<string, unknown>).__FP__ = { version: "0.1.0" };
```

- [ ] **Step 6: Иконка**

`app-icon.svg` — ручка «Пульта»:
```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <rect width="1024" height="1024" rx="180" fill="#2A2A28"/>
  <circle cx="512" cy="540" r="330" fill="#BDBAB2"/>
  <circle cx="512" cy="540" r="270" fill="#D8D6D0"/>
  <rect x="482" y="250" width="60" height="230" rx="10" fill="#FF5A1F" transform="rotate(28 512 540)"/>
  <circle cx="512" cy="540" r="36" fill="#2A2A28"/>
</svg>
```

`scripts/make-icon.mjs`:
```js
import { readFileSync, writeFileSync } from "node:fs";
import { Resvg } from "@resvg/resvg-js";

const svg = readFileSync("app-icon.svg", "utf8");
const png = new Resvg(svg, { fitTo: { mode: "width", value: 1024 } }).render().asPng();
writeFileSync("app-icon.png", png);
console.log("app-icon.png written");
```

- [ ] **Step 7: Каркас Tauri**

`src-tauri/Cargo.toml`:
```toml
[package]
name = "foundry-performance"
version = "0.1.0"
description = "Foundry Performance"
edition = "2021"

[lib]
name = "foundry_performance_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
url = "2"
windows = { version = "0.61", features = ["Win32_Graphics_Dxgi", "Win32_Globalization"] }

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["macros", "rt"] }

[profile.release]
codegen-units = 1
lto = true
opt-level = "s"
panic = "abort"
strip = true
```

`src-tauri/build.rs`:
```rust
fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_state",
            "save_server",
            "delete_server",
            "save_settings",
            "probe_server",
            "launch",
            "clear_notice",
            "report_telemetry",
        ]),
    ))
    .expect("failed to run tauri-build");
}
```

`src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Foundry Performance",
  "version": "0.1.0",
  "identifier": "com.nikif.foundryperformance",
  "build": {
    "beforeDevCommand": "npm run build:agent && npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build:agent && npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self' data:; img-src 'self' data:; connect-src ipc: http://ipc.localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.ico"],
    "windows": {
      "webviewInstallMode": { "type": "downloadBootstrapper" },
      "nsis": { "installMode": "currentUser", "languages": ["Russian", "English"] }
    }
  }
}
```

`src-tauri/capabilities/launcher.json`:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "launcher",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-close",
    "core:window:allow-destroy",
    "core:window:allow-minimize",
    "core:window:allow-start-dragging",
    "allow-get-state",
    "allow-save-server",
    "allow-delete-server",
    "allow-save-settings",
    "allow-probe-server",
    "allow-launch",
    "allow-clear-notice"
  ]
}
```

`src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    foundry_performance_lib::run()
}
```

`src-tauri/src/lib.rs` (заменяется в Task 9):
```rust
use tauri::{WebviewUrl, WebviewWindowBuilder};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Foundry Performance")
                .inner_size(900.0, 620.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Foundry Performance");
}
```

- [ ] **Step 8: Установить зависимости, сгенерировать иконки, собрать**

```bash
npm install
npm run icon
npm run build:agent
cargo check --manifest-path src-tauri/Cargo.toml
```
Expected: `npm run icon` создаёт `src-tauri/icons/*`; `cargo check` завершается без ошибок (предупреждения о неиспользуемых разрешениях допустимы).

- [ ] **Step 9: Смоук-запуск**

Run: `npm run tauri dev` (в фоне), дождаться окна «Foundry Performance» с текстом «Foundry Performance», закрыть.
Expected: окно открывается; в логе нет паники.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri 2 + Svelte 5 + agent build

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 1: Модель данных и профили (Rust)

**Files:**
- Create: `src-tauri/src/model.rs`, `src-tauri/src/profile.rs`
- Modify: `src-tauri/src/lib.rs` (добавить `pub mod model; pub mod profile;` в начало файла)

**Interfaces:**
- Produces:
  - `model::{ProfileId, VideoMode, PrimeLevel, AngleBackend, Levers, Overrides, EngineSettings, Locale, Theme, Settings, Server, FpsSummary, BenchResult, ServerStats, Notice}`
  - `AngleBackend::flag_value(self) -> &'static str`, `AngleBackend::next(self) -> AngleBackend`
  - `Notice::new(key: &str) -> Notice`, `Notice::with(self, k: &str, v: impl Into<String>) -> Notice`
  - `profile::preset(id: ProfileId) -> Levers`
  - `profile::apply(base: Levers, o: &Overrides) -> Levers` (включает sanitize)
  - `profile::sanitize(l: Levers) -> Levers`
  - `profile::resolve_all(settings: &Settings, server: Option<&Server>) -> BTreeMap<ProfileId, Levers>`
  - `profile::active_id(settings: &Settings, server: Option<&Server>) -> ProfileId`
  - `profile::presets() -> BTreeMap<ProfileId, Levers>`

- [ ] **Step 1: Написать `model.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProfileId {
    Quality,
    Balance,
    Potato,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoMode {
    Play,
    PauseUnfocused,
    Static,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrimeLevel {
    Soft,
    Medium,
    Aggressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AngleBackend {
    #[serde(rename = "d3d11")]
    D3d11,
    #[serde(rename = "d3d11on12")]
    D3d11on12,
    #[serde(rename = "gl")]
    Gl,
    #[serde(rename = "vulkan")]
    Vulkan,
}

impl AngleBackend {
    pub fn flag_value(self) -> &'static str {
        match self {
            AngleBackend::D3d11 => "d3d11",
            AngleBackend::D3d11on12 => "d3d11on12",
            AngleBackend::Gl => "gl",
            AngleBackend::Vulkan => "vulkan",
        }
    }

    /// Порядок отката при отказе WebGL: d3d11 → d3d11on12 → gl → vulkan → d3d11.
    pub fn next(self) -> AngleBackend {
        match self {
            AngleBackend::D3d11 => AngleBackend::D3d11on12,
            AngleBackend::D3d11on12 => AngleBackend::Gl,
            AngleBackend::Gl => AngleBackend::Vulkan,
            AngleBackend::Vulkan => AngleBackend::D3d11,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Levers {
    pub perf_mode: u8,
    pub max_fps: u32,
    pub res_min: f32,
    pub res_max: f32,
    pub adaptive: bool,
    pub pixel_ratio_scaling: bool,
    pub light_animation: bool,
    pub vision_animation: bool,
    pub mipmap: bool,
    pub video: VideoMode,
    pub ui_blur: bool,
    pub sequencer: bool,
    pub fxmaster: bool,
    pub unfocused_fps: u32,
    pub prime: PrimeLevel,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Overrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perf_mode: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub res_min: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub res_max: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_ratio_scaling: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_animation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision_animation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mipmap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_blur: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequencer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fxmaster: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unfocused_fps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prime: Option<PrimeLevel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EngineSettings {
    pub angle: AngleBackend,
    pub disk_cache_mb: u32,
    pub extra_args: String,
}

impl Default for EngineSettings {
    fn default() -> Self {
        EngineSettings { angle: AngleBackend::D3d11, disk_cache_mb: 2048, extra_args: String::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Locale {
    #[default]
    Auto,
    Ru,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    Auto,
    Day,
    Night,
}

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub schema: u32,
    pub profile: ProfileId,
    pub overrides: Overrides,
    pub engine: EngineSettings,
    pub locale: Locale,
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            schema: SCHEMA,
            profile: ProfileId::Balance,
            overrides: Overrides::default(),
            engine: EngineSettings::default(),
            locale: Locale::Auto,
            theme: Theme::Auto,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub profile: Option<ProfileId>,
    #[serde(default)]
    pub overrides: Overrides,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FpsSummary {
    pub avg: f32,
    pub low1: f32,
    pub profile: ProfileId,
    pub at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchResult {
    pub avg: f32,
    pub low1: f32,
    pub min: f32,
    pub profile: ProfileId,
    pub at: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ServerStats {
    pub last_session: Option<FpsSummary>,
    pub last_bench: Option<BenchResult>,
}

/// Сообщение для ЖК лаунчера: ключ i18n + параметры.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub key: String,
    pub params: BTreeMap<String, String>,
}

impl Notice {
    pub fn new(key: &str) -> Notice {
        Notice { key: key.to_string(), params: BTreeMap::new() }
    }

    pub fn with(mut self, k: &str, v: impl Into<String>) -> Notice {
        self.params.insert(k.to_string(), v.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enums_serialize_as_camel_case_strings() {
        assert_eq!(serde_json::to_string(&ProfileId::Potato).unwrap(), "\"potato\"");
        assert_eq!(serde_json::to_string(&VideoMode::PauseUnfocused).unwrap(), "\"pauseUnfocused\"");
        assert_eq!(serde_json::to_string(&AngleBackend::D3d11on12).unwrap(), "\"d3d11on12\"");
    }

    #[test]
    fn angle_fallback_cycles_through_all_backends() {
        let mut a = AngleBackend::D3d11;
        let mut seen = vec![a];
        for _ in 0..3 {
            a = a.next();
            seen.push(a);
        }
        assert_eq!(seen, vec![AngleBackend::D3d11, AngleBackend::D3d11on12, AngleBackend::Gl, AngleBackend::Vulkan]);
        assert_eq!(a.next(), AngleBackend::D3d11);
    }

    #[test]
    fn settings_tolerate_missing_fields() {
        let s: Settings = serde_json::from_str(r#"{"profile":"potato"}"#).unwrap();
        assert_eq!(s.profile, ProfileId::Potato);
        assert_eq!(s.engine.angle, AngleBackend::D3d11);
        assert_eq!(s.schema, SCHEMA);
    }

    #[test]
    fn empty_overrides_serialize_to_empty_object() {
        assert_eq!(serde_json::to_string(&Overrides::default()).unwrap(), "{}");
    }
}
```

- [ ] **Step 2: Написать падающие тесты `profile.rs`**

```rust
use crate::model::*;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests {
    use super::*;

    fn server(profile: Option<ProfileId>, overrides: Overrides) -> Server {
        Server { id: "s1".into(), name: "S".into(), url: "https://x".into(), profile, overrides }
    }

    #[test]
    fn presets_match_spec_table() {
        let q = preset(ProfileId::Quality);
        assert_eq!((q.perf_mode, q.max_fps, q.res_min, q.res_max, q.adaptive), (2, 60, 1.0, 1.0, false));
        let b = preset(ProfileId::Balance);
        assert_eq!((b.perf_mode, b.res_min, b.res_max, b.adaptive, b.vision_animation), (1, 0.7, 1.0, true, false));
        assert_eq!((b.video, b.ui_blur, b.unfocused_fps), (VideoMode::PauseUnfocused, false, 15));
        let p = preset(ProfileId::Potato);
        assert_eq!((p.perf_mode, p.max_fps, p.res_min, p.res_max), (0, 45, 0.55, 0.85));
        assert_eq!((p.light_animation, p.mipmap, p.video, p.sequencer, p.fxmaster), (false, false, VideoMode::Static, false, false));
    }

    #[test]
    fn server_profile_wins_over_global() {
        let s = Settings { profile: ProfileId::Quality, ..Settings::default() };
        assert_eq!(active_id(&s, Some(&server(Some(ProfileId::Potato), Overrides::default()))), ProfileId::Potato);
        assert_eq!(active_id(&s, Some(&server(None, Overrides::default()))), ProfileId::Quality);
        assert_eq!(active_id(&s, None), ProfileId::Quality);
    }

    #[test]
    fn server_overrides_apply_after_global_overrides() {
        let s = Settings {
            overrides: Overrides { max_fps: Some(30), mipmap: Some(false), ..Overrides::default() },
            ..Settings::default()
        };
        let srv = server(None, Overrides { max_fps: Some(50), ..Overrides::default() });
        let all = resolve_all(&s, Some(&srv));
        let b = all[&ProfileId::Balance];
        assert_eq!(b.max_fps, 50);
        assert!(!b.mipmap);
        assert_eq!(all[&ProfileId::Quality].max_fps, 50);
    }

    #[test]
    fn sanitize_clamps_values() {
        let l = sanitize(Levers { perf_mode: 9, max_fps: 1, res_min: 1.4, res_max: 0.1, unfocused_fps: 0, ..preset(ProfileId::Balance) });
        assert_eq!(l.perf_mode, 3);
        assert_eq!(l.max_fps, 10);
        assert_eq!(l.unfocused_fps, 5);
        assert!(l.res_min >= 0.25 && l.res_max <= 1.0 && l.res_min <= l.res_max);
    }

    #[test]
    fn presets_contain_all_three() {
        assert_eq!(presets().len(), 3);
    }
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml profile`
Expected: FAIL — `preset`, `active_id`, `resolve_all`, `sanitize`, `presets` не найдены.

- [ ] **Step 3: Реализация `profile.rs`** (над блоком тестов)

```rust
pub fn preset(id: ProfileId) -> Levers {
    match id {
        ProfileId::Quality => Levers {
            perf_mode: 2,
            max_fps: 60,
            res_min: 1.0,
            res_max: 1.0,
            adaptive: false,
            pixel_ratio_scaling: true,
            light_animation: true,
            vision_animation: true,
            mipmap: true,
            video: VideoMode::Play,
            ui_blur: true,
            sequencer: true,
            fxmaster: true,
            unfocused_fps: 60,
            prime: PrimeLevel::Soft,
        },
        ProfileId::Balance => Levers {
            perf_mode: 1,
            max_fps: 60,
            res_min: 0.7,
            res_max: 1.0,
            adaptive: true,
            pixel_ratio_scaling: false,
            light_animation: true,
            vision_animation: false,
            mipmap: true,
            video: VideoMode::PauseUnfocused,
            ui_blur: false,
            sequencer: true,
            fxmaster: true,
            unfocused_fps: 15,
            prime: PrimeLevel::Medium,
        },
        ProfileId::Potato => Levers {
            perf_mode: 0,
            max_fps: 45,
            res_min: 0.55,
            res_max: 0.85,
            adaptive: true,
            pixel_ratio_scaling: false,
            light_animation: false,
            vision_animation: false,
            mipmap: false,
            video: VideoMode::Static,
            ui_blur: false,
            sequencer: false,
            fxmaster: false,
            unfocused_fps: 10,
            prime: PrimeLevel::Aggressive,
        },
    }
}

pub fn presets() -> BTreeMap<ProfileId, Levers> {
    [ProfileId::Quality, ProfileId::Balance, ProfileId::Potato]
        .into_iter()
        .map(|id| (id, preset(id)))
        .collect()
}

pub fn sanitize(mut l: Levers) -> Levers {
    l.perf_mode = l.perf_mode.min(3);
    l.max_fps = l.max_fps.clamp(10, 240);
    l.unfocused_fps = l.unfocused_fps.clamp(5, 240);
    l.res_min = l.res_min.clamp(0.25, 1.0);
    l.res_max = l.res_max.clamp(0.25, 1.0);
    if l.res_min > l.res_max {
        std::mem::swap(&mut l.res_min, &mut l.res_max);
    }
    l
}

pub fn apply(mut b: Levers, o: &Overrides) -> Levers {
    if let Some(v) = o.perf_mode { b.perf_mode = v; }
    if let Some(v) = o.max_fps { b.max_fps = v; }
    if let Some(v) = o.res_min { b.res_min = v; }
    if let Some(v) = o.res_max { b.res_max = v; }
    if let Some(v) = o.adaptive { b.adaptive = v; }
    if let Some(v) = o.pixel_ratio_scaling { b.pixel_ratio_scaling = v; }
    if let Some(v) = o.light_animation { b.light_animation = v; }
    if let Some(v) = o.vision_animation { b.vision_animation = v; }
    if let Some(v) = o.mipmap { b.mipmap = v; }
    if let Some(v) = o.video { b.video = v; }
    if let Some(v) = o.ui_blur { b.ui_blur = v; }
    if let Some(v) = o.sequencer { b.sequencer = v; }
    if let Some(v) = o.fxmaster { b.fxmaster = v; }
    if let Some(v) = o.unfocused_fps { b.unfocused_fps = v; }
    if let Some(v) = o.prime { b.prime = v; }
    sanitize(b)
}

pub fn active_id(settings: &Settings, server: Option<&Server>) -> ProfileId {
    server.and_then(|s| s.profile).unwrap_or(settings.profile)
}

/// Все три профиля с наложенными глобальными и серверными правками —
/// агенту нужны все, чтобы переключать профиль прямо в игре.
pub fn resolve_all(settings: &Settings, server: Option<&Server>) -> BTreeMap<ProfileId, Levers> {
    presets()
        .into_iter()
        .map(|(id, base)| {
            let mut l = apply(base, &settings.overrides);
            if let Some(s) = server {
                l = apply(l, &s.overrides);
            }
            (id, l)
        })
        .collect()
}
```

Добавить в начало `lib.rs`:
```rust
pub mod model;
pub mod profile;
```

- [ ] **Step 4: Тесты проходят**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS (все тесты `model` и `profile`).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src
git commit -m "feat(core): data model and performance profiles

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Флаги движка (Rust)

**Files:**
- Create: `src-tauri/src/engine_flags.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod engine_flags;`)

**Interfaces:**
- Consumes: `model::{EngineSettings, AngleBackend}`
- Produces: `engine_flags::build(engine: &EngineSettings) -> String`, `engine_flags::safe() -> String`

Примечание: при вызове `additional_browser_args` Tauri/wry перестаёт передавать свои умолчания
`--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`, поэтому мы добавляем их сами.

- [ ] **Step 1: Падающие тесты**

```rust
use crate::model::EngineSettings;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AngleBackend;

    #[test]
    fn default_flags_contain_performance_set() {
        let f = build(&EngineSettings::default());
        for flag in [
            "--use-angle=d3d11",
            "--enable-gpu-rasterization",
            "--enable-zero-copy",
            "--ignore-gpu-blocklist",
            "--force-high-performance-gpu",
            "--disable-renderer-backgrounding",
            "--disk-cache-size=2147483648",
            "--js-flags=--max-old-space-size=4096",
            "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
        ] {
            assert!(f.split(' ').any(|x| x == flag), "missing {flag} in {f}");
        }
    }

    #[test]
    fn angle_backend_is_configurable() {
        let e = EngineSettings { angle: AngleBackend::Vulkan, ..EngineSettings::default() };
        assert!(build(&e).contains("--use-angle=vulkan"));
    }

    #[test]
    fn extra_args_replace_same_key_and_append_new() {
        let e = EngineSettings { extra_args: "--use-angle=gl --foo --bad".into(), ..EngineSettings::default() };
        let f = build(&e);
        assert!(f.contains("--use-angle=gl"));
        assert!(!f.contains("--use-angle=d3d11"));
        // заменённый флаг переносится в конец, в порядке extra_args
        assert!(f.ends_with("--use-angle=gl --foo --bad"));
    }

    #[test]
    fn extra_args_ignore_non_flags() {
        let e = EngineSettings { extra_args: "rm -rf x".into(), ..EngineSettings::default() };
        let f = build(&e);
        assert!(!f.contains("rm"));
        assert!(!f.contains(" x"));
    }

    #[test]
    fn safe_mode_has_only_compat_flags() {
        let f = safe();
        assert!(!f.contains("--use-angle"));
        assert!(f.contains("--allow-insecure-localhost"));
    }
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml engine_flags`
Expected: FAIL — `build`/`safe` не найдены.

- [ ] **Step 2: Реализация** (над тестами)

```rust
const COMPAT: [&str; 3] = [
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
    "--allow-insecure-localhost",
    "--allow-running-insecure-content",
];

fn key(flag: &str) -> &str {
    flag.split('=').next().unwrap_or(flag)
}

pub fn build(engine: &EngineSettings) -> String {
    let mut flags: Vec<String> = vec![
        format!("--use-angle={}", engine.angle.flag_value()),
        "--enable-gpu-rasterization".into(),
        "--enable-zero-copy".into(),
        "--ignore-gpu-blocklist".into(),
        "--force-high-performance-gpu".into(),
        "--disable-renderer-backgrounding".into(),
        format!("--disk-cache-size={}", u64::from(engine.disk_cache_mb) * 1024 * 1024),
        "--js-flags=--max-old-space-size=4096".into(),
    ];
    flags.extend(COMPAT.iter().map(|s| s.to_string()));

    for extra in engine.extra_args.split_whitespace().filter(|a| a.starts_with("--")) {
        match flags.iter().position(|f| key(f) == key(extra)) {
            Some(i) => {
                flags.remove(i);
                flags.push(extra.to_string());
            }
            None => flags.push(extra.to_string()),
        }
    }
    flags.join(" ")
}

pub fn safe() -> String {
    COMPAT.join(" ")
}
```

Добавить `pub mod engine_flags;` в `lib.rs`.

- [ ] **Step 3: Тесты проходят**

Run: `cargo test --manifest-path src-tauri/Cargo.toml engine_flags`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src
git commit -m "feat(core): Chromium flag builder for the game engine

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Хранилище (Rust)

**Files:**
- Create: `src-tauri/src/store.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod store;`)

**Interfaces:**
- Consumes: `model::{Settings, Server, ServerStats, ProfileId, Notice, SCHEMA}`
- Produces:
  - `store::Store::new(dir: PathBuf) -> Store`, `Store::default_dir() -> PathBuf`, `store::engine_dir() -> PathBuf`
  - `Store::load_settings(&self, fallback: ProfileId) -> (Settings, Option<Notice>)`
  - `Store::save_settings(&self, s: &Settings) -> std::io::Result<()>`
  - `Store::load_servers(&self) -> (Vec<Server>, Option<Notice>)`, `Store::save_servers(&self, v: &[Server]) -> io::Result<()>`
  - `Store::load_stats(&self) -> (BTreeMap<String, ServerStats>, Option<Notice>)`, `Store::save_stats(&self, v: &BTreeMap<String, ServerStats>) -> io::Result<()>`
  - Notice-ключ при порче файла: `notice.storeCorrupted` с параметром `file`.

- [ ] **Step 1: Падающие тесты**

```rust
use crate::model::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> (tempfile::TempDir, Store) {
        let d = tempfile::tempdir().unwrap();
        let s = Store::new(d.path().to_path_buf());
        (d, s)
    }

    #[test]
    fn missing_settings_use_fallback_profile() {
        let (_d, s) = tmp();
        let (settings, notice) = s.load_settings(ProfileId::Potato);
        assert_eq!(settings.profile, ProfileId::Potato);
        assert!(notice.is_none());
    }

    #[test]
    fn settings_roundtrip() {
        let (_d, s) = tmp();
        let mut st = Settings::default();
        st.profile = ProfileId::Quality;
        st.engine.extra_args = "--foo".into();
        s.save_settings(&st).unwrap();
        assert_eq!(s.load_settings(ProfileId::Potato).0, st);
    }

    #[test]
    fn corrupted_file_is_backed_up_and_reported() {
        let (d, s) = tmp();
        fs::write(d.path().join("servers.json"), "{not json").unwrap();
        let (servers, notice) = s.load_servers();
        assert!(servers.is_empty());
        let n = notice.unwrap();
        assert_eq!(n.key, "notice.storeCorrupted");
        assert_eq!(n.params["file"], "servers.json");
        assert!(d.path().join("servers.json.bak").exists());
    }

    #[test]
    fn servers_and_stats_roundtrip() {
        let (_d, s) = tmp();
        let srv = vec![Server { id: "a".into(), name: "A".into(), url: "https://a".into(), profile: None, overrides: Overrides::default() }];
        s.save_servers(&srv).unwrap();
        assert_eq!(s.load_servers().0, srv);

        let mut stats = BTreeMap::new();
        stats.insert("a".to_string(), ServerStats::default());
        s.save_stats(&stats).unwrap();
        assert_eq!(s.load_stats().0, stats);
    }

    #[test]
    fn save_creates_directory() {
        let d = tempfile::tempdir().unwrap();
        let s = Store::new(d.path().join("nested"));
        s.save_settings(&Settings::default()).unwrap();
        assert!(d.path().join("nested/settings.json").exists());
    }
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml store`
Expected: FAIL — `Store` не найден.

- [ ] **Step 2: Реализация** (над тестами)

```rust
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct ServersFile {
    schema: u32,
    servers: Vec<Server>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct StatsFile {
    schema: u32,
    servers: BTreeMap<String, ServerStats>,
}

pub struct Store {
    dir: PathBuf,
}

pub fn engine_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("FoundryPerformance").join("engine")
}

impl Store {
    pub fn new(dir: PathBuf) -> Store {
        Store { dir }
    }

    pub fn default_dir() -> PathBuf {
        let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        base.join("FoundryPerformance")
    }

    /// `Ok(None)` — файла нет; `Err(notice)` — файл повреждён (переименован в .bak).
    fn read<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, Notice> {
        let path = self.dir.join(name);
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(Notice::new("notice.storeCorrupted").with("file", name)),
        };
        match serde_json::from_str(&text) {
            Ok(v) => Ok(Some(v)),
            Err(_) => {
                let _ = fs::rename(&path, self.dir.join(format!("{name}.bak")));
                Err(Notice::new("notice.storeCorrupted").with("file", name))
            }
        }
    }

    fn write<T: Serialize>(&self, name: &str, value: &T) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let tmp = self.dir.join(format!("{name}.tmp"));
        let json = serde_json::to_string_pretty(value).map_err(io::Error::other)?;
        fs::write(&tmp, json)?;
        fs::rename(&tmp, self.dir.join(name))
    }

    pub fn load_settings(&self, fallback: ProfileId) -> (Settings, Option<Notice>) {
        let fresh = Settings { profile: fallback, ..Settings::default() };
        match self.read::<Settings>("settings.json") {
            Ok(Some(mut s)) => {
                s.schema = SCHEMA;
                (s, None)
            }
            Ok(None) => (fresh, None),
            Err(n) => (fresh, Some(n)),
        }
    }

    pub fn save_settings(&self, s: &Settings) -> io::Result<()> {
        self.write("settings.json", s)
    }

    pub fn load_servers(&self) -> (Vec<Server>, Option<Notice>) {
        match self.read::<ServersFile>("servers.json") {
            Ok(Some(f)) => (f.servers, None),
            Ok(None) => (Vec::new(), None),
            Err(n) => (Vec::new(), Some(n)),
        }
    }

    pub fn save_servers(&self, servers: &[Server]) -> io::Result<()> {
        self.write("servers.json", &ServersFile { schema: SCHEMA, servers: servers.to_vec() })
    }

    pub fn load_stats(&self) -> (BTreeMap<String, ServerStats>, Option<Notice>) {
        match self.read::<StatsFile>("stats.json") {
            Ok(Some(f)) => (f.servers, None),
            Ok(None) => (BTreeMap::new(), None),
            Err(n) => (BTreeMap::new(), Some(n)),
        }
    }

    pub fn save_stats(&self, stats: &BTreeMap<String, ServerStats>) -> io::Result<()> {
        self.write("stats.json", &StatsFile { schema: SCHEMA, servers: stats.clone() })
    }
}
```

Добавить `pub mod store;` в `lib.rs`.

- [ ] **Step 3: Тесты проходят**

Run: `cargo test --manifest-path src-tauri/Cargo.toml store`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src
git commit -m "feat(core): JSON store with corruption backup

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: GPU, язык системы, проверка сервера (Rust)

**Files:**
- Create: `src-tauri/src/gpu.rs`, `src-tauri/src/locale.rs`, `src-tauri/src/probe.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod gpu; pub mod locale; pub mod probe;`)

**Interfaces:**
- Consumes: `model::{ProfileId, Locale}`
- Produces:
  - `gpu::GpuInfo { name: String, vram_mb: u64, vendor_id: u32 }` (Serialize camelCase)
  - `gpu::detect() -> Option<GpuInfo>`, `gpu::recommend(g: Option<&GpuInfo>) -> ProfileId`
  - `locale::effective(l: Locale) -> &'static str` (`"ru"` | `"en"`)
  - `probe::normalize_url(input: &str) -> Result<url::Url, String>` (ошибка — i18n-ключ `err.urlInvalid`)
  - `probe::ProbeResult { reachable, foundry, active: bool, version, world, system: Option<String>, users: Option<u32> }` (Serialize camelCase)
  - `probe::parse_status(body: &str) -> Option<ProbeResult>`
  - `async probe::probe(input: &str) -> ProbeResult`

- [ ] **Step 1: Падающие тесты `gpu.rs`**

```rust
use crate::model::ProfileId;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
    pub vendor_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(vram_mb: u64, vendor_id: u32) -> GpuInfo {
        GpuInfo { name: "x".into(), vram_mb, vendor_id }
    }

    #[test]
    fn recommendation_by_vram_and_vendor() {
        assert_eq!(recommend(Some(&g(12 * 1024, 0x10DE))), ProfileId::Quality);
        assert_eq!(recommend(Some(&g(6 * 1024, 0x10DE))), ProfileId::Balance);
        assert_eq!(recommend(Some(&g(3 * 1024, 0x10DE))), ProfileId::Potato);
        assert_eq!(recommend(Some(&g(8 * 1024, 0x8086))), ProfileId::Potato);
        assert_eq!(recommend(None), ProfileId::Balance);
    }
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml gpu`
Expected: FAIL — `recommend` не найден.

- [ ] **Step 2: Реализация `gpu.rs`** (под структурой, над тестами)

```rust
const VENDOR_INTEL: u32 = 0x8086;
const VENDOR_MICROSOFT_BASIC: u32 = 0x1414;

pub fn recommend(gpu: Option<&GpuInfo>) -> ProfileId {
    match gpu {
        None => ProfileId::Balance,
        Some(g) if g.vendor_id == VENDOR_INTEL => ProfileId::Potato,
        Some(g) if g.vram_mb >= 8 * 1024 => ProfileId::Quality,
        Some(g) if g.vram_mb >= 4 * 1024 => ProfileId::Balance,
        Some(_) => ProfileId::Potato,
    }
}

/// Адаптер с наибольшей выделенной VRAM, без программных.
pub fn detect() -> Option<GpuInfo> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE};

    unsafe {
        let factory: IDXGIFactory1 = CreateDXGIFactory1().ok()?;
        let mut best: Option<GpuInfo> = None;
        let mut i = 0u32;
        while let Ok(adapter) = factory.EnumAdapters1(i) {
            i += 1;
            let Ok(desc) = adapter.GetDesc1() else { continue };
            if desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 || desc.VendorId == VENDOR_MICROSOFT_BASIC {
                continue;
            }
            let len = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
            let info = GpuInfo {
                name: String::from_utf16_lossy(&desc.Description[..len]),
                vram_mb: (desc.DedicatedVideoMemory as u64) / (1024 * 1024),
                vendor_id: desc.VendorId,
            };
            if best.as_ref().map_or(true, |b| info.vram_mb > b.vram_mb) {
                best = Some(info);
            }
        }
        best
    }
}
```

- [ ] **Step 3: `locale.rs`** (без отдельного теста — тонкая обёртка над WinAPI; `effective` для явных значений покрыт тестом)

```rust
use crate::model::Locale;

const LANG_RUSSIAN: u16 = 0x19;

pub fn effective(l: Locale) -> &'static str {
    match l {
        Locale::Ru => "ru",
        Locale::En => "en",
        Locale::Auto => {
            let lang = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
            if lang & 0x3ff == LANG_RUSSIAN { "ru" } else { "en" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_locales() {
        assert_eq!(effective(Locale::Ru), "ru");
        assert_eq!(effective(Locale::En), "en");
        assert!(["ru", "en"].contains(&effective(Locale::Auto)));
    }
}
```

- [ ] **Step 4: Падающие тесты `probe.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub reachable: bool,
    pub foundry: bool,
    pub active: bool,
    pub version: Option<String>,
    pub world: Option<String>,
    pub system: Option<String>,
    pub users: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_scheme_and_strips_foundry_routes() {
        assert_eq!(normalize_url("vtt.example.com").unwrap().as_str(), "https://vtt.example.com/");
        assert_eq!(normalize_url("  http://192.168.1.40:30000/game ").unwrap().as_str(), "http://192.168.1.40:30000/");
        assert_eq!(normalize_url("https://host/foundry/join").unwrap().as_str(), "https://host/foundry/");
        assert_eq!(normalize_url("localhost:30000").unwrap().as_str(), "http://localhost:30000/");
    }

    #[test]
    fn normalize_rejects_garbage() {
        assert_eq!(normalize_url("").unwrap_err(), "err.urlInvalid");
        assert_eq!(normalize_url("ftp://x").unwrap_err(), "err.urlInvalid");
        assert_eq!(normalize_url("not a url at all").unwrap_err(), "err.urlInvalid");
    }

    #[test]
    fn parse_active_world() {
        let r = parse_status(r#"{"active":true,"version":"14.349","world":"strahd","system":"dnd5e","users":3}"#).unwrap();
        assert!(r.reachable && r.foundry && r.active);
        assert_eq!(r.version.as_deref(), Some("14.349"));
        assert_eq!(r.world.as_deref(), Some("strahd"));
        assert_eq!(r.users, Some(3));
    }

    #[test]
    fn parse_no_world_and_non_foundry() {
        let r = parse_status(r#"{"active":false,"version":"14.349"}"#).unwrap();
        assert!(r.foundry && !r.active);
        assert!(parse_status(r#"{"hello":"world"}"#).is_none());
        assert!(parse_status("<html>").is_none());
    }
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml probe`
Expected: FAIL — функции не найдены.

- [ ] **Step 5: Реализация `probe.rs`** (над тестами)

```rust
const FOUNDRY_ROUTES: [&str; 5] = ["game", "join", "setup", "auth", "players"];

pub fn normalize_url(input: &str) -> Result<Url, String> {
    let s = input.trim();
    if s.is_empty() || s.contains(char::is_whitespace) {
        return Err("err.urlInvalid".into());
    }
    let with_scheme = if s.contains("://") {
        s.to_string()
    } else if s.starts_with("localhost") || s.starts_with("127.") || s.starts_with("192.168.") || s.starts_with("10.") {
        format!("http://{s}")
    } else {
        format!("https://{s}")
    };
    let mut url = Url::parse(&with_scheme).map_err(|_| "err.urlInvalid".to_string())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("err.urlInvalid".into());
    }
    let mut segs: Vec<String> = url.path_segments().map(|p| p.filter(|x| !x.is_empty()).map(String::from).collect()).unwrap_or_default();
    if segs.last().is_some_and(|l| FOUNDRY_ROUTES.contains(&l.as_str())) {
        segs.pop();
    }
    let path = if segs.is_empty() { "/".to_string() } else { format!("/{}/", segs.join("/")) };
    url.set_path(&path);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

#[derive(Deserialize)]
struct RawStatus {
    active: Option<bool>,
    version: Option<String>,
    world: Option<String>,
    system: Option<String>,
    users: Option<u32>,
}

pub fn parse_status(body: &str) -> Option<ProbeResult> {
    let raw: RawStatus = serde_json::from_str(body).ok()?;
    let active = raw.active?;
    raw.version.as_ref()?;
    Some(ProbeResult { reachable: true, foundry: true, active, version: raw.version, world: raw.world, system: raw.system, users: raw.users })
}

pub async fn probe(input: &str) -> ProbeResult {
    let Ok(base) = normalize_url(input) else { return ProbeResult::default() };
    let Ok(status_url) = base.join("api/status") else { return ProbeResult::default() };
    let client = match reqwest::Client::builder().timeout(Duration::from_secs(3)).build() {
        Ok(c) => c,
        Err(_) => return ProbeResult::default(),
    };
    match client.get(status_url).send().await {
        Err(_) => ProbeResult::default(),
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            parse_status(&body).unwrap_or(ProbeResult { reachable: true, ..ProbeResult::default() })
        }
    }
}
```

Добавить `pub mod gpu; pub mod locale; pub mod probe;` в `lib.rs`.

- [ ] **Step 6: Тесты проходят + ручная проверка DXGI**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS.

Добавить временный тест в `gpu.rs`, запустить и удалить:
```rust
#[test]
#[ignore]
fn print_detected_gpu() {
    println!("{:?}", detect());
}
```
Run: `cargo test --manifest-path src-tauri/Cargo.toml print_detected_gpu -- --ignored --nocapture`
Expected: `Some(GpuInfo { name: "NVIDIA GeForce RTX 5070", vram_mb: ~12000, vendor_id: 4318 })`. Тест оставить (он `#[ignore]` и полезен для диагностики).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src
git commit -m "feat(core): GPU detection, system locale, Foundry status probe

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Ядро агента — типы, счётчик кадров, адаптивное разрешение

**Files:**
- Create: `agent/src/types.ts`, `agent/src/foundry.ts`, `agent/src/fps-meter.ts`, `agent/src/fps-meter.test.ts`, `agent/src/adaptive.ts`, `agent/src/adaptive.test.ts`

**Interfaces:**
- Produces:
  - `types.ts`: `ProfileId`, `VideoMode`, `PrimeLevel`, `Levers`, `Boot`
  - `foundry.ts`: `g: Record<string, any>` (globalThis)
  - `fps-meter.ts`: `class FrameStats { constructor(capacity: number); push(ms: number): void; recent(count: number): number[]; avgMs(count: number): number; get size(): number; clear(): void }`, `low1(samplesMs: number[]): number`, `summarize(samplesMs: number[]): { avg: number; low1: number; min: number }`
  - `adaptive.ts`: `AdaptiveConfig`, `AdaptiveState`, `configFor(l: Levers): AdaptiveConfig`, `initAdaptive(c: AdaptiveConfig): AdaptiveState`, `stepAdaptive(s: AdaptiveState, c: AdaptiveConfig, avgMs: number, now: number): AdaptiveState`, `resetTimers(s: AdaptiveState): AdaptiveState`

- [ ] **Step 1: Типы и доступ к глобалам**

`agent/src/types.ts`:
```ts
export type ProfileId = "quality" | "balance" | "potato";
export type VideoMode = "play" | "pauseUnfocused" | "static";
export type PrimeLevel = "soft" | "medium" | "aggressive";

/** Зеркало Rust `model::Levers` (camelCase). */
export interface Levers {
	perfMode: 0 | 1 | 2 | 3;
	maxFps: number;
	resMin: number;
	resMax: number;
	adaptive: boolean;
	pixelRatioScaling: boolean;
	lightAnimation: boolean;
	visionAnimation: boolean;
	mipmap: boolean;
	video: VideoMode;
	uiBlur: boolean;
	sequencer: boolean;
	fxmaster: boolean;
	unfocusedFps: number;
	prime: PrimeLevel;
}

/** Зеркало Rust `agent::Boot`; кладётся в `window.__FP_BOOT__`. */
export interface Boot {
	serverId: string;
	profileId: ProfileId;
	presets: Record<ProfileId, Levers>;
	locale: "ru" | "en";
	/** Базовый замер: рычаги не применяются, core-настройки сбрасываются к умолчаниям Foundry. */
	measureOnly: boolean;
}
```

`agent/src/foundry.ts`:
```ts
/** Глобалы Foundry (game, canvas, Hooks, PIXI) нетипизированы и появляются не сразу. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const g = globalThis as unknown as Record<string, any>;
```

- [ ] **Step 2: Падающие тесты счётчика кадров**

`agent/src/fps-meter.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import { FrameStats, low1, summarize } from "./fps-meter";

describe("FrameStats", () => {
	it("keeps the most recent samples in order", () => {
		const s = new FrameStats(3);
		[10, 20, 30, 40].forEach((x) => s.push(x));
		expect(s.recent(3)).toEqual([20, 30, 40]);
		expect(s.recent(2)).toEqual([30, 40]);
		expect(s.size).toBe(3);
	});

	it("averages the last N samples", () => {
		const s = new FrameStats(10);
		[10, 20, 30].forEach((x) => s.push(x));
		expect(s.avgMs(2)).toBe(25);
		expect(s.avgMs(100)).toBe(20);
	});

	it("returns 0 average when empty", () => {
		expect(new FrameStats(4).avgMs(4)).toBe(0);
	});
});

describe("low1 / summarize", () => {
	it("computes 1% low from the slowest frames", () => {
		const samples = Array.from({ length: 100 }, () => 10);
		samples[0] = 50;
		expect(low1(samples)).toBe(20);
	});

	it("summarizes fps from frame times", () => {
		const r = summarize([10, 10, 20, 40]);
		expect(r.avg).toBeCloseTo(1000 / 20, 5);
		expect(r.min).toBeCloseTo(25, 5);
		expect(r.low1).toBeCloseTo(25, 5);
	});

	it("handles empty input", () => {
		expect(summarize([])).toEqual({ avg: 0, low1: 0, min: 0 });
	});
});
```

Run: `npx vitest run agent/src/fps-meter.test.ts`
Expected: FAIL — модуль `./fps-meter` не найден.

- [ ] **Step 3: Реализация `agent/src/fps-meter.ts`**

```ts
export class FrameStats {
	private buf: Float64Array;
	private next = 0;
	private count = 0;

	constructor(capacity: number) {
		this.buf = new Float64Array(capacity);
	}

	push(ms: number): void {
		this.buf[this.next] = ms;
		this.next = (this.next + 1) % this.buf.length;
		if (this.count < this.buf.length) this.count++;
	}

	get size(): number {
		return this.count;
	}

	recent(count: number): number[] {
		const n = Math.min(count, this.count);
		const out = new Array<number>(n);
		for (let i = 0; i < n; i++) {
			const idx = (this.next - n + i + this.buf.length) % this.buf.length;
			out[i] = this.buf[idx];
		}
		return out;
	}

	avgMs(count: number): number {
		const r = this.recent(count);
		if (r.length === 0) return 0;
		let sum = 0;
		for (const x of r) sum += x;
		return sum / r.length;
	}

	clear(): void {
		this.next = 0;
		this.count = 0;
	}
}

/** FPS по самым медленным 1% кадров. */
export function low1(samplesMs: number[]): number {
	if (samplesMs.length === 0) return 0;
	const sorted = [...samplesMs].sort((a, b) => b - a);
	const k = Math.max(1, Math.floor(sorted.length * 0.01));
	let sum = 0;
	for (let i = 0; i < k; i++) sum += sorted[i];
	return 1000 / (sum / k);
}

export function summarize(samplesMs: number[]): { avg: number; low1: number; min: number } {
	if (samplesMs.length === 0) return { avg: 0, low1: 0, min: 0 };
	let sum = 0;
	let worst = 0;
	for (const x of samplesMs) {
		sum += x;
		if (x > worst) worst = x;
	}
	return { avg: 1000 / (sum / samplesMs.length), low1: low1(samplesMs), min: 1000 / worst };
}
```

Run: `npx vitest run agent/src/fps-meter.test.ts`
Expected: PASS.

- [ ] **Step 4: Падающие тесты адаптивного контроллера**

`agent/src/adaptive.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import { type AdaptiveConfig, initAdaptive, resetTimers, stepAdaptive } from "./adaptive";

const cfg: AdaptiveConfig = {
	min: 0.7,
	max: 1,
	step: 0.05,
	targetMs: 1000 / 60,
	overRatio: 1.15,
	stableRatio: 1.05,
	downAfterMs: 2000,
	upAfterMs: 5000,
	cooldownMs: 1000,
	probeBackoffMs: 30000
};
const SLOW = 30;
const OK = 16.7;

function run(state = initAdaptive(cfg), from: number, to: number, ms: number) {
	let s = state;
	for (let t = from; t <= to; t += 250) s = stepAdaptive(s, cfg, ms, t);
	return s;
}

describe("adaptive resolution", () => {
	it("starts at max", () => {
		expect(initAdaptive(cfg).scale).toBe(1);
	});

	it("ignores short spikes", () => {
		expect(run(undefined, 0, 1500, SLOW).scale).toBe(1);
	});

	it("steps down after sustained slow frames", () => {
		expect(run(undefined, 0, 2000, SLOW).scale).toBe(0.95);
	});

	it("never goes below min", () => {
		expect(run(undefined, 0, 60000, SLOW).scale).toBe(0.7);
	});

	it("steps up after stable frames and never above max", () => {
		const low = run(undefined, 0, 60000, SLOW);
		const s = run(low, 60250, 65250, OK);
		expect(s.scale).toBe(0.75);
		expect(run(low, 60250, 400000, OK).scale).toBe(1);
	});

	it("blocks a scale that failed right after a probe", () => {
		let s = run(undefined, 0, 2000, SLOW); // 0.95 at t=2000
		s = run(s, 2250, 7250, OK); // probe up to 1.0 at t=7250
		expect(s.scale).toBe(1);
		s = run(s, 7500, 9500, SLOW); // fails, back to 0.95
		expect(s.scale).toBe(0.95);
		s = run(s, 9750, 30000, OK); // stable but 1.0 is blocked
		expect(s.scale).toBe(0.95);
		s = run(s, 30250, 45000, OK); // backoff expired
		expect(s.scale).toBe(1);
	});

	it("resetTimers clears pending windows", () => {
		const s = resetTimers(run(undefined, 0, 1500, SLOW));
		expect(s.overSince).toBeNull();
		expect(s.stableSince).toBeNull();
	});
});
```

Run: `npx vitest run agent/src/adaptive.test.ts`
Expected: FAIL — модуль `./adaptive` не найден.

- [ ] **Step 5: Реализация `agent/src/adaptive.ts`**

```ts
import type { Levers } from "./types";

export interface AdaptiveConfig {
	min: number;
	max: number;
	step: number;
	targetMs: number;
	/** Кадр «медленный», если avg > targetMs × overRatio. */
	overRatio: number;
	/** Кадр «стабильный», если avg ≤ targetMs × stableRatio. */
	stableRatio: number;
	downAfterMs: number;
	upAfterMs: number;
	cooldownMs: number;
	/** На сколько блокируется масштаб, который не выдержал пробного повышения. */
	probeBackoffMs: number;
}

export interface AdaptiveState {
	scale: number;
	overSince: number | null;
	stableSince: number | null;
	lastChange: number;
	lastUpAt: number;
	blockedScale: number | null;
	blockedUntil: number;
}

export function configFor(l: Levers): AdaptiveConfig {
	return {
		min: l.resMin,
		max: l.resMax,
		step: 0.05,
		targetMs: 1000 / l.maxFps,
		overRatio: 1.15,
		stableRatio: 1.05,
		downAfterMs: 2000,
		upAfterMs: 5000,
		cooldownMs: 1000,
		probeBackoffMs: 30000
	};
}

export function initAdaptive(c: AdaptiveConfig): AdaptiveState {
	return {
		scale: c.max,
		overSince: null,
		stableSince: null,
		lastChange: -Infinity,
		lastUpAt: -Infinity,
		blockedScale: null,
		blockedUntil: 0
	};
}

export function resetTimers(s: AdaptiveState): AdaptiveState {
	return { ...s, overSince: null, stableSince: null };
}

const round2 = (x: number) => Math.round(x * 100) / 100;

/**
 * Когда FPS упирается в лимит, время кадра не может стать меньше цели,
 * поэтому повышение идёт «пробой»: после 5 с стабильности пробуем шаг вверх;
 * если он сразу проваливается — этот масштаб блокируется на probeBackoffMs.
 */
export function stepAdaptive(s: AdaptiveState, c: AdaptiveConfig, avgMs: number, now: number): AdaptiveState {
	const over = avgMs > c.targetMs * c.overRatio;
	const stable = avgMs <= c.targetMs * c.stableRatio;
	const cooled = now - s.lastChange >= c.cooldownMs;

	if (over) {
		const overSince = s.overSince ?? now;
		if (now - overSince >= c.downAfterMs && cooled && s.scale > c.min) {
			const failedProbe = now - s.lastUpAt <= c.downAfterMs * 2;
			return {
				...s,
				scale: round2(Math.max(c.min, s.scale - c.step)),
				overSince: null,
				stableSince: null,
				lastChange: now,
				blockedScale: failedProbe ? s.scale : s.blockedScale,
				blockedUntil: failedProbe ? now + c.probeBackoffMs : s.blockedUntil
			};
		}
		return { ...s, overSince, stableSince: null };
	}

	if (stable) {
		const stableSince = s.stableSince ?? now;
		const next = round2(Math.min(c.max, s.scale + c.step));
		const blocked = s.blockedScale !== null && next >= s.blockedScale && now < s.blockedUntil;
		if (now - stableSince >= c.upAfterMs && cooled && s.scale < c.max && !blocked) {
			return { ...s, scale: next, overSince: null, stableSince: null, lastChange: now, lastUpAt: now };
		}
		return { ...s, stableSince, overSince: null };
	}

	return resetTimers(s);
}
```

- [ ] **Step 6: Тесты проходят**

Run: `npx vitest run agent/src`
Expected: PASS (fps-meter + adaptive).

- [ ] **Step 7: Commit**

```bash
git add agent/src
git commit -m "feat(agent): frame stats and adaptive resolution controller

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Агент — клиентские настройки Foundry и модулей

**Files:**
- Create: `agent/src/client-settings.ts`, `agent/src/client-settings.test.ts`, `agent/src/modules.ts`, `agent/src/modules.test.ts`

**Interfaces:**
- Consumes: `types.ts` (`Levers`, `PrimeLevel`)
- Produces:
  - `CORE_KEYS` (объект ключей), `coreValues(l: Levers): Record<string, unknown>`
  - `writePreboot(storage: { setItem(k: string, v: string): void }, values: Record<string, unknown>): void`
  - `interface SettingsHost`, `interface ReconcileResult { applied: string[]; missing: string[]; failed: string[] }`
  - `reconcile(game: SettingsHost, values: Record<string, unknown>): Promise<ReconcileResult>`
  - `defaultValues(game: SettingsHost, keys: string[]): Record<string, unknown>` — умолчания Foundry для зарегистрированных ключей
  - `MODULE_SETTINGS`, `PRIME_MODULE`, `PRIME_SETTINGS: Record<PrimeLevel, Record<string, unknown>>`
  - `moduleValues(isActive: (id: string) => boolean, l: Levers): Record<string, unknown>`

Ключи ниже — рабочие гипотезы (спецификация §4.2). Task 11 сверяет их с дампом реального сервера.
`reconcile` применяет только зарегистрированные ключи, поэтому ошибка в имени не ломает загрузку.

- [ ] **Step 1: Падающие тесты**

`agent/src/client-settings.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import { CORE_KEYS, coreValues, defaultValues, reconcile, type SettingsHost, writePreboot } from "./client-settings";
import type { Levers } from "./types";

const levers: Levers = {
	perfMode: 1,
	maxFps: 60,
	resMin: 0.7,
	resMax: 1,
	adaptive: true,
	pixelRatioScaling: false,
	lightAnimation: true,
	visionAnimation: false,
	mipmap: true,
	video: "pauseUnfocused",
	uiBlur: false,
	sequencer: true,
	fxmaster: true,
	unfocusedFps: 15,
	prime: "medium"
};

function fakeGame(registered: Record<string, unknown>, failing: string[] = []) {
	const values = new Map(Object.entries(registered));
	const sets: string[] = [];
	const game: SettingsHost = {
		settings: {
			settings: new Map([...values.keys()].map((k) => [k, { default: `default-of-${k}` }])),
			get: (ns, key) => values.get(`${ns}.${key}`),
			set: async (ns, key, v) => {
				const full = `${ns}.${key}`;
				if (failing.includes(full)) throw new Error("invalid");
				sets.push(full);
				values.set(full, v);
				return v;
			}
		}
	};
	return { game, sets, values };
}

describe("coreValues", () => {
	it("maps levers onto core setting keys", () => {
		const v = coreValues(levers);
		expect(v[CORE_KEYS.perfMode]).toBe(1);
		expect(v[CORE_KEYS.maxFps]).toBe(60);
		expect(v[CORE_KEYS.visionAnimation]).toBe(false);
		expect(Object.keys(v)).toHaveLength(6);
	});
});

describe("writePreboot", () => {
	it("stores JSON-encoded values", () => {
		const store = new Map<string, string>();
		writePreboot({ setItem: (k, v) => store.set(k, v) }, { "core.maxFPS": 45, "core.mipmap": false });
		expect(store.get("core.maxFPS")).toBe("45");
		expect(store.get("core.mipmap")).toBe("false");
	});
});

describe("reconcile", () => {
	it("sets only registered keys that differ", async () => {
		const { game, sets } = fakeGame({ "core.maxFPS": 60, "core.mipmap": true });
		const r = await reconcile(game, { "core.maxFPS": 60, "core.mipmap": false, "core.unknown": 1 });
		expect(sets).toEqual(["core.mipmap"]);
		expect(r.applied.sort()).toEqual(["core.maxFPS", "core.mipmap"]);
		expect(r.missing).toEqual(["core.unknown"]);
		expect(r.failed).toEqual([]);
	});

	it("isolates failures per key", async () => {
		const { game } = fakeGame({ "core.maxFPS": 60, "core.mipmap": true }, ["core.maxFPS"]);
		const r = await reconcile(game, { "core.maxFPS": 240, "core.mipmap": false });
		expect(r.failed).toEqual(["core.maxFPS"]);
		expect(r.applied).toEqual(["core.mipmap"]);
	});
});

describe("defaultValues", () => {
	it("returns Foundry defaults for registered keys only", () => {
		const { game } = fakeGame({ "core.maxFPS": 30 });
		expect(defaultValues(game, ["core.maxFPS", "core.nope"])).toEqual({ "core.maxFPS": "default-of-core.maxFPS" });
	});
});
```

`agent/src/modules.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import { moduleValues } from "./modules";
import type { Levers } from "./types";

const base = { sequencer: false, fxmaster: false, prime: "aggressive" } as Levers;

describe("moduleValues", () => {
	it("includes settings only for active modules", () => {
		const v = moduleValues((id) => id === "sequencer", base);
		expect(v).toEqual({ "sequencer.effectsEnabled": false });
	});

	it("inverts fxmaster disable flag", () => {
		const v = moduleValues((id) => id === "fxmaster", { ...base, fxmaster: true });
		expect(v["fxmaster.disableAll"]).toBe(false);
	});

	it("returns nothing when no modules are active", () => {
		expect(moduleValues(() => false, base)).toEqual({});
	});
});
```

Run: `npx vitest run agent/src/client-settings.test.ts agent/src/modules.test.ts`
Expected: FAIL — модули не найдены.

- [ ] **Step 2: Реализация `agent/src/client-settings.ts`**

```ts
import type { Levers } from "./types";

export const CORE_KEYS = {
	perfMode: "core.performanceMode",
	maxFps: "core.maxFPS",
	pixelRatioScaling: "core.pixelRatioResolutionScaling",
	lightAnimation: "core.lightAnimation",
	visionAnimation: "core.visionAnimation",
	mipmap: "core.mipmap"
} as const;

export function coreValues(l: Levers): Record<string, unknown> {
	return {
		[CORE_KEYS.perfMode]: l.perfMode,
		[CORE_KEYS.maxFps]: l.maxFps,
		[CORE_KEYS.pixelRatioScaling]: l.pixelRatioScaling,
		[CORE_KEYS.lightAnimation]: l.lightAnimation,
		[CORE_KEYS.visionAnimation]: l.visionAnimation,
		[CORE_KEYS.mipmap]: l.mipmap
	};
}

/** Foundry хранит client-настройки в localStorage как JSON под ключом `namespace.key`. */
export function writePreboot(storage: { setItem(k: string, v: string): void }, values: Record<string, unknown>): void {
	for (const [k, v] of Object.entries(values)) storage.setItem(k, JSON.stringify(v));
}

export interface SettingsHost {
	settings: {
		settings: Map<string, { default?: unknown }>;
		get(ns: string, key: string): unknown;
		set(ns: string, key: string, value: unknown): Promise<unknown>;
	};
}

export interface ReconcileResult {
	applied: string[];
	missing: string[];
	failed: string[];
}

export async function reconcile(game: SettingsHost, values: Record<string, unknown>): Promise<ReconcileResult> {
	const result: ReconcileResult = { applied: [], missing: [], failed: [] };
	for (const [full, value] of Object.entries(values)) {
		if (!game.settings.settings.has(full)) {
			result.missing.push(full);
			continue;
		}
		const dot = full.indexOf(".");
		const ns = full.slice(0, dot);
		const key = full.slice(dot + 1);
		try {
			if (game.settings.get(ns, key) !== value) await game.settings.set(ns, key, value);
			result.applied.push(full);
		} catch {
			result.failed.push(full);
		}
	}
	return result;
}

export function defaultValues(game: SettingsHost, keys: string[]): Record<string, unknown> {
	const out: Record<string, unknown> = {};
	for (const k of keys) {
		const cfg = game.settings.settings.get(k);
		if (cfg) out[k] = cfg.default;
	}
	return out;
}
```

- [ ] **Step 3: Реализация `agent/src/modules.ts`**

```ts
import type { Levers, PrimeLevel } from "./types";

export interface ModuleSetting {
	module: string;
	key: string;
	value: (l: Levers) => unknown;
}

export const MODULE_SETTINGS: ModuleSetting[] = [
	{ module: "sequencer", key: "sequencer.effectsEnabled", value: (l) => l.sequencer },
	{ module: "fxmaster", key: "fxmaster.disableAll", value: (l) => !l.fxmaster }
];

export const PRIME_MODULE = "fvtt-perf-optim";

/** Значения клиентских настроек Prime Performance по уровню. Заполняется в Task 11 по дампу. */
export const PRIME_SETTINGS: Record<PrimeLevel, Record<string, unknown>> = {
	soft: {},
	medium: {},
	aggressive: {}
};

export function moduleValues(isActive: (id: string) => boolean, l: Levers): Record<string, unknown> {
	const out: Record<string, unknown> = {};
	for (const m of MODULE_SETTINGS) if (isActive(m.module)) out[m.key] = m.value(l);
	if (isActive(PRIME_MODULE)) Object.assign(out, PRIME_SETTINGS[l.prime]);
	return out;
}
```

- [ ] **Step 4: Тесты проходят**

Run: `npx vitest run agent/src`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add agent/src
git commit -m "feat(agent): Foundry client settings and module levers

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Агент — рычаги blur, видео, фокус, разрешение

**Files:**
- Create: `agent/src/levers/blur.ts`, `agent/src/levers/blur.test.ts`, `agent/src/levers/video.ts`, `agent/src/levers/video.test.ts`, `agent/src/levers/focus.ts`, `agent/src/levers/focus.test.ts`, `agent/src/levers/resolution.ts`, `agent/src/levers/resolution.test.ts`

**Interfaces:**
- Consumes: `types.ts` (`VideoMode`)
- Produces:
  - `setUiBlur(doc: Document, enabled: boolean): void`
  - `interface VideoLike { paused: boolean; pause(): void; play(): Promise<void> }`, `collectVideos(canvas: any): HTMLVideoElement[]`, `class VideoController { constructor(find: () => VideoLike[]); setMode(m: VideoMode): void; setFocused(f: boolean): void; sync(): void }`
  - `class FocusThrottle { constructor(getTicker: () => { maxFPS: number } | undefined, focusedFps: number, unfocusedFps: number); setFps(focused: number, unfocused: number): void; setFocused(f: boolean): void; get focused(): boolean; apply(): void }`
  - `class CanvasResolution { constructor(getRenderer: () => any, baseResolution: () => number, win: { dispatchEvent(e: Event): boolean }); apply(scale: number): boolean; get scale(): number; targetFor(scale: number): number }`

- [ ] **Step 1: Падающие тесты**

`agent/src/levers/blur.test.ts`:
```ts
// @vitest-environment happy-dom
import { describe, expect, it } from "vitest";
import { setUiBlur } from "./blur";

describe("setUiBlur", () => {
	it("injects and removes the no-blur stylesheet idempotently", () => {
		setUiBlur(document, false);
		setUiBlur(document, false);
		expect(document.querySelectorAll("#fp-no-blur")).toHaveLength(1);
		expect(document.getElementById("fp-no-blur")!.textContent).toContain("backdrop-filter:none");
		setUiBlur(document, true);
		expect(document.getElementById("fp-no-blur")).toBeNull();
	});
});
```

`agent/src/levers/video.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import { VideoController, type VideoLike } from "./video";

function vid(paused = false): VideoLike & { plays: number } {
	return {
		paused,
		plays: 0,
		pause() {
			this.paused = true;
		},
		async play() {
			this.paused = false;
			this.plays++;
		}
	};
}

describe("VideoController", () => {
	it("static mode pauses everything", () => {
		const v = [vid(), vid()];
		const c = new VideoController(() => v);
		c.setMode("static");
		expect(v.every((x) => x.paused)).toBe(true);
	});

	it("pauseUnfocused pauses on blur and resumes only what it paused", () => {
		const playing = vid();
		const userPaused = vid(true);
		const c = new VideoController(() => [playing, userPaused]);
		c.setMode("pauseUnfocused");
		expect(playing.paused).toBe(false);
		c.setFocused(false);
		expect(playing.paused).toBe(true);
		c.setFocused(true);
		expect(playing.paused).toBe(false);
		expect(userPaused.paused).toBe(true);
		expect(userPaused.plays).toBe(0);
	});

	it("play mode resumes videos paused by static mode", () => {
		const v = vid();
		const c = new VideoController(() => [v]);
		c.setMode("static");
		c.setMode("play");
		expect(v.paused).toBe(false);
	});
});
```

`agent/src/levers/focus.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import { FocusThrottle } from "./focus";

describe("FocusThrottle", () => {
	it("switches ticker maxFPS on focus changes", () => {
		const ticker = { maxFPS: 0 };
		const f = new FocusThrottle(() => ticker, 60, 15);
		f.apply();
		expect(ticker.maxFPS).toBe(60);
		f.setFocused(false);
		expect(ticker.maxFPS).toBe(15);
		f.setFps(45, 10);
		expect(ticker.maxFPS).toBe(10);
		f.setFocused(true);
		expect(ticker.maxFPS).toBe(45);
	});

	it("tolerates a missing ticker", () => {
		const f = new FocusThrottle(() => undefined, 60, 15);
		expect(() => f.setFocused(false)).not.toThrow();
	});
});
```

`agent/src/levers/resolution.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import { CanvasResolution } from "./resolution";

describe("CanvasResolution", () => {
	it("sets renderer resolution relative to base and triggers resize", () => {
		const renderer = { resolution: 1.5 };
		const events: string[] = [];
		const r = new CanvasResolution(() => renderer, () => 1.5, { dispatchEvent: (e) => (events.push(e.type), true) });
		expect(r.apply(0.8)).toBe(true);
		expect(renderer.resolution).toBe(1.2);
		expect(events).toEqual(["resize"]);
		expect(r.scale).toBe(0.8);
	});

	it("skips work when already at target and clamps to 0.25", () => {
		const renderer = { resolution: 1 };
		const events: string[] = [];
		const r = new CanvasResolution(() => renderer, () => 1, { dispatchEvent: (e) => (events.push(e.type), true) });
		r.apply(1);
		expect(events).toEqual([]);
		expect(r.targetFor(0.1)).toBe(0.25);
	});

	it("returns false without a renderer", () => {
		const r = new CanvasResolution(() => undefined, () => 1, { dispatchEvent: () => true });
		expect(r.apply(0.5)).toBe(false);
	});
});
```

Run: `npx vitest run agent/src/levers`
Expected: FAIL — модули не найдены.

- [ ] **Step 2: Реализация**

`agent/src/levers/blur.ts`:
```ts
const ID = "fp-no-blur";
const CSS = "*,*::before,*::after{backdrop-filter:none!important;-webkit-backdrop-filter:none!important}";

export function setUiBlur(doc: Document, enabled: boolean): void {
	const existing = doc.getElementById(ID);
	if (enabled) {
		existing?.remove();
		return;
	}
	if (existing) return;
	const style = doc.createElement("style");
	style.id = ID;
	style.textContent = CSS;
	(doc.head ?? doc.documentElement).appendChild(style);
}
```

`agent/src/levers/video.ts`:
```ts
import type { VideoMode } from "../types";

export interface VideoLike {
	paused: boolean;
	pause(): void;
	play(): Promise<void>;
}

/** Видео-текстуры сцены: фон, тайлы, токены (PIXI v7 и v8). */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function collectVideos(canvas: any): HTMLVideoElement[] {
	const out = new Set<HTMLVideoElement>();
	const add = (v: unknown) => {
		if (typeof HTMLVideoElement !== "undefined" && v instanceof HTMLVideoElement) out.add(v);
	};
	const meshes = canvas?.primary?.videoMeshes;
	if (meshes) {
		for (const m of meshes) {
			add(m?.sourceElement);
			add(m?.texture?.baseTexture?.resource?.source);
			add(m?.texture?.source?.resource);
		}
	}
	return [...out];
}

export class VideoController {
	private mode: VideoMode = "play";
	private focused = true;
	private pausedByUs = new Set<VideoLike>();

	constructor(private find: () => VideoLike[]) {}

	setMode(m: VideoMode): void {
		this.mode = m;
		this.sync();
	}

	setFocused(f: boolean): void {
		this.focused = f;
		this.sync();
	}

	sync(): void {
		const shouldPause = this.mode === "static" || (this.mode === "pauseUnfocused" && !this.focused);
		for (const v of this.find()) {
			if (shouldPause) {
				if (!v.paused) {
					v.pause();
					this.pausedByUs.add(v);
				}
			} else if (this.pausedByUs.has(v)) {
				this.pausedByUs.delete(v);
				v.play().catch(() => {});
			}
		}
	}
}
```

`agent/src/levers/focus.ts`:
```ts
export class FocusThrottle {
	private isFocused = true;

	constructor(
		private getTicker: () => { maxFPS: number } | undefined,
		private focusedFps: number,
		private unfocusedFps: number
	) {}

	get focused(): boolean {
		return this.isFocused;
	}

	setFps(focused: number, unfocused: number): void {
		this.focusedFps = focused;
		this.unfocusedFps = unfocused;
		this.apply();
	}

	setFocused(f: boolean): void {
		this.isFocused = f;
		this.apply();
	}

	apply(): void {
		const t = this.getTicker();
		if (t) t.maxFPS = this.isFocused ? this.focusedFps : this.unfocusedFps;
	}
}
```

`agent/src/levers/resolution.ts`:
```ts
/**
 * Меняет разрешение рендерера PIXI (только канвас; DOM-интерфейс Foundry не затрагивается)
 * и шлёт `resize`, чтобы Foundry пересчитал свои render-текстуры штатным путём.
 * Работоспособность на v14 проверяется в Task 10.
 */
export class CanvasResolution {
	private current = 1;

	constructor(
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		private getRenderer: () => any,
		private baseResolution: () => number,
		private win: { dispatchEvent(e: Event): boolean }
	) {}

	get scale(): number {
		return this.current;
	}

	targetFor(scale: number): number {
		return Math.max(0.25, Math.round(this.baseResolution() * scale * 100) / 100);
	}

	apply(scale: number): boolean {
		const r = this.getRenderer();
		if (!r) return false;
		this.current = scale;
		const target = this.targetFor(scale);
		if (Math.abs(r.resolution - target) < 0.005) return true;
		r.resolution = target;
		this.win.dispatchEvent(new Event("resize"));
		return true;
	}
}
```

- [ ] **Step 3: Тесты проходят**

Run: `npx vitest run agent/src`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add agent/src
git commit -m "feat(agent): blur, video, focus and canvas resolution levers

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Агент — HUD, «Замер», телеметрия, диагностика, точка входа

**Files:**
- Create: `agent/src/bench.ts`, `agent/src/bridge.ts`, `agent/src/diag.ts`, `agent/src/i18n.ts`, `agent/src/hud.ts`, `agent/src/bridge.test.ts`
- Modify: `agent/src/main.ts` (полная замена)

**Interfaces:**
- Consumes: всё из Task 5–7.
- Produces:
  - `bridge.ts`: `type Report`, `send(report: Report): void` — вызывает `__TAURI_INTERNALS__.invoke("report_telemetry", { report })`
  - `bench.ts`: `runBench(canvas: any, rec: { start(): void; stop(): number[] }, durationMs?: number): Promise<{avg,low1,min}>`
  - `diag.ts`: `collectDiag(extra: Record<string, unknown>): Record<string, unknown>`, `downloadDiag(data): void`
  - `i18n.ts`: `type Strings`, `strings(locale: "ru" | "en"): Strings`, `fmt(s: string, vars: Record<string, string | number>): string`
  - `hud.ts`: `class Hud { constructor(t: Strings, h: { onProfile(id: ProfileId): void; onBench(): void }); toggle(): void; toggleMenu(): void; update(d: HudData): void; status(text: string): void }`, `interface HudData`
  - Глобал страницы `window.__FP__ = { version, toggleHud(), setProfile(id), bench(), diag() }` — Rust (трей) вызывает `window.__FP__?.toggleHud()`.
  - Горячие клавиши: F9 — HUD, Shift+F9 — меню, F10 — диагностика.

- [ ] **Step 1: Падающий тест моста**

`agent/src/bridge.test.ts`:
```ts
import { afterEach, describe, expect, it, vi } from "vitest";
import { send } from "./bridge";

const g = globalThis as Record<string, unknown>;

afterEach(() => {
	delete g.__TAURI_INTERNALS__;
});

describe("send", () => {
	it("invokes report_telemetry when IPC is available", () => {
		const invoke = vi.fn().mockResolvedValue(null);
		g.__TAURI_INTERNALS__ = { invoke };
		send({ kind: "profileChanged", profile: "potato" });
		expect(invoke).toHaveBeenCalledWith("report_telemetry", { report: { kind: "profileChanged", profile: "potato" } });
	});

	it("is a no-op without IPC and swallows rejections", async () => {
		expect(() => send({ kind: "webglLost", early: true })).not.toThrow();
		g.__TAURI_INTERNALS__ = { invoke: vi.fn().mockRejectedValue(new Error("denied")) };
		expect(() => send({ kind: "webglLost", early: true })).not.toThrow();
	});
});
```

Run: `npx vitest run agent/src/bridge.test.ts`
Expected: FAIL — модуль не найден.

- [ ] **Step 2: `agent/src/bridge.ts`**

```ts
import type { ProfileId } from "./types";

export type Report =
	| { kind: "session"; avg: number; low1: number; profile: ProfileId }
	| { kind: "bench"; avg: number; low1: number; min: number; profile: ProfileId }
	| { kind: "profileChanged"; profile: ProfileId }
	| { kind: "webglLost"; early: boolean };

/** Единственный канал наружу. Если Tauri не выдал IPC этому origin — тихо ничего не делаем. */
export function send(report: Report): void {
	const internals = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ as
		| { invoke?: (cmd: string, args: unknown) => Promise<unknown> }
		| undefined;
	if (typeof internals?.invoke !== "function") return;
	try {
		internals.invoke("report_telemetry", { report }).catch(() => {});
	} catch {
		/* IPC недоступен для этого origin */
	}
}
```

Run: `npx vitest run agent/src/bridge.test.ts`
Expected: PASS.

- [ ] **Step 3: `agent/src/bench.ts`**

```ts
import { summarize } from "./fps-meter";

/** 30 с панорамирования и зума по сцене; фиксирует времена кадров через rec. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function runBench(canvas: any, rec: { start(): void; stop(): number[] }, durationMs = 30000) {
	const r = canvas.dimensions.sceneRect ?? canvas.dimensions.rect;
	const at = (fx: number, fy: number, scale: number) => ({ x: r.x + r.width * fx, y: r.y + r.height * fy, scale });
	const route = [at(0.5, 0.5, 0.5), at(0.2, 0.2, 1), at(0.8, 0.2, 1.5), at(0.8, 0.8, 1), at(0.2, 0.8, 0.75), at(0.5, 0.5, 1)];
	const segment = durationMs / route.length;
	rec.start();
	for (const p of route) await canvas.animatePan({ ...p, duration: segment });
	return summarize(rec.stop());
}
```

- [ ] **Step 4: `agent/src/i18n.ts`**

```ts
import type { ProfileId } from "./types";

const ru = {
	profile: { quality: "КАЧ", balance: "БАЛ", potato: "КРТ" } as Record<ProfileId, string>,
	profileLong: { quality: "Качество", balance: "Баланс", potato: "Картошка" } as Record<ProfileId, string>,
	base: "БАЗА",
	menuTitle: "ПРОФИЛЬ",
	bench: "ЗАМЕР 30 С",
	benchRunning: "ЗАМЕР… НЕ ТРОГАЙТЕ КАРТУ",
	benchDone: "AVG {avg} · 1% {low} · MIN {min}",
	benchNoScene: "НЕТ АКТИВНОЙ СЦЕНЫ",
	missing: "ПРОПУЩЕНО НАСТРОЕК: {n}",
	diagSaved: "ДИАГНОСТИКА СОХРАНЕНА В ЗАГРУЗКИ",
	hint: "F9 HUD · SHIFT+F9 МЕНЮ · F10 ДИАГН."
};

export type Strings = typeof ru;

const en: Strings = {
	profile: { quality: "QLT", balance: "BAL", potato: "POT" },
	profileLong: { quality: "Quality", balance: "Balance", potato: "Potato" },
	base: "BASE",
	menuTitle: "PROFILE",
	bench: "BENCH 30 S",
	benchRunning: "BENCHMARK… HANDS OFF THE MAP",
	benchDone: "AVG {avg} · 1% {low} · MIN {min}",
	benchNoScene: "NO ACTIVE SCENE",
	missing: "SETTINGS SKIPPED: {n}",
	diagSaved: "DIAGNOSTICS SAVED TO DOWNLOADS",
	hint: "F9 HUD · SHIFT+F9 MENU · F10 DIAG"
};

export function strings(locale: "ru" | "en"): Strings {
	return locale === "ru" ? ru : en;
}

export function fmt(s: string, vars: Record<string, string | number>): string {
	return s.replace(/\{(\w+)\}/g, (_, k: string) => String(vars[k] ?? ""));
}
```

- [ ] **Step 5: `agent/src/hud.ts`**

```ts
import type { Strings } from "./i18n";
import type { ProfileId } from "./types";

export interface HudData {
	fps: number;
	low1: number;
	ms: number;
	res: number;
	/** null — базовый замер (профиль не применён). */
	profile: ProfileId | null;
	spark: number[];
}

const POS_KEY = "fp.hud.pos";
const PROFILES: ProfileId[] = ["quality", "balance", "potato"];

const CSS = `
:host{all:initial}
.hud{position:fixed;z-index:2147483000;width:184px;background:#1E2A1E;color:#B8F28A;border:2px solid #D8D6D0;
 font:500 11px/1.35 'Martian Mono',ui-monospace,Consolas,monospace;padding:8px 10px;user-select:none;cursor:grab;
 font-variant-numeric:tabular-nums}
.hud[hidden]{display:none}
.row{display:flex;justify-content:space-between;gap:8px}
.big{font-size:26px;line-height:1}
.dim{color:#7FA866}
canvas{display:block;width:100%;height:18px;margin:5px 0}
.menu{margin-top:8px;border-top:1px solid #3C5A3C;padding-top:6px;display:grid;gap:4px;cursor:default}
.menu[hidden]{display:none}
button{all:unset;cursor:pointer;padding:3px 6px;border:1px solid #3C5A3C;text-align:center}
button[aria-pressed=true]{background:#B8F28A;color:#1E2A1E}
button:focus-visible{outline:2px solid #FF5A1F}
.bench{border-color:#FF5A1F;color:#FF5A1F}
.status{color:#FF5A1F;min-height:1em;white-space:normal}
@media (prefers-reduced-motion:no-preference){button{transition:background .12s}}
`;

export class Hud {
	private host = document.createElement("fp-hud");
	private box: HTMLDivElement;
	private el: Record<"fps" | "low" | "res" | "ms" | "prof" | "status", HTMLElement>;
	private menu: HTMLDivElement;
	private spark: HTMLCanvasElement;
	private buttons = new Map<ProfileId, HTMLButtonElement>();

	constructor(
		private t: Strings,
		private h: { onProfile(id: ProfileId): void; onBench(): void }
	) {
		const root = this.host.attachShadow({ mode: "closed" });
		root.innerHTML = `<style>${CSS}</style>
<div class="hud" hidden>
 <div class="row"><span class="big" data-k="fps">---</span><span>FPS</span></div>
 <canvas width="160" height="18"></canvas>
 <div class="row"><span>1%LOW <b data-k="low">--</b></span><span>RES <b data-k="res">--</b></span></div>
 <div class="row dim"><span><b data-k="ms">--</b> MS</span><span data-k="prof"></span></div>
 <div class="status" data-k="status"></div>
 <div class="menu" hidden><div class="dim">${t.menuTitle}</div></div>
</div>`;
		this.box = root.querySelector(".hud")!;
		this.menu = root.querySelector(".menu")!;
		this.spark = root.querySelector("canvas")!;
		const q = (k: string) => root.querySelector<HTMLElement>(`[data-k=${k}]`)!;
		this.el = { fps: q("fps"), low: q("low"), res: q("res"), ms: q("ms"), prof: q("prof"), status: q("status") };

		for (const id of PROFILES) {
			const b = document.createElement("button");
			b.textContent = t.profileLong[id];
			b.onclick = () => h.onProfile(id);
			this.buttons.set(id, b);
			this.menu.appendChild(b);
		}
		const bench = document.createElement("button");
		bench.className = "bench";
		bench.textContent = t.bench;
		bench.onclick = () => h.onBench();
		this.menu.appendChild(bench);
		const hint = document.createElement("div");
		hint.className = "dim";
		hint.textContent = t.hint;
		this.menu.appendChild(hint);

		this.restorePosition();
		this.enableDrag();
		(document.body ?? document.documentElement).appendChild(this.host);
	}

	toggle(): void {
		this.box.hidden = !this.box.hidden;
	}

	toggleMenu(): void {
		this.box.hidden = false;
		this.menu.hidden = !this.menu.hidden;
	}

	status(text: string): void {
		this.el.status.textContent = text;
		this.box.hidden = false;
	}

	update(d: HudData): void {
		if (this.box.hidden) return;
		this.el.fps.textContent = String(Math.round(d.fps)).padStart(3, "0");
		this.el.low.textContent = String(Math.round(d.low1));
		this.el.res.textContent = `${Math.round(d.res * 100)}%`;
		this.el.ms.textContent = d.ms.toFixed(1);
		this.el.prof.textContent = d.profile ? this.t.profile[d.profile] : this.t.base;
		for (const [id, b] of this.buttons) b.setAttribute("aria-pressed", String(id === d.profile));
		this.drawSpark(d.spark);
	}

	private drawSpark(fpsValues: number[]): void {
		const ctx = this.spark.getContext("2d");
		if (!ctx) return;
		const { width, height } = this.spark;
		ctx.clearRect(0, 0, width, height);
		const n = fpsValues.length;
		if (n === 0) return;
		const w = width / n;
		const top = Math.max(60, ...fpsValues);
		for (let i = 0; i < n; i++) {
			const v = fpsValues[i];
			const bh = Math.max(1, (v / top) * height);
			ctx.fillStyle = v < 30 ? "#FF5A1F" : "#B8F28A";
			ctx.fillRect(i * w, height - bh, Math.max(1, w - 1), bh);
		}
	}

	private restorePosition(): void {
		let pos = { right: 12, top: 12 };
		try {
			const raw = localStorage.getItem(POS_KEY);
			if (raw) pos = JSON.parse(raw);
		} catch {
			/* хранилище недоступно — позиция по умолчанию */
		}
		this.box.style.right = `${pos.right}px`;
		this.box.style.top = `${pos.top}px`;
	}

	private enableDrag(): void {
		this.box.addEventListener("pointerdown", (e) => {
			if ((e.target as HTMLElement).closest("button")) return;
			const startX = e.clientX;
			const startY = e.clientY;
			const startRight = parseFloat(this.box.style.right);
			const startTop = parseFloat(this.box.style.top);
			this.box.setPointerCapture(e.pointerId);
			const move = (ev: PointerEvent) => {
				this.box.style.right = `${Math.max(0, startRight - (ev.clientX - startX))}px`;
				this.box.style.top = `${Math.max(0, startTop + (ev.clientY - startY))}px`;
			};
			const up = () => {
				this.box.removeEventListener("pointermove", move);
				this.box.removeEventListener("pointerup", up);
				try {
					localStorage.setItem(POS_KEY, JSON.stringify({ right: parseFloat(this.box.style.right), top: parseFloat(this.box.style.top) }));
				} catch {
					/* не критично */
				}
			};
			this.box.addEventListener("pointermove", move);
			this.box.addEventListener("pointerup", up);
		});
	}
}
```

- [ ] **Step 6: `agent/src/diag.ts`**

```ts
import { g } from "./foundry";
import { collectVideos } from "./levers/video";

export const AGENT_VERSION = "0.1.0";

function plain(v: unknown): unknown {
	try {
		return JSON.parse(JSON.stringify(v));
	} catch {
		return String(v);
	}
}

export function collectDiag(extra: Record<string, unknown>): Record<string, unknown> {
	const game = g.game;
	const canvas = g.canvas;
	const r = canvas?.app?.renderer;
	const settings: unknown[] = [];
	if (game?.settings?.settings) {
		for (const [key, cfg] of game.settings.settings) {
			if (cfg.scope === "world") continue;
			let value: unknown;
			try {
				value = game.settings.get(cfg.namespace, cfg.key);
			} catch {
				value = "<error>";
			}
			settings.push({
				key,
				scope: cfg.scope,
				type: cfg.type?.name ?? typeof cfg.default,
				default: plain(cfg.default),
				value: plain(value),
				choices: cfg.choices ? Object.keys(cfg.choices) : undefined,
				range: plain(cfg.range)
			});
		}
	}
	let gpu: string | undefined;
	try {
		const gl = r?.gl;
		const ext = gl?.getExtension("WEBGL_debug_renderer_info");
		gpu = ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : undefined;
	} catch {
		gpu = undefined;
	}
	const modules = game?.modules ? [...game.modules].filter((m: any) => m.active).map((m: any) => ({ id: m.id, version: m.version })) : [];
	return {
		at: new Date().toISOString(),
		agent: AGENT_VERSION,
		foundry: game?.version ?? game?.release?.version,
		system: game?.system?.id,
		pixi: g.PIXI?.VERSION,
		devicePixelRatio: window.devicePixelRatio,
		renderer: { type: r?.type, resolution: r?.resolution, screen: [r?.screen?.width, r?.screen?.height], gpu },
		canvasPerformance: plain(canvas?.performance),
		hasVideoMeshes: Boolean(canvas?.primary?.videoMeshes),
		videoCount: collectVideos(canvas).length,
		modules,
		settings,
		...extra
	};
}

export function downloadDiag(data: Record<string, unknown>): void {
	const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
	const a = document.createElement("a");
	a.href = URL.createObjectURL(blob);
	a.download = `fp-diag-${Date.now()}.json`;
	document.body.appendChild(a);
	a.click();
	a.remove();
	setTimeout(() => URL.revokeObjectURL(a.href), 5000);
}
```

- [ ] **Step 7: `agent/src/main.ts` (полная замена)**

```ts
import { configFor, initAdaptive, resetTimers, stepAdaptive } from "./adaptive";
import { runBench } from "./bench";
import { send } from "./bridge";
import { coreValues, defaultValues, reconcile, writePreboot } from "./client-settings";
import { AGENT_VERSION, collectDiag, downloadDiag } from "./diag";
import { g } from "./foundry";
import { FrameStats, summarize } from "./fps-meter";
import { Hud } from "./hud";
import { fmt, strings } from "./i18n";
import { setUiBlur } from "./levers/blur";
import { FocusThrottle } from "./levers/focus";
import { CanvasResolution } from "./levers/resolution";
import { collectVideos, VideoController } from "./levers/video";
import { moduleValues } from "./modules";
import type { Boot, ProfileId } from "./types";

function whenDom(fn: () => void): void {
	if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", fn, { once: true });
	else fn();
}

function waitFor<T>(get: () => T | undefined, fn: (v: T) => void, timeoutMs = 120000): void {
	const started = Date.now();
	const tick = () => {
		const v = get();
		if (v) fn(v);
		else if (Date.now() - started < timeoutMs) setTimeout(tick, 50);
	};
	tick();
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

function start(boot: Boot): void {
	const t = strings(boot.locale);
	let profileId: ProfileId = boot.profileId;
	let levers = boot.presets[profileId];
	let adaptiveCfg = configFor(levers);
	let adaptive = initAdaptive(adaptiveCfg);
	let readyAt = 0;
	let skipped = 0;
	let benchSamples: number[] | null = null;
	let tickerAttached = false;
	// Базовый замер: геттеры ниже отдают «ничего», и рычаги становятся no-op
	let measureOnly = boot.measureOnly;
	const stats = new FrameStats(4096);
	const sparkline: number[] = [];

	const resolution = new CanvasResolution(
		() => (measureOnly ? undefined : g.canvas?.app?.renderer),
		() => (levers.pixelRatioScaling ? window.devicePixelRatio || 1 : 1),
		window
	);
	const video = new VideoController(() => (measureOnly ? [] : collectVideos(g.canvas)));
	const focus = new FocusThrottle(() => (measureOnly ? undefined : g.canvas?.app?.ticker), levers.maxFps, levers.unfocusedFps);

	// Фаза 1: до загрузки Foundry
	if (!measureOnly) {
		try {
			writePreboot(localStorage, coreValues(levers));
		} catch {
			/* localStorage недоступен — reconcile применит на ready */
		}
	}
	whenDom(() => {
		if (!measureOnly) setUiBlur(document, levers.uiBlur);
		const probe = document.createElement("canvas").getContext("webgl2");
		if (!probe && !sessionStorage.getItem("fp.nowebgl")) {
			sessionStorage.setItem("fp.nowebgl", "1");
			send({ kind: "webglLost", early: true });
		}
	});

	let hud: Hud | null = null;
	const ensureHud = () => (hud ??= new Hud(t, { onProfile: (id) => void setProfile(id, true), onBench: () => void bench() }));

	window.addEventListener(
		"keydown",
		(e) => {
			if (e.key === "F9" && e.shiftKey) ensureHud().toggleMenu();
			else if (e.key === "F9") ensureHud().toggle();
			else if (e.key === "F10") void diag();
			else return;
			e.preventDefault();
			e.stopPropagation();
		},
		true
	);

	const targetScale = () => (levers.adaptive ? adaptive.scale : levers.resMax);

	async function applySettings(): Promise<void> {
		if (!g.game?.settings) return;
		const isActive = (id: string) => Boolean(g.game.modules?.get(id)?.active);
		const values = measureOnly
			? defaultValues(g.game, Object.keys(coreValues(levers)))
			: { ...coreValues(levers), ...moduleValues(isActive, levers) };
		const r = await reconcile(g.game, values);
		skipped = r.missing.length + r.failed.length;
		if (skipped > 0) console.info("[Foundry Performance] skipped settings", r);
	}

	function attachTicker(): void {
		const ticker = g.canvas?.app?.ticker;
		if (!ticker || tickerAttached) return;
		tickerAttached = true;
		ticker.add(() => {
			const ms = ticker.deltaMS;
			stats.push(ms);
			benchSamples?.push(ms);
		});
		const view = g.canvas.app.renderer?.view ?? g.canvas.app.canvas;
		view?.addEventListener?.("webglcontextlost", () => send({ kind: "webglLost", early: performance.now() - readyAt < 60000 }));
		focus.apply();
	}

	function loop(): void {
		const window1s = Math.max(10, Math.round(levers.maxFps));
		const avg = stats.avgMs(window1s);
		if (levers.adaptive && !measureOnly && focus.focused && avg > 0) {
			const next = stepAdaptive(adaptive, adaptiveCfg, avg, performance.now());
			if (next.scale !== adaptive.scale) resolution.apply(next.scale);
			adaptive = next;
		}
		if (avg > 0) {
			sparkline.push(1000 / avg);
			if (sparkline.length > 40) sparkline.shift();
		}
		hud?.update({
			fps: avg > 0 ? 1000 / avg : 0,
			low1: summarize(stats.recent(window1s * 5)).low1,
			ms: avg,
			res: resolution.scale,
			profile: measureOnly ? null : profileId,
			spark: sparkline
		});
	}

	function sessionReport(): void {
		if (measureOnly || !focus.focused || stats.size < 60) return;
		const s = summarize(stats.recent(Math.round(levers.maxFps * 30)));
		send({ kind: "session", avg: s.avg, low1: s.low1, profile: profileId });
	}

	function onFocusChange(focused: boolean): void {
		focus.setFocused(focused);
		video.setFocused(focused);
		adaptive = resetTimers(adaptive);
	}

	async function setProfile(id: ProfileId, persist: boolean): Promise<void> {
		measureOnly = false;
		profileId = id;
		levers = boot.presets[id];
		adaptiveCfg = configFor(levers);
		adaptive = initAdaptive(adaptiveCfg);
		try {
			writePreboot(localStorage, coreValues(levers));
		} catch {
			/* см. фазу 1 */
		}
		setUiBlur(document, levers.uiBlur);
		video.setMode(levers.video);
		focus.setFps(levers.maxFps, levers.unfocusedFps);
		await applySettings();
		resolution.apply(targetScale());
		if (skipped > 0) hud?.status(fmt(t.missing, { n: skipped }));
		if (persist) send({ kind: "profileChanged", profile: id });
	}

	async function bench(): Promise<void> {
		const h = ensureHud();
		if (!g.canvas?.ready || !g.canvas?.dimensions) {
			h.status(t.benchNoScene);
			return;
		}
		h.status(t.benchRunning);
		const recorder = {
			start: () => {
				benchSamples = [];
			},
			stop: () => {
				const s = benchSamples ?? [];
				benchSamples = null;
				return s;
			}
		};
		let out: { avg: number; low1: number; min: number };
		try {
			out = await runBench(g.canvas, recorder);
		} catch {
			benchSamples = null;
			h.status(t.benchNoScene);
			return;
		}
		h.status(fmt(t.benchDone, { avg: Math.round(out.avg), low: Math.round(out.low1), min: Math.round(out.min) }));
		if (!measureOnly) send({ kind: "bench", avg: out.avg, low1: out.low1, min: out.min, profile: profileId });
	}

	async function diag(): Promise<void> {
		const r = g.canvas?.app?.renderer;
		const before = { scale: resolution.scale, rendererResolution: r?.resolution };
		let resolutionTest: unknown = "no-renderer";
		if (r) {
			resolution.apply(0.8);
			await sleep(700);
			resolutionTest = { target: resolution.targetFor(0.8), after: r.resolution, screen: [r.screen?.width, r.screen?.height] };
			resolution.apply(before.scale);
		}
		downloadDiag(
			collectDiag({ fp: { profileId, levers, adaptive, skipped, before, resolutionTest, tickerAttached, bootProfiles: Object.keys(boot.presets) } })
		);
		ensureHud().status(t.diagSaved);
	}

	waitFor(
		() => g.Hooks,
		(hooks) => {
			hooks.once("ready", async () => {
				readyAt = performance.now();
				video.setMode(levers.video);
				await applySettings();
				attachTicker();
				resolution.apply(targetScale());
				window.addEventListener("blur", () => onFocusChange(false));
				window.addEventListener("focus", () => onFocusChange(true));
				document.addEventListener("visibilitychange", () => onFocusChange(!document.hidden));
				setInterval(loop, 250);
				setInterval(sessionReport, 30000);
				setInterval(() => video.sync(), 2000);
				if (skipped > 0) ensureHud().status(fmt(t.missing, { n: skipped }));
			});
			hooks.on("canvasReady", () => {
				attachTicker();
				resolution.apply(targetScale());
				video.sync();
			});
		}
	);

	g.__FP__ = {
		version: AGENT_VERSION,
		toggleHud: () => ensureHud().toggle(),
		setProfile: (id: ProfileId) => setProfile(id, true),
		bench,
		diag
	};
}

const boot = g.__FP_BOOT__ as Boot | undefined;
if (boot && !g.__FP__) start(boot);
```

- [ ] **Step 8: Сборка и тесты**

```bash
npx vitest run agent/src
npm run build:agent
npx tsc --noEmit -p tsconfig.json
```
Expected: тесты PASS; `agent/dist/agent.js` создан (≈ 10–20 КБ); `tsc` без ошибок.

- [ ] **Step 9: Commit**

```bash
git add agent/src
git commit -m "feat(agent): HUD, benchmark, telemetry bridge, diagnostics, entry point

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Rust — запуск игры, окна, телеметрия, трей, команды

**Files:**
- Create: `src-tauri/src/agent.rs`, `src-tauri/src/telemetry.rs`, `src-tauri/src/state.rs`, `src-tauri/src/windows.rs`, `src-tauri/src/tray.rs`, `src-tauri/src/commands.rs`, `src-tauri/capabilities/game.json`
- Modify: `src-tauri/src/lib.rs` (полная замена)

**Interfaces:**
- Consumes: Task 1–4 (model, profile, engine_flags, store, gpu, locale, probe); `agent/dist/agent.js` (Task 8).
- Produces (команды Tauri; JS-имена аргументов в camelCase):
  - `get_state() -> StateDto { settings, servers, stats, presets, gpu, recommended, notice, locale, version }`
  - `save_server(server: Server) -> Result<Vec<Server>, String>` (ошибки: `err.nameRequired`, `err.urlInvalid`, `err.saveFailed`)
  - `delete_server(id: String) -> Result<Vec<Server>, String>`
  - `save_settings(settings: Settings) -> Result<Settings, String>`
  - `probe_server(url: String) -> ProbeResult`
  - `launch(serverId: String, safeMode: bool) -> Result<(), String>` (ошибки: `err.serverMissing`, `err.urlInvalid`, `notice.launchFailed`)
  - `clear_notice() -> ()`
  - `report_telemetry(report: Report) -> Result<(), String>` — только из окна `game`
  - CLI: `foundry-performance.exe --open <url> [--safe | --baseline]`
- `agent::Boot { server_id, profile_id, presets, locale, measure_only }`

- [ ] **Step 1: `agent.rs` с тестом**

```rust
use crate::model::{Levers, ProfileId};
use serde::Serialize;
use std::collections::BTreeMap;

pub const AGENT_JS: &str = include_str!("../../agent/dist/agent.js");

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Boot<'a> {
    pub server_id: &'a str,
    pub profile_id: ProfileId,
    pub presets: &'a BTreeMap<ProfileId, Levers>,
    pub locale: &'a str,
    pub measure_only: bool,
}

pub fn script(boot: &Boot) -> String {
    let json = serde_json::to_string(boot).expect("Boot is always serializable");
    format!("window.__FP_BOOT__ = {json};\n{AGENT_JS}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile;

    #[test]
    fn script_embeds_boot_before_agent() {
        let presets = profile::presets();
        let s = script(&Boot { server_id: "a1", profile_id: ProfileId::Balance, presets: &presets, locale: "ru", measure_only: false });
        assert!(s.starts_with("window.__FP_BOOT__ = {"));
        assert!(s.contains("\"profileId\":\"balance\""));
        assert!(s.contains("\"measureOnly\":false"));
        assert!(s.contains("\"potato\":{"));
        assert!(s.ends_with(AGENT_JS));
    }
}
```

- [ ] **Step 2: Падающие тесты `telemetry.rs`**

```rust
use crate::model::*;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Report {
    Session { avg: f32, low1: f32, profile: ProfileId },
    Bench { avg: f32, low1: f32, min: f32, profile: ProfileId },
    ProfileChanged { profile: ProfileId },
    WebglLost { early: bool },
}

#[derive(Debug, Default, PartialEq)]
pub struct Effects {
    pub stats: bool,
    pub servers: bool,
    pub settings: bool,
    pub notice: Option<Notice>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srv(id: &str) -> Server {
        Server { id: id.into(), name: "S".into(), url: "https://s".into(), profile: None, overrides: Overrides::default() }
    }

    #[test]
    fn parses_agent_json() {
        let r: Report = serde_json::from_str(r#"{"kind":"bench","avg":41.2,"low1":20.5,"min":12.0,"profile":"potato"}"#).unwrap();
        assert_eq!(r, Report::Bench { avg: 41.2, low1: 20.5, min: 12.0, profile: ProfileId::Potato });
        let r: Report = serde_json::from_str(r#"{"kind":"webglLost","early":true}"#).unwrap();
        assert_eq!(r, Report::WebglLost { early: true });
    }

    #[test]
    fn rejects_non_finite_or_absurd_numbers() {
        assert!(!validate(&Report::Session { avg: f32::NAN, low1: 1.0, profile: ProfileId::Balance }));
        assert!(!validate(&Report::Bench { avg: 5000.0, low1: 1.0, min: 1.0, profile: ProfileId::Balance }));
        assert!(validate(&Report::Session { avg: 58.0, low1: 31.0, profile: ProfileId::Balance }));
    }

    #[test]
    fn session_and_bench_update_stats() {
        let (mut stats, mut servers, mut settings) = (BTreeMap::new(), vec![srv("a")], Settings::default());
        let fx = apply(&Report::Session { avg: 50.0, low1: 30.0, profile: ProfileId::Balance }, "a", 7, &mut stats, &mut servers, &mut settings);
        assert!(fx.stats);
        assert_eq!(stats["a"].last_session.unwrap().at, 7);
        apply(&Report::Bench { avg: 40.0, low1: 20.0, min: 10.0, profile: ProfileId::Potato }, "a", 8, &mut stats, &mut servers, &mut settings);
        assert_eq!(stats["a"].last_bench.unwrap().min, 10.0);
    }

    #[test]
    fn profile_change_persists_on_server() {
        let (mut stats, mut servers, mut settings) = (BTreeMap::new(), vec![srv("a")], Settings::default());
        let fx = apply(&Report::ProfileChanged { profile: ProfileId::Potato }, "a", 0, &mut stats, &mut servers, &mut settings);
        assert!(fx.servers);
        assert_eq!(servers[0].profile, Some(ProfileId::Potato));
        let fx = apply(&Report::ProfileChanged { profile: ProfileId::Potato }, "adhoc", 0, &mut stats, &mut servers, &mut settings);
        assert!(!fx.servers);
    }

    #[test]
    fn early_webgl_loss_switches_angle_backend() {
        let (mut stats, mut servers, mut settings) = (BTreeMap::new(), vec![srv("a")], Settings::default());
        let fx = apply(&Report::WebglLost { early: true }, "a", 0, &mut stats, &mut servers, &mut settings);
        assert!(fx.settings);
        assert_eq!(settings.engine.angle, AngleBackend::D3d11on12);
        let n = fx.notice.unwrap();
        assert_eq!((n.key.as_str(), n.params["to"].as_str()), ("notice.engineFallback", "d3d11on12"));
        let fx = apply(&Report::WebglLost { early: false }, "a", 0, &mut stats, &mut servers, &mut settings);
        assert_eq!(fx, Effects::default());
    }
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml telemetry`
Expected: FAIL — `validate`/`apply` не найдены (сначала добавить `pub mod agent; pub mod telemetry;` в `lib.rs`, иначе тесты не соберутся).

- [ ] **Step 3: Реализация `telemetry.rs`** (над тестами)

```rust
fn fps_ok(x: f32) -> bool {
    x.is_finite() && (0.0..=1000.0).contains(&x)
}

pub fn validate(r: &Report) -> bool {
    match r {
        Report::Session { avg, low1, .. } => fps_ok(*avg) && fps_ok(*low1),
        Report::Bench { avg, low1, min, .. } => fps_ok(*avg) && fps_ok(*low1) && fps_ok(*min),
        Report::ProfileChanged { .. } | Report::WebglLost { .. } => true,
    }
}

pub fn apply(
    report: &Report,
    server_id: &str,
    now: u64,
    stats: &mut BTreeMap<String, ServerStats>,
    servers: &mut [Server],
    settings: &mut Settings,
) -> Effects {
    let mut fx = Effects::default();
    match *report {
        Report::Session { avg, low1, profile } => {
            stats.entry(server_id.to_string()).or_default().last_session = Some(FpsSummary { avg, low1, profile, at: now });
            fx.stats = true;
        }
        Report::Bench { avg, low1, min, profile } => {
            stats.entry(server_id.to_string()).or_default().last_bench = Some(BenchResult { avg, low1, min, profile, at: now });
            fx.stats = true;
        }
        Report::ProfileChanged { profile } => {
            if let Some(s) = servers.iter_mut().find(|s| s.id == server_id) {
                s.profile = Some(profile);
                fx.servers = true;
            }
        }
        Report::WebglLost { early: true } => {
            let from = settings.engine.angle;
            settings.engine.angle = from.next();
            fx.settings = true;
            fx.notice = Some(
                Notice::new("notice.engineFallback")
                    .with("from", from.flag_value())
                    .with("to", settings.engine.angle.flag_value()),
            );
        }
        Report::WebglLost { early: false } => {}
    }
    fx
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml telemetry agent`
Expected: PASS.

- [ ] **Step 4: `state.rs`, `windows.rs`, `tray.rs`**

`src-tauri/src/state.rs`:
```rust
use crate::gpu::GpuInfo;
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
}

pub struct AppState {
    pub store: Store,
    pub gpu: Option<GpuInfo>,
    pub data: Mutex<Data>,
}
```

`src-tauri/src/windows.rs`:
```rust
use std::path::PathBuf;
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub const LAUNCHER: &str = "main";
pub const GAME: &str = "game";

pub fn open_launcher(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(LAUNCHER) {
        w.show()?;
        w.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, LAUNCHER, WebviewUrl::App("index.html".into()))
        .title("Foundry Performance")
        .inner_size(900.0, 620.0)
        .min_inner_size(820.0, 580.0)
        .decorations(false)
        .center()
        .build()?;
    Ok(())
}

pub struct GameLaunch {
    pub url: url::Url,
    pub title: String,
    pub browser_args: String,
    pub init_script: Option<String>,
    pub data_dir: PathBuf,
}

/// Отдельная папка данных = отдельный процесс браузера WebView2 со своими флагами.
pub fn open_game(app: &AppHandle, l: GameLaunch) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(GAME) {
        w.set_focus()?;
        return Ok(());
    }
    let mut b = WebviewWindowBuilder::new(app, GAME, WebviewUrl::External(l.url))
        .title(l.title)
        .inner_size(1280.0, 800.0)
        .maximized(true)
        .data_directory(l.data_dir)
        .additional_browser_args(&l.browser_args)
        .disable_drag_drop_handler()
        .general_autofill_enabled(false)
        .on_new_window(|_url, _features| NewWindowResponse::Allow);
    if let Some(script) = l.init_script {
        b = b.initialization_script(script);
    }
    b.build()?;
    Ok(())
}
```

`src-tauri/src/tray.rs`:
```rust
use crate::windows;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

pub fn create(app: &AppHandle, lang: &str) -> tauri::Result<()> {
    let (hud, launcher, quit) = if lang == "ru" {
        ("Показать / скрыть HUD", "Вернуться в лаунчер", "Выход")
    } else {
        ("Toggle HUD", "Back to launcher", "Quit")
    };
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "hud", hud, true, None::<&str>)?,
            &MenuItem::with_id(app, "launcher", launcher, true, None::<&str>)?,
            &MenuItem::with_id(app, "quit", quit, true, None::<&str>)?,
        ],
    )?;
    let mut builder = TrayIconBuilder::with_id("main").tooltip("Foundry Performance").menu(&menu).show_menu_on_left_click(true);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "hud" => {
                if let Some(w) = app.get_webview_window(windows::GAME) {
                    let _ = w.eval("window.__FP__ && window.__FP__.toggleHud()");
                }
            }
            "launcher" => match app.get_webview_window(windows::GAME) {
                // закрытие игры само откроет лаунчер (см. lib.rs, WindowEvent::Destroyed)
                Some(w) => {
                    let _ = w.destroy();
                }
                None => {
                    let _ = windows::open_launcher(app);
                }
            },
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
```

- [ ] **Step 5: Падающие тесты `commands.rs` (сборка параметров запуска)**

```rust
use crate::model::*;
use crate::state::AppState;
use crate::{agent, engine_flags, gpu, locale, probe, profile, store, telemetry, windows};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State, Webview};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    Normal,
    /// Без агента, стандартные флаги — чтобы исключить наше влияние.
    Safe,
    /// Стандартные флаги + агент только для измерений.
    Baseline,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srv() -> Server {
        Server { id: "a1".into(), name: "Страд".into(), url: "vtt.example.com/game".into(), profile: Some(ProfileId::Potato), overrides: Overrides::default() }
    }

    #[test]
    fn normal_launch_uses_tuned_flags_and_agent() {
        let l = build_launch(&Settings::default(), &srv(), LaunchMode::Normal).unwrap();
        assert_eq!(l.url.as_str(), "https://vtt.example.com/");
        assert!(l.browser_args.contains("--use-angle=d3d11"));
        let script = l.init_script.unwrap();
        assert!(script.contains("\"profileId\":\"potato\""));
        assert!(script.contains("\"measureOnly\":false"));
        assert!(l.title.contains("Страд"));
    }

    #[test]
    fn safe_launch_has_no_agent_and_stock_flags() {
        let l = build_launch(&Settings::default(), &srv(), LaunchMode::Safe).unwrap();
        assert!(l.init_script.is_none());
        assert_eq!(l.browser_args, engine_flags::safe());
    }

    #[test]
    fn baseline_launch_measures_only() {
        let l = build_launch(&Settings::default(), &srv(), LaunchMode::Baseline).unwrap();
        assert_eq!(l.browser_args, engine_flags::safe());
        assert!(l.init_script.unwrap().contains("\"measureOnly\":true"));
    }

    #[test]
    fn invalid_url_is_rejected() {
        let s = Server { url: "::::".into(), ..srv() };
        assert_eq!(build_launch(&Settings::default(), &s, LaunchMode::Normal).unwrap_err(), "err.urlInvalid");
    }

    #[test]
    fn parse_cli_args() {
        let a = |v: &[&str]| parse_cli(v.iter().map(|s| s.to_string()));
        assert_eq!(a(&["--open", "vtt.x"]), Some(("vtt.x".to_string(), LaunchMode::Normal)));
        assert_eq!(a(&["--safe", "--open", "vtt.x"]), Some(("vtt.x".to_string(), LaunchMode::Safe)));
        assert_eq!(a(&["--open", "vtt.x", "--baseline"]), Some(("vtt.x".to_string(), LaunchMode::Baseline)));
        assert_eq!(a(&[]), None);
    }
}
```

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands`
Expected: FAIL — `build_launch`/`parse_cli` не найдены (добавить `pub mod commands; pub mod state; pub mod windows; pub mod tray;` в `lib.rs`).

- [ ] **Step 6: Реализация `commands.rs`** (над тестами)

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDto {
    settings: Settings,
    servers: Vec<Server>,
    stats: BTreeMap<String, ServerStats>,
    presets: BTreeMap<ProfileId, Levers>,
    gpu: Option<gpu::GpuInfo>,
    recommended: ProfileId,
    notice: Option<Notice>,
    locale: &'static str,
    version: String,
}

pub fn build_launch(settings: &Settings, server: &Server, mode: LaunchMode) -> Result<windows::GameLaunch, String> {
    let url = probe::normalize_url(&server.url)?;
    let presets = profile::resolve_all(settings, Some(server));
    let boot = agent::Boot {
        server_id: &server.id,
        profile_id: profile::active_id(settings, Some(server)),
        presets: &presets,
        locale: locale::effective(settings.locale),
        measure_only: mode == LaunchMode::Baseline,
    };
    Ok(windows::GameLaunch {
        url,
        title: format!("{} — Foundry Performance", server.name),
        browser_args: match mode {
            LaunchMode::Normal => engine_flags::build(&settings.engine),
            LaunchMode::Safe | LaunchMode::Baseline => engine_flags::safe(),
        },
        init_script: match mode {
            LaunchMode::Safe => None,
            LaunchMode::Normal | LaunchMode::Baseline => Some(agent::script(&boot)),
        },
        data_dir: store::engine_dir(),
    })
}

pub fn parse_cli(args: impl Iterator<Item = String>) -> Option<(String, LaunchMode)> {
    let mut url = None;
    let mut mode = LaunchMode::Normal;
    let mut it = args;
    while let Some(a) = it.next() {
        match a.as_str() {
            "--open" => url = it.next(),
            "--safe" => mode = LaunchMode::Safe,
            "--baseline" => mode = LaunchMode::Baseline,
            _ => {}
        }
    }
    url.map(|u| (u, mode))
}

fn start_game(app: &AppHandle, state: &AppState, server: &Server, mode: LaunchMode) -> Result<(), String> {
    let launch = {
        let mut d = state.data.lock().expect("state poisoned");
        d.current_server = Some(server.id.clone());
        d.session_fallback_done = false;
        build_launch(&d.settings, server, mode)?
    };
    windows::open_game(app, launch).map_err(|e| {
        eprintln!("[foundry-performance] open_game failed: {e}");
        "notice.launchFailed".to_string()
    })
}

/// Запуск из командной строки: `--open <url> [--safe|--baseline]`.
pub fn launch_adhoc(app: &AppHandle, state: &AppState, url: &str, mode: LaunchMode) -> Result<(), String> {
    let server = Server { id: "adhoc".into(), name: url.to_string(), url: url.to_string(), profile: None, overrides: Overrides::default() };
    start_game(app, state, &server, mode)
}

#[tauri::command]
pub fn get_state(app: AppHandle, state: State<'_, AppState>) -> StateDto {
    let d = state.data.lock().expect("state poisoned");
    StateDto {
        settings: d.settings.clone(),
        servers: d.servers.clone(),
        stats: d.stats.clone(),
        presets: profile::presets(),
        gpu: state.gpu.clone(),
        recommended: gpu::recommend(state.gpu.as_ref()),
        notice: d.notice.clone(),
        locale: locale::effective(d.settings.locale),
        version: app.package_info().version.to_string(),
    }
}

#[tauri::command]
pub fn save_server(state: State<'_, AppState>, server: Server) -> Result<Vec<Server>, String> {
    let name = server.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 60 || server.id.trim().is_empty() {
        return Err("err.nameRequired".into());
    }
    let url = probe::normalize_url(&server.url)?;
    let clean = Server { name, url: url.to_string(), ..server };
    let mut d = state.data.lock().expect("state poisoned");
    match d.servers.iter_mut().find(|s| s.id == clean.id) {
        Some(s) => *s = clean,
        None => d.servers.push(clean),
    }
    state.store.save_servers(&d.servers).map_err(|_| "err.saveFailed".to_string())?;
    Ok(d.servers.clone())
}

#[tauri::command]
pub fn delete_server(state: State<'_, AppState>, id: String) -> Result<Vec<Server>, String> {
    let mut d = state.data.lock().expect("state poisoned");
    d.servers.retain(|s| s.id != id);
    d.stats.remove(&id);
    state.store.save_servers(&d.servers).map_err(|_| "err.saveFailed".to_string())?;
    let _ = state.store.save_stats(&d.stats);
    Ok(d.servers.clone())
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    let mut d = state.data.lock().expect("state poisoned");
    d.settings = Settings { schema: SCHEMA, ..settings };
    state.store.save_settings(&d.settings).map_err(|_| "err.saveFailed".to_string())?;
    Ok(d.settings.clone())
}

#[tauri::command]
pub async fn probe_server(url: String) -> probe::ProbeResult {
    probe::probe(&url).await
}

/// async: на Windows создание окна из синхронной команды может взаимно заблокироваться.
#[tauri::command]
pub async fn launch(app: AppHandle, state: State<'_, AppState>, server_id: String, safe_mode: bool) -> Result<(), String> {
    let server = {
        let d = state.data.lock().expect("state poisoned");
        d.servers.iter().find(|s| s.id == server_id).cloned().ok_or_else(|| "err.serverMissing".to_string())?
    };
    let mode = if safe_mode { LaunchMode::Safe } else { LaunchMode::Normal };
    start_game(&app, &state, &server, mode)
}

#[tauri::command]
pub fn clear_notice(state: State<'_, AppState>) {
    state.data.lock().expect("state poisoned").notice = None;
}

#[tauri::command]
pub fn report_telemetry(webview: Webview, state: State<'_, AppState>, report: telemetry::Report) -> Result<(), String> {
    if webview.label() != windows::GAME || !telemetry::validate(&report) {
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
    Ok(())
}
```

- [ ] **Step 7: `lib.rs` (полная замена) и `capabilities/game.json`**

`src-tauri/src/lib.rs`:
```rust
pub mod agent;
pub mod commands;
pub mod engine_flags;
pub mod gpu;
pub mod locale;
pub mod model;
pub mod probe;
pub mod profile;
pub mod state;
pub mod store;
pub mod telemetry;
pub mod tray;
pub mod windows;

use state::{AppState, Data};
use std::sync::Mutex;
use tauri::{Manager, RunEvent, WindowEvent};

pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let store = store::Store::new(store::Store::default_dir());
            let gpu = gpu::detect();
            let (settings, n1) = store.load_settings(gpu::recommend(gpu.as_ref()));
            let (servers, n2) = store.load_servers();
            let (stats, n3) = store.load_stats();
            let lang = locale::effective(settings.locale);
            app.manage(AppState {
                store,
                gpu,
                data: Mutex::new(Data {
                    settings,
                    servers,
                    stats,
                    notice: n1.or(n2).or(n3),
                    current_server: None,
                    session_fallback_done: false,
                }),
            });
            tray::create(app.handle(), lang)?;
            match commands::parse_cli(std::env::args().skip(1)) {
                Some((url, mode)) => commands::launch_adhoc(app.handle(), &app.state::<AppState>(), &url, mode)?,
                None => windows::open_launcher(app.handle())?,
            }
            Ok(())
        })
        .on_window_event(|window, event| match (window.label(), event) {
            // Пользователь закрыл лаунчер крестиком — выходим. Программный destroy()
            // при запуске игры CloseRequested не порождает.
            (windows::LAUNCHER, WindowEvent::CloseRequested { .. }) => window.app_handle().exit(0),
            (windows::GAME, WindowEvent::Destroyed) => {
                if let Some(s) = window.app_handle().try_state::<AppState>() {
                    s.data.lock().expect("state poisoned").current_server = None;
                }
                let _ = windows::open_launcher(window.app_handle());
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::save_server,
            commands::delete_server,
            commands::save_settings,
            commands::probe_server,
            commands::launch,
            commands::clear_notice,
            commands::report_telemetry
        ])
        .build(tauri::generate_context!())
        .expect("error while building Foundry Performance");

    // Между закрытием лаунчера и открытием игры окон может не остаться — не выходим.
    // Явный выход — только app.exit(0) (крестик лаунчера или «Выход» в трее).
    app.run(|_app, event| {
        if let RunEvent::ExitRequested { code: None, api, .. } = event {
            api.prevent_exit();
        }
    });
}
```

`src-tauri/capabilities/game.json`:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "game-telemetry",
  "windows": ["game"],
  "remote": {
    "urls": ["https://*:*/*", "http://*:*/*"]
  },
  "permissions": ["allow-report-telemetry"]
}
```

- [ ] **Step 8: Все тесты и сборка**

```bash
npm run build:agent
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```
Expected: PASS; clippy без предупреждений (при предупреждениях — исправить по подсказке clippy).

- [ ] **Step 9: Смоук-запуск по CLI**

```bash
npm run tauri build -- --no-bundle
./src-tauri/target/release/foundry-performance.exe --open https://foundryvtt.com
```
Expected: открывается развёрнутое окно `https://foundryvtt.com — Foundry Performance` (это не сервер Foundry: хук `ready` не наступит, агент только ждёт); закрытие окна открывает лаунчер; закрытие лаунчера завершает процесс. Появилась папка `%LOCALAPPDATA%\FoundryPerformance\engine\`. Фактическое действие флагов движка проверяется в Task 10 по `renderer.gpu` в дампе.

- [ ] **Step 10: Commit**

```bash
git add src-tauri
git commit -m "feat(core): game launch, window lifecycle, telemetry, tray, commands

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 10: Проверка осуществимости на реальном сервере v14 (вместе с пользователем)

**Files:** нет изменений кода. Результат — файлы `fp-diag-*.json` и таблица замеров от пользователя, сохранённые в `docs/superpowers/spike/`.

Вход на сервер выполняет **пользователь** (агент-исполнитель не вводит пароли).

- [ ] **Step 1: Передать пользователю сборку и инструкцию**

Сборка: `src-tauri/target/release/foundry-performance.exe` (из Task 9). Инструкция пользователю (в чат):

1. Запустить `foundry-performance.exe --open <адрес сервера>`, войти в мир, открыть самую тяжёлую сцену.
2. F9 — должен появиться HUD. Записать, работает ли он.
3. F10 — сохранится `fp-diag-….json` в «Загрузки». Прислать файл.
4. Shift+F9 → «Замер 30 с», не трогать мышь. Записать строку результата (профиль по умолчанию).
5. В меню Shift+F9 переключить на «Картошку», повторить замер.
6. Закрыть игру. Запустить `foundry-performance.exe --open <адрес> --baseline`, та же сцена, Shift+F9 → замер. Это «как во FLC».
7. Визуально: не стал ли интерфейс Foundry нечитаемым без blur; ставятся ли на паузу видеофоны в «Картошке»; не мыльная ли карта.
8. По возможности повторить шаги 4–6 на машине друга с GTX 1060.

- [ ] **Step 2: Разобрать дамп** — проверить по `fp-diag-*.json`:

| Что | Где в дампе | Ожидание |
|---|---|---|
| Версия Foundry, PIXI | `foundry`, `pixi` | 14.x; PIXI 7 или 8 |
| Ключи core | `settings[].key` | `core.performanceMode`, `core.maxFPS`, `core.pixelRatioResolutionScaling`, `core.lightAnimation`, `core.visionAnimation`, `core.mipmap` |
| Диапазон maxFPS | `settings[key=core.maxFPS].range` | вмещает 10–60 |
| Ключи модулей | `settings[].key`, начинающиеся с `sequencer.`, `fxmaster.`, `fvtt-perf-optim.` | существуют |
| Смена разрешения | `fp.resolutionTest.after` ≈ `fp.resolutionTest.target` | совпадают (±0.01) |
| Видео | `hasVideoMeshes`, `videoCount` | `true`, > 0 на анимированной сцене |
| GPU/ANGLE | `renderer.gpu` | содержит `Direct3D11` и имя видеокарты |
| Пропущенные | `fp.skipped` | 0 после Task 11 |
| Телеметрия (IPC для удалённого origin) | `%APPDATA%\FoundryPerformance\stats.json` после «Замера» | есть запись `adhoc` с `lastBench` |
| Смена профиля на лету | визуально после Shift+F9 → «Картошка» | качество света/теней меняется без перезагрузки |

- [ ] **Step 3: Зафиксировать выводы**

Создать `docs/superpowers/spike/2026-xx-xx-findings.md` (дата фактическая) с: таблицей из Step 2 (факт), таблицей замеров (baseline / профиль по умолчанию / картошка: avg, 1% low, min; для обеих машин), списком расхождений, которые чинит Task 11. Положить рядом дамп(ы). Commit:

```bash
git add docs/superpowers/spike
git commit -m "docs: v14 feasibility findings

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 11: Исправления по итогам проверки

**Files (по результатам Task 10):**
- Modify: `agent/src/client-settings.ts` (`CORE_KEYS`), `agent/src/modules.ts` (`MODULE_SETTINGS`, `PRIME_SETTINGS`), `agent/src/levers/resolution.ts`, `agent/src/levers/video.ts`, соответствующие `*.test.ts`

Каждое расхождение из findings закрывается отдельным циклом «тест → правка → тест → коммит». Правила:

- [ ] **Step 1: Ключи настроек.** Для каждого ключа из findings, которого нет в дампе, найти фактический ключ с тем же смыслом (по `key`/`choices`/`type`/`default`) и заменить в `CORE_KEYS` или `MODULE_SETTINGS`. Если значение модуля инвертировано (например, «disable»), поменять `value`. Обновить ожидания в `client-settings.test.ts`/`modules.test.ts` на новые ключи, прогнать `npx vitest run agent/src`, закоммитить `fix(agent): align setting keys with Foundry v14`.

- [ ] **Step 2: Prime Performance.** Если модуль `fvtt-perf-optim` есть в `modules` дампа, заполнить `PRIME_SETTINGS` значениями его клиентских настроек из дампа: `soft` — значения по умолчанию модуля (`default`), `aggressive` — максимально производительные значения из `choices`/`range` (минимальные разрешения слоёв освещения, все оптимизации включены), `medium` — середина между ними. Добавить в `modules.test.ts` тест: `moduleValues(id => id === "fvtt-perf-optim", {...base, prime: "aggressive"})` содержит все ключи `PRIME_SETTINGS.aggressive`. Если модуля нет — оставить объекты пустыми и записать это в findings. Коммит `feat(agent): Prime Performance presets`.

- [ ] **Step 3: Разрешение канваса.** Если `resolutionTest.after` ≠ `target` (Foundry сбрасывает разрешение на `resize`), переключить `CanvasResolution.apply` на стратегию без события: `r.resolution = target; r.resize(r.screen.width, r.screen.height);` и повторить Task 10 Step 1 п.3 (F10) у пользователя. Если и она не держится — стратегия «до загрузки»: в фазе 1 агента переопределить `devicePixelRatio`
  ```ts
  Object.defineProperty(window, "devicePixelRatio", { get: () => base * levers.resMax, configurable: true });
  ```
  при `pixelRatioScaling = true` для Foundry, а адаптивность отключить (`adaptive = false` в пресетах Rust и пометка в findings). Обновить `resolution.test.ts` под выбранную стратегию. Коммит `fix(agent): canvas resolution strategy for v14`.

- [ ] **Step 4: Видео.** Если `videoCount` = 0 на сцене с видеофоном, найти в дампе/консоли фактическое место видео (например, `canvas.primary.background.sourceElement`, `canvas.tiles.placeables[].mesh.sourceElement`) и добавить эти пути в `collectVideos`. Тест: фейковый `canvas` с видео по новому пути (happy-dom, `document.createElement("video")`). Коммит `fix(agent): find v14 video textures`.

- [ ] **Step 5: Телеметрия.** Если записи в `stats.json` нет — IPC не выдан удалённому origin. Проверить в консоли игрового окна (F12) `typeof window.__TAURI_INTERNALS__?.invoke` и ошибку вызова. Если причина в шаблоне URL — поправить `remote.urls` в `capabilities/game.json` (например, перечислить `https://*` и `http://*`) и пересобрать. Если IPC для удалённых страниц недоступен в принципе — по спецификации §3.4 телеметрия отключается: `send()` уже молча ничего не делает, ЖК лаунчера показывает «замеров пока нет»; записать это в findings. Коммит `fix(core): telemetry capability for remote origin` (если были правки).

- [ ] **Step 6: Смена профиля на лету.** Если после Shift+F9 часть настроек (например, Performance Mode) применяется только после перезагрузки, показать в HUD подсказку с кнопкой. В `agent/src/i18n.ts` добавить `reload: "ПЕРЕЗАГРУЗИТЬ ДЛЯ ПОЛНОГО ЭФФЕКТА"` / `reload: "RELOAD FOR FULL EFFECT"`, в `Hud` — метод:
  ```ts
  	offerReload(): void {
  		const b = document.createElement("button");
  		b.textContent = this.t.reload;
  		b.onclick = () => location.reload();
  		this.menu.appendChild(b);
  		this.menu.hidden = false;
  		this.box.hidden = false;
  	}
  ```
  и вызвать `hud?.offerReload()` в конце `setProfile` в `main.ts` (только когда `persist === true`). Так как `writePreboot` уже записал новые значения в localStorage, перезагрузка поднимет Foundry с новым профилем. Коммит `feat(agent): offer reload after live profile switch`.

- [ ] **Step 7: Пересборка и повторная проверка** — `npm run build:agent && npm run tauri build -- --no-bundle`; пользователь повторяет F10: `fp.skipped` = 0, `resolutionTest` совпадает. Обновить findings, коммит.

---

### Task 12: Основа лаунчера — типы, правки рычагов, i18n, API, состояние, стили

**Files:**
- Create: `src/lib/types.ts`, `src/lib/levers.ts`, `src/lib/levers.test.ts`, `src/lib/i18n/ru.json`, `src/lib/i18n/en.json`, `src/lib/i18n/parity.test.ts`, `src/lib/i18n.svelte.ts`, `src/lib/api.ts`, `src/lib/mock.ts`, `src/lib/store.svelte.ts`, `src/styles/tokens.css`, `src/styles/base.css`
- Modify: `src/main.ts`

**Interfaces:**
- Consumes: команды Task 9 (имена и формы DTO), типы агента `agent/src/types.ts`.
- Produces:
  - `types.ts`: `ProfileId, VideoMode, PrimeLevel, Levers` (реэкспорт из агента), `Overrides, AngleBackend, EngineSettings, LocalePref, ThemePref, Settings, Server, FpsSummary, BenchResult, ServerStats, Notice, GpuInfo, StateDto, ProbeResult, LedState`
  - `levers.ts`: `type Scope`, `applyOverrides`, `profileFor`, `baseFor`, `overridesFor`, `resolved`, `withOverrides`, `countOverrides`
  - `i18n.svelte.ts`: `t(key: string, vars?: Record<string, string | number>): string`, `setLocale(l: "ru" | "en"): void`
  - `api.ts`: `api.{getState, saveServer, deleteServer, saveSettings, probe, launch, clearNotice}`, `windowControls.{minimize, close, destroy}`
  - `store.svelte.ts`: `app` (экземпляр `AppStore`), `type Screen`

- [ ] **Step 1: `src/lib/types.ts`**

```ts
export type { Levers, PrimeLevel, ProfileId, VideoMode } from "../../agent/src/types";
import type { Levers, ProfileId } from "../../agent/src/types";

export type Overrides = Partial<Levers>;
export type AngleBackend = "d3d11" | "d3d11on12" | "gl" | "vulkan";
export type LocalePref = "auto" | "ru" | "en";
export type ThemePref = "auto" | "day" | "night";
export type LedState = "ok" | "warn" | "err" | "off" | "pending";

export interface EngineSettings {
	angle: AngleBackend;
	diskCacheMb: number;
	extraArgs: string;
}

export interface Settings {
	schema: number;
	profile: ProfileId;
	overrides: Overrides;
	engine: EngineSettings;
	locale: LocalePref;
	theme: ThemePref;
}

export interface Server {
	id: string;
	name: string;
	url: string;
	profile: ProfileId | null;
	overrides: Overrides;
}

export interface FpsSummary {
	avg: number;
	low1: number;
	profile: ProfileId;
	at: number;
}

export interface BenchResult extends FpsSummary {
	min: number;
}

export interface ServerStats {
	lastSession: FpsSummary | null;
	lastBench: BenchResult | null;
}

export interface Notice {
	key: string;
	params: Record<string, string>;
}

export interface GpuInfo {
	name: string;
	vramMb: number;
	vendorId: number;
}

export interface StateDto {
	settings: Settings;
	servers: Server[];
	stats: Record<string, ServerStats>;
	presets: Record<ProfileId, Levers>;
	gpu: GpuInfo | null;
	recommended: ProfileId;
	notice: Notice | null;
	locale: "ru" | "en";
	version: string;
}

export interface ProbeResult {
	reachable: boolean;
	foundry: boolean;
	active: boolean;
	version: string | null;
	world: string | null;
	system: string | null;
	users: number | null;
}
```

- [ ] **Step 2: Падающие тесты `levers.ts`**

`src/lib/levers.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import { baseFor, countOverrides, resolved, withOverrides } from "./levers";
import type { Levers, StateDto } from "./types";

const preset = (maxFps: number, mipmap: boolean) => ({ maxFps, mipmap, resMin: 0.7 }) as Levers;

const dto = {
	settings: { profile: "balance", overrides: { maxFps: 30 } },
	servers: [
		{ id: "a", profile: "potato", overrides: { mipmap: true } },
		{ id: "b", profile: null, overrides: {} }
	],
	presets: { quality: preset(60, true), balance: preset(60, true), potato: preset(45, false) }
} as unknown as StateDto;

describe("levers resolution", () => {
	it("global scope = preset + global overrides", () => {
		expect(resolved(dto, { kind: "global" }).maxFps).toBe(30);
		expect(baseFor(dto, { kind: "global" }).maxFps).toBe(60);
	});

	it("server scope = own profile + global overrides + server overrides", () => {
		const r = resolved(dto, { kind: "server", id: "a" });
		expect(r.maxFps).toBe(30);
		expect(r.mipmap).toBe(true);
		expect(baseFor(dto, { kind: "server", id: "a" }).mipmap).toBe(false);
	});

	it("server without profile inherits global profile", () => {
		expect(resolved(dto, { kind: "server", id: "b" }).mipmap).toBe(true);
	});
});

describe("withOverrides", () => {
	it("drops an override equal to the base value (float-safe)", () => {
		const base = preset(60, true);
		expect(withOverrides({ resMin: 0.5 }, base, { resMin: 0.7000000001 })).toEqual({});
	});

	it("adds differing values and keeps others", () => {
		const base = preset(60, true);
		expect(withOverrides({ mipmap: false }, base, { maxFps: 45 })).toEqual({ mipmap: false, maxFps: 45 });
	});

	it("counts overrides", () => {
		expect(countOverrides({ maxFps: 45, mipmap: false })).toBe(2);
		expect(countOverrides({})).toBe(0);
	});
});
```

Run: `npx vitest run src/lib/levers.test.ts`
Expected: FAIL — модуль не найден.

- [ ] **Step 3: `src/lib/levers.ts`**

```ts
import type { Levers, Overrides, ProfileId, Server, StateDto } from "./types";

export type Scope = { kind: "global" } | { kind: "server"; id: string };

export function applyOverrides(base: Levers, o: Overrides): Levers {
	const defined = Object.entries(o).filter(([, v]) => v !== undefined && v !== null);
	return { ...base, ...Object.fromEntries(defined) } as Levers;
}

function serverOf(dto: StateDto, scope: Scope): Server | null {
	return scope.kind === "server" ? (dto.servers.find((s) => s.id === scope.id) ?? null) : null;
}

export function profileFor(dto: StateDto, scope: Scope): ProfileId {
	return serverOf(dto, scope)?.profile ?? dto.settings.profile;
}

/** То, от чего считаются правки текущего уровня: для сервера это уже профиль + глобальные правки. */
export function baseFor(dto: StateDto, scope: Scope): Levers {
	const preset = dto.presets[profileFor(dto, scope)];
	return scope.kind === "global" ? preset : applyOverrides(preset, dto.settings.overrides);
}

export function overridesFor(dto: StateDto, scope: Scope): Overrides {
	return scope.kind === "global" ? dto.settings.overrides : (serverOf(dto, scope)?.overrides ?? {});
}

export function resolved(dto: StateDto, scope: Scope): Levers {
	return applyOverrides(baseFor(dto, scope), overridesFor(dto, scope));
}

const same = (a: unknown, b: unknown) =>
	typeof a === "number" && typeof b === "number" ? Math.abs(a - b) < 1e-6 : a === b;

export function withOverrides(o: Overrides, base: Levers, patch: Partial<Levers>): Overrides {
	const next: Record<string, unknown> = { ...o };
	for (const [k, v] of Object.entries(patch)) {
		if (same(base[k as keyof Levers], v)) delete next[k];
		else next[k] = v;
	}
	return next as Overrides;
}

export function countOverrides(o: Overrides): number {
	return Object.values(o).filter((v) => v !== undefined && v !== null).length;
}
```

Run: `npx vitest run src/lib/levers.test.ts`
Expected: PASS.

- [ ] **Step 4: Словари**

`src/lib/i18n/ru.json`:
```json
{
  "window.minimize": "Свернуть",
  "window.close": "Закрыть",
  "main.slots": "Серверы",
  "main.emptySlot": "пустой слот",
  "main.noServer": "Добавьте сервер, чтобы начать",
  "main.addSlot": "+ СЛОТ",
  "main.launch": "ЗАПУСК",
  "main.launchSafe": "БЕЗОПАСНО",
  "main.launching": "ОТКРЫВАЕМ…",
  "main.safeHint": "SHIFT — безопасный режим, без оптимизаций",
  "main.tune": "НАСТРОЙКА",
  "main.lastSession": "последний сеанс",
  "main.lastBench": "последний замер",
  "main.noData": "замеров пока нет",
  "profile.quality": "КАЧ",
  "profile.balance": "БАЛ",
  "profile.potato": "КРТ",
  "profile.quality.long": "Качество",
  "profile.balance.long": "Баланс",
  "profile.potato.long": "Картошка",
  "profile.inherit": "КАК ВЕЗДЕ",
  "knob.label": "Профиль производительности",
  "led.ok": "онлайн · {world}",
  "led.idle": "сервер без мира",
  "led.err": "нет связи",
  "led.notFoundry": "это не Foundry",
  "led.pending": "проверка…",
  "slot.editShort": "ИЗМ",
  "slot.editAria": "Изменить слот {name}",
  "slot.new": "НОВЫЙ СЛОТ",
  "slot.edit": "СЛОТ {code}",
  "slot.name": "Название",
  "slot.url": "Адрес сервера",
  "slot.namePlaceholder": "Проклятие Страда",
  "slot.urlPlaceholder": "vtt.example.com:30000",
  "slot.profile": "Профиль",
  "slot.save": "СОХРАНИТЬ",
  "slot.cancel": "ОТМЕНА",
  "slot.delete": "УДАЛИТЬ",
  "slot.deleteConfirm": "ЕЩЁ РАЗ — УДАЛИТЬ",
  "slot.tune": "Тонкая настройка слота →",
  "tuning.title": "НАСТРОЙКА",
  "tuning.global": "ВСЕ СЕРВЕРЫ",
  "tuning.back": "← НАЗАД",
  "tuning.profile": "ПРОФИЛЬ",
  "tuning.changed": "+{n} ИЗМ",
  "tuning.reset": "сброс к профилю",
  "tuning.modifiedLegend": "● — изменено вручную",
  "tuning.resolution": "РАЗРЕШ.",
  "tuning.maxFps": "МАКС FPS",
  "tuning.unfocused": "БЕЗ ФОКУСА",
  "tuning.perfMode": "PERF MODE",
  "tuning.adaptive": "Адаптивное разрешение",
  "tuning.pixelRatio": "Масштаб под DPI",
  "tuning.lightAnimation": "Анимация света",
  "tuning.visionAnimation": "Анимация зрения",
  "tuning.mipmap": "Мипмапы",
  "tuning.uiBlur": "Blur интерфейса",
  "tuning.video": "ВИДЕОФОНЫ",
  "tuning.prime": "PRIME PERFORMANCE",
  "tuning.engine": "ДВИЖОК",
  "tuning.angle": "ANGLE",
  "tuning.cache": "КЭШ",
  "tuning.extraArgs": "Доп. флаги Chromium",
  "tuning.extraArgsHint": "--flag=value через пробел; применяются при следующем запуске",
  "tuning.language": "ЯЗЫК",
  "tuning.theme": "ТЕМА",
  "tuning.engineLine": "ДВИЖОК: ANGLE {angle} · КЭШ {cache} ГБ",
  "video.play": "ИГРАЮТ",
  "video.pauseUnfocused": "ПАУЗА БЕЗ ФОКУСА",
  "video.static": "СТАТИКА",
  "prime.soft": "МЯГКО",
  "prime.medium": "СРЕДНЕ",
  "prime.aggressive": "МАКС",
  "theme.auto": "АВТО",
  "theme.day": "ДЕНЬ",
  "theme.night": "НОЧЬ",
  "locale.auto": "АВТО",
  "gpu.unknown": "GPU не определён",
  "unit.gb": "ГБ",
  "err.nameRequired": "Введите название",
  "err.urlInvalid": "Не похоже на адрес. Пример: vtt.example.com:30000",
  "err.saveFailed": "Не удалось сохранить на диск",
  "err.serverMissing": "Слот не найден — обновите список",
  "notice.storeCorrupted": "Файл {file} был повреждён — сохранена копия .bak",
  "notice.engineFallback": "WebGL упал на {from} — движок переключён на {to}",
  "notice.launchFailed": "Игра не открылась. Если только что закрыли её — подождите пару секунд"
}
```

`src/lib/i18n/en.json`:
```json
{
  "window.minimize": "Minimize",
  "window.close": "Close",
  "main.slots": "Servers",
  "main.emptySlot": "empty slot",
  "main.noServer": "Add a server to get started",
  "main.addSlot": "+ SLOT",
  "main.launch": "LAUNCH",
  "main.launchSafe": "SAFE MODE",
  "main.launching": "OPENING…",
  "main.safeHint": "SHIFT — safe mode, no optimizations",
  "main.tune": "TUNING",
  "main.lastSession": "last session",
  "main.lastBench": "last benchmark",
  "main.noData": "no measurements yet",
  "profile.quality": "QLT",
  "profile.balance": "BAL",
  "profile.potato": "POT",
  "profile.quality.long": "Quality",
  "profile.balance.long": "Balance",
  "profile.potato.long": "Potato",
  "profile.inherit": "SAME AS ALL",
  "knob.label": "Performance profile",
  "led.ok": "online · {world}",
  "led.idle": "no world loaded",
  "led.err": "unreachable",
  "led.notFoundry": "not a Foundry server",
  "led.pending": "checking…",
  "slot.editShort": "EDIT",
  "slot.editAria": "Edit slot {name}",
  "slot.new": "NEW SLOT",
  "slot.edit": "SLOT {code}",
  "slot.name": "Name",
  "slot.url": "Server address",
  "slot.namePlaceholder": "Curse of Strahd",
  "slot.urlPlaceholder": "vtt.example.com:30000",
  "slot.profile": "Profile",
  "slot.save": "SAVE",
  "slot.cancel": "CANCEL",
  "slot.delete": "DELETE",
  "slot.deleteConfirm": "AGAIN TO DELETE",
  "slot.tune": "Tune this slot →",
  "tuning.title": "TUNING",
  "tuning.global": "ALL SERVERS",
  "tuning.back": "← BACK",
  "tuning.profile": "PROFILE",
  "tuning.changed": "+{n} MOD",
  "tuning.reset": "reset to profile",
  "tuning.modifiedLegend": "● — changed manually",
  "tuning.resolution": "RES.",
  "tuning.maxFps": "MAX FPS",
  "tuning.unfocused": "UNFOCUSED",
  "tuning.perfMode": "PERF MODE",
  "tuning.adaptive": "Adaptive resolution",
  "tuning.pixelRatio": "DPI scaling",
  "tuning.lightAnimation": "Light animation",
  "tuning.visionAnimation": "Vision animation",
  "tuning.mipmap": "Mipmaps",
  "tuning.uiBlur": "UI blur",
  "tuning.video": "VIDEO BACKGROUNDS",
  "tuning.prime": "PRIME PERFORMANCE",
  "tuning.engine": "ENGINE",
  "tuning.angle": "ANGLE",
  "tuning.cache": "CACHE",
  "tuning.extraArgs": "Extra Chromium flags",
  "tuning.extraArgsHint": "--flag=value separated by spaces; applied on next launch",
  "tuning.language": "LANGUAGE",
  "tuning.theme": "THEME",
  "tuning.engineLine": "ENGINE: ANGLE {angle} · CACHE {cache} GB",
  "video.play": "PLAY",
  "video.pauseUnfocused": "PAUSE UNFOCUSED",
  "video.static": "STATIC",
  "prime.soft": "SOFT",
  "prime.medium": "MEDIUM",
  "prime.aggressive": "MAX",
  "theme.auto": "AUTO",
  "theme.day": "DAY",
  "theme.night": "NIGHT",
  "locale.auto": "AUTO",
  "gpu.unknown": "GPU not detected",
  "unit.gb": "GB",
  "err.nameRequired": "Enter a name",
  "err.urlInvalid": "Doesn't look like an address. Example: vtt.example.com:30000",
  "err.saveFailed": "Couldn't save to disk",
  "err.serverMissing": "Slot not found — refresh the list",
  "notice.storeCorrupted": "{file} was damaged — a .bak copy was kept",
  "notice.engineFallback": "WebGL crashed on {from} — engine switched to {to}",
  "notice.launchFailed": "The game didn't open. If you just closed it, wait a couple of seconds"
}
```

`src/lib/i18n/parity.test.ts`:
```ts
import { describe, expect, it } from "vitest";
import en from "./en.json";
import ru from "./ru.json";

const vars = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe("i18n dictionaries", () => {
	it("have identical keys", () => {
		expect(Object.keys(en).sort()).toEqual(Object.keys(ru).sort());
	});

	it("use the same placeholders", () => {
		for (const k of Object.keys(ru) as (keyof typeof ru)[]) expect(vars(en[k]), k).toEqual(vars(ru[k]));
	});
});
```

Run: `npx vitest run src/lib`
Expected: PASS.

- [ ] **Step 5: `src/lib/i18n.svelte.ts`**

```ts
import en from "./i18n/en.json";
import ru from "./i18n/ru.json";

const dicts: Record<"ru" | "en", Record<string, string>> = { ru, en };
let current = $state<"ru" | "en">("ru");

export function setLocale(l: "ru" | "en"): void {
	current = l;
	document.documentElement.lang = l;
}

export function t(key: string, vars?: Record<string, string | number>): string {
	const s = dicts[current][key] ?? dicts.ru[key] ?? key;
	return vars ? s.replace(/\{(\w+)\}/g, (_, k: string) => String(vars[k] ?? "")) : s;
}
```

- [ ] **Step 6: `src/lib/api.ts` и `src/lib/mock.ts`**

`src/lib/api.ts`:
```ts
import type { ProbeResult, Server, Settings, StateDto } from "./types";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** В браузерном превью (vite dev без Tauri) команды обслуживает mock.ts. */
async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	if (inTauri) {
		const { invoke } = await import("@tauri-apps/api/core");
		return invoke<T>(cmd, args);
	}
	const { mockInvoke } = await import("./mock");
	return (await mockInvoke(cmd, args ?? {})) as T;
}

export const api = {
	getState: () => call<StateDto>("get_state"),
	saveServer: (server: Server) => call<Server[]>("save_server", { server }),
	deleteServer: (id: string) => call<Server[]>("delete_server", { id }),
	saveSettings: (settings: Settings) => call<Settings>("save_settings", { settings }),
	probe: (url: string) => call<ProbeResult>("probe_server", { url }),
	launch: (serverId: string, safeMode: boolean) => call<void>("launch", { serverId, safeMode }),
	clearNotice: () => call<void>("clear_notice")
};

async function currentWindow() {
	if (!inTauri) return null;
	const { getCurrentWindow } = await import("@tauri-apps/api/window");
	return getCurrentWindow();
}

export const windowControls = {
	minimize: async () => void (await currentWindow())?.minimize(),
	close: async () => void (await currentWindow())?.close(),
	destroy: async () => void (await currentWindow())?.destroy()
};
```

`src/lib/mock.ts`:
```ts
import type { Levers, ProfileId, Server, Settings, StateDto } from "./types";

const lv = (o: Partial<Levers>): Levers => ({
	perfMode: 1,
	maxFps: 60,
	resMin: 0.7,
	resMax: 1,
	adaptive: true,
	pixelRatioScaling: false,
	lightAnimation: true,
	visionAnimation: false,
	mipmap: true,
	video: "pauseUnfocused",
	uiBlur: false,
	sequencer: true,
	fxmaster: true,
	unfocusedFps: 15,
	prime: "medium",
	...o
});

/** Копия пресетов из src-tauri/src/profile.rs — только для превью в браузере. */
const presets: Record<ProfileId, Levers> = {
	quality: lv({ perfMode: 2, resMin: 1, adaptive: false, pixelRatioScaling: true, visionAnimation: true, video: "play", uiBlur: true, unfocusedFps: 60, prime: "soft" }),
	balance: lv({}),
	potato: lv({ perfMode: 0, maxFps: 45, resMin: 0.55, resMax: 0.85, lightAnimation: false, mipmap: false, video: "static", sequencer: false, fxmaster: false, unfocusedFps: 10, prime: "aggressive" })
};

let settings: Settings = {
	schema: 1,
	profile: "balance",
	overrides: {},
	engine: { angle: "d3d11", diskCacheMb: 2048, extraArgs: "" },
	locale: "auto",
	theme: "auto"
};

let servers: Server[] = [
	{ id: "a1", name: "Проклятие Страда", url: "https://vtt.example.com/", profile: null, overrides: {} },
	{ id: "a2", name: "Ваншот по пятницам", url: "http://192.168.1.40:30000/", profile: "quality", overrides: { maxFps: 45 } }
];

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function mockInvoke(cmd: string, args: Record<string, unknown>): Promise<unknown> {
	await delay(120);
	switch (cmd) {
		case "get_state":
			return {
				settings,
				servers,
				stats: { a1: { lastSession: { avg: 52.4, low1: 31, profile: "balance", at: 0 }, lastBench: null } },
				presets,
				gpu: { name: "NVIDIA GeForce GTX 1060 6GB", vramMb: 6144, vendorId: 0x10de },
				recommended: "balance",
				notice: null,
				locale: "ru",
				version: "0.1.0-preview"
			} satisfies StateDto;
		case "save_server": {
			const s = args.server as Server;
			if (!s.name.trim()) throw "err.nameRequired";
			if (!/^(https?:\/\/)?[\w.-]+(:\d+)?(\/.*)?$/.test(s.url.trim())) throw "err.urlInvalid";
			servers = servers.some((x) => x.id === s.id) ? servers.map((x) => (x.id === s.id ? s : x)) : [...servers, s];
			return servers;
		}
		case "delete_server":
			servers = servers.filter((s) => s.id !== args.id);
			return servers;
		case "save_settings":
			settings = args.settings as Settings;
			return settings;
		case "probe_server":
			await delay(700);
			return String(args.url).includes("192.168")
				? { reachable: false, foundry: false, active: false, version: null, world: null, system: null, users: null }
				: { reachable: true, foundry: true, active: true, version: "14.349", world: "strahd", system: "dnd5e", users: 3 };
		case "launch":
			console.info("[preview] launch", args);
			return null;
		case "clear_notice":
			return null;
		default:
			throw `unknown command ${cmd}`;
	}
}
```

- [ ] **Step 7: `src/lib/store.svelte.ts`**

```ts
import { api, windowControls } from "./api";
import { setLocale, t } from "./i18n.svelte";
import { baseFor, overridesFor, type Scope, withOverrides } from "./levers";
import type { Levers, Overrides, ProbeResult, ProfileId, Server, Settings, StateDto } from "./types";

export type Screen = "main" | "tuning" | "slot";

const snap = <T>(v: T): T => $state.snapshot(v) as T;
const systemLocale = (): "ru" | "en" => (navigator.language.toLowerCase().startsWith("ru") ? "ru" : "en");

class AppStore {
	dto = $state<StateDto | null>(null);
	selectedId = $state<string | null>(null);
	screen = $state<Screen>("main");
	editing = $state<Server | null>(null);
	scope = $state<Scope>({ kind: "global" });
	probes = $state<Record<string, ProbeResult | "pending">>({});
	error = $state<string | null>(null);
	launching = $state(false);

	get selected(): Server | null {
		return this.dto?.servers.find((s) => s.id === this.selectedId) ?? null;
	}

	get message(): string | null {
		if (this.error) return t(this.error);
		const n = this.dto?.notice;
		return n ? t(n.key, n.params) : null;
	}

	async load(): Promise<void> {
		const dto = await api.getState();
		this.dto = dto;
		this.applyLocale();
		this.selectedId = dto.servers[0]?.id ?? null;
		for (const s of dto.servers) void this.probe(s);
	}

	private applyLocale(): void {
		const pref = this.dto?.settings.locale ?? "auto";
		setLocale(pref === "auto" ? systemLocale() : pref);
	}

	async probe(s: Server): Promise<void> {
		this.probes[s.id] = "pending";
		this.probes[s.id] = await api.probe(s.url);
	}

	async dismiss(): Promise<void> {
		if (this.error) {
			this.error = null;
			return;
		}
		if (this.dto?.notice) {
			await api.clearNotice();
			this.dto.notice = null;
		}
	}

	newSlot(): void {
		this.editing = { id: crypto.randomUUID(), name: "", url: "", profile: null, overrides: {} };
		this.screen = "slot";
	}

	editSlot(s: Server): void {
		this.editing = snap(s);
		this.screen = "slot";
	}

	openTuning(scope: Scope): void {
		this.scope = scope;
		this.screen = "tuning";
	}

	closeScreen(): void {
		this.screen = "main";
		this.editing = null;
	}

	/** @returns i18n-ключ ошибки или null */
	async saveServer(s: Server): Promise<string | null> {
		try {
			const servers = await api.saveServer(snap(s));
			this.dto!.servers = servers;
			this.selectedId = s.id;
			this.closeScreen();
			const saved = servers.find((x) => x.id === s.id);
			if (saved) void this.probe(saved);
			return null;
		} catch (e) {
			return String(e);
		}
	}

	async deleteServer(id: string): Promise<void> {
		try {
			this.dto!.servers = await api.deleteServer(id);
			delete this.dto!.stats[id];
			if (this.selectedId === id) this.selectedId = this.dto!.servers[0]?.id ?? null;
			this.closeScreen();
		} catch (e) {
			this.error = String(e);
		}
	}

	async saveSettings(patch: Partial<Settings>): Promise<void> {
		try {
			this.dto!.settings = await api.saveSettings({ ...snap(this.dto!.settings), ...patch });
			this.applyLocale();
		} catch (e) {
			this.error = String(e);
		}
	}

	private async patchServer(id: string, patch: Partial<Server>): Promise<void> {
		const s = this.dto!.servers.find((x) => x.id === id);
		if (!s) return;
		try {
			this.dto!.servers = await api.saveServer({ ...snap(s), ...patch });
		} catch (e) {
			this.error = String(e);
		}
	}

	/** Ручка на главном экране: профиль выбранного сервера (или глобальный, если серверов нет). */
	async setKnob(p: ProfileId): Promise<void> {
		const s = this.selected;
		if (s) await this.patchServer(s.id, { profile: p });
		else await this.saveSettings({ profile: p });
	}

	async setScopeProfile(p: ProfileId | null): Promise<void> {
		if (this.scope.kind === "global") {
			if (p) await this.saveSettings({ profile: p });
		} else await this.patchServer(this.scope.id, { profile: p });
	}

	private async saveOverrides(next: Overrides): Promise<void> {
		if (this.scope.kind === "global") await this.saveSettings({ overrides: next });
		else await this.patchServer(this.scope.id, { overrides: next });
	}

	async setLevers(patch: Partial<Levers>): Promise<void> {
		const dto = this.dto!;
		await this.saveOverrides(withOverrides(snap(overridesFor(dto, this.scope)), baseFor(dto, this.scope), patch));
	}

	async resetOverrides(): Promise<void> {
		await this.saveOverrides({});
	}

	async launch(safe: boolean): Promise<void> {
		const s = this.selected;
		if (!s || this.launching) return;
		this.launching = true;
		try {
			await api.launch(s.id, safe);
			await windowControls.destroy();
		} catch (e) {
			this.error = String(e);
		} finally {
			this.launching = false;
		}
	}
}

export const app = new AppStore();
```

- [ ] **Step 8: Стили и подключение**

`src/styles/tokens.css`:
```css
:root {
	--panel: #d8d6d0;
	--face: #e6e4de;
	--face-2: #f2f0eb;
	--ink: #2a2a28;
	--ink-2: #6e6b64;
	--ink-3: #9e9b93;
	--line: #bdbab2;
	--inverse-bg: #2a2a28;
	--inverse-fg: #e6e4de;
	--signal: #ff5a1f;
	--signal-deep: #b8420f;
	--signal-ink: #1a1a18;
	--signal-text: #b8420f;
	--lcd-bg: #1e2a1e;
	--lcd-fg: #b8f28a;
	--lcd-dim: #7fa866;
	--lcd-ghost: #2f422f;
	--led-ok: #3fa34d;
	--led-warn: #e0a100;
	--led-err: #d2361b;
	--led-off: #bdbab2;
	--font-ui: "Geologica", system-ui, sans-serif;
	--font-mono: "Martian Mono", ui-monospace, Consolas, monospace;
	--ease-detent: cubic-bezier(0.3, 1.6, 0.5, 1);
	--ease-out: cubic-bezier(0.2, 0.8, 0.2, 1);
	color-scheme: light;
}

:root[data-theme="night"] {
	--panel: #111110;
	--face: #1c1c1a;
	--face-2: #262623;
	--ink: #e6e4de;
	--ink-2: #a19e96;
	--ink-3: #6e6b64;
	--line: #3a3a36;
	--inverse-bg: #e6e4de;
	--inverse-fg: #1c1c1a;
	--signal-text: #ff7a45;
	--led-off: #3a3a36;
	color-scheme: dark;
}
```

`src/styles/base.css`:
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
	background: var(--panel);
	color: var(--ink);
	font: 400 13px/1.4 var(--font-ui);
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
	outline: 2px solid var(--signal);
	outline-offset: 2px;
}

.mono {
	font-family: var(--font-mono);
	font-size: 11px;
	letter-spacing: 0.02em;
}

.dot {
	display: inline-block;
	width: 6px;
	height: 6px;
	margin-left: 6px;
	border-radius: 50%;
	background: var(--signal);
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

`src/main.ts` (замена):
```ts
import "@fontsource/geologica/400.css";
import "@fontsource/geologica/500.css";
import "@fontsource/geologica/800.css";
import "@fontsource/martian-mono/400.css";
import "@fontsource/martian-mono/500.css";
import "./styles/tokens.css";
import "./styles/base.css";
import { mount } from "svelte";
import App from "./App.svelte";

const app = mount(App, { target: document.getElementById("app")! });

export default app;
```

- [ ] **Step 9: Проверки и коммит**

```bash
npx vitest run
npm run check
git add src
git commit -m "feat(ui): launcher foundation — types, levers, i18n, api, state, tokens

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: тесты PASS; `svelte-check` — 0 ошибок (App.svelte пока заглушка).

---

### Task 13: Компоненты «Пульта»

**Files:**
- Create: `src/components/TitleBar.svelte`, `Lcd.svelte`, `Led.svelte`, `Knob.svelte`, `Slot.svelte`, `LaunchButton.svelte`, `Fader.svelte`, `Toggle.svelte`, `Segmented.svelte`

**Interfaces:**
- Consumes: `t` (i18n), `windowControls` (api), типы из `src/lib/types.ts`.
- Produces (props):
  - `TitleBar { channel: string }`
  - `Lcd { value: number | null; unit: string; caption?: string; digits?: number }`
  - `Led { state: LedState; label: string }`
  - `Knob { value: ProfileId; onchange: (p: ProfileId) => void }`
  - `Slot { server: Server; code: string; selected: boolean; probe: ProbeResult | "pending" | undefined; onselect(): void; onlaunch(): void; onedit(): void }`
  - `LaunchButton { busy: boolean; onlaunch: (safe: boolean) => void }`
  - `Fader { label: string; value: number; min: number; max: number; step: number; format: (v: number) => string; modified?: boolean; onchange: (v: number) => void }`
  - `Toggle { label: string; checked: boolean; modified?: boolean; onchange: (v: boolean) => void }`
  - `Segmented<T> { label: string; options: { value: T; label: string }[]; value: T; modified?: boolean; vertical?: boolean; onchange: (v: T) => void }`

- [ ] **Step 1: `TitleBar.svelte`, `Led.svelte`, `Lcd.svelte`**

`src/components/TitleBar.svelte`:
```svelte
<script lang="ts">
	import { windowControls } from "../lib/api";
	import { t } from "../lib/i18n.svelte";

	let { channel }: { channel: string } = $props();
</script>

<header class="bar" data-tauri-drag-region>
	<b class="brand" data-tauri-drag-region>FOUNDRY/PERFORMANCE</b>
	<span class="mono ch" data-tauri-drag-region>{channel}</span>
	<div class="ctl">
		<button class="mono" aria-label={t("window.minimize")} onclick={() => windowControls.minimize()}>—</button>
		<button class="mono close" aria-label={t("window.close")} onclick={() => windowControls.close()}>✕</button>
	</div>
</header>

<style>
	.bar {
		display: flex;
		align-items: center;
		gap: 14px;
		height: 40px;
		padding-left: 18px;
		background: var(--face);
	}
	.brand {
		font-weight: 800;
		font-size: 13px;
		letter-spacing: 0.14em;
	}
	.ch {
		margin-left: auto;
		color: var(--ink-2);
	}
	.ctl {
		display: flex;
		height: 100%;
	}
	.ctl button {
		width: 46px;
		height: 100%;
		display: grid;
		place-items: center;
	}
	.ctl button:hover {
		background: var(--face-2);
	}
	.ctl .close:hover {
		background: var(--signal);
		color: var(--signal-ink);
	}
</style>
```

`src/components/Led.svelte`:
```svelte
<script lang="ts">
	import type { LedState } from "../lib/types";

	let { state, label }: { state: LedState; label: string } = $props();
</script>

<span class="led {state}" role="img" aria-label={label} title={label}></span>

<style>
	.led {
		flex: none;
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--led-off);
	}
	.ok {
		background: var(--led-ok);
	}
	.warn {
		background: var(--led-warn);
	}
	.err {
		background: var(--led-err);
	}
	.pending {
		background: var(--signal);
		animation: blink 0.9s steps(2, start) infinite;
	}
	@keyframes blink {
		to {
			visibility: hidden;
		}
	}
</style>
```

`src/components/Lcd.svelte` (цифры «перекатываются» барабаном):
```svelte
<script lang="ts">
	let { value, unit, caption, digits = 3 }: { value: number | null; unit: string; caption?: string; digits?: number } = $props();

	const REEL = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
	const cells = $derived(
		value === null
			? Array.from({ length: digits }, () => null)
			: String(Math.min(10 ** digits - 1, Math.max(0, Math.round(value))))
					.padStart(digits, "0")
					.split("")
					.map(Number)
	);
</script>

<div class="lcd" role="img" aria-label={value === null ? `— ${unit}` : `${Math.round(value)} ${unit}`}>
	<div class="digits">
		{#each cells as d, i (i)}
			<span class="cell">
				{#if d === null}
					<span class="ghost">-</span>
				{:else}
					<span class="reel" style:transform={`translateY(${-d}em)`}>
						{#each REEL as n (n)}<span>{n}</span>{/each}
					</span>
				{/if}
			</span>
		{/each}
		<span class="unit">{unit}</span>
	</div>
	{#if caption}<div class="cap">{caption}</div>{/if}
</div>

<style>
	.lcd {
		background: var(--lcd-bg);
		color: var(--lcd-fg);
		padding: 12px 14px 10px;
		font-family: var(--font-mono);
	}
	.digits {
		display: flex;
		align-items: baseline;
		justify-content: flex-end;
		font-size: 36px;
		line-height: 1;
		font-variant-numeric: tabular-nums;
	}
	.cell {
		display: inline-block;
		width: 0.74em;
		height: 1em;
		overflow: hidden;
		text-align: center;
	}
	.reel {
		display: flex;
		flex-direction: column;
		transition: transform 0.45s var(--ease-out);
	}
	.reel span {
		height: 1em;
	}
	.ghost {
		color: var(--lcd-ghost);
	}
	.unit {
		margin-left: 6px;
		font-size: 11px;
		color: var(--lcd-dim);
	}
	.cap {
		margin-top: 6px;
		font-size: 11px;
		text-align: right;
		color: var(--lcd-dim);
	}
</style>
```

- [ ] **Step 2: `Knob.svelte`, `Slot.svelte`, `LaunchButton.svelte`**

`src/components/Knob.svelte`:
```svelte
<script lang="ts">
	import { t } from "../lib/i18n.svelte";
	import type { ProfileId } from "../lib/types";

	let { value, onchange }: { value: ProfileId; onchange: (p: ProfileId) => void } = $props();

	const ORDER: ProfileId[] = ["quality", "balance", "potato"];
	const ANGLE: Record<ProfileId, number> = { quality: -60, balance: 0, potato: 60 };

	function step(delta: number) {
		const i = ORDER.indexOf(value) + delta;
		if (i >= 0 && i < ORDER.length) onchange(ORDER[i]);
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === "ArrowRight" || e.key === "ArrowUp") step(1);
		else if (e.key === "ArrowLeft" || e.key === "ArrowDown") step(-1);
		else return;
		e.preventDefault();
	}
</script>

<div class="wrap">
	<div
		class="knob"
		role="slider"
		tabindex="0"
		aria-label={t("knob.label")}
		aria-valuemin={0}
		aria-valuemax={2}
		aria-valuenow={ORDER.indexOf(value)}
		aria-valuetext={t(`profile.${value}.long`)}
		{onkeydown}
		onwheel={(e) => step(e.deltaY > 0 ? 1 : -1)}
		onclick={() => step(value === "potato" ? -2 : 1)}
	>
		<div class="cap" style:transform={`rotate(${ANGLE[value]}deg)`}><i></i></div>
		{#each ORDER as p (p)}<span class="tick" style:transform={`rotate(${ANGLE[p]}deg)`}></span>{/each}
	</div>
	<div class="scale mono">
		{#each ORDER as p (p)}
			<button class:on={p === value} onclick={() => onchange(p)}>{t(`profile.${p}`)}</button>
		{/each}
	</div>
</div>

<style>
	.wrap {
		display: grid;
		gap: 10px;
		justify-items: center;
	}
	.knob {
		position: relative;
		width: 116px;
		height: 116px;
		border-radius: 50%;
		background: var(--line);
		display: grid;
		place-items: center;
		cursor: pointer;
	}
	.cap {
		width: 88px;
		height: 88px;
		border-radius: 50%;
		background: var(--inverse-bg);
		position: relative;
		transition: transform 0.32s var(--ease-detent);
	}
	.cap i {
		position: absolute;
		left: 50%;
		top: 8px;
		width: 5px;
		height: 28px;
		margin-left: -2.5px;
		background: var(--signal);
	}
	.tick {
		position: absolute;
		inset: 0;
		pointer-events: none;
	}
	.tick::before {
		content: "";
		position: absolute;
		left: 50%;
		top: -9px;
		width: 2px;
		height: 6px;
		margin-left: -1px;
		background: var(--ink-3);
	}
	.scale {
		display: flex;
		justify-content: space-between;
		width: 100%;
	}
	.scale button {
		padding: 3px 6px;
		color: var(--ink-2);
	}
	.scale button.on {
		color: var(--signal-text);
	}
</style>
```

`src/components/Slot.svelte`:
```svelte
<script lang="ts">
	import { t } from "../lib/i18n.svelte";
	import type { LedState, ProbeResult, Server } from "../lib/types";
	import Led from "./Led.svelte";

	let {
		server,
		code,
		selected,
		probe,
		onselect,
		onlaunch,
		onedit
	}: {
		server: Server;
		code: string;
		selected: boolean;
		probe: ProbeResult | "pending" | undefined;
		onselect: () => void;
		onlaunch: () => void;
		onedit: () => void;
	} = $props();

	const led = $derived<LedState>(
		probe === undefined ? "off" : probe === "pending" ? "pending" : !probe.reachable || !probe.foundry ? "err" : probe.active ? "ok" : "warn"
	);
	const status = $derived(
		probe === undefined
			? ""
			: probe === "pending"
				? t("led.pending")
				: !probe.reachable
					? t("led.err")
					: !probe.foundry
						? t("led.notFoundry")
						: probe.active
							? t("led.ok", { world: probe.world ?? "" })
							: t("led.idle")
	);
	const host = $derived(server.url.replace(/^https?:\/\//, "").replace(/\/$/, ""));
</script>

<div
	class="slot"
	class:selected
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
	<span class="code mono">{code}</span>
	<span class="body">
		<span class="name">{server.name}</span>
		<span class="meta mono">{host}{#if status}&nbsp;· {status}{/if}</span>
	</span>
	<button
		class="edit mono"
		aria-label={t("slot.editAria", { name: server.name })}
		onclick={(e) => {
			e.stopPropagation();
			onedit();
		}}>{t("slot.editShort")}</button
	>
	<Led state={led} label={status} />
</div>

<style>
	.slot {
		display: grid;
		grid-template-columns: 34px minmax(0, 1fr) auto 12px;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		background: var(--face-2);
		cursor: pointer;
		transition: background 0.12s;
	}
	.slot:hover {
		background: var(--face);
	}
	.slot.selected {
		background: var(--inverse-bg);
		color: var(--inverse-fg);
	}
	.code {
		color: var(--ink-2);
	}
	.selected .code,
	.selected .meta {
		color: var(--ink-3);
	}
	.body {
		display: grid;
		min-width: 0;
	}
	.name {
		font-weight: 500;
		font-size: 17px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.meta {
		color: var(--ink-2);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.edit {
		padding: 4px 8px;
		border: 1px solid var(--line);
		opacity: 0;
		transition: opacity 0.12s;
	}
	.slot:hover .edit,
	.slot:focus-within .edit,
	.selected .edit {
		opacity: 1;
	}
</style>
```

`src/components/LaunchButton.svelte`:
```svelte
<script lang="ts">
	import { t } from "../lib/i18n.svelte";

	let { busy, onlaunch }: { busy: boolean; onlaunch: (safe: boolean) => void } = $props();
	let shift = $state(false);

	$effect(() => {
		const track = (e: KeyboardEvent) => (shift = e.shiftKey);
		const reset = () => (shift = false);
		window.addEventListener("keydown", track);
		window.addEventListener("keyup", track);
		window.addEventListener("blur", reset);
		return () => {
			window.removeEventListener("keydown", track);
			window.removeEventListener("keyup", track);
			window.removeEventListener("blur", reset);
		};
	});
</script>

<div class="group">
	<button class="launch" class:safe={shift} aria-busy={busy} onclick={(e) => onlaunch(e.shiftKey)}>
		<span>{busy ? t("main.launching") : shift ? t("main.launchSafe") : t("main.launch")}</span>
		<span aria-hidden="true">▶</span>
	</button>
	<span class="hint mono">{t("main.safeHint")}</span>
</div>

<style>
	.group {
		display: grid;
		gap: 6px;
	}
	.launch {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 15px 18px;
		background: var(--signal);
		color: var(--signal-ink);
		font-weight: 800;
		font-size: 20px;
		letter-spacing: 0.06em;
		box-shadow: 0 3px 0 var(--signal-deep);
		transition:
			transform 0.06s,
			box-shadow 0.06s,
			background 0.12s;
	}
	.launch:active {
		transform: translateY(3px);
		box-shadow: 0 0 0 var(--signal-deep);
	}
	.launch.safe {
		background: var(--inverse-bg);
		color: var(--inverse-fg);
		box-shadow: 0 3px 0 var(--ink-3);
	}
	.launch[aria-busy="true"] {
		cursor: progress;
	}
	.hint {
		color: var(--ink-2);
	}
</style>
```

- [ ] **Step 3: `Fader.svelte`, `Toggle.svelte`, `Segmented.svelte`**

`src/components/Fader.svelte` (значение отправляется на `change`, пока тянешь — только отображается):
```svelte
<script lang="ts">
	let {
		label,
		value,
		min,
		max,
		step,
		format,
		modified = false,
		onchange
	}: {
		label: string;
		value: number;
		min: number;
		max: number;
		step: number;
		format: (v: number) => string;
		modified?: boolean;
		onchange: (v: number) => void;
	} = $props();

	let live = $state(0);
	$effect(() => {
		live = value;
	});
</script>

<label class="fader">
	<span class="lbl mono">{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
	<input
		type="range"
		{min}
		{max}
		{step}
		value={live}
		aria-valuetext={format(live)}
		oninput={(e) => (live = Number(e.currentTarget.value))}
		onchange={(e) => onchange(Number(e.currentTarget.value))}
	/>
	<span class="val mono">{format(live)}</span>
</label>

<style>
	.fader {
		display: grid;
		justify-items: center;
		align-content: start;
		gap: 10px;
		padding: 14px 8px 12px;
		background: var(--face);
	}
	.lbl,
	.val {
		white-space: nowrap;
	}
	input {
		appearance: none;
		writing-mode: vertical-lr;
		direction: rtl;
		width: 28px;
		height: 108px;
		margin: 0;
		background: linear-gradient(var(--ink), var(--ink)) center / 6px 100% no-repeat;
		cursor: ns-resize;
	}
	input::-webkit-slider-runnable-track {
		background: transparent;
	}
	input::-webkit-slider-thumb {
		appearance: none;
		width: 28px;
		height: 13px;
		background: var(--signal);
		border: 1px solid var(--signal-ink);
	}
</style>
```

`src/components/Toggle.svelte`:
```svelte
<script lang="ts">
	let {
		label,
		checked,
		modified = false,
		onchange
	}: { label: string; checked: boolean; modified?: boolean; onchange: (v: boolean) => void } = $props();
</script>

<button class="toggle" role="switch" aria-checked={checked} onclick={() => onchange(!checked)}>
	<span>{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
	<span class="sw" class:on={checked}><i></i></span>
</button>

<style>
	.toggle {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		width: 100%;
		padding: 4px 0;
		text-align: left;
	}
	.sw {
		position: relative;
		flex: none;
		width: 32px;
		height: 16px;
		background: var(--inverse-bg);
	}
	.sw i {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 12px;
		height: 12px;
		background: var(--inverse-fg);
		transition:
			transform 0.14s var(--ease-detent),
			background 0.14s;
	}
	.sw.on i {
		transform: translateX(16px);
		background: var(--signal);
	}
</style>
```

`src/components/Segmented.svelte`:
```svelte
<script lang="ts" generics="T extends string | number">
	let {
		label,
		options,
		value,
		modified = false,
		vertical = false,
		onchange
	}: {
		label: string;
		options: { value: T; label: string }[];
		value: T;
		modified?: boolean;
		vertical?: boolean;
		onchange: (v: T) => void;
	} = $props();
</script>

<div class="seg" class:vertical role="radiogroup" aria-label={label}>
	<span class="lbl mono">{label}{#if modified}<i class="dot" aria-hidden="true"></i>{/if}</span>
	<div class="opts">
		{#each options as o (o.value)}
			<button class="mono" role="radio" aria-checked={o.value === value} class:on={o.value === value} onclick={() => onchange(o.value)}>
				{o.label}
			</button>
		{/each}
	</div>
</div>

<style>
	.seg {
		display: grid;
		gap: 6px;
	}
	.opts {
		display: flex;
		flex-wrap: wrap;
		gap: 3px;
	}
	.vertical {
		justify-items: center;
		align-content: start;
		padding: 14px 8px 12px;
		background: var(--face);
	}
	.vertical .opts {
		flex-direction: column;
		width: 76px;
	}
	button {
		padding: 4px 8px;
		border: 1px solid var(--ink-3);
		text-align: center;
	}
	button.on {
		background: var(--inverse-bg);
		color: var(--inverse-fg);
		border-color: var(--inverse-bg);
	}
</style>
```

- [ ] **Step 4: Проверка типов и коммит**

```bash
npm run check
git add src/components
git commit -m "feat(ui): Pult components — LCD, knob, slots, faders, toggles

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: `svelte-check` — 0 ошибок.

---

### Task 14: Экраны и сборка лаунчера

**Files:**
- Create: `src/screens/MainScreen.svelte`, `src/screens/TuningScreen.svelte`, `src/screens/SlotEditor.svelte`, `.claude/launch.json`
- Modify: `src/App.svelte` (полная замена)

**Interfaces:**
- Consumes: `app` (store), компоненты Task 13, `levers.ts`.

- [ ] **Step 1: `src/screens/MainScreen.svelte`**

```svelte
<script lang="ts">
	import Knob from "../components/Knob.svelte";
	import LaunchButton from "../components/LaunchButton.svelte";
	import Lcd from "../components/Lcd.svelte";
	import Slot from "../components/Slot.svelte";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";

	const dto = $derived(app.dto!);
	const sel = $derived(app.selected);
	const profile = $derived(sel?.profile ?? dto.settings.profile);
	const stats = $derived(sel ? dto.stats[sel.id] : undefined);
	const lcdValue = $derived(stats?.lastBench?.avg ?? stats?.lastSession?.avg ?? null);
	const lcdCaption = $derived(stats?.lastBench ? t("main.lastBench") : stats?.lastSession ? t("main.lastSession") : t("main.noData"));
	const gpuLine = $derived(
		dto.gpu
			? `${dto.gpu.name} · ${Math.round(dto.gpu.vramMb / 1024)} ${t("unit.gb")} · ${dto.settings.engine.angle.toUpperCase()}`
			: t("gpu.unknown")
	);
	const code = (i: number) => `A${i + 1}`;
</script>

<div class="main">
	<section class="left">
		<div class="slots" role="listbox" aria-label={t("main.slots")}>
			{#each dto.servers as s, i (s.id)}
				<Slot
					server={s}
					code={code(i)}
					selected={s.id === app.selectedId}
					probe={app.probes[s.id]}
					onselect={() => (app.selectedId = s.id)}
					onlaunch={() => {
						app.selectedId = s.id;
						void app.launch(false);
					}}
					onedit={() => app.editSlot(s)}
				/>
			{/each}
			<button class="empty" onclick={() => app.newSlot()}>
				<span class="mono">{code(dto.servers.length)}</span>
				<span>{dto.servers.length ? t("main.emptySlot") : t("main.noServer")}</span>
				<span class="mono">{t("main.addSlot")}</span>
			</button>
		</div>
		{#if app.message}
			<div class="notice mono" role="status">
				<span>{app.message}</span>
				<button aria-label={t("window.close")} onclick={() => app.dismiss()}>✕</button>
			</div>
		{/if}
		<LaunchButton busy={app.launching} onlaunch={(safe) => (sel ? app.launch(safe) : app.newSlot())} />
	</section>

	<aside class="right">
		<Lcd value={lcdValue} unit="FPS" caption={lcdCaption} />
		<Knob value={profile} onchange={(p) => app.setKnob(p)} />
		<div class="gpu mono">{gpuLine}</div>
		<button class="tune mono" onclick={() => app.openTuning(sel ? { kind: "server", id: sel.id } : { kind: "global" })}>
			{t("main.tune")} →
		</button>
	</aside>
</div>

<style>
	.main {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 232px;
		gap: 2px;
		height: 100%;
		min-height: 0;
	}
	.left,
	.right {
		background: var(--face);
		padding: 18px;
		min-height: 0;
	}
	.left {
		display: grid;
		grid-template-rows: minmax(0, 1fr) auto auto;
		gap: 12px;
	}
	.slots {
		display: grid;
		align-content: start;
		gap: 6px;
		overflow-y: auto;
		min-height: 0;
	}
	.empty {
		display: grid;
		grid-template-columns: 34px 1fr auto;
		gap: 12px;
		align-items: center;
		padding: 12px 14px;
		border: 1px dashed var(--ink-3);
		color: var(--ink-2);
		text-align: left;
	}
	.empty:hover {
		border-color: var(--signal);
		color: var(--ink);
	}
	.notice {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		padding: 8px 12px;
		background: var(--lcd-bg);
		color: var(--signal);
	}
	.right {
		display: grid;
		align-content: start;
		justify-items: stretch;
		gap: 18px;
	}
	.gpu {
		color: var(--ink-2);
		text-align: center;
	}
	.tune {
		padding: 10px;
		border: 1px solid var(--ink-3);
	}
	.tune:hover {
		border-color: var(--signal);
	}
</style>
```

- [ ] **Step 2: `src/screens/TuningScreen.svelte`**

```svelte
<script lang="ts">
	import Fader from "../components/Fader.svelte";
	import Segmented from "../components/Segmented.svelte";
	import Toggle from "../components/Toggle.svelte";
	import { t } from "../lib/i18n.svelte";
	import { countOverrides, overridesFor, profileFor, resolved } from "../lib/levers";
	import { app } from "../lib/store.svelte";
	import type { AngleBackend, Levers, LocalePref, PrimeLevel, ProfileId, ThemePref, VideoMode } from "../lib/types";

	const dto = $derived(app.dto!);
	const scope = $derived(app.scope);
	const l = $derived(resolved(dto, scope));
	const over = $derived(overridesFor(dto, scope));
	const n = $derived(countOverrides(over));
	const mod = (...keys: (keyof Levers)[]) => keys.some((k) => over[k] !== undefined && over[k] !== null);
	const serverIndex = $derived(scope.kind === "server" ? dto.servers.findIndex((s) => s.id === scope.id) : -1);
	const server = $derived(serverIndex >= 0 ? dto.servers[serverIndex] : null);
	const title = $derived(server ? `A${serverIndex + 1} ${server.name}` : t("tuning.global"));
	const pct = (v: number) => `${Math.round(v * 100)}%`;

	const profiles = $derived(
		(["quality", "balance", "potato"] as ProfileId[]).map((p) => ({ value: p as ProfileId | "inherit", label: t(`profile.${p}.long`).toUpperCase() }))
	);
	const profileOptions = $derived(server ? [{ value: "inherit" as const, label: t("profile.inherit") }, ...profiles] : profiles);
	const profileValue = $derived<ProfileId | "inherit">(server ? (server.profile ?? "inherit") : dto.settings.profile);

	const videoOptions = (["play", "pauseUnfocused", "static"] as VideoMode[]).map((v) => ({ value: v, label: t(`video.${v}`) }));
	const primeOptions = (["soft", "medium", "aggressive"] as PrimeLevel[]).map((v) => ({ value: v, label: t(`prime.${v}`) }));
	const perfOptions = [3, 2, 1, 0].map((v) => ({ value: v, label: ["LOW", "MED", "HIGH", "MAX"][v] }));
	const angleOptions = (["d3d11", "d3d11on12", "gl", "vulkan"] as AngleBackend[]).map((v) => ({ value: v, label: v.toUpperCase() }));
	const cacheOptions = [1024, 2048, 4096].map((v) => ({ value: v, label: `${v / 1024} ${t("unit.gb")}` }));
	const localeOptions = (["auto", "ru", "en"] as LocalePref[]).map((v) => ({ value: v, label: v === "auto" ? t("locale.auto") : v.toUpperCase() }));
	const themeOptions = (["auto", "day", "night"] as ThemePref[]).map((v) => ({ value: v, label: t(`theme.${v}`) }));
</script>

<div class="tuning">
	<header class="head">
		<button class="mono back" onclick={() => app.closeScreen()}>{t("tuning.back")}</button>
		<b>{t("tuning.title")} · {title}</b>
		<span class="mono">
			{t("tuning.profile")}
			{t(`profile.${profileFor(dto, scope)}`)}
			{#if n > 0}<span class="badge">{t("tuning.changed", { n })}</span>{/if}
		</span>
	</header>

	<div class="body">
		<div class="profile">
			<Segmented
				label={t("tuning.profile")}
				options={profileOptions}
				value={profileValue}
				onchange={(v) => app.setScopeProfile(v === "inherit" ? null : v)}
			/>
		</div>

		<div class="faders">
			<Fader
				label={t("tuning.resolution")}
				value={l.resMin}
				min={0.4}
				max={1}
				step={0.05}
				format={(v) => (l.adaptive ? `${pct(v)}–${pct(l.resMax)}` : pct(v))}
				modified={mod("resMin", "resMax")}
				onchange={(v) => app.setLevers(l.adaptive ? { resMin: Math.min(v, l.resMax) } : { resMin: v, resMax: v })}
			/>
			<Fader
				label={t("tuning.maxFps")}
				value={l.maxFps}
				min={20}
				max={60}
				step={5}
				format={(v) => String(v)}
				modified={mod("maxFps")}
				onchange={(v) => app.setLevers({ maxFps: v })}
			/>
			<Fader
				label={t("tuning.unfocused")}
				value={l.unfocusedFps}
				min={5}
				max={60}
				step={5}
				format={(v) => String(v)}
				modified={mod("unfocusedFps")}
				onchange={(v) => app.setLevers({ unfocusedFps: v })}
			/>
			<Segmented
				vertical
				label={t("tuning.perfMode")}
				options={perfOptions}
				value={l.perfMode}
				modified={mod("perfMode")}
				onchange={(v) => app.setLevers({ perfMode: v as Levers["perfMode"] })}
			/>
		</div>

		<div class="toggles">
			<Toggle label={t("tuning.adaptive")} checked={l.adaptive} modified={mod("adaptive")} onchange={(v) => app.setLevers({ adaptive: v })} />
			<Toggle label={t("tuning.lightAnimation")} checked={l.lightAnimation} modified={mod("lightAnimation")} onchange={(v) => app.setLevers({ lightAnimation: v })} />
			<Toggle label={t("tuning.visionAnimation")} checked={l.visionAnimation} modified={mod("visionAnimation")} onchange={(v) => app.setLevers({ visionAnimation: v })} />
			<Toggle label={t("tuning.mipmap")} checked={l.mipmap} modified={mod("mipmap")} onchange={(v) => app.setLevers({ mipmap: v })} />
			<Toggle label={t("tuning.pixelRatio")} checked={l.pixelRatioScaling} modified={mod("pixelRatioScaling")} onchange={(v) => app.setLevers({ pixelRatioScaling: v })} />
			<Toggle label={t("tuning.uiBlur")} checked={l.uiBlur} modified={mod("uiBlur")} onchange={(v) => app.setLevers({ uiBlur: v })} />
			<Toggle label="Sequencer" checked={l.sequencer} modified={mod("sequencer")} onchange={(v) => app.setLevers({ sequencer: v })} />
			<Toggle label="FXMaster" checked={l.fxmaster} modified={mod("fxmaster")} onchange={(v) => app.setLevers({ fxmaster: v })} />
		</div>

		<div class="rows">
			<Segmented label={t("tuning.video")} options={videoOptions} value={l.video} modified={mod("video")} onchange={(v) => app.setLevers({ video: v })} />
			<Segmented label={t("tuning.prime")} options={primeOptions} value={l.prime} modified={mod("prime")} onchange={(v) => app.setLevers({ prime: v })} />
		</div>

		{#if !server}
			<div class="rows engine">
				<Segmented
					label={t("tuning.angle")}
					options={angleOptions}
					value={dto.settings.engine.angle}
					onchange={(v) => app.saveSettings({ engine: { ...dto.settings.engine, angle: v } })}
				/>
				<Segmented
					label={t("tuning.cache")}
					options={cacheOptions}
					value={dto.settings.engine.diskCacheMb}
					onchange={(v) => app.saveSettings({ engine: { ...dto.settings.engine, diskCacheMb: v } })}
				/>
				<label class="extra">
					<span class="mono">{t("tuning.extraArgs")}</span>
					<input
						class="mono"
						value={dto.settings.engine.extraArgs}
						placeholder="--flag=value"
						spellcheck="false"
						onchange={(e) => app.saveSettings({ engine: { ...dto.settings.engine, extraArgs: e.currentTarget.value } })}
					/>
					<span class="mono hint">{t("tuning.extraArgsHint")}</span>
				</label>
				<Segmented label={t("tuning.language")} options={localeOptions} value={dto.settings.locale} onchange={(v) => app.saveSettings({ locale: v })} />
				<Segmented label={t("tuning.theme")} options={themeOptions} value={dto.settings.theme} onchange={(v) => app.saveSettings({ theme: v })} />
			</div>
		{/if}
	</div>

	<footer class="foot mono">
		<span>{t("tuning.engineLine", { angle: dto.settings.engine.angle.toUpperCase(), cache: dto.settings.engine.diskCacheMb / 1024 })}</span>
		<span>
			{t("tuning.modifiedLegend")} ·
			<button class="reset" onclick={() => app.resetOverrides()}>{t("tuning.reset")}</button>
		</span>
	</footer>
</div>

<style>
	.tuning {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto;
		gap: 2px;
		height: 100%;
		min-height: 0;
	}
	.head,
	.foot {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 10px 18px;
		background: var(--face);
	}
	.head b {
		font-weight: 800;
		letter-spacing: 0.08em;
		margin-right: auto;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.back:hover,
	.reset:hover {
		color: var(--signal-text);
	}
	.badge {
		margin-left: 6px;
		padding: 1px 6px;
		background: var(--signal);
		color: var(--signal-ink);
	}
	.body {
		display: grid;
		gap: 2px;
		overflow-y: auto;
		min-height: 0;
	}
	.profile,
	.toggles,
	.rows {
		background: var(--face);
		padding: 12px 18px;
	}
	.faders {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 2px;
	}
	.toggles {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 6px 28px;
	}
	.rows {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 14px 28px;
	}
	.extra {
		grid-column: 1 / -1;
		display: grid;
		gap: 6px;
	}
	.extra input {
		height: 34px;
		padding: 0 10px;
		background: var(--face-2);
		border: 1px solid var(--line);
	}
	.hint {
		color: var(--ink-2);
	}
	.foot {
		justify-content: space-between;
		color: var(--ink-2);
	}
</style>
```

- [ ] **Step 3: `src/screens/SlotEditor.svelte`**

```svelte
<script lang="ts">
	import Segmented from "../components/Segmented.svelte";
	import { t } from "../lib/i18n.svelte";
	import { app } from "../lib/store.svelte";
	import type { ProfileId } from "../lib/types";

	const draft = $derived(app.editing!);
	const index = $derived(app.dto!.servers.findIndex((s) => s.id === draft.id));
	const isNew = $derived(index < 0);
	let errors = $state<{ name?: string; url?: string }>({});
	let confirmDelete = $state(false);
	let saving = $state(false);

	const profileOptions = $derived([
		{ value: "inherit" as ProfileId | "inherit", label: t("profile.inherit") },
		...(["quality", "balance", "potato"] as ProfileId[]).map((p) => ({ value: p as ProfileId | "inherit", label: t(`profile.${p}.long`).toUpperCase() }))
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

<form class="editor" onsubmit={save} novalidate>
	<header class="head">
		<b>{isNew ? t("slot.new") : t("slot.edit", { code: `A${index + 1}` })}</b>
	</header>

	<div class="fields">
		<label class="field">
			<span class="mono">{t("slot.name")}</span>
			<input
				bind:value={draft.name}
				placeholder={t("slot.namePlaceholder")}
				maxlength="60"
				aria-invalid={Boolean(errors.name)}
				oninput={() => (errors.name = undefined)}
			/>
			{#if errors.name}<span class="err mono" role="alert">{errors.name}</span>{/if}
		</label>
		<label class="field">
			<span class="mono">{t("slot.url")}</span>
			<input
				class="mono"
				bind:value={draft.url}
				placeholder={t("slot.urlPlaceholder")}
				spellcheck="false"
				aria-invalid={Boolean(errors.url)}
				oninput={() => (errors.url = undefined)}
			/>
			{#if errors.url}<span class="err mono" role="alert">{errors.url}</span>{/if}
		</label>
		<Segmented
			label={t("slot.profile")}
			options={profileOptions}
			value={draft.profile ?? "inherit"}
			onchange={(v) => (draft.profile = v === "inherit" ? null : v)}
		/>
		{#if !isNew}
			<button type="button" class="link mono" onclick={() => app.openTuning({ kind: "server", id: draft.id })}>{t("slot.tune")}</button>
		{/if}
	</div>

	<footer class="actions">
		<button type="submit" class="primary" aria-busy={saving}>{t("slot.save")}</button>
		<button type="button" class="secondary mono" onclick={() => app.closeScreen()}>{t("slot.cancel")}</button>
		{#if !isNew}
			<button
				type="button"
				class="danger mono"
				class:armed={confirmDelete}
				onclick={() => (confirmDelete ? app.deleteServer(draft.id) : (confirmDelete = true))}
				onblur={() => (confirmDelete = false)}
			>
				{confirmDelete ? t("slot.deleteConfirm") : t("slot.delete")}
			</button>
		{/if}
	</footer>
</form>

<style>
	.editor {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr) auto;
		gap: 2px;
		height: 100%;
	}
	.head,
	.fields,
	.actions {
		background: var(--face);
		padding: 14px 18px;
	}
	.head b {
		font-weight: 800;
		letter-spacing: 0.08em;
	}
	.fields {
		display: grid;
		align-content: start;
		gap: 18px;
		max-width: 560px;
		width: 100%;
	}
	.field {
		display: grid;
		gap: 6px;
	}
	input {
		height: 42px;
		padding: 0 12px;
		background: var(--face-2);
		border: 1px solid var(--line);
		font-size: 15px;
	}
	input[aria-invalid="true"] {
		border-color: var(--led-err);
	}
	.err {
		color: var(--led-err);
	}
	.link {
		justify-self: start;
		color: var(--signal-text);
	}
	.actions {
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.primary {
		padding: 11px 22px;
		background: var(--signal);
		color: var(--signal-ink);
		font-weight: 800;
		letter-spacing: 0.06em;
	}
	.secondary,
	.danger {
		padding: 10px 14px;
		border: 1px solid var(--ink-3);
	}
	.danger {
		margin-left: auto;
		color: var(--led-err);
		border-color: var(--led-err);
	}
	.danger.armed {
		background: var(--led-err);
		color: var(--face);
	}
</style>
```

- [ ] **Step 4: `src/App.svelte` (полная замена)**

```svelte
<script lang="ts">
	import TitleBar from "./components/TitleBar.svelte";
	import MainScreen from "./screens/MainScreen.svelte";
	import SlotEditor from "./screens/SlotEditor.svelte";
	import TuningScreen from "./screens/TuningScreen.svelte";
	import { app } from "./lib/store.svelte";

	void app.load();

	$effect(() => {
		const pref = app.dto?.settings.theme ?? "auto";
		const mq = matchMedia("(prefers-color-scheme: dark)");
		const apply = () => {
			document.documentElement.dataset.theme = pref === "auto" ? (mq.matches ? "night" : "day") : pref;
		};
		apply();
		mq.addEventListener("change", apply);
		return () => mq.removeEventListener("change", apply);
	});
</script>

<div class="frame">
	<TitleBar channel={app.dto ? `v${app.dto.version}` : ""} />
	<main class="screen">
		{#if !app.dto}
			<div class="boot mono">…</div>
		{:else if app.screen === "tuning"}
			<TuningScreen />
		{:else if app.screen === "slot" && app.editing}
			<SlotEditor />
		{:else}
			<MainScreen />
		{/if}
	</main>
</div>

<style>
	.frame {
		display: grid;
		grid-template-rows: 40px minmax(0, 1fr);
		gap: 2px;
		height: 100vh;
		padding: 0 2px 2px;
		background: var(--panel);
	}
	.screen {
		min-height: 0;
	}
	.boot {
		display: grid;
		place-items: center;
		height: 100%;
		background: var(--face);
		color: var(--ink-2);
	}
</style>
```

- [ ] **Step 5: Превью в браузере и визуальная проверка**

`.claude/launch.json`:
```json
{
  "version": "0.0.1",
  "configurations": [
    { "name": "launcher-preview", "runtimeExecutable": "npm", "runtimeArgs": ["run", "dev"], "port": 1420 }
  ]
}
```

Запустить превью (`preview_start` с `name: "launcher-preview"`), установить размер 900×620 (`resize_window`), сделать скриншоты и проверить:
1. Главный экран: два слота (A1 выделен, светодиод мигает, затем зелёный; A2 — красный), пунктирный пустой слот, оранжевая кнопка «ЗАПУСК», ЖК «052 FPS» с подписью «последний сеанс», ручка в положении БАЛ, строка GPU.
2. Клик по ручке → стрелка поворачивается с «пружиной», подпись КРТ подсвечивается; ЖК-цифры не прыгают (барабан).
3. Зажатый Shift → кнопка графитовая «БЕЗОПАСНО».
4. «НАСТРОЙКА →» для A1: фейдеры (вертикальные, оранжевые бегунки), тумблеры, видео/Prime; смена фейдера ставит оранжевую точку и бейдж «+1 ИЗМ»; «сброс к профилю» убирает.
5. «ИЗМ» у A2 → редактор; пустое название → ошибка у поля; «УДАЛИТЬ» требует второго клика.
6. Тема: `resize_window` с `colorScheme: "dark"` → ночная панель; всё читаемо.
7. Язык: в глобальной настройке EN → все подписи на английском.
8. Консоль браузера (`read_console_messages`, onlyErrors) — пусто.

Исправить найденные визуальные дефекты в соответствующих компонентах (только CSS/разметка), повторить скриншот.

- [ ] **Step 6: Проверки и коммит**

```bash
npx vitest run
npm run check
npm run build
git add src .claude/launch.json
git commit -m "feat(ui): main, tuning and slot editor screens

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

- [ ] **Step 7: Сквозная проверка в Tauri**

`npm run tauri dev`: окно без системной рамки, перетаскивается за шапку, «—» сворачивает, «✕» завершает процесс. Добавить слот `https://foundryvtt.com` (светодиод станет красным — это не сервер Foundry: `/api/status` не отдаёт JSON Foundry) → «ЗАПУСК»: лаунчер закрывается, открывается игровое окно; закрыть игровое окно → лаунчер вернулся. В трее: «Выход» завершает процесс.

---

### Task 15: Упаковка и README

**Files:**
- Create: `scripts/package-portable.ps1`, `README.md`

- [ ] **Step 1: `scripts/package-portable.ps1`**

```powershell
$ErrorActionPreference = "Stop"
$version = (Get-Content src-tauri/tauri.conf.json -Raw | ConvertFrom-Json).version
$exe = "src-tauri/target/release/foundry-performance.exe"
if (-not (Test-Path $exe)) { throw "Build first: npm run tauri build" }
New-Item -ItemType Directory -Force dist-portable | Out-Null
$zip = "dist-portable/FoundryPerformance-$version-portable.zip"
if (Test-Path $zip) { Remove-Item $zip }
Compress-Archive -Path $exe -DestinationPath $zip
Write-Output "Created $zip"
```

- [ ] **Step 2: `README.md`**

````markdown
# Foundry Performance

Клиент для Foundry VTT (Windows), который выжимает FPS из слабых видеокарт: отдельно
настроенный движок WebView2 и агент внутри страницы Foundry, применяющий профиль
производительности. Сделан для тяжёлых миров на v14.

## Установка

Скачайте `Foundry Performance_x.y.z_x64-setup.exe` (устанавливается без прав администратора)
или portable-архив. Нужен WebView2 — в Windows 10/11 он уже есть.

## Как пользоваться

- **Слоты** — ваши серверы. Двойной клик или «ЗАПУСК» открывает игру; лаунчер закрывается,
  чтобы не занимать память, и возвращается, когда вы закроете игру.
- **Ручка** — профиль выбранного сервера: КАЧ (качество), БАЛ (баланс), КРТ (картошка).
- **НАСТРОЙКА** — любой рычаг можно поменять вручную; изменённые отмечены точкой.
- **Shift + ЗАПУСК** — безопасный режим: без оптимизаций, чтобы проверить, не мешаем ли мы.

В игре:

| Клавиша | Действие |
|---|---|
| F9 | HUD: FPS, 1% low, разрешение карты, профиль |
| Shift+F9 | меню: смена профиля на лету, «Замер 30 с» |
| F10 | диагностический файл в «Загрузки» (для отчёта об ошибке) |

## Командная строка

```
foundry-performance.exe --open <адрес> [--safe | --baseline]
```
`--baseline` — стандартный движок и настройки Foundry по умолчанию, агент только измеряет
(для честного сравнения «до/после»).

## Сборка

Нужны Node 24+, Rust (stable-msvc) и MSVC Build Tools.

```bash
npm install
npm run tauri dev         # разработка
npm run tauri build       # установщик NSIS в src-tauri/target/release/bundle/nsis
npm run portable          # zip в dist-portable/
npm test                  # тесты агента и лаунчера
cargo test --manifest-path src-tauri/Cargo.toml
```
````

- [ ] **Step 3: Сборка установщика и portable**

```bash
npm run tauri build
npm run portable
ls -la src-tauri/target/release/bundle/nsis dist-portable
```
Expected: `Foundry Performance_0.1.0_x64-setup.exe` и `FoundryPerformance-0.1.0-portable.zip`; размер exe ≈ 5–10 МБ.

- [ ] **Step 4: Commit**

```bash
git add scripts README.md
git commit -m "chore: installer, portable package, README

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 16: Итоговый замер (с пользователем)

- [ ] **Step 1:** Передать пользователю установщик. На GTX 1060 (друг) и RTX 5070, одна и та же тяжёлая сцена:
  1. FLC: визуальная оценка FPS (в FLC нет счётчика — можно открыть консоль Foundry и включить `canvas.activateFPSMeter()` через F12).
  2. `--baseline` → Shift+F9 → «Замер».
  3. Профиль «Баланс» → «Замер».
  4. Профиль «Картошка» → «Замер».
- [ ] **Step 2:** Записать таблицу в `docs/superpowers/spike/…-findings.md` (раздел «Итог»), добавить в README раздел «Результаты» с цифрами. Commit `docs: benchmark results`.
- [ ] **Step 3:** Если «Баланс» на 1060 не даёт заметного прироста относительно baseline — завести задачи на следующий шаг (например, уменьшить `resMin` у «Баланса», проверить `d3d11on12`), без изменения кода в рамках этого плана.
