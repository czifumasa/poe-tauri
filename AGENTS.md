# poe-tauri Project Rules

## Scope and intent

- **Goal**: Port Exile-UI's AHK overlay automation to a **native Linux overlay** using **Tauri v2 + Rust + React**.
- **Non-goals**: Preserve AHK runtime behavior as-is. AHK is the reference for _feature semantics_, not the
  implementation.

## Repository layout (current reality)

- **Frontend (UI)**: `src/` (React + TypeScript + Vite)
  - Uses Tauri IPC via `@tauri-apps/api/core` `invoke`.
  - Multiple "views" exist via query params (`getViewMode()` in `src/App.tsx`):
    - `index.html?view=overlay` (overlay window content)
    - `index.html?view=hintTooltip` (hint tooltip window content)
    - default "main" window
- **Backend (native)**: `src-tauri/src/`
  - `main.rs` is a thin entrypoint calling `poe_tauri_lib::run()`.
  - `lib.rs` is only wiring (setup + command registration); feature code lives in modules:
    - `commands/` (thin Tauri command functions), `window/` (window creation + platform integration), `error/`
      (shared error types), `persistence/` (settings store), `leveling_guide/`, `timer/`
- **Legacy reference**: the original AHK scripts live outside this repo (Exile-UI); feature domains are listed below.
- **Data/assets**: `src-tauri/data/`
  - Includes `data/english/poe1/level_tracker/{areas.json, default_guide.json}` and
    `data/english/poe2/level_tracker/{areas.json, default_guide.json}`.
  - `data/img/` exists but is empty right now (legacy expects a large `img/...` tree).

## Tauri / windowing rules (overlay-specific)

- **Windows** (labels in `src-tauri/src/window/identifiers.rs`):
  - **`main`**: the regular app window.
  - **`overlay`**: transparent, always-on-top, skip-taskbar overlay panel (layer-shell on Wayland).
  - **`hint-tooltip`**: small non-focusable tooltip window.
- **Linux layer-shell**:
  - On Linux/Wayland, use `gtk-layer-shell` to place windows in `Layer::Overlay`
    (`src-tauri/src/window/native_window/layer_shell_native_window.rs`). The backend is chosen at startup in
    `window::init_native_window()` (layer-shell if supported, else X11).
  - All GTK calls must happen on the GTK main thread via `window.run_on_main_thread(...)` (helper:
    `window/native_window/gtk_util.rs`).
- **Cross-platform guardrails**:
  - Keep Linux-specific logic behind `#[cfg(...)]` in `window/native_window/`.
  - Provide safe fallback behavior on other OSes (`default_native_window.rs`, `win32_native_window.rs`).

## IPC/command API rules (Rust <-> TS)

- **Tauri commands are the contract**:
  - Every command must return a typed result that TS can consume.
  - Prefer `Result<T, CommandError>` for anything that can fail.
- **Error model** (`src-tauri/src/error.rs`):
  - Do not throw strings.
  - Use the serializable `CommandError` struct with:
    - `kind`: stable identifier for programmatic handling (snake-case string IDs like
      `overlay_panel_window_create_failed`)
    - `message`: human-readable detail
  - Keep `kind` names stable once used by the UI.
- **State and side-effects**:
  - Side effects (window creation, GTK config, file IO, log watching) stay in Rust.
  - UI stays in TS/React and calls Rust through `invoke`.
  - Prefer idempotent "ensure" APIs (pattern: `ensure_overlay_window`, `ensure_hint_tooltip_window`).

## TypeScript / React rules (project-enforced)

- **TypeScript strictness is mandatory** (`tsconfig.json` has `strict: true`).
- ESLint enforces:
  - `@typescript-eslint/explicit-function-return-type: error`
  - `@typescript-eslint/typedef: error`
  - `@typescript-eslint/no-explicit-any: error`
- **Project convention**:
  - Every function must have an explicit return type (including callbacks where practical).
  - Never use `any`. Use concrete types or narrow `unknown`.
  - Keep view logic explicit (pattern: `getViewMode()` + conditional rendering in `src/App.tsx`).
- **IPC usage**:
  - Prefer `invoke<T>(...)` with explicit generic return type where relevant.
  - Avoid "stringly-typed" parameters; pass structured objects (`{ width, height }`, `{ position }`, etc.).

## Data and assets rules

- **Treat `src-tauri/data/` as authoritative static data** reused from Exile-UI.
- **Do not hardcode legacy paths** like `img\\Recognition (...)` in new Rust logic; resolve paths via Tauri path APIs.
- **Level tracker data**:
  - `areas.json` is a structured list of acts/zones (PoE1 entries may include `lvl`, `map_name`, `crafting_recipe`).
  - `default_guide.json` is a DSL-like structure of instruction line arrays and conditional blocks
    (`{ condition: [...], lines: [...] }`).
  - Parsing should treat these files as data, not code.

## Legacy AHK to Rust mapping rules (how to port safely)

- **AHK modules define feature domains**. Keep the same boundaries when designing Rust modules:
  - **Input/hotkeys**: AHK `Hotkey`, `GetKeySC/VK`, modifier conversion
  - **Screen checks**: image/pixel detection, resolution scaling, black-bar compensation
  - **OCR**: capture region + preprocess + OCR engine + tooltip rendering
  - **Leveling tracker**: guide DSL rendering, act/area progression, timers
  - **Settings**: port as structured config + UI panels rather than 1:1 behavior
- **Replace AHK global state** (`vars`, `settings`, `db`) with explicit state structs:
  - Keep state immutable in pure logic functions.
  - Put shared mutable state behind Tauri managed state (`app.manage(...)`) only when needed.
- **Replace AHK timers/threads** with Tauri-managed background workers with clear ownership (e.g.
  `leveling_guide/log_watcher.rs`, timer auto-save thread).
- **Replace GDI+/ImageSearch/PixelSearch** with Linux-friendly primitives behind interfaces, so Windows support remains
  possible later.

## Code organization rules

- Keep `src-tauri/src/lib.rs` as wiring only; new feature work goes into Rust modules (`commands/`, `window/`, etc.).
- **Functional bias**:
  - Prefer pure functions for parsing/transformations.
  - Keep platform calls at edges (GTK, Tauri window ops, filesystem).

## Development workflows

- Tauri dev (starts vite + debug binary): `npm run tauri:dev`
- Frontend dev server only: `npm run dev` (vite, fixed port `1420`)
- Build frontend: `npm run build`; packaged release: `npm run tauri:build`
- Formatting/lint: `npm run quality:fix` (prettier then eslint `--fix`) (Mandatory after every code change)

Gotcha: a debug binary run **without** the vite server shows "Could not connect to localhost" in every view — debug
builds load `devUrl` (`http://localhost:1420`), not the bundled `dist/`. Use `tauri:dev` or a release build.

## Runtime files (Linux)

- Settings: `~/.local/share/com.czifumasa.poe-tauri/settings.json` (release) / `settings.dev.json` (debug)
- Logs: `~/.local/share/com.czifumasa.poe-tauri/logs/poe-tauri.log` / `poe-tauri.dev.log`

On Wayland the overlay/hint windows are wlr-layer-shell surfaces; positions are stored as layer-shell margins there,
absolute coordinates on X11/Windows.

## Naming & terminology rules (project-specific)

- Use **consistent terms**:
  - **overlay**: the always-on-top overlay window (`overlay`)
  - **hint tooltip**: the `hint-tooltip` window
  - **view mode**: `main | overlay | hintTooltip` (TS union in `src/App.tsx`)
- Command names should be verb-first and stable: `show_overlay`, `hide_overlay`, `set_overlay_panel_size`,
  `overlay_set_position`, etc.
