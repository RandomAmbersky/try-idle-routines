# Random base, three gather missions, lifecycle — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the spec in `docs/superpowers/specs/2026-05-11-random-base-three-gather-missions-design.md`: random base and three gather sites per new game, remove a mission from the active list when gathering completes, autonomous squad picks the next closest mission by outbound route length with `(row, col)` tie-break, map/UI/viewport/selection updated accordingly, deterministic tests.

**Architecture:** Move shared map dimensions to `constants` so `core` never imports `ui`. Move **pure grid routing** (`cell_step_toward`, Bresenham, `route_outbound_cells_from`, `outbound_route_len`) into a new **`map_geometry`** module (depends only on `constants`) so `core/world_gen.rs` can call `crate::map_geometry::outbound_route_len` without a **`core` → `ui` cycle**. `ui/map_layout.rs` keeps ratatui viewport helpers and **re-exports** geometry for existing `crate::ui::route_outbound_cells` call sites during migration. Extend `Game` with `base_cell`, `active_missions: Vec<(u16, u16)>`, drop `available_gather_missions` in favor of list emptiness + squad state. Add `rand` for `Game::new()`; tests use `Game::new_from_layout_for_test`. At the **start** of each `Game::tick`, set `gathering_just_completed = false`; set it to `true` in `simulate_second` when gathering completes; `App` reads the flag **after** `tick` returns to clear `Selection::Mission`.

**Tech Stack:** Rust 2024 edition, `ratatui`, `crossterm`, new dependency `rand` (pin e.g. `rand = "0.8"` in `Cargo.toml`).

---

## File structure (before tasks)

| File | Responsibility |
|------|------------------|
| `Cargo.toml` | Add `rand` dependency. |
| `src/constants.rs` (new) | `MAP_WIDTH`, `MAP_HEIGHT` — single source shared by `core` and `ui`. |
| `src/main.rs` | `mod constants;` and `mod map_geometry;` near top (order: `constants` before `core` if `core` needs it). |
| `src/map_geometry.rs` (new) | Pure `(col,row)` routing: `cell_step_toward`, `bresenham_inclusive`, `route_outbound_cells_from`, `outbound_route_len` — copied from current `map_layout.rs`, using `crate::constants::{MAP_WIDTH, MAP_HEIGHT}` only. |
| `src/ui/map_layout.rs` | Import map size from `crate::constants`; delegate routing to `crate::map_geometry` and `pub use` the route helpers so `crate::ui::route_outbound_cells_from` stays available; viewport helpers take base + mission set + optional squad cell. |
| `src/core/mod.rs` | `Game` / `World` data and `simulate_second` loop; layout generation entry for `Game::new`; test constructor; mission removal; closest-mission dispatch; `gathering_just_completed` flag. |
| `src/core/world_gen.rs` (new) | `generate_base_and_three_missions<R: Rng>` resampling until four distinct cells; `pick_closest_mission_index` using `crate::map_geometry::outbound_route_len` and `crate::constants` for bounds. |
| `src/app/mod.rs` | After `game.tick(...)`, if flag then `if selection == Mission { selection = None }`; sync route dimensions from constants; remove reliance on static `cell_for_*`. |
| `src/ui/mod.rs` | `map_text`, `squad_cell_on_map`, exports; mission hit-test using `game.active_missions` + current target cell for `!` glyph rules. |
| `src/ui/detail.rs` | Mission panel: remaining count, optional coordinate list; remove `available_gather_missions` display. |

---

### Task 1: Shared map constants

**Files:**
- Create: `src/constants.rs`
- Modify: `src/main.rs`
- Modify: `src/ui/map_layout.rs`

- [ ] **Step 1: Add `src/constants.rs`**

```rust
//! Shared map dimensions (logical grid). Used by `core` world generation and `ui` layout.

pub const MAP_WIDTH: u16 = 100;
pub const MAP_HEIGHT: u16 = 100;
```

- [ ] **Step 2: Wire module in `src/main.rs`**

After `mod app;`, add:

```rust
mod constants;
```

- [ ] **Step 3: Re-point `src/ui/map_layout.rs`**

Remove local `pub const MAP_WIDTH` / `MAP_HEIGHT` and add:

```rust
pub use crate::constants::{MAP_HEIGHT, MAP_WIDTH};
```

Keep `map_bounds()` returning `Rect::new(0, 0, MAP_WIDTH, MAP_HEIGHT)`.

- [ ] **Step 4: Verify build**

Run:

```bash
cd /Users/random/restore/try-idle-routines && cargo build
```

Expected: **success** (no warnings required to be zero).

- [ ] **Step 5: Commit**

```bash
git add src/constants.rs src/main.rs src/ui/map_layout.rs
git commit -m "refactor: centralize map dimensions in constants module"
```

---

### Task 2: `map_geometry` — pure routing (no `ui`, no ratatui)

**Files:**
- Create: `src/map_geometry.rs`
- Modify: `src/main.rs` (`mod map_geometry;`)
- Modify: `src/ui/map_layout.rs` (delegate + re-export)

- [ ] **Step 1: Add `src/map_geometry.rs`**

Move **verbatim** (adjusting only `inner` width/height) these pieces from the current `src/ui/map_layout.rs`: `cell_step_toward`, `bresenham_inclusive`, then add:

```rust
//! King-adjacent grid routing. Depends only on `constants` so `core` can use it without importing `ui`.

use crate::constants::{MAP_HEIGHT, MAP_WIDTH};

// ... cell_step_toward, bresenham_inclusive (use MAP_WIDTH, MAP_HEIGHT as max bounds) ...

pub fn route_outbound_cells_from(base: (u16, u16), mission: (u16, u16)) -> Vec<(u16, u16)> {
    if MAP_WIDTH == 0 || MAP_HEIGHT == 0 {
        return Vec::new();
    }
    let start = cell_step_toward(base, mission);
    bresenham_inclusive(start, mission, MAP_WIDTH, MAP_HEIGHT)
}

pub fn outbound_route_len(base: (u16, u16), mission: (u16, u16)) -> usize {
    route_outbound_cells_from(base, mission).len()
}
```

- [ ] **Step 2: Tests in `map_geometry.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn route_steps_are_one_cell_apart(r: &[(u16, u16)]) -> bool {
        r.windows(2).all(|w| {
            let (a, b) = (w[0], w[1]);
            let dc = a.0.abs_diff(b.0);
            let dr = a.1.abs_diff(b.1);
            dc <= 1 && dr <= 1 && (dc + dr > 0)
        })
    }

    #[test]
    fn outbound_route_respects_custom_base_and_mission() {
        let base = (10u16, 50u16);
        let mission = (12u16, 48u16);
        let r = route_outbound_cells_from(base, mission);
        assert!(!r.is_empty());
        assert_eq!(*r.last().unwrap(), mission);
        assert!(route_steps_are_one_cell_apart(&r));
    }

    #[test]
    fn route_len_matches_vec_len() {
        let base = (5u16, 5u16);
        let m = (20u16, 8u16);
        assert_eq!(outbound_route_len(base, m), route_outbound_cells_from(base, m).len());
    }
}
```

Run: `cargo test outbound_route_respects` — Expected: **PASS** after implementation.

- [ ] **Step 3: Wire `map_layout.rs`**

Add `pub use crate::map_geometry::{outbound_route_len, route_outbound_cells_from};` and change existing `route_outbound_cells()` to call `route_outbound_cells_from(cell_for_base(), cell_for_mission())`. Remove duplicate private `bresenham_inclusive` / inlined logic now living in `map_geometry` (keep `map_bounds`, viewport, `cell_for_base` until Task 9 cleanup).

Run: `cargo test` — Expected: existing `route_steps_are_one_cell_apart` in `map_layout` tests still **PASS**.

- [ ] **Step 4: Commit**

```bash
git add src/map_geometry.rs src/main.rs src/ui/map_layout.rs
git commit -m "refactor: extract map_geometry for routing without ui dependency"
```

---

### Task 3: `pick_closest_mission_index` + world generation

**Files:**
- Create: `src/core/world_gen.rs`
- Modify: `src/core/mod.rs` (`mod world_gen;`)
- Modify: `Cargo.toml` (`rand = "0.8"`)

- [ ] **Step 1: Add dependency**

```toml
rand = "0.8"
```

Run `cargo build`. Expected: **success**.

- [ ] **Step 2: Tie-break test (complete, no placeholders)**

In `src/core/world_gen.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::map_geometry::outbound_route_len;

    use super::*;

    #[test]
    fn pick_closest_tie_breaks_by_row_then_col() {
        let base = (50u16, 50u16);
        let mut pair: Option<((u16, u16), (u16, u16))> = None;
        'outer: for r1 in 0u16..80u16 {
            for c1 in 0u16..80u16 {
                let a = (c1, r1);
                if a == base {
                    continue;
                }
                let la = outbound_route_len(base, a);
                for r2 in 0u16..80u16 {
                    for c2 in 0u16..80u16 {
                        let b = (c2, r2);
                        if b == base || b == a {
                            continue;
                        }
                        if outbound_route_len(base, b) == la {
                            pair = Some((a, b));
                            break 'outer;
                        }
                    }
                }
            }
        }
        let (a, b) = pair.expect("find two distinct cells with equal route len from base");
        assert_eq!(outbound_route_len(base, a), outbound_route_len(base, b));
        let missions = vec![a, b];
        let idx = pick_closest_mission_index(base, &missions).unwrap();
        let want = if (a.1, a.0) <= (b.1, b.0) { 0 } else { 1 };
        assert_eq!(idx, want);
    }
}
```

Leave `pick_closest_mission_index` unimplemented first. Run: `cargo test pick_closest_tie_breaks` — Expected: **FAIL** (not found or wrong index).

- [ ] **Step 3: Implement `world_gen.rs`**

```rust
use rand::Rng;

use crate::constants::{MAP_HEIGHT, MAP_WIDTH};
use crate::map_geometry::outbound_route_len;

/// Tie-break: lexicographically smallest `(row, col)` = `(cell.1, cell.0)` for `(col, row)` tuples.
pub fn pick_closest_mission_index(base: (u16, u16), missions: &[(u16, u16)]) -> Option<usize> {
    if missions.is_empty() {
        return None;
    }
    let mut best_i = 0usize;
    let mut best_len = usize::MAX;
    let mut best_key = (u16::MAX, u16::MAX);
    for (i, &cell) in missions.iter().enumerate() {
        let len = outbound_route_len(base, cell);
        let key = (cell.1, cell.0);
        let better = len < best_len
            || (len == best_len && key < best_key)
            || (len == best_len && key == best_key && i < best_i);
        if better {
            best_len = len;
            best_i = i;
            best_key = key;
        }
    }
    Some(best_i)
}

pub fn generate_base_and_three_missions<R: Rng>(rng: &mut R) -> ((u16, u16), Vec<(u16, u16)>) {
    loop {
        let bc = rng.gen_range(0..MAP_WIDTH);
        let br = rng.gen_range(0..MAP_HEIGHT);
        let base = (bc, br);
        let mut missions = Vec::with_capacity(3);
        let mut ok = true;
        for _ in 0..3 {
            let mut tries = 0u32;
            loop {
                let c = rng.gen_range(0..MAP_WIDTH);
                let r = rng.gen_range(0..MAP_HEIGHT);
                let m = (c, r);
                tries += 1;
                if tries > 10_000 {
                    ok = false;
                    break;
                }
                if m != base && !missions.contains(&m) {
                    missions.push(m);
                    break;
                }
            }
            if !ok {
                break;
            }
        }
        if ok && missions.len() == 3 {
            return (base, missions);
        }
    }
}
```

Add `#[test] fn generates_three_distinct_missions_and_base()` using `StdRng::seed_from_u64(12345)` asserting `len == 3`, all distinct, none equals base.

Run: `cargo test` — Expected: **all PASS**.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/core/world_gen.rs src/core/mod.rs
git commit -m "feat(core): world RNG layout and closest-mission picker"
```

---

### Task 4: `Game` layout fields and `simulate_second` behavior

**Files:**
- Modify: `src/core/mod.rs`

- [ ] **Step 1: Extend `World` / `Game`**

Replace `World { available_gather_missions }` with something like:

```rust
#[derive(Debug)]
pub struct World {
    pub base_cell: (u16, u16),
    pub active_missions: Vec<(u16, u16)>,
}

#[derive(Debug)]
pub struct Game {
    pub world: World,
    // ...
    /// True for one `tick` after gathering completes (silver granted, MovingToBase begins).
    pub gathering_just_completed: bool,
}
```

Add test-only or `pub` constructor:

```rust
impl Game {
    pub fn new_from_layout_for_test(base_cell: (u16, u16), active_missions: Vec<(u16, u16)>) -> Self {
        let mut g = Game {
            world: World { base_cell, active_missions },
            // ... ticks, accum, route empty, route_map_w/h from constants, units default
            gathering_just_completed: false,
        };
        g.route_map_w = crate::constants::MAP_WIDTH;
        g.route_map_h = crate::constants::MAP_HEIGHT;
        g
    }
}
```

`Game::new()` should call `generate_base_and_three_missions(&mut thread_rng())` and fill `world`, then set `route_map_w/h`, leave `route_to_mission` empty until first dispatch.

- [ ] **Step 2: Write failing integration test `no_dispatch_when_no_missions`**

```rust
#[test]
fn no_dispatch_when_missions_empty() {
    let mut g = Game::new_from_layout_for_test((5, 5), vec![]);
    g.units.squads[0].state = SquadState::IdleAtBase;
    for _ in 0..20 {
        g.tick(SIMULATED_SECOND_MS);
    }
    assert!(matches!(g.units.squads[0].state, SquadState::IdleAtBase));
    assert!(g.route_to_mission.is_empty());
}
```

Run test — implement minimal `IdleAtBase` branch: **do not** start `MovingToMission` when `active_missions.is_empty()`.

- [ ] **Step 3: Implement dispatch from idle**

When `IdleAtBase` and `!active_missions.is_empty()`:

1. `let idx = pick_closest_mission_index(world.base_cell, &world.active_missions).unwrap();`
2. `let mission = world.active_missions[idx];`
3. `route_to_mission = crate::map_geometry::route_outbound_cells_from(world.base_cell, mission);`
4. Set squad `MovingToMission`, `path_index = 0`.

**Do not** remove mission from list yet.

Remove all uses of `available_gather_missions`.

- [ ] **Step 4: Gathering completion removes mission**

In `Gathering { seconds_left }` when `seconds_left == 1`:

- Add silver (unchanged).
- Set `gathering_just_completed = true`.
- Remove **one** mission from `world.active_missions` where `mission_cell == *route_to_mission.last().expect("non-empty while gathering on site")` (if route empty edge case, match spec: treat as instant gather site = base — should not happen if dispatch always builds non-empty route when distance > 0; if base == mission illegal).

Then build a **fresh** return polyline `mission → base` with `route_outbound_cells_from(mission, base_cell)`, prepend `mission` so `path_index == 0` is the gather cell, set `MovingToBase`, `path_index = 0`, and advance **forward** along that vector each second (not reverse of outbound).

- [ ] **Step 5: `tick` and `gathering_just_completed`**

Clear the flag **once at the very start** of `tick` (before the `while self.accum_ms >= SIMULATED_SECOND_MS` loop). Inside `simulate_second`, when gathering completes (`seconds_left == 1` branch), set `gathering_just_completed = true`. If one `tick` processes multiple simulated seconds, only the **last** `simulate_second` in that tick can leave the flag true — acceptable because gathering completion is at most one transition per tick for a single squad.

```rust
pub fn tick(&mut self, ms: u64) {
    self.gathering_just_completed = false;
    self.accum_ms += ms;
    while self.accum_ms >= SIMULATED_SECOND_MS {
        self.accum_ms -= SIMULATED_SECOND_MS;
        self.ticks += 1;
        self.simulate_second();
    }
}
```

`App` reads `gathering_just_completed` **after** `game.tick(...)` returns.

- [ ] **Step 6: Update existing tests in `core/mod.rs`**

Replace `sync_route_like_app` + `available_gather_missions` assertions with `new_from_layout_for_test` using a fixed base and 1–3 missions, or seed RNG in `Game::new()` test with predictable layout (prefer constructor).

Run: `cargo test` — Expected: **all PASS**.

- [ ] **Step 7: Commit**

```bash
git add src/core/mod.rs
git commit -m "feat(core): multi-mission gather loop and removal on completion"
```

---

### Task 5: App route sync + selection clear

**Files:**
- Modify: `src/app/mod.rs`

- [ ] **Step 1: After every `game.tick` in `run` loop**

Where `Action::Tick` and idle-running path call `tick`, add:

```rust
if self.game.gathering_just_completed && self.selection == Selection::Mission {
    self.selection = Selection::None;
}
```

(Also after `Action::Step` when paused.)

- [ ] **Step 2: `sync_game_route`**

Replace dimension check: if `game.route_map_w != MAP_WIDTH`, set `route_map_*` from `crate::constants` and **recompute** `route_to_mission` only when needed — preferably **remove** `sync_game_route`’s reassignment of full route from static `route_outbound_cells()`; route now owned entirely by `core` from dispatch. If `sync_game_route` becomes redundant, delete it and clamp `path_index` only inside `core` when map dimensions change (or never if dims constant).

Minimal change: **`sync_game_route` only sets `route_map_w/h` from constants** and clamps `path_index` to `route_to_mission.len().saturating_sub(1)` without overwriting `route_to_mission`.

- [ ] **Step 3: Mouse `map_target_at_cell`**

Replace static mission check with: mission hit if `(col,row)` is in `game.world.active_missions` **or** is current gather/outbound target still shown with `!` — hit-test only **active list** per spec (returning: mission cell not in list → **Empty** unless squad occupies).

Wire `ui::map_target_at_cell` signature to `map_target_at_cell(col, row, game: &Game) -> MapTarget` in `ui/mod.rs` or `map_layout.rs`, update `app/mod.rs` call site.

- [ ] **Step 4: Run `cargo test` + manual `cargo run`**

Expected: tests pass; app runs.

- [ ] **Step 5: Commit**

```bash
git add src/app/mod.rs
git commit -m "fix(app): clear mission selection when site completes; sync route with core"
```

---

### Task 6: Map rendering, squad cell, viewport

**Files:**
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/map_layout.rs` (`map_view_origin` signature)

- [ ] **Step 1: `map_view_origin`**

Change to accept `base: (u16,u16)`, `mission_cells: &[(u16,u16)]`, `squad: Option<(u16,u16)>`, compute bounding box of all points, then apply same 1D viewport logic extended to min/max across all columns and rows (implement helper: collect all relevant coords into min/max col/row, expand by viewport size like current segment logic).

- [ ] **Step 2: `map_text`**

- Draw `B` at `game.world.base_cell`.
- For each `(c,r)` in `active_missions`, draw `M` unless that cell is the **current** outbound/gather target (squad `MovingToMission` with destination last cell of route, or `Gathering` on that cell) then `!`.
- Draw squads: `squad_cell_on_map` uses `game.world.base_cell` for idle; route indices for move/return; gathering on mission cell (same as today’s “toward base” step rule — align with spec: gathering occupies **mission** cell glyph co-location with `!`; squad `S` draws on that cell).

- [ ] **Step 3: `squad_cell_on_map`**

```rust
SquadState::IdleAtBase => Some(game.world.base_cell),
```

Remove `cell_step_toward(base, mission)` for idle.

- [ ] **Step 4: Update `render_tests`**

Use `Game::new_from_layout_for_test` with known coordinates so buffer assertions still find `B` and an `M`.

Run: `cargo test`

- [ ] **Step 5: Commit**

```bash
git add src/ui/mod.rs src/ui/map_layout.rs
git commit -m "feat(ui): map glyphs and viewport for dynamic missions"
```

---

### Task 7: Detail panel + remove stale counter

**Files:**
- Modify: `src/ui/detail.rs`

- [ ] **Step 1: Mission panel text**

Replace `Available: {}` with `available_gather_missions` by:

```rust
lines.push(Line::from(format!(
    "Remaining sites: {}",
    game.world.active_missions.len()
)));
```

Optionally list coordinates in compact form on following lines.

- [ ] **Step 2: Fix tests importing `Game::new`**

If `Game::new()` now random, tests that need deterministic state should use `new_from_layout_for_test`.

Run: `cargo test`

- [ ] **Step 3: Commit**

```bash
git add src/ui/detail.rs
git commit -m "fix(ui): mission detail matches active mission list"
```

---

### Task 8: Remove deprecated static layout helpers (cleanup)

**Files:**
- Modify: `src/ui/map_layout.rs`
- Modify: tests across crate

- [ ] **Step 1:** Remove or `#[cfg(test)]` gate `cell_for_base`, `cell_for_mission`, and `route_outbound_cells()` if nothing in production uses them after Tasks 4–6.

- [ ] **Step 2:** `cargo test` and commit `refactor: drop static base/mission helpers after dynamic layout`.

---

## Self-review (plan vs spec)

| Spec section | Tasks covering it |
|--------------|-------------------|
| Random base + 3 missions, distinct | Task 3 `generate_base_and_three_missions`, Task 4 `Game::new` |
| Remove mission at gather completion | Task 4 Step 4 |
| Shortest route next + `(row,col)` tie-break | Task 3 `pick_closest_mission_index`, Task 4 dispatch |
| No respawn when empty | Task 4 Step 2 test + idle branch |
| Return path new mission→base polyline | Task 2 `route_outbound_cells_from` reused; forward walk |
| No `core` → `ui` cycle | Task 2 `map_geometry` module |
| Map glyphs + `S` over `B` at idle | Task 6 |
| Viewport over all points | Task 6 Step 1 |
| Mission click only if active | Task 5 Step 3 |
| Clear `Selection::Mission` when site completes | Task 4 flag + Task 5 Step 1 |
| Detail remaining count | Task 7 |
| Deterministic tests / RNG injection | Tasks 3–4 constructors and seeded RNG |

**Placeholder scan:** None intentional; tie-break test discovers an equal-length pair by brute force so coordinates need no hand-tuning.

**Type consistency:** `Game::world.base_cell` and mission tuples are `(u16, u16)` matching `route_outbound_cells_from` throughout.

---

## Execution handoff

**Plan complete and saved to `docs/superpowers/plans/2026-05-11-random-base-three-gather-missions-plan.md`. Two execution options:**

1. **Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration. **REQUIRED SUB-SKILL:** superpowers:subagent-driven-development.

2. **Inline Execution** — execute tasks in this session using executing-plans with checkpoints. **REQUIRED SUB-SKILL:** superpowers:executing-plans.

**Which approach?**
