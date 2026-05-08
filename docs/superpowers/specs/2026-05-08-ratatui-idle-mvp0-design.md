# Rust + ratatui idle game — MVP-0 (Hello World) design

Date: 2026-05-08

## Goal

Create the first runnable MVP (“MVP-0”) of a future idle game as a terminal UI (TUI) app using:

- `ratatui` for UI
- `crossterm` for terminal backend + input

This MVP is intentionally minimal: it should render a “Hello world” screen (with a basic multi-panel layout that will later host Map/Base/Units), handle keyboard input, and exit cleanly.

## Non-goals (for MVP-0)

- Any actual simulation/idle ticking
- Persistence/saves
- Real map generation, base building, unit management logic
- Mouse support
- Config system

## Success criteria (Definition of Done)

- `cargo run` launches a TUI in the terminal.
- The screen renders:
  - A framed layout with 2–3 panels (placeholders for Map/Base/Units).
  - A visible “Hello world” label (or similar minimal copy).
  - A small hint line for controls (e.g. “Press q to quit”).
- Pressing `q` exits the app.
- The terminal state is fully restored after exit (no broken echo/cursor/alternate screen issues), including:
  - raw mode disabled
  - alternate screen disabled
  - cursor shown
- Cleanup happens even on errors (best effort; do not leave the terminal broken).

## User interaction model (MVP-0)

- Keyboard only.
- Minimal action set:
  - `q` → Quit
  - Other keys ignored (reserved for future use)

## Architecture

### Crate layout (single crate)

Use a single Rust crate, but establish boundaries immediately to avoid a “single main.rs blob” refactor later.

Proposed module layout:

- `src/main.rs`
  - Minimal wiring: initialize, create `App`, call `app.run()`, handle top-level error reporting.
- `src/core/mod.rs`
  - UI-agnostic game/domain layer (state + pure logic).
  - Owns the “future anchors” (`world`, `base`, `units`) and any rules that evolve them.
  - Must not depend on terminal/UI crates; designed to be reusable with other frontends later.
- `src/app/mod.rs`
  - Application shell / runtime wiring for the chosen frontend (ratatui+crossterm).
  - Holds `core::Game` (or similar) as the source of truth.
  - `run()` main loop (render → input → action dispatch).
- `src/ui/mod.rs`
  - `render(frame, &App)` — pure rendering (no input reading, no timing).
  - Owns layout composition (Map/Base/Units panels) and drawing.
- `src/input/mod.rs`
  - Reads `crossterm` events and translates them into app-level actions.
  - Defines `enum Action { Quit /*, ...future */ }`.
- `src/tui/mod.rs`
  - Terminal lifecycle wrapper:
    - enter: raw mode + alternate screen + optional cursor hide
    - leave: restore terminal state
  - Must be responsible for ensuring cleanup on drop / scope exit.

### Main loop (MVP-0)

High-level flow:

1. Enter terminal UI mode (raw + alternate screen).
2. Loop:
   - Draw UI via `ui::render`.
   - Read input event(s) and map to `Action`.
   - Apply action:
     - `Quit` → break loop
     - otherwise no-op
3. Leave terminal UI mode (restore terminal state).

Input strategy (MVP-0):

- Blocking input read is acceptable (no need for tick timers yet).
- Later MVPs can introduce non-blocking polling + tick events without changing UI module boundaries.

### State model (stub for future)

Even in MVP-0, the domain layer in `core` should reserve “future state anchors” as empty placeholders to keep the direction clear:

- `world` (future map / global state)
- `base` (future base state)
- `units` (future unit roster/state)

They can be empty structs or `Option<()>`-like placeholders; no behavior required in MVP-0.

`app` should treat `core` as the source of truth, and avoid duplicating domain state inside UI-specific types.

## Error handling & terminal safety

The terminal must not be left in a broken state.

Design requirements:

- Terminal teardown happens in a dedicated wrapper (`tui` module) using RAII (Drop) or an explicit `restore()` in a `defer`-like pattern.
- App-level errors should bubble up, but always after terminal restore has run.

## Dependencies (high level)

- `ratatui`
- `crossterm`

(Exact versions will be selected during implementation planning; keep MVP-0 compatible with stable Rust.)

## Testing

MVP-0 testing is manual:

- Run `cargo run`, verify rendering.
- Press `q` to quit.
- Type in terminal afterward to confirm echo/cursor behavior is normal.

Future tests (not required in MVP-0):

- Unit tests for input mapping (`KeyEvent` → `Action`).
- Snapshot-like tests for layout functions (if/when UI becomes complex).

## Visual companion notes (project hygiene)

If the visual companion creates `.superpowers/` artifacts under the repo, ensure they are not committed (add `.superpowers/` to `.gitignore`).

