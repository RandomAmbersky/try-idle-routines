# idle-tui — design: random base, three gather missions, lifecycle + routing

Date: 2026-05-11

## Goal

Specify **domain and map behavior** for:

- A **random base position** and **three random gather-mission sites** on the logical map at **new game** start.
- **Removing** a mission from the active set when it **ends** (gathering completes: silver applied, squad begins return).
- **Choosing the next mission** for the autonomous squad as the **remaining** site **closest to the base**, where “closest” is defined by **shortest outbound route length** (same routing rules as today).
- **No new missions** spawn after the initial three are cleared; the run ends the “mission content” until a **new game**.

This spec is **gameplay + map truth**. It **updates** map/mission assumptions documented in `2026-05-11-tui-map-mouse-selection-design.md` (which described **one** fixed mission cell for MVP). Implementation should reconcile selection and hit-testing with **multiple** mission cells and **dynamic** removal.

## Relationship to other specs

- **MVP-0 / MVP-1:** time model (`tick`, pause, step) unchanged.
- **Vision / gather MVP** (`2026-05-09-idle-ufo-inspired-vision-design.md`, gather plan): still **one squad**, **one resource (Silver)** on the base, **gather** mission type only.
- **Map + mouse selection** (`2026-05-11-tui-map-mouse-selection-design.md`): layout and selection patterns stay; **mission sites** become **up to three** cells drawn from domain state, not a single hard-coded coordinate. Where the older spec says idle squads are **not** on the map, **this document wins:** idle squads are shown **on the base cell** (`S` over `B`) so placement stays correct with a random base and no fixed “staging” cell off-base.

## Out of scope (explicit)

- Multiple squads, mission types other than gather, player-assigned targets, combat/failure, persistence across runs beyond “new game regenerates layout”.
- Procedural respawn of missions mid-run after all three are done.

## World generation (new game)

- **Map bounds:** use existing logical size (`MAP_WIDTH`, `MAP_HEIGHT`); all placements are valid grid cells inside those bounds.
- **Base:** one cell `(col, row)` chosen uniformly at random from all map cells (or from an implementation-chosen non-edge margin if desired—if so, document the rule in the implementation plan; default is **any cell**).
- **Missions:** exactly **three** distinct gather sites. Each site is one cell. Constraints:
  - Each mission cell **≠** base cell.
  - All four cells (base + three missions) are **pairwise distinct**.
- **Sampling:** resample until constraints hold (simple loop with a cap and panic/fallback only if mathematically impossible—on a 100×100 grid with four distinct cells this is always satisfiable).
- **Randomness:** use one RNG type seeded from `thread_rng` (or OS) for normal play; tests must use a **deterministic** seed or injected RNG so CI does not flake.

## Domain: mission list

- Store an **ordered or unordered list** of active gather missions; each entry needs at least **map coordinates** (and optionally a stable **id** for logging/tests).
- **No separate** “mission reappears” counter is required for map truth: availability is **`active_missions.is_empty()` is false** for “there exists a mission the squad can run next”, combined with squad idle logic in the implementation plan.

## Mission end (timing)

- When **gathering** on site completes for the **current target** (silver payout applied, squad state transitions to **returning to base**):
  - Remove **that** mission from the active list immediately.
  - The **mission marker** for that cell **must not** be drawn and **must not** be selectable on the map while the squad is returning or afterward.

## Routing

- **Outbound:** from base to the **chosen** mission cell using the same geometric rules as the current codebase: step off base toward mission, then a **king-adjacent** polyline (e.g. Bresenham inclusive) from the first step through the mission cell.
- **Return:** traverse the same cell sequence **backward** at the same **one cell per simulated second** rate as today.
- While returning, the route still references cells that included the mission site; the **glyph** for the mission is gone, but the squad **still walks** those coordinates until home.

## Next mission selection (squad idle at base)

- When the squad is **idle at base** and the active mission list is **non-empty**:
  - Compute the **outbound route length** (number of cells in the outbound path vector, after any deduplication/consistency rules used by the engine) from **current base** to each **remaining** mission cell.
  - Select the mission with **minimum** route length.
  - **Tie-break:** if several missions share the same minimum length, choose the cell with **lexicographically smallest** `(row, col)` (row first, then col) so ordering is deterministic.

## Autonomous dispatch

- When the squad becomes **idle at base** and missions remain, the squad **starts** the next mission on the next applicable tick using the **closest** rule above (same “autonomous loop” spirit as the current gather MVP).
- When the list is **empty**, the squad **stays idle**; no automatic mission creation.

## Map rendering

- **Base:** one glyph (e.g. `B`) at the generated base cell.
- **Missions:** one glyph per **active** mission (e.g. `M`, or `!` when any squad is actively engaged with **that** cell if the product wants per-site urgency—**default:** reuse a single mission glyph set: `M` when no squad on route to this cell, `!` when this cell is the **current** outbound/gather target; exact styling is implementation detail).
- **Squads:** position from state + current route while **MovingToMission**, **Gathering**, or **MovingToBase**. While **IdleAtBase**, the squad occupies the **base cell** for map purposes (replacing the fixed “one step toward mission” rule, which does not apply when missions are dynamic or absent). **Glyph overlap at base:** draw **base** then **squad** so **`S` wins** over `B` when both share the cell (player always sees the unit at home).

## Viewport (`map_view_origin`)

- The visible map window should keep **base**, **all active mission cells**, and the **squad** position in view when the terminal is smaller than the full logical map, generalizing the current “segment between base and mission” heuristic to **the set** `{base} ∪ missions ∪ squad cell` (e.g. bounding box padding + tie-breaks documented in code comments if needed).

## Selection and hit-testing

- **Mission cell click** resolves to mission **only** if that cell is still in the active mission list.
- Otherwise the cell behaves as **empty** for selection purposes (or as **squad** if the squad occupies it).
- If the player had **`Selection::Mission`** (or a future per-mission selection) pointing at a site that **just** left the active list, **clear selection to `None`** on the same tick boundary where the mission is removed, so the detail pane does not describe a non-existent mission.

## Detail panel

- **Mission** summary should reflect **remaining count** and/or list coordinates (implementation choice); must not claim a mission exists after it has been removed.

## Testing expectations

- **Unit tests** with fixed RNG: three missions + base placed deterministically; verify **closest** choice by comparing route lengths; verify **removal** on gather completion and **no marker** during `MovingToBase`; verify **empty list** stops dispatch.
- **Property or example test:** tie-break `(row, col)` when two missions are equidistant by route length.

## Summary

| Topic | Rule |
|-------|------|
| Start | Random base + 3 random distinct gather cells |
| Mission done | Remove from list at gather completion (silver applied); no map marker |
| Next target | Shortest outbound route from base; tie `(row, col)` |
| All cleared | Squad idle; no new missions until new game |
| Return path | Same geometry as today, backward, mission cell not shown as `M` |
