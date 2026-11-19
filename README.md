<div align="center">

**English** · [Русский](README.ru.md)

<img src="app-icon.svg" width="96" height="96" alt="Foundry Performance" />

# Foundry Performance

**A lightweight Windows client for Foundry VTT that squeezes maximum FPS out of low-end GPUs**

Dedicated WebView2 engine profile + real-time in-page optimization agent.  
Built for heavy worlds and scenes on **Foundry VTT v14**.

[![Latest Release](https://img.shields.io/github/v/release/TacticalOtaku/FoundryPerformance?style=flat-square&label=Release&color=E08256)](https://github.com/TacticalOtaku/FoundryPerformance/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/TacticalOtaku/FoundryPerformance/ci.yml?style=flat-square&label=CI)](https://github.com/TacticalOtaku/FoundryPerformance/actions/workflows/ci.yml)
![Platform](https://img.shields.io/badge/Windows-10%20%2F%2011%20(WebView2)-262D40?style=flat-square)
![Stack](https://img.shields.io/badge/Tauri%20v2-Svelte%205%20%C2%B7%20Rust-343D58?style=flat-square)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-3b82f6?style=flat-square)](LICENSE)

[Features](#features) • [Installation](#installation) • [Profiles](#performance-profiles) • [In-Game Hotkeys](#in-game-hotkeys) • [CLI](#command-line) • [Development](#development--build)

</div>

---

## Features

- **Isolated WebView2 runtime** — Launches Foundry with an isolated data profile and GPU tuning flags (`ANGLE`: `D3D11` / `D3D11on12` / `GL` / `Vulkan`, GPU rasterization, zero-copy, background throttling disabled, configurable `1 / 2 / 4 GB` disk cache, and extra Chromium flags).
- **Real-time Foundry agent** — Controls PIXI renderer resolution (including **adaptive resolution scaling** aware of monitor refresh rate), bypasses Foundry's `60 FPS` cap up to `240 FPS`, throttles background FPS when unfocused, disables heavy UI `backdrop-filter` blur, pauses video backgrounds, and toggles client-side `Sequencer` / `FXMaster` effects.
- **Server slots & status probing** — Displays live server status (including `Sqyre` hosting, LAN IPs, and `localhost`), remembers your last launched slot, and stores the latest FPS summary.
- **GPU check & automatic fallback** — Checks `WebGL2` and hardware acceleration in the launcher header; automatically falls back to the next ANGLE backend if an early WebGL crash occurs.
- **Tactile UI** — Light / Dark / System themes, 10 accent colors, English & Russian UI, and one-click engine cache cleanup without wiping logins or world settings.
- **Single instance & signed updates** — Re-launching focuses or routes URLs to the active instance; updates download from GitHub Releases and verify both `SHA-256` and `minisign` signatures.

---

## Installation

### Standard Installer (No admin rights required)

1. Download [`FoundryPerformance-x.y.z.exe`](https://github.com/TacticalOtaku/FoundryPerformance/releases/latest) and run it — the executable acts as its own installer.
2. Choose an install folder (defaults to `%LOCALAPPDATA%\Programs\FoundryPerformance`) and select shortcuts.
3. Running a newer `.exe` over an existing installation updates it in place; automatic updates are also checked on the startup splash screen.

> **Uninstall:** via **Windows Settings → Apps** or `FoundryPerformance.exe --uninstall`.  
> Servers, settings, and cache stay intact unless you check "also delete my data". Unrelated files in the folder are never touched.

### Portable Edition

Download `FoundryPerformance-x.y.z-portable.zip` — it contains the `.exe` alongside a `portable` marker file (launches straight into the launcher without installing).

> Requires **Microsoft Edge WebView2 Runtime** (preinstalled on most Windows 10/11 systems; the app offers a download prompt if missing).

---

## Usage

- **Server slots** — Click a slot to select it; double-click, press `Enter`, or click **LAUNCH** to start Foundry (the launcher closes to save memory and reopens once all game windows close).
- **Profile knob** — Quickly switch presets (**Quality**, **Balance**, **Potato**) for the selected server, or return to **Custom** with your manual tweaks preserved.
- **Tuning** — Fine-tune individual levers globally or per server slot, adjust engine flags, manage disk cache, and pick language/theme.
- **`Shift + LAUNCH`** — **Safe mode**: starts Foundry with stock engine flags and resets managed `core.*` client settings to verify baseline behavior.

### Performance Profiles

| Setting | **Quality** | **Balance** | **Potato** |
|---|---|---|---|
| Foundry performance mode | `HIGH` (`2`) | `MED` (`1`) | `LOW` (`0`) |
| Max FPS / Unfocused | `60` / `60` | `60` / `15` | `45` / `10` |
| Canvas resolution scale | `100%` (fixed) | `70%–100%` (auto) | `55%–85%` (auto) |
| DPI scaling | On | Off | Off |
| Light / Vision animations | On / On | On / Off | Off / Off |
| Mipmaps / UI blur | On / On | On / Off | Off / Off |
| Video backgrounds | Play | Pause unfocused | Static |
| `Sequencer` / `FXMaster` effects | On / On | On / On | Off / Off |

### In-Game Hotkeys

| Key | Action |
|---|---|
| `F9` | Toggle performance **HUD**: FPS, 1% low, frame time, canvas scale, graph, active profile |
| `Shift + F9` | Open HUD menu: switch profile on the fly or run a **30s scene benchmark** |
| `F10` | Save a diagnostic snapshot (`fp-diag-*.json`) to **Downloads** |
| `F11` | Toggle borderless fullscreen for the active game window |

---

## Command Line

```powershell
FoundryPerformance.exe --open <url> [--safe | --baseline]
```

| Flag | Purpose |
|---|---|
| `--open <url>` | Open a server URL directly, skipping the launcher (forwards to an existing instance) |
| `--safe` | Safe mode: stock engine flags and reset managed `core.*` settings |
| `--baseline` | Baseline benchmark: stock flags/settings with metrics collection only |
| `--portable` | Force portable mode without installer flow |
| `--uninstall` | Open the uninstaller screen |

---

## Development & Build

Requires **Node.js 24+**, **Rust (`stable-msvc`)**, and **MSVC Build Tools** (for the Windows native shell).

```bash
npm install                                      # Install JS dependencies
npm run tauri dev                                # Launch app in dev mode
npm run dev                                      # Preview launcher UI in browser
npm run check                                    # Svelte / TypeScript typecheck
npm test                                         # Agent & frontend tests (Vitest)
cargo test --manifest-path src-tauri/Cargo.toml  # Rust unit tests (Windows)
npm run dist                                     # Build dist-release/ .exe and -portable.zip
```

<details>
<summary><b>Browser UI preview flags (<code>npm run dev</code>)</b></summary>

- `?kit` — Tactile UI component showcase
- `?mode=install` / `?mode=install&state=upgrade` / `?mode=uninstall` — installer/uninstaller views
- `?splash&update=1` / `?splash&update=bad` / `?splash&update=fail` — updater splash states
- `?gpu=ok|software|none` — simulated GPU status

</details>

### Project Layout

```text
├── src/            # Launcher UI (Svelte 5, state & profile management)
├── agent/          # In-page Foundry VTT agent (adaptive resolution, HUD, benchmark)
├── src-tauri/      # Tauri v2 host (WebView2 flags, installer, auto-updater, server probe)
├── scripts/        # Packaging, icons, and update manifest tooling
└── CHANGELOG.md    # Release history (embedded into launcher and releases)
```

---

## Feedback & License

Pull Requests are disabled on this repository; bug reports and suggestions are welcome via [GitHub Issues](https://github.com/TacticalOtaku/FoundryPerformance/issues) (see [CONTRIBUTING.md](CONTRIBUTING.md)).

Released under the **GNU General Public License v3.0** ([LICENSE](LICENSE)).

<div align="center">

Full changelog in [CHANGELOG.md](CHANGELOG.md) • Binaries on [GitHub Releases](https://github.com/TacticalOtaku/FoundryPerformance/releases)

</div>
