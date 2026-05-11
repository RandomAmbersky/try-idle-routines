# Squad cargo capacity and mission silver pool

**Status:** Approved intent (2026-05-11). Extends the gather/silver MVP described in [2026-05-09 idle UFO-inspired vision](2026-05-09-idle-ufo-inspired-vision-design.md) and implemented in `src/core/mod.rs`.

## Problem

The current loop moves silver straight into the base when a timed gather completes and removes the mission from the map in the same step. There is no squad carry limit and no partial depletion of a mission site.

## Goals

1. Each **squad** has a **cargo hold** with a maximum **capacity** and a current **cargo** amount (silver carried, not yet at the base).
2. Each **gather mission** has a finite **silver pool** on site. The mission **stays on the map** until that pool reaches **zero** (fully extracted by squad activity).
3. **Partial runs** are allowed: the squad may fill its hold (or exhaust the site) and then travel; silver on the base increases **only when the squad unloads at the base**, not when gathering finishes on the mission cell.

## Non-goals (this spec)

- Multiple squads splitting one mission pool concurrently (MVP remains one squad unless a later spec adds fleet rules).
- Different mission types, combat, or player-assigned routes beyond existing autonomous picking.
- Persistence/save format changes beyond what the core model needs.

## Data model

### Squad

- `cargo_silver: u64` — silver currently in the hold (0 ..= `cargo_capacity`).
- `cargo_capacity: u64` — maximum hold size (MVP may use a constant; storing on `Squad` keeps future per-squad upgrades straightforward).

### Mission (gather site)

Replace “mission as bare coordinates” with a structure that includes at least:

- `cell: (u16, u16)` — map position (unchanged semantics for routing).
- `silver_remaining: u64` — pool not yet taken by squads.

Optional for UI: `silver_initial: u64` (set at spawn) to show fill percentage; if omitted, UI can show only `silver_remaining` or derive from world generation.

## Simulation rules

### Transfer on gather completion

When a gather **work interval** completes on a mission cell (same time granularity as today’s `Gathering` phase):

1. Compute `room = cargo_capacity - cargo_silver`.
2. Compute `take = min(SILVER_PER_GATHER, silver_remaining, room)` (exact constant name may match existing code; the rule is **bounded by mission left, hold room, and per-tick payout cap**).
3. Apply: `cargo_silver += take`, `silver_remaining -= take`.
4. Do **not** add to `Base.silver` here.

### After a gather step: where the squad goes next

Evaluate in this **order**:

1. **Hold full** (`cargo_silver == cargo_capacity`): compute route **to base**, enter return/move phase; on arrival at base, move `cargo_silver` into `Base.silver` and clear hold.
2. **Current mission pool empty** (`silver_remaining == 0` for the active site):
   - If there exists **another** active mission with `silver_remaining > 0`: pick the **closest** such mission by path distance **from the squad’s current cell** (the mission cell just emptied), using the same routing/grid metric as outbound pathfinding. Excluding missions with `silver_remaining == 0`.
   - **Else** (no other missions with silver): compute route **to base** and unload as above, **even if the hold is not full**.

3. **Otherwise** (hold not full and site still has silver): continue gathering on the **same** cell (same gathering phase semantics as now).

### Mission lifetime on the map

- Remove a mission from the active list **only when** `silver_remaining == 0` after applying extraction for that site (typically when the squad leaves the decision point after a gather that emptied the pool, or equivalent consistent moment — implementation must not drop the cell while silver remains).

### Base unload

- When the squad reaches the base with `cargo_silver > 0`, add that amount to `Base.silver` and set `cargo_silver` to 0.

## Edge cases

- **No missions with silver, empty hold, idle at base:** no travel (unchanged).
- **Mission emptied, hold partial, no other missions:** route to base and unload (user-confirmed).
- **Arithmetic:** use `saturating_*` where appropriate to avoid panics on mis-tuned constants.

## UI

- **Base panel:** warehouse silver (unchanged meaning).
- **Units / detail:** show `cargo_silver / cargo_capacity` for the squad.
- **Map / mission detail:** show remaining site silver or a simple fraction if `silver_initial` exists.

## Testing (acceptance)

1. **Partial mission, two sites:** First site depletes before hold is full; squad routes to second site without visiting base; base silver still 0 until a later unload.
2. **Hold full, mission not empty:** Squad returns to base; after unload, base increases; mission remains on map with reduced pool.
3. **Last mission emptied, hold partial, no others:** Squad returns to base with partial cargo; mission removed from active list.
4. **Pool exhaustion removes mission:** After total extracted equals initial pool for a cell, that mission is no longer in `active_missions`.

## Relationship to prior MVP

- Replaces “silver credited at gather complete” with “silver credited at base unload”.
- Replaces “remove mission on every successful gather” with “remove mission only when `silver_remaining == 0`”.

## Mission pick when chaining sites

When the current site is empty and the hold is not full: choose the next mission with `silver_remaining > 0` that is **closest from the squad’s current cell** (empty site). First outbound from base still uses closest-from-base (existing behavior).
