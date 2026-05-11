# idle-tui — design: map + mouse selection + detail column

Date: 2026-05-11

## Goal

Specify the **terminal UI** for presenting **one spatial map** (left) and a **narrow detail column** (right), with **mouse-driven selection** aligned to the existing Rust + **ratatui** stack and the current **gather / base / squads** domain.

This document is a **UI/UX + application-layer** spec. It **does not** add player-assigned movement waypoints or multi-squad-per-mission gameplay; those are explicitly **out of scope** for the implementation step that follows this spec, but **`Selection::Squad`** and list-driven entry exist so later features can attach without redesigning selection.

## Relationship to other specs

- **MVP-0 / MVP-1** (shell, tick, pause) remain the time model baseline.
- **Vision / gather MVP** (`2026-05-09-idle-ufo-inspired-vision-design.md`, gather plan) define **what** is simulated; this spec defines **how** it is shown and how the player **inspects** state.

## Layout (screen)

- **Footer:** unchanged in spirit (mode, ticks, key hints); exact copy can evolve.
- **Body:** horizontal split:
  - **Left — Map:** large share of width (concrete ratio is an implementation constant, e.g. ~75% / ~25%, tuned for readability).
  - **Right — Detail:** narrow column for numeric/text detail, scroll/wrap behavior as constrained by terminal width.

**Minimum width:** the detail column enforces a **minimum width** so the **[X]** close control and short labels remain usable; if the terminal is too narrow, text **wraps** or **truncates** with predictable behavior (implementation detail).

## Architecture: simulation vs presentation (recommended split)

**Option B (chosen):** core **`Game`** holds **gameplay truth** only (resources, squads, mission availability, tick-driven state). It does **not** store screen coordinates or player selection.

- **Application / TUI state** holds **`Selection`**: `None` | `Base` | `Mission` | `Squad`.
- A small **layout** layer (pure functions colocated with UI or a dedicated module) **deterministically** maps:
  - `Game` + terminal **inner map rect** → which **character** occupies each grid cell;
  - terminal **mouse (x, y)** → **map cell** → **map hit target** (`Base` cell, `Mission` cell, or empty).

This keeps the domain testable without pixel knowledge, while keeping hit-testing **consistent** with what is drawn.

## Map semantics (MVP)

### Grid

- The map is a **rectangular character grid** inside the map widget’s **inner** rectangle (excluding block borders and title).
- **One character per cell** for walkable display cells.

### Entities on the map

| Entity | Cells on map | Notes |
|--------|----------------|------|
| **Base** | **Exactly one** cell | Clicking this cell selects **`Base`**. |
| **Mission site** (“alert” in product language) | **Exactly one** cell in MVP | Represents the **resource gathering** mission point. Clicking this cell always selects **`Mission`**, including while a squad is in **`Gathering`** on that site. |
| **Squads in `IdleAtBase`** | **Not drawn** as separate tokens | They are listed in the **detail** UI when **`Base`** is selected. |
| **Squads in `Gathering`** | **Visually co-located** with the mission cell | Still **one** map cell for the site; **map click** resolves to **`Mission`** only (see below). |

### Multi-squad-per-mission

**Out of MVP.** The spec allows **future** multiple squads assigned to one mission; map click policy may need revision then. MVP assumes **at most one** squad context on a mission site for display text.

## Input: mouse, selection, and cancel

### Mouse

- Enable **mouse event delivery** for the TUI session (e.g. via crossterm), with capture enabled/disabled as appropriate, and **always** disable mouse capture on **clean exit**.

### Map clicks

- **Click inside map inner area on the base cell** → `Selection::Base`.
- **Click inside map inner area on the mission cell** → `Selection::Mission` (always, even during **`Gathering`**).
- **Click inside map inner area on an empty cell** → **clear selection** (`Selection::None`).

### Cancel selection

- **Esc** → `Selection::None`.
- **[X]** in the detail column → `Selection::None` (dedicated **hit rectangle** for **[X]**).
- **Empty map cell click** → `Selection::None` (same as above).

### Clicks outside targets

- Clicks **outside** the map inner rect, **outside** **[X]**, and **outside** intentionally clickable **list rows** (see below): **do not change** `Selection`.

### Keyboard parity (MVP)

- **Esc** is required for cancel.
- **No requirement** in this MVP to replicate every mouse action with keyboard navigation; optional follow-up: focusable list rows and arrow keys.

## Detail column: content by `Selection`

### `None`

- Short hint text (e.g. how to select base or mission).
- **[X]** is **hidden** or **disabled** (no hit target) when there is nothing to cancel.

### `Base`

- Base **stockpile** (for current MVP: **Silver** and any other fields already in `Game`).
- **Roster** of squads that are **`IdleAtBase`**: per squad, **label**, **state**, and **numeric fields** already available from the domain (no invented stats).
- **Clicking a squad row** in this roster → `Selection::Squad` for that squad id.

### `Mission`

- Mission site **identity** (MVP: single gathering site).
- **`available_gather_missions`** (and any existing mission-related fields).
- While **`Gathering`** on this site: additional lines such as **time remaining** and **which squad is on site** (MVP: one squad).
- **Clicking the “squad on site” row** (when shown) → `Selection::Squad` for that squad id.

**Important:** **Map cell click** for the mission site remains **`Mission`**; entering **`Squad`** from mission context is via the **detail list row** only in MVP.

### `Squad`

- **Full squad summary** using existing simulation fields, with **room in the layout** for future controls (e.g. waypoint assignment). **Waypoint assignment is not implemented** in the step immediately following this spec.

Clearing selection from **`Squad`** (**Esc**, **[X]**, empty map click) goes to **`None`** (no requirement to return to `Base`/`Mission`).

## Rendering and hit-testing notes

- **Hit-testing** uses the **same** inner rectangle and grid dimensions as **rendering**.
- **Z-order** is trivial at the map level in MVP because **only one** of {base character, mission character, optional squad glyph during gathering} needs to be **drawn** per cell policy:
  - Mission cell **during `Gathering`**: still **one** cell; **glyph policy** is an implementation choice (e.g. show mission as primary, or a fused symbol) as long as **click** resolves to **`Mission`**.
- **Resize:** recompute layout each frame; clamp grid if inner area is too small, with a **defined** fallback (e.g. fewer visible rows, or a one-line warning inside the map block).

## Testing strategy

- **Unit tests** for pure helpers:
  - mouse `(x, y)` + known inner rect → cell index or miss;
  - cell index → `Base` / `Mission` / empty.
- **Optional:** golden strings for a few `(Game, Selection)` pairs for the detail column (no mandatory terminal harness).

## Non-goals (this spec / immediate implementation)

- Player-authored **movement waypoints** or pathing UI.
- **Multiple mission sites** or **multiple squads per mission** on the map.
- **Mouse hover** tooltips or animations beyond static TUI redraw.

## Open decisions left to implementation plan

- Exact **percent split** and **minimum** detail width constants.
- Exact **glyphs** for base/mission/gathering cell and **truncation** rules for long text in the detail column.
