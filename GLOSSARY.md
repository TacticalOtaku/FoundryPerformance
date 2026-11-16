# Foundry Performance

A Windows desktop client and in-page optimization agent for Foundry VTT that tunes browser flags, canvas resolution, frame caps, and heavy Foundry modules for smoother frame rates on weak GPUs.

## Language

**Launcher**:
The desktop interface where a player manages slots, profiles, engine settings, and updates before opening the game window.
_Avoid_: Dashboard, control panel, main app

**Slot**:
A saved Foundry VTT server entry holding a display name, entry URL, optional discovered game URL, optional profile, and slot-level overrides.
_Avoid_: Server card, bookmark, endpoint

**Slot Status**:
The probed availability state of a slot (`running`, `idle`, `stopped`, `unknown`, or `checking`).
_Avoid_: Server health, ping state, connection state

**Profile**:
A named preset of performance levers (`Quality`, `Balance`, `Potato`) representing a coherent performance-versus-visuals target.
_Avoid_: Preset tier, mode, configuration

**Levers**:
The complete resolved set of Foundry, rendering, video, and module optimization parameters active for a session.
_Avoid_: Tweaks, flags, options

**Override**:
A manual adjustment to an individual lever layered on top of the active profile at the global or slot scope.
_Avoid_: Delta, patch, diff

**Scope**:
The level where a profile selection or lever override applies: either global (`All servers`) or a specific slot.
_Avoid_: Layer, context, tier

**Engine Settings**:
Browser-level WebView2 settings applied at game window creation: ANGLE graphics backend, disk cache size, and extra Chromium flags.
_Avoid_: Runtime flags, browser config

**Agent**:
The script running inside the Foundry VTT game window that applies levers, monitors frame times, renders the in-game HUD, and reports telemetry back to the launcher.
_Avoid_: Plugin, extension, worker

**Safe Mode**:
A launch mode using stock browser flags where the agent only removes managed `core.*` client settings from local storage so Foundry runs with its own defaults.
_Avoid_: Recovery mode, clean mode

**Baseline**:
A measurement-only launch mode using stock browser flags and Foundry defaults while the agent records FPS without applying levers.
_Avoid_: Dry run, control mode

**Bench**:
A standardized 30-second camera pan-and-zoom measurement across the active scene that records average, 1% low, and minimum FPS.
_Avoid_: Stress test, profiling run
