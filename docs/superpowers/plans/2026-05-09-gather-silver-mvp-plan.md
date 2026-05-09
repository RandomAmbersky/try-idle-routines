# Gather / Silver MVP (autonomous squad loop) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the first gameplay slice from [2026-05-09 idle UFO-inspired vision](../specs/2026-05-09-idle-ufo-inspired-vision-design.md): one squad autonomously takes **gather** missions from the map, spends **3 simulated seconds** on site, returns **Silver** to the base, and repeats while MVP-1’s **tick(1000ms)** advances time.

**Architecture:** Extend `core::Game` only (no new crates). Mission type is implicit (`Gather`). One squad, tracked in `units`. Each **completed simulated second** (same moment `ticks` increments) runs `Game::simulate_second()` after counter math. Silver is stored on `Base`; available gather slots on `World`; squad phase in `Units`. UI replaces `Debug` dumps with readable summaries for Map / Base / Units.

**Tech Stack:** Rust 2024 edition, `ratatui`, `crossterm`. Tests in-module under `cfg(test)` in `src/core/mod.rs`.

**Fixed MVP constants (explicit):**

| Constant | Value |
|----------|------:|
| `GATHER_DURATION_SECS` | `3` |
| `SILVER_PER_GATHER` | `10` |
| Initial `available_gather_missions` in `Game::new` | `1` |

---

## File structure (what changes where)

**Modify:**

| File | Responsibility |
|------|----------------|
| `src/core/mod.rs` | `Base.silver`, `World.available_gather_missions`, `Squad` / `SquadState`, `Units` with exactly one squad, `simulate_second()`, wire into `tick()` inner second loop; unit tests |
| `src/ui/mod.rs` | Map: mission availability + squad status; Base: Silver count; Units: squad phase text |

**No mandatory changes:**

- `src/app/mod.rs` — still dispatches `Action::Tick` / `Step` → `game.tick(1000)`
- `src/input/mod.rs`, `src/tui/mod.rs`, `src/main.rs`

---

### Task 1: `Base::silver`

**Files:**

- Modify: `src/core/mod.rs`
- Test: `src/core/mod.rs` (`cfg(test)`)

- [ ] **Step 1: Write failing test**

Add to `mod tests`:

```rust
    #[test]
    fn new_game_base_has_zero_silver() {
        let g = Game::new();
        assert_eq!(g.base.silver, 0);
    }
```

- [ ] **Step 2: Run test, expect FAIL**

Run: `cargo test new_game_base_has_zero_silver -- --nocapture`

Expected: compile error `no field silver on type Base`.

- [ ] **Step 3: Add field**

```rust
#[derive(Debug, Default)]
pub struct Base {
    pub silver: u64,
}
```

Ensure `Game::new()` still initializes `base: Base::default()` so silver stays `0`.

- [ ] **Step 4: Run test, expect PASS**

Run: `cargo test new_game_base_has_zero_silver`

- [ ] **Step 5: Commit**

```bash
git add src/core/mod.rs
git commit -m "feat(core): add Base.silver counter"
```

---

### Task 2: `World::available_gather_missions` seeded in `Game::new`

**Files:**

- Modify: `src/core/mod.rs`

- [ ] **Step 1: Write failing test**

```rust
    #[test]
    fn new_game_has_one_gather_mission_available() {
        let g = Game::new();
        assert_eq!(g.world.available_gather_missions, 1);
    }
```

- [ ] **Step 2: Run test, expect FAIL**

Run: `cargo test new_game_has_one_gather_mission_available`

Expected: missing field or wrong value (`0` from `Default`).

- [ ] **Step 3: Add field and seed**

```rust
#[derive(Debug, Default)]
pub struct World {
    pub available_gather_missions: u32,
}
```

Replace `world: World::default()` inside `Game::new()` with explicit struct literal:

```rust
            world: World {
                available_gather_missions: 1,
            },
```

Leave `#[derive(Default)]` on `World` so unrelated code can still use `World::default()` with `0` missions if needed.

- [ ] **Step 4: Run full core tests**

Run: `cargo test --lib`

(If the crate has no `[lib]` and only bins, run `cargo test` — this project exposes tests from `src/core/mod.rs` only when compiled as part of the binary crate’s module tree; verify with `cargo test`.)

Expected: PASS for all tests in `core::tests`.

- [ ] **Step 5: Commit**

```bash
git add src/core/mod.rs
git commit -m "feat(core): world gather mission availability + initial pool"
```

---

### Task 3: `Squad`, `SquadState`, `Units` (single idle squad)

**Files:**

- Modify: `src/core/mod.rs`

- [ ] **Step 1: Write failing tests**

Append:

```rust
    #[test]
    fn new_game_single_squad_idle_at_base() {
        let g = Game::new();
        assert_eq!(g.units.squads.len(), 1);
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);
    }
```

- [ ] **Step 2: Run test, expect FAIL**

Run: `cargo test new_game_single_squad_idle_at_base`

- [ ] **Step 3: Add domain types**

Place **above** `Game` struct:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquadState {
    IdleAtBase,
    Gathering { seconds_left: u32 },
}

#[derive(Debug)]
pub struct Squad {
    pub state: SquadState,
}

#[derive(Debug)]
pub struct Units {
    pub squads: Vec<Squad>,
}

impl Default for Units {
    fn default() -> Self {
        Self {
            squads: vec![Squad {
                state: SquadState::IdleAtBase,
            }],
        }
    }
}
```

Remove `#[derive(Default)]` from the old empty `Units` if it conflicts — `Units` must use the manual `Default` above.

Ensure `#[derive(Default)]` on `World`, `Base` remains valid.

- [ ] **Step 4: Run `cargo test`**

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/mod.rs
git commit -m "feat(core): squad state and single-squad Units"
```

---

### Task 4: `simulate_second` — autonomous gather loop

**Files:**

- Modify: `src/core/mod.rs`

**Simulation rule (single canonical semantics):**

- Each **simulated second** (each time `ticks` increments inside `tick`), run `simulate_second` **after** `ticks += 1`.
- **Idle**: if `available_gather_missions > 0`, decrement it by 1 and set `Gathering { seconds_left: GATHER_DURATION_SECS }` (starts at **3**). No countdown in the **same** second as departure.
- **Gathering { n }**:
  - If `n == 1`: add `SILVER_PER_GATHER` to base, squad → `IdleAtBase`, `available_gather_missions += 1` (respawn).
  - Else (`n >= 2`): squad → `Gathering { seconds_left: n - 1 }`.

State timeline from cold start: after sim second 1 → `Gathering { 3 }`; after 2 → `{ 2 }`; after 3 → `{ 1 }`; after 4 → Idle + Silver + mission back on map.

- [ ] **Step 1: Add constants**

At top of `src/core/mod.rs` (after inner doc / before structs):

```rust
const GATHER_DURATION_SECS: u32 = 3;
const SILVER_PER_GATHER: u64 = 10;
```

- [ ] **Step 2: Write failing integration test**

Add to `mod tests`:

```rust
    #[test]
    fn autonomous_gather_loop_adds_silver_every_gather_cycle() {
        let mut g = Game::new();
        assert_eq!(g.world.available_gather_missions, 1);
        assert_eq!(g.base.silver, 0);

        g.tick(1000); // simulated second 1: depart
        assert_eq!(g.world.available_gather_missions, 0);
        assert_eq!(
            g.units.squads[0].state,
            SquadState::Gathering {
                seconds_left: GATHER_DURATION_SECS,
            }
        );

        g.tick(1000); // 2 → 3 → 2
        assert_eq!(
            g.units.squads[0].state,
            SquadState::Gathering {
                seconds_left: GATHER_DURATION_SECS - 1,
            }
        );

        g.tick(1000); // 3 → 2 → 1
        assert_eq!(
            g.units.squads[0].state,
            SquadState::Gathering { seconds_left: 1 }
        );

        g.tick(1000); // 4: resolve
        assert_eq!(g.units.squads[0].state, SquadState::IdleAtBase);
        assert_eq!(g.base.silver, SILVER_PER_GATHER);
        assert_eq!(g.world.available_gather_missions, 1);
    }
```

- [ ] **Step 3: Run test, expect FAIL**

Run: `cargo test autonomous_gather_loop_adds_silver_every_gather_cycle`

Expected: FAIL (missing `simulate_second`, wrong state, or zero silver).

- [ ] **Step 4: Implement `simulate_second` and call it from `tick`**

Extend `Game` impl:

```rust
impl Game {
    pub fn tick(&mut self, ms: u64) {
        self.accum_ms += ms;
        while self.accum_ms >= 1000 {
            self.accum_ms -= 1000;
            self.ticks += 1;
            self.simulate_second();
        }
    }

    fn simulate_second(&mut self) {
        let squad = &mut self.units.squads[0];
        match squad.state {
            SquadState::IdleAtBase => {
                if self.world.available_gather_missions > 0 {
                    self.world.available_gather_missions -= 1;
                    squad.state = SquadState::Gathering {
                        seconds_left: GATHER_DURATION_SECS,
                    };
                }
            }
            SquadState::Gathering { seconds_left } => match seconds_left {
                1 => {
                    self.base.silver = self.base.silver.saturating_add(SILVER_PER_GATHER);
                    squad.state = SquadState::IdleAtBase;
                    self.world.available_gather_missions =
                        self.world.available_gather_missions.saturating_add(1);
                }
                n => {
                    squad.state = SquadState::Gathering {
                        seconds_left: n - 1,
                    };
                }
            },
        }
    }
}
```

If `pub fn tick` already exists from MVP-1, **merge**: keep accumulator/`ticks` logic and insert `self.simulate_second();` once per consumed second inside the `while`.

- [ ] **Step 5: Run full test suite**

Run: `cargo test`

Expected: PASS (including tick tests and `autonomous_gather_loop_adds_silver_every_gather_cycle`).

- [ ] **Step 6: (Optional)** Add second cycle: four more `g.tick(1000)` asserting `silver == 2 * SILVER_PER_GATHER`.

- [ ] **Step 7: Commit**

```bash
git add src/core/mod.rs
git commit -m "feat(core): autonomous gather missions and Silver payout"
```

---

### Task 5: HUD text for Map / Base / Units

**Files:**

- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Replace `Debug` panel bodies with summaries**

Replace `use crate::core::Game;` with:

```rust
use crate::core::{Game, SquadState};
```

Add helpers (same module, below `render` or above it):

```rust
fn squad_primary_line(game: &Game) -> String {
    match game.units.squads[0].state {
        SquadState::IdleAtBase => String::from("Squad A: idle at base"),
        SquadState::Gathering { seconds_left } => {
            format!("Squad A: gathering ({seconds_left} s left on site)")
        }
    }
}

fn format_map_panel(game: &Game) -> String {
    format!(
        "Gather missions available: {}\n{}",
        game.world.available_gather_missions,
        squad_primary_line(game)
    )
}

fn format_base_panel(game: &Game) -> String {
    format!("Silver: {}", game.base.silver)
}
```

Wire panels:

```rust
    let map = Paragraph::new(format_map_panel(game))
        .block(Block::default().title("Map").borders(Borders::ALL));
    let base = Paragraph::new(format_base_panel(game))
        .block(Block::default().title("Base").borders(Borders::ALL));
    let units = Paragraph::new(squad_primary_line(game))
        .block(Block::default().title("Units").borders(Borders::ALL));
```

- [ ] **Step 2: Run `cargo build`**

Expected: SUCCESS.

- [ ] **Step 3: Smoke `cargo run`**

Expect: Map shows availability + squad line; Base shows Silver; Units mirrors squad status; footer unchanged.

Quit with `q`.

- [ ] **Step 4: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat(ui): readable gather/silver MVP panels"
```

---

## Self-review (plan vs vision spec)

| Spec requirement | Covered by |
|------------------|-----------|
| Autonomous squad picks gather mission | Task 4 Idle branch |
| Complete and return → base Silver | Task 4 `SILVER_PER_GATHER` |
| Repeat loop | Idle → gather again Task 4 test + second cycle optional test below |
| One mission type, one resource | Constants + structs |
| Map / Base / Units anchors | Task 5 |
| Build on MVP-1 tick | `tick()` unchanged at call sites; simulation inside |

**Optional extra test** (recommended but not blocking): extend Task 4 with second `tick(1000)×4` block asserting `silver == 20`.

**Placeholder scan:** None intentional; numeric constants fixed in table above.

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-09-gather-silver-mvp-plan.md`. Two execution options:

**1. Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — run tasks sequentially in this session using executing-plans, batch execution with checkpoints.

Which approach?
