% Rust + ratatui idle game — MVP-1 (Tick timer + pause/step) design

Date: 2026-05-08

## Goal

Introduce a 1-second tick timer into the app runtime and surface it as `Action::Tick`, while keeping all “game logic” minimal: a tick counter in `core`.

Add a pause mode and a single-step action (apply exactly 1000ms once) while paused.

## Non-goals (for MVP-1)

- Any real simulation rules beyond incrementing a counter
- Persistence/saves
- Performance optimizations, multi-threading, or background workers
- Complex input rebinding or config systems

## Success criteria (Definition of Done)

- `cargo run` launches the TUI as in MVP-0.
- In **Running** mode, the app produces a tick every 1 second even with no user input.
- `core` maintains a counter that increments once per simulated second.
- Pressing:
  - `q` quits cleanly (terminal restored)
  - `p` toggles **Running** ↔ **Paused**
  - `n` performs a single step of **1000ms** when paused (no effect while running)
- UI footer shows:
  - current mode (running/paused)
  - current tick counter
  - controls hint: `q quit | p pause | n step`

## Architecture changes from MVP-0

### Action model

Extend the app-level action set to support timing and pause/step controls.

`src/input/mod.rs`:

- `enum Action` becomes:
  - `Quit`
  - `TogglePause`
  - `Step`
  - `Tick`
  - `None`

### Runtime loop (app)

The `App` owns the “mode” of execution:

- **Running**: input read uses a timeout of 1 second; timeout expiration produces `Action::Tick`.
- **Paused**: input read is blocking; no `Tick` is produced.

`src/app/mod.rs` main loop becomes:

- draw UI
- read action (blocking or tick-aware depending on mode)
- dispatch:
  - `Quit` → break
  - `TogglePause` → toggle mode
  - `Tick` → call `core::Game::tick(1000)`
  - `Step` → if paused, call `core::Game::tick(1000)`; otherwise no-op
  - `None` → no-op

### Input layer (single-threaded, no timers)

Timing remains single-threaded and uses `crossterm::event::poll` with a timeout:

- If an input event arrives before the timeout, read it and map it to an `Action`.
- If the timeout expires with no events, return `Action::Tick`.

This keeps the app simple (no background threads/channels) and preserves terminal safety.

### Domain layer (core): `tick(ms)`

The domain owns the simulation-time API.

`src/core/mod.rs`:

- `Game` gains:
  - `ticks: u64` — total number of simulated seconds elapsed
  - `accum_ms: u64` — accumulated partial milliseconds not yet converted to a full second
- `fn tick(&mut self, ms: u64)` updates:
  - `accum_ms += ms`
  - while `accum_ms >= 1000`:
    - `ticks += 1`
    - `accum_ms -= 1000`

For MVP-1 the runtime always calls `tick(1000)`, but the accumulator design avoids fragile assumptions and supports future sub-second polling or variable frame times without changing the domain API.

## UI changes

Minimal UI changes:

- Footer line becomes a compact status + help:
  - `mode: running|paused | ticks: <N> | q quit | p pause | n step`

## Error handling & terminal safety

No change from MVP-0:

- Use the existing `tui` RAII wrapper to restore the terminal on exit and on errors.
- Remain single-threaded to avoid shutdown races.

## Testing

Manual verification:

- Run `cargo run`
- Observe `ticks` increases once per second in running mode
- Press `p` and confirm ticks stop increasing
- Press `n` repeatedly and confirm ticks increments by 1 per keypress (only while paused)
- Press `p` again to resume auto ticking
- Press `q` to exit; confirm terminal is restored

