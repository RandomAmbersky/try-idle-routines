# MVP-1 Tick/Pause/Step Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a 1-second tick action, pause/resume, and a single-step (1000ms) while paused, backed by a minimal `core` tick counter and displayed in the TUI footer.

**Architecture:** Keep the app single-threaded. In running mode, use `crossterm::event::poll` with a 1s timeout to generate `Action::Tick`. In paused mode, use blocking input reads and allow `Action::Step` to advance the core by exactly 1000ms.

**Tech Stack:** Rust 2024 edition, `ratatui`, `crossterm`.

---

## File structure (what changes where)

**Modify:**
- `src/core/mod.rs`
  - Add `Game` tick state (`ticks`, `accum_ms`)
  - Add `Game::tick(ms: u64)`
  - Add unit tests for tick accumulation
- `src/input/mod.rs`
  - Extend `Action` with `Tick`, `TogglePause`, `Step`
  - Add a tick-aware read function using `event::poll`
  - Add tests for mapping `KeyEvent` → `Action` (pure helper)
- `src/app/mod.rs`
  - Add run-mode state (`Running`/`Paused`)
  - Switch input read strategy by mode
  - Dispatch `Tick`/`Step` into `core::Game::tick(1000)`
- `src/ui/mod.rs`
  - Update footer to show mode + tick counter + controls

**No changes expected:**
- `src/tui/mod.rs` (terminal lifecycle remains RAII)
- `src/main.rs` (wiring unchanged)

---

### Task 1: Add `core::Game::tick(ms)` with a tested counter

**Files:**
- Modify: `src/core/mod.rs`
- Test: `src/core/mod.rs` (module tests)

- [ ] **Step 1: Write failing tests for tick accumulation**

Add this test module at the bottom of `src/core/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_1000ms_increments_once() {
        let mut g = Game::new();
        g.tick(1000);
        assert_eq!(g.ticks, 1);
        assert_eq!(g.accum_ms, 0);
    }

    #[test]
    fn tick_accumulates_partial_ms() {
        let mut g = Game::new();
        g.tick(400);
        assert_eq!(g.ticks, 0);
        assert_eq!(g.accum_ms, 400);

        g.tick(600);
        assert_eq!(g.ticks, 1);
        assert_eq!(g.accum_ms, 0);
    }

    #[test]
    fn tick_can_roll_over_multiple_seconds() {
        let mut g = Game::new();
        g.tick(2500);
        assert_eq!(g.ticks, 2);
        assert_eq!(g.accum_ms, 500);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
- `cargo test`

Expected:
- FAIL, because `Game::tick`, `ticks`, and `accum_ms` do not exist yet.

- [ ] **Step 3: Implement minimal tick state + `tick(ms)`**

Update `Game` to include tick fields and implement `tick`:

```rust
#[derive(Debug)]
pub struct Game {
    pub world: World,
    pub base: Base,
    pub units: Units,
    pub ticks: u64,
    pub accum_ms: u64,
}

impl Game {
    pub fn new() -> Self {
        Self {
            world: World::default(),
            base: Base::default(),
            units: Units::default(),
            ticks: 0,
            accum_ms: 0,
        }
    }

    pub fn tick(&mut self, ms: u64) {
        self.accum_ms = self.accum_ms.saturating_add(ms);
        while self.accum_ms >= 1000 {
            self.ticks += 1;
            self.accum_ms -= 1000;
        }
    }
}
```

Notes:
- `saturating_add` prevents `u64` overflow panics; not expected in MVP, but it’s cheap safety.

- [ ] **Step 4: Run tests to verify they pass**

Run:
- `cargo test`

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/core/mod.rs
git commit -m "feat(core): add tick(ms) counter"
```

---

### Task 2: Add `Action` variants and tick-aware input read (TDD around mapping)

**Files:**
- Modify: `src/input/mod.rs`
- Test: `src/input/mod.rs`

- [ ] **Step 1: Refactor input mapping into a pure helper + write failing tests**

Replace ad-hoc match in `read_action_blocking` with a pure helper, and test it.

Add this test module to `src/input/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    #[test]
    fn map_q_to_quit() {
        assert_eq!(action_from_key(key(KeyCode::Char('q'))), Action::Quit);
    }

    #[test]
    fn map_p_to_toggle_pause() {
        assert_eq!(
            action_from_key(key(KeyCode::Char('p'))),
            Action::TogglePause
        );
    }

    #[test]
    fn map_n_to_step() {
        assert_eq!(action_from_key(key(KeyCode::Char('n'))), Action::Step);
    }

    #[test]
    fn other_keys_are_none() {
        assert_eq!(action_from_key(key(KeyCode::Left)), Action::None);
        assert_eq!(action_from_key(key(KeyCode::Char('x'))), Action::None);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:
- `cargo test`

Expected:
- FAIL: `action_from_key` and new `Action` variants don’t exist.

- [ ] **Step 3: Implement the new `Action` enum + mapping helper**

Update `Action` and add:

```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    TogglePause,
    Step,
    Tick,
    None,
}

pub fn action_from_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('p') => Action::TogglePause,
        KeyCode::Char('n') => Action::Step,
        _ => Action::None,
    }
}
```

- [ ] **Step 4: Add a tick-aware read function (poll with timeout)**

Add:

```rust
use std::time::Duration;

pub fn read_action_tick_aware(timeout_ms: u64) -> std::io::Result<Action> {
    if event::poll(Duration::from_millis(timeout_ms))? {
        match event::read()? {
            Event::Key(key) => Ok(action_from_key(key)),
            _ => Ok(Action::None),
        }
    } else {
        Ok(Action::Tick)
    }
}
```

Keep the existing blocking read but route through the helper:

```rust
pub fn read_action_blocking() -> std::io::Result<Action> {
    match event::read()? {
        Event::Key(key) => Ok(action_from_key(key)),
        _ => Ok(Action::None),
    }
}
```

- [ ] **Step 5: Run tests**

Run:
- `cargo test`

Expected:
- PASS

- [ ] **Step 6: Commit**

```bash
git add src/input/mod.rs
git commit -m "feat(input): add tick/pause/step actions"
```

---

### Task 3: Add app run modes + dispatch tick/step into core

**Files:**
- Modify: `src/app/mod.rs`
- Modify: `src/input/mod.rs` (imports/usage only, if needed)

- [ ] **Step 1: Add run-mode state to `App`**

In `src/app/mod.rs`, change `run(self)` to `run(mut self)` and add a small enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Running,
    Paused,
}
```

Initialize `let mut mode = RunMode::Running;` before the loop.

- [ ] **Step 2: Switch action read strategy by mode**

In the loop, replace `read_action_blocking()` call with:

```rust
let action = match mode {
    RunMode::Running => crate::input::read_action_tick_aware(1000)?,
    RunMode::Paused => crate::input::read_action_blocking()?,
};
```

- [ ] **Step 3: Dispatch actions**

Replace the existing `match` with:

```rust
match action {
    Action::Quit => break,
    Action::TogglePause => {
        mode = match mode {
            RunMode::Running => RunMode::Paused,
            RunMode::Paused => RunMode::Running,
        };
    }
    Action::Tick => {
        self.game.tick(1000);
    }
    Action::Step => {
        if mode == RunMode::Paused {
            self.game.tick(1000);
        }
    }
    Action::None => {}
}
```

- [ ] **Step 4: Run and smoke-test manually**

Run:
- `cargo run`

Expected manual behavior:
- ticks increase once per second
- `p` pauses/resumes
- while paused, `n` increments ticks by 1 per press
- `q` quits and restores terminal

- [ ] **Step 5: Commit**

```bash
git add src/app/mod.rs
git commit -m "feat(app): add tick loop with pause and step"
```

---

### Task 4: Display mode + ticks in footer

**Files:**
- Modify: `src/ui/mod.rs`
- (Optional) Modify: `src/app/mod.rs` if UI needs mode passed in

Because `ui::render` currently receives only `&Game`, we should keep UI minimal by deriving mode text in `app` and threading it through `ui::render`.

- [ ] **Step 1: Update `ui::render` signature to accept mode**

Change:
- `pub fn render(frame: &mut Frame, game: &Game)`
to:
- `pub fn render(frame: &mut Frame, game: &Game, mode: &str)`

Update `app` draw call accordingly:

```rust
let mode_label = match mode {
    RunMode::Running => "running",
    RunMode::Paused => "paused",
};
terminal.draw(|f| ui::render(f, &self.game, mode_label))?;
```

- [ ] **Step 2: Update footer line**

Replace footer help line with:

```rust
let help = Paragraph::new(Line::from(format!(
    "mode: {} | ticks: {} | q quit | p pause | n step",
    mode, game.ticks
)))
.style(Style::default());
```

- [ ] **Step 3: Run manual check**

Run:
- `cargo run`

Expected:
- Footer shows mode changes on `p`
- Ticks visible and changing in running/step in paused

- [ ] **Step 4: Commit**

```bash
git add src/ui/mod.rs src/app/mod.rs
git commit -m "feat(ui): show tick counter and mode"
```

---

## Plan self-review

**Spec coverage check (MVP-1):**
- 1s tick without input: Task 3 (tick-aware poll) ✅
- Pause/resume: Task 3 (`TogglePause`) ✅
- Step 1000ms only while paused: Task 3 (`Step` gated by mode) ✅
- `core::tick(ms)` + counter: Task 1 ✅
- Footer shows mode/ticks/controls: Task 4 ✅
- Terminal safety unchanged: no `tui` changes, still RAII ✅

**Placeholder scan:** no TBD/TODO steps; each task includes concrete code/commands ✅

**Type consistency:** `Action::{Quit,TogglePause,Step,Tick,None}` used consistently; `Game::{ticks,accum_ms,tick}` defined before use ✅

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-08-ratatui-idle-mvp1-tick.md`. Two execution options:

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?

