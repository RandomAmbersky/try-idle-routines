# Squad capacity and mission silver pool Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement [2026-05-11 squad capacity / mission silver pool spec](../specs/2026-05-11-squad-capacity-mission-silver-pool-design.md): squad cargo hold with capacity, per-mission silver pools, silver credited only at base unload, missions removed only when depleted, and routing (full hold → base; empty site + other missions → closest next from current cell; else base even with partial hold).

**Architecture:** Introduce `GatherMission { cell, silver_initial, silver_remaining }` in `core`, extend `Squad` with `cargo_silver` and `cargo_capacity`, replace `World.active_missions: Vec<(u16,u16)>` with `Vec<GatherMission>`. Centralize routing decisions in `Game::simulate_second` after each gather interval. Extend `world_gen` for pool-filled missions and `pick_closest_gather_mission_index(from, missions)` (silver > 0, same tie-break as today). UI reads `m.cell` and shows cargo plus optional mission pool text.

**Tech Stack:** Rust 2024, existing crate layout (`src/core/mod.rs`, `src/core/world_gen.rs`, `src/ui/mod.rs`, `src/ui/detail.rs`), `cargo test` at repo root.

**Worktree:** If you use git worktrees for isolation, create one before Task 1; otherwise implement on your current branch.

---

## File map

| File | Role |
|------|------|
| `src/core/mod.rs` | `GatherMission`, `World`, `Squad`, `simulate_second` gather/unload/routing, `pub const` defaults, `new_from_layout_for_test`, in-module tests |
| `src/core/world_gen.rs` | `generate_base_and_three_missions` builds `GatherMission` with default pool; `pick_closest_gather_mission_index`; update existing world_gen tests |
| `src/ui/mod.rs` | `map_viewport_points`, `map_target_at_cell`, `map_text` mission iteration, render test layout |
| `src/ui/detail.rs` | Mission panel copy, squad detail cargo line |

No changes required to `src/app/mod.rs` for correctness (`gathering_just_completed` stays on gather interval completion).

---

### Task 1: Domain types and constants (`GatherMission`, `Squad` cargo, `World`)

**Files:**
- Modify: `src/core/mod.rs` (structs, imports, `World`, `Squad`, `Units::default`)
- Modify: `src/core/world_gen.rs` (signature stubs only if needed to compile — prefer full Task 2 in same session to avoid broken `main`)

**Note:** To keep `cargo build` green after this task, either complete Task 2 in the same commit or temporarily add a `type MissionCell = (u16,u16);` bridge — **preferred:** implement Task 1 and Task 2 together in one commit (combine Task 1 + 2 steps below into one pass).

- [ ] **Step 1: Add types and constants in `src/core/mod.rs`**

After `SILVER_PER_GATHER`, add:

```rust
/// Initial and remaining silver on a gather site (spec: mission stays until `silver_remaining == 0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GatherMission {
    pub cell: (u16, u16),
    pub silver_initial: u64,
    pub silver_remaining: u64,
}

impl GatherMission {
    pub fn new(cell: (u16, u16), silver: u64) -> Self {
        Self {
            cell,
            silver_initial: silver,
            silver_remaining: silver,
        }
    }
}

pub const DEFAULT_MISSION_SILVER_POOL: u64 = 100;
pub const DEFAULT_SQUAD_CARGO_CAPACITY: u64 = 30;
```

Change `World` to:

```rust
#[derive(Debug, Default)]
pub struct World {
    pub base_cell: (u16, u16),
    pub active_missions: Vec<GatherMission>,
}
```

Change `Squad` to:

```rust
#[derive(Debug)]
pub struct Squad {
    pub state: SquadState,
    pub path_index: usize,
    pub cargo_silver: u64,
    pub cargo_capacity: u64,
}
```

Update `Units::default` squad initializer:

```rust
squads: vec![Squad {
    state: SquadState::IdleAtBase,
    path_index: 0,
    cargo_silver: 0,
    cargo_capacity: DEFAULT_SQUAD_CARGO_CAPACITY,
}],
```

- [ ] **Step 2: Run `cargo check`**

Run: `cargo check`

Expected: **FAIL** with errors in `world_gen.rs` and `simulate_second` / tests until Task 2–3 are done.

---

### Task 2: `world_gen` — generation and closest-index picker

**Files:**
- Modify: `src/core/world_gen.rs`
- Modify: `src/core/mod.rs` (`pub use` line: export `pick_closest_gather_mission_index` instead of or in addition to tuple-based picker — **remove** `pick_closest_mission_index` once all call sites use the new function)

Replace `pick_closest_mission_index` with:

```rust
/// Missions with `silver_remaining == 0` are ignored. Tie-break: shortest `outbound_route_len`, then lexicographic `(row, col)` on `cell`, then lower index (same policy as the old tuple picker).
pub fn pick_closest_gather_mission_index(
    from: (u16, u16),
    missions: &[GatherMission],
) -> Option<usize> {
    let mut best_i: Option<usize> = None;
    let mut best_len = usize::MAX;
    let mut best_key = (u16::MAX, u16::MAX);
    for (i, m) in missions.iter().enumerate() {
        if m.silver_remaining == 0 {
            continue;
        }
        let len = outbound_route_len(from, m.cell);
        let key = (m.cell.1, m.cell.0);
        let better = match best_i {
            None => true,
            Some(j) => {
                len < best_len
                    || (len == best_len && key < best_key)
                    || (len == best_len && key == best_key && i < j)
            }
        };
        if better {
            best_len = len;
            best_i = Some(i);
            best_key = key;
        }
    }
    best_i
}
```

Update `generate_base_and_three_missions` return type and loop body so each accepted `m` becomes:

```rust
missions.push(GatherMission::new(m, crate::core::DEFAULT_MISSION_SILVER_POOL));
```

(use `super::GatherMission` and constant from parent module depending on your import style — avoid a `core` → `core` cycle: either move constants to `world_gen` and re-export from `mod.rs`, or pass pool as argument; **simplest:** `use super::{GatherMission, DEFAULT_MISSION_SILVER_POOL};` from `world_gen` which is a submodule of `core`)

Update `generate_base_and_three_missions` signature:

```rust
pub fn generate_base_and_three_missions<R: Rng>(rng: &mut R) -> ((u16, u16), Vec<GatherMission>)
```

Update `src/core/world_gen.rs` tests `pick_closest_tie_breaks_by_row_then_col` to build missions:

```rust
let missions = vec![GatherMission::new(a, 1), GatherMission::new(b, 1)];
let idx = pick_closest_gather_mission_index(base, &missions).unwrap();
```

and `generates_three_distinct_missions_and_base` to assert on `missions[i].cell` and `missions[i].silver_remaining == DEFAULT_MISSION_SILVER_POOL`.

- [ ] **Step 3: Run `cargo test -p try-idle-routines world_gen`**

Run: `cargo test world_gen`

Adjust package name if the crate is not `try-idle-routines` (use `cargo test world_gen` from repo root).

Expected: PASS for `world_gen` tests once `GatherMission` imports resolve.

---

### Task 3: `Game::new_from_layout_for_test` and `IdleAtBase` dispatch

**Files:**
- Modify: `src/core/mod.rs`

Change signature:

```rust
pub fn new_from_layout_for_test(
    base_cell: (u16, u16),
    active_missions: Vec<GatherMission>,
) -> Self
```

`IdleAtBase` branch: replace `active_missions.is_empty()` with **no dispatch when no mission has silver**:

```rust
if !self
    .world
    .active_missions
    .iter()
    .any(|m| m.silver_remaining > 0)
{
    return;
}
let mission_i = pick_closest_gather_mission_index(
    self.world.base_cell,
    &self.world.active_missions,
)
.expect("non-empty silver mission list has a closest mission");
let mission_cell = self.world.active_missions[mission_i].cell;
self.route_to_mission =
    crate::map_geometry::route_outbound_cells_from(self.world.base_cell, mission_cell);
```

- [ ] **Step 4: Run `cargo check`**

Expected: still FAIL on gathering branch and tests referencing tuples.

---

### Task 4: Gather completion, mission update, routing, base unload

**Files:**
- Modify: `src/core/mod.rs` (`simulate_second` for `Gathering` and `MovingToBase`)

**4a. `Gathering` when `seconds_left == 1`**

Replace the body that adds to `base.silver`, removes mission unconditionally, and always routes home with logic equivalent to:

```rust
self.gathering_just_completed = true;
let mission_cell = self
    .route_to_mission
    .last()
    .copied()
    .expect("gathering requires non-empty route ending on mission cell");

let idx = self
    .world
    .active_missions
    .iter()
    .position(|m| m.cell == mission_cell)
    .expect("gathering on a cell that exists in active_missions");

let squad = &mut self.units.squads[0];
let room = squad.cargo_capacity.saturating_sub(squad.cargo_silver);
let pool = self.world.active_missions[idx].silver_remaining;
let take = SILVER_PER_GATHER.min(pool).min(room);
squad.cargo_silver = squad.cargo_silver.saturating_add(take);
self.world.active_missions[idx].silver_remaining = self.world.active_missions[idx]
    .silver_remaining
    .saturating_sub(take);

if self.world.active_missions[idx].silver_remaining == 0 {
    self.world.active_missions.remove(idx);
}

let squad = &mut self.units.squads[0];
let hold_full = squad.cargo_silver >= squad.cargo_capacity;
let site_empty = !self
    .world
    .active_missions
    .iter()
    .any(|m| m.cell == mission_cell);

if hold_full {
    self.start_route_to_base_from(mission_cell);
    squad.state = SquadState::MovingToBase;
    squad.path_index = 0;
} else if site_empty {
    if let Some(next_i) =
        pick_closest_gather_mission_index(mission_cell, &self.world.active_missions)
    {
        let next_cell = self.world.active_missions[next_i].cell;
        self.route_to_mission =
            crate::map_geometry::route_outbound_cells_from(mission_cell, next_cell);
        squad.state = SquadState::MovingToMission;
        squad.path_index = 0;
    } else {
        self.start_route_to_base_from(mission_cell);
        squad.state = SquadState::MovingToBase;
        squad.path_index = 0;
    }
} else {
    squad.state = SquadState::Gathering {
        seconds_left: GATHER_DURATION_SECS,
    };
    squad.path_index = 0;
}
```

Add a private helper on `impl Game`:

```rust
fn start_route_to_base_from(&mut self, from_cell: (u16, u16)) {
    let steps = crate::map_geometry::route_outbound_cells_from(from_cell, self.world.base_cell);
    let mut home = Vec::with_capacity(steps.len().saturating_add(1));
    home.push(from_cell);
    home.extend(steps);
    self.route_to_mission = home;
}
```

(Reuse the same construction you already use for return routes.)

**4b. `MovingToBase` when squad reaches base**

When advancing to `IdleAtBase` (both the `route_len == 0` early return path and the `path_index` reached `last` path), **before** setting idle:

```rust
let squad = &mut self.units.squads[0];
let unload = squad.cargo_silver;
if unload > 0 {
    self.base.silver = self.base.silver.saturating_add(unload);
    squad.cargo_silver = 0;
}
squad.state = SquadState::IdleAtBase;
squad.path_index = 0;
```

- [ ] **Step 5: Run `cargo test`**

Expected: several existing `core` tests still fail until updated in Task 5.

---

### Task 5: Rewrite `core` tests for new semantics

**Files:**
- Modify: `src/core/mod.rs` (`#[cfg(test)] mod tests`)

Replace tuple layouts with `GatherMission::new((c, r), pool)`.

**5a. Update `new_from_layout_for_test` call sites inside `core` tests**

Example:

```rust
let mut g = Game::new_from_layout_for_test(
    (10, 50),
    vec![
        GatherMission::new((12, 48), 20),
        GatherMission::new((20, 50), 100),
    ],
);
```

**5b. Replace `autonomous_gather_loop_adds_silver_every_gather_cycle`**

New name and behavior: e.g. `autonomous_loop_unloads_silver_at_base_and_removes_depleted_missions` — drive ticks until two unload cycles match expected `base.silver` and mission count (derive expected totals from pool sizes and `SILVER_PER_GATHER` / capacity).

**5c. Replace `gather_completion_drops_mission_from_active_list_while_returning`**

Rename to `gather_completion_keeps_mission_until_pool_empty` (or split into two tests):

- Layout: one mission with `silver_remaining > SILVER_PER_GATHER` after one take.
- After one gather tick: `base.silver == 0`, `cargo_silver == SILVER_PER_GATHER`, mission still in list with reduced pool, state `Gathering` or `MovingToBase` per spec (if hold not full and pool left → still `Gathering`).

Concrete scenario:

```rust
let mission = (14u16, 50u16);
let mut g = Game::new_from_layout_for_test(
    (10, 50),
    vec![GatherMission::new(mission, SILVER_PER_GATHER + 5), GatherMission::new((30, 50), 10)],
);
g.route_to_mission = crate::map_geometry::route_outbound_cells_from(g.world.base_cell, mission);
g.units.squads[0].state = SquadState::Gathering { seconds_left: 1 };
g.units.squads[0].cargo_capacity = 100;
g.tick(SIMULATED_SECOND_MS);
assert_eq!(g.base.silver, 0);
assert_eq!(g.units.squads[0].cargo_silver, SILVER_PER_GATHER);
assert!(g.world.active_missions.iter().any(|m| m.cell == mission && m.silver_remaining == 5));
assert!(matches!(
    g.units.squads[0].state,
    SquadState::Gathering { .. }
));
```

**5d. Add acceptance tests from spec**

1. `chain_to_second_mission_without_base_silver_when_first_depletes`: pools `(12,48): 20`, `(20,50): 100`, capacity `30`. After first site empty, `base.silver == 0`, `cargo_silver > 0`, squad `MovingToMission` toward second cell (assert `route_to_mission.last()`).

2. `hold_full_returns_while_mission_has_silver_left`: one mission pool `100`, capacity `25` — after gather completion that fills hold, `MovingToBase`, mission still listed with `silver_remaining == 75` (adjust for exact `take` arithmetic).

3. `last_mission_empty_partial_hold_returns_to_base`: single mission pool `15`, capacity `40` — after depletion, `MovingToBase`, mission removed, `cargo_silver == 15`, then after full return `base.silver == 15`.

- [ ] **Step 6: Run `cargo test`**

Run: `cargo test`

Expected: PASS all.

- [ ] **Step 7: Commit**

```bash
git add src/core/mod.rs src/core/world_gen.rs
git commit -m "feat(core): squad cargo, mission silver pools, routing"
```

---

### Task 6: UI — map and detail

**Files:**
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/detail.rs`

**6a. `map_viewport_points`**

Replace:

```rust
pts.extend_from_slice(&game.world.active_missions);
```

with:

```rust
pts.extend(game.world.active_missions.iter().map(|m| m.cell));
```

**6b. `map_target_at_cell`**

Replace `.any(|&m| m == (col, row))` with `.any(|m| m.cell == (col, row))`.

**6c. `map_text` mission loop**

```rust
for m in &game.world.active_missions {
    let (mc, mr) = m.cell;
    // same glyph logic using (mc, mr)
}
```

**6d. `detail.rs` — `Selection::Mission`**

After the "Mission" title, add a line for total remaining silver, e.g.:

```rust
let total_site: u64 = game.world.active_missions.iter().map(|m| m.silver_remaining).sum();
lines.push(Line::from(format!("Silver on sites (sum): {total_site}")));
```

Keep "Remaining sites" as `game.world.active_missions.len()` (all active have silver > 0 once core removes depleted entries).

**6e. `Selection::Squad`**

After state lines, if squad resolved:

```rust
lines.push(Line::from(format!(
    "Cargo: {} / {}",
    squad.cargo_silver, squad.cargo_capacity
)));
```

**6f. `Selection::Base` roster (optional)**

For idle squads, append cargo `0 / capacity` for consistency.

- [ ] **Step 8: Run `cargo test`**

Expected: PASS including `ui::render_tests`.

**6g. Update `render_uses_provided_layout_for_map_detail_and_footer` in `src/ui/mod.rs`**

```rust
use crate::core::{Game, GatherMission, DEFAULT_MISSION_SILVER_POOL};

let game = Game::new_from_layout_for_test(
    (1, 50),
    vec![
        GatherMission::new((75, 50), DEFAULT_MISSION_SILVER_POOL),
        GatherMission::new((80, 55), DEFAULT_MISSION_SILVER_POOL),
        GatherMission::new((70, 45), DEFAULT_MISSION_SILVER_POOL),
    ],
);
let (mission_col, mission_row) = game.world.active_missions[0].cell;
```

- [ ] **Step 9: Commit**

```bash
git add src/ui/mod.rs src/ui/detail.rs
git commit -m "feat(ui): show cargo and mission silver; map uses mission cells"
```

---

## Spec coverage (self-review)

| Spec section | Task |
|--------------|------|
| Squad `cargo_silver` / `cargo_capacity` | Task 1, 6e |
| Mission pool + `silver_initial` | Task 1 (`GatherMission`), Task 2 generation |
| Transfer `take = min(SILVER_PER_GATHER, pool, room)` | Task 4a |
| No base credit on gather | Task 4a |
| Routing order full / empty+next / continue gather | Task 4a |
| Next mission closest from current cell | Task 4a (`pick_closest_gather_mission_index(mission_cell, ...)` ) |
| No other missions → base partial hold | Task 4a `else` branch |
| Remove mission only when pool 0 | Task 4a remove after subtract |
| Unload at base | Task 4b |
| UI cargo + site silver | Task 6 |
| Acceptance tests | Task 5d |

**Placeholder scan:** None intentional; all code blocks are concrete.

**Type consistency:** `GatherMission` used in `World`, `world_gen`, UI via `.cell`; picker name `pick_closest_gather_mission_index` everywhere.

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-11-squad-capacity-mission-silver-pool-plan.md`. Two execution options:

**1. Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration. **REQUIRED SUB-SKILL:** superpowers:subagent-driven-development.

**2. Inline Execution** — run tasks in this session with superpowers:executing-plans and checkpoints between tasks.

Which approach do you want?
