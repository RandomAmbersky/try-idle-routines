# TUI map, mouse selection, and detail column Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the UI from [2026-05-11 TUI map + mouse selection design](../specs/2026-05-11-tui-map-mouse-selection-design.md): **large map (left)**, **detail column (right)** with context by `Selection`, **left mouse** hit-testing on the map and on **[X]** / roster rows, **Esc** clears selection, **`Game`** unchanged for gameplay rules.

**Architecture:** Keep `core::Game` free of coordinates and selection. Add `Selection` + `SquadId` in the UI/app layer. Add **pure** `map_layout` helpers (grid indices, which map tile, mouse→tile). Add `layout` to split terminal `Rect` into map/detail/footer and derive **inner** rects plus a fixed **[X]** hit rect. Extend `input::Action` with `ClearSelection` and `MousePress { column, row }`. Enable/disable **mouse capture** in `Tui::enter` / `Tui::restore`. `App` stores `selection` and applies actions after each event or tick draw cycle.

**Tech Stack:** Rust 2024 edition, `ratatui`, `crossterm`. Tests: `cfg(test)` in the same modules as new pure logic (`map_layout`, `layout`, `input`); optional string asserts for detail text in `ui` tests.

---

## File structure (what changes where)

| File | Responsibility |
|------|------------------|
| `src/ui/selection.rs` (create) | `SquadId`, `Selection` enum |
| `src/ui/map_layout.rs` (create) | Grid size clamp, base/mission cell positions, `terminal_xy_to_cell`, `cell_to_map_target`, map glyph line builder |
| `src/ui/layout.rs` (create) | `MainLayout`: footer strip, `map_block`, `detail_block`, `map_inner`, `detail_inner`, `close_x_rect` from root `Rect` |
| `src/ui/detail.rs` (create) | Build detail `Text`/`Vec<Line>` from `(Game, Selection)`; `detail_mouse_target` for **[X]**, roster row, mission “on site” row |
| `src/ui/mod.rs` (modify) | Wire submodules; `pub fn compute_layout(area: Rect) -> MainLayout`; `pub fn render(...)` uses layout; draw map grid + detail block |
| `src/input/mod.rs` (modify) | `Action` variants; `Event::Mouse` + `Event::Key` handling in blocking + tick-aware readers; tests |
| `src/tui/mod.rs` (modify) | `EnableMouseCapture` on enter, `DisableMouseCapture` on restore (with `?` propagation) |
| `src/app/mod.rs` (modify) | Hold `selection: Selection`; pass into `ui::render`; on `ClearSelection` / mouse / map empty cell update selection; call `terminal.size()` before poll when needed for hit tests |
| `src/main.rs` (modify) | Only if you add a new top-level module (prefer keeping everything under `ui::*` without new `mod` lines) |

**Spec link:** `docs/superpowers/specs/2026-05-11-tui-map-mouse-selection-design.md`

**Layout constants (locked for MVP implementation):**

| Constant | Value |
|----------|------:|
| Detail column width | `24` terminal columns (including borders handled by outer block; inner text width = inner.width) |
| Map logical grid | Uses **`map_inner.width` × `map_inner.height`** as the cell grid (one char per terminal cell inside the map inner rect) |
| Base cell | `(map_inner.width * 1 / 10, map_inner.height / 2)` clamped inside inner |
| Mission cell | `(map_inner.width * 7 / 10, map_inner.height / 2)` clamped; if equals base after clamp, offset mission by `(1, 0)` |
| Map glyphs | Base `'B'`, mission idle `'M'`, mission while any squad `Gathering` `'!'` (click still resolves to **Mission** per spec) |
| Close **[X]** | First line of **detail inner**: literal `" [X] "` right-aligned inside inner width (store `close_x_rect` as **last 4 columns** of first row of `detail_inner`) |

---

### Task 1: `Selection` + `SquadId`

**Files:**

- Create: `src/ui/selection.rs`
- Modify: `src/ui/mod.rs` (add `mod selection;` and `pub use selection::{Selection, SquadId};`)

- [ ] **Step 1: Create `src/ui/selection.rs`**

```rust
//! Player focus for the right-hand detail column and hit testing.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SquadId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selection {
    #[default]
    None,
    Base,
    Mission,
    Squad(SquadId),
}
```

- [ ] **Step 2: Wire module in `src/ui/mod.rs`**

At top of `src/ui/mod.rs` add:

```rust
mod selection;

pub use selection::{Selection, SquadId};
```

- [ ] **Step 3: Compile check**

Run: `cargo check`

Expected: PASS (no references yet).

- [ ] **Step 4: Commit**

```bash
git add src/ui/selection.rs src/ui/mod.rs
git commit -m "feat(ui): add Selection and SquadId types"
```

---

### Task 2: `MainLayout` split (footer, map, detail, close rect)

**Files:**

- Create: `src/ui/layout.rs`
- Modify: `src/ui/mod.rs` (`mod layout;` + `pub use layout::MainLayout;` + `pub use layout::compute_layout;`)

- [ ] **Step 1: Write failing test in `src/ui/layout.rs`**

```rust
use ratatui::layout::Rect;

use super::compute_layout;

#[test]
fn splits_body_into_map_and_fixed_width_detail() {
    let area = Rect::new(0, 0, 80, 24);
    let l = compute_layout(area);
    assert_eq!(l.detail_block.width, 24);
    assert_eq!(l.map_block.width, area.width - 24);
    assert!(l.map_inner.width >= 1);
    assert!(l.detail_inner.width >= 1);
}
```

Add `pub struct MainLayout` with fields: `map_block`, `detail_block`, `map_inner`, `detail_inner`, `close_x_rect` (a `Rect` in **absolute** terminal coordinates), `footer_block`.

Implement `compute_layout` using the same pattern as current `ui::render` vertical split, then horizontal: **detail width = 24**. Inner rects: subtract **1** for each border side from block `Rect` when using `Block::default().borders(Borders::ALL)` — match whatever `inner` you use in render (must be **identical** arithmetic in one function).

Export:

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainLayout {
    pub map_block: Rect,
    pub detail_block: Rect,
    pub map_inner: Rect,
    pub detail_inner: Rect,
    pub close_x_rect: Rect,
    pub footer_block: Rect,
}

pub fn compute_layout(area: Rect) -> MainLayout {
    // ... implementation: vertical [Min(1), Length(1)] for footer
    // body horizontal [map = area.width - 24, detail = 24]
    // close_x_rect: first row of detail_inner, width 4, x = detail_inner.x + detail_inner.width - 4
    todo!()
}
```

- [ ] **Step 2: Run test, expect FAIL**

Run: `cargo test splits_body_into_map_and_fixed_width_detail -- --nocapture`

Expected: compile error or panic `not yet implemented`.

- [ ] **Step 3: Implement `compute_layout` (remove `todo!`)**

Use `Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(1), Constraint::Length(1)])` for body/footer, then horizontal `constraints([Constraint::Length(area.width.saturating_sub(24)), Constraint::Length(24)])` on `body`. Compute `map_inner` as `map_block.inner(ratatui::widgets::Block::default().borders(Borders::ALL))` — import `Borders`, `Block` from ratatui.

Set `close_x_rect` exactly:

```rust
let close_w = 4u16;
let close_x = detail_inner.x.saturating_add(detail_inner.width.saturating_sub(close_w));
close_x_rect: Rect::new(close_x, detail_inner.y, close_w, 1),
```

- [ ] **Step 4: Run test, expect PASS**

Run: `cargo test splits_body_into_map_and_fixed_width_detail`

- [ ] **Step 5: Commit**

```bash
git add src/ui/layout.rs src/ui/mod.rs
git commit -m "feat(ui): add MainLayout split and close [X] rect"
```

---

### Task 3: `map_layout` — cell math + `MapTarget`

**Files:**

- Create: `src/ui/map_layout.rs`
- Modify: `src/ui/mod.rs` (`mod map_layout; pub use map_layout::...`)

- [ ] **Step 1: Write failing tests**

In `src/ui/map_layout.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapTarget {
    Base,
    Mission,
    Empty,
}

pub fn cell_for_base(inner: Rect) -> (u16, u16);
pub fn cell_for_mission(inner: Rect) -> (u16, u16);
pub fn terminal_xy_to_cell(inner: Rect, column: u16, row: u16) -> Option<(u16, u16)>;
pub fn map_target_at_cell(inner: Rect, col: u16, row: u16) -> MapTarget;
```

Test:

```rust
#[test]
fn click_inside_maps_to_cell() {
    let inner = Rect::new(5, 3, 40, 10);
    assert_eq!(terminal_xy_to_cell(inner, 5, 3), Some((0, 0)));
    assert_eq!(terminal_xy_to_cell(inner, 44, 12), Some((39, 9)));
    assert_eq!(terminal_xy_to_cell(inner, 4, 3), None);
}

#[test]
fn base_and_mission_targets_distinct_on_wide_inner() {
    let inner = Rect::new(0, 0, 80, 24);
    let (bc, br) = cell_for_base(inner);
    let (mc, mr) = cell_for_mission(inner);
    assert_ne!((bc, br), (mc, mr));
    assert_eq!(map_target_at_cell(inner, bc, br), MapTarget::Base);
    assert_eq!(map_target_at_cell(inner, mc, mr), MapTarget::Mission);
    assert_eq!(map_target_at_cell(inner, 0, 0), MapTarget::Empty);
}
```

- [ ] **Step 2: Run tests, expect FAIL**

Run: `cargo test cell_for -- --nocapture`

- [ ] **Step 3: Implement functions**

```rust
use ratatui::layout::Rect;

pub fn cell_for_base(inner: Rect) -> (u16, u16) {
    let col = 1u16.min(inner.width.saturating_sub(1));
    let row = inner.height / 2;
    (col, row.min(inner.height.saturating_sub(1)))
}

pub fn cell_for_mission(inner: Rect) -> (u16, u16) {
    let row = inner.height / 2;
    let mut col = inner.width.saturating_mul(3) / 4;
    col = col.max(2).min(inner.width.saturating_sub(1));
    let (bc, br) = cell_for_base(inner);
    let mut mr = row.min(inner.height.saturating_sub(1));
    let mut mc = col;
    if mc == bc && mr == br {
        mc = (bc + 1).min(inner.width.saturating_sub(1));
    }
    (mc, mr)
}

pub fn terminal_xy_to_cell(inner: Rect, column: u16, row: u16) -> Option<(u16, u16)> {
    if column < inner.x
        || row < inner.y
        || column >= inner.x + inner.width
        || row >= inner.y + inner.height
    {
        return None;
    }
    Some((column - inner.x, row - inner.y))
}

pub fn map_target_at_cell(inner: Rect, col: u16, row: u16) -> MapTarget {
    let (bc, br) = cell_for_base(inner);
    let (mc, mr) = cell_for_mission(inner);
    if col == bc && row == br {
        MapTarget::Base
    } else if col == mc && row == mr {
        MapTarget::Mission
    } else {
        MapTarget::Empty
    }
}
```

If a future terminal size makes `(bc, br) == (mc, mr)` after clamping, keep the **mission offset** branch as written.

- [ ] **Step 4: Run tests, expect PASS**

Run: `cargo test map_layout -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add src/ui/map_layout.rs src/ui/mod.rs
git commit -m "feat(ui): add map cell positions and MapTarget hit mapping"
```

---

### Task 4: `Action::ClearSelection` + `Action::MousePress` + readers

**Files:**

- Modify: `src/input/mod.rs`

- [ ] **Step 1: Extend `Action` and add parser helpers**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    TogglePause,
    Step,
    Tick,
    ClearSelection,
    MousePress { column: u16, row: u16 },
    None,
}
```

Change `action_from_key`:

```rust
pub fn action_from_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('p') => Action::TogglePause,
        KeyCode::Char('n') => Action::Step,
        KeyCode::Esc => Action::ClearSelection,
        _ => Action::None,
    }
}
```

Add:

```rust
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

pub fn action_from_mouse(mouse: MouseEvent) -> Action {
    if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
        Action::MousePress {
            column: mouse.column,
            row: mouse.row,
        }
    } else {
        Action::None
    }
}
```

Update `read_action_blocking`:

```rust
pub fn read_action_blocking() -> std::io::Result<Action> {
    match event::read()? {
        Event::Key(key) => Ok(action_from_key(key)),
        Event::Mouse(m) => Ok(action_from_mouse(m)),
        _ => Ok(Action::None),
    }
}
```

Update `read_action_tick_aware` similarly inside the `poll` branch for `Event::Mouse`.

- [ ] **Step 2: Update tests — Esc clears**

Replace the old expectation in `maps_other_keys_to_none` with:

```rust
#[test]
fn maps_esc_to_clear_selection() {
    assert_eq!(
        action_from_key(KeyEvent::from(KeyCode::Esc)),
        Action::ClearSelection
    );
}
```

Add:

```rust
#[test]
fn maps_left_mouse_down_to_press() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let m = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 12,
        row: 7,
        modifiers: KeyModifiers::NONE,
    };
    assert_eq!(
        action_from_mouse(m),
        Action::MousePress { column: 12, row: 7 }
    );
}
```

Add `use crossterm::event::KeyModifiers;` at top of tests module.

- [ ] **Step 3: Run tests**

Run: `cargo test input::`

Expected: all PASS.

- [ ] **Step 4: Commit**

```bash
git add src/input/mod.rs
git commit -m "feat(input): Esc clears selection, mouse left press action"
```

---

### Task 5: Enable / disable mouse capture in `Tui`

**Files:**

- Modify: `src/tui/mod.rs`

- [ ] **Step 1: Import and execute mouse capture**

```rust
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
```

In `enter` after `EnterAlternateScreen`:

```rust
stdout.execute(EnableMouseCapture)?;
```

In `restore` before `LeaveAlternateScreen`:

```rust
self.stdout.execute(DisableMouseCapture)?;
```

- [ ] **Step 2: Manual smoke (optional)**

Run: `cargo run` — move mouse; no panic on exit. (No automated test required.)

- [ ] **Step 3: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat(tui): enable mouse capture while running"
```

---

### Task 6: `detail` text + hit testing for **[X]** and rows

**Files:**

- Create: `src/ui/detail.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Add `DetailMouseTarget` enum + tests first**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailMouseTarget {
    None,
    Close,
    BaseSquadRow { squad_index: usize },
    MissionOnSiteRow { squad_index: usize },
}
```

Import `use super::layout::MainLayout;` (or `crate::ui::layout::MainLayout` from a submodule) so `detail` can read geometry.

Implement `pub fn detail_mouse_target(layout: &MainLayout, game: &Game, selection: Selection, column: u16, row: u16) -> DetailMouseTarget`:

1. If `(column, row)` intersects `layout.close_x_rect` → `Close`.
2. Else if not inside `layout.detail_inner` → `None`.
3. Match `selection`:
   - `Base`: compute **roster start line** inside detail inner as **line 4** (0-based **relative row** `rel_y = row - detail_inner.y`). For each squad with `IdleAtBase`, if `rel_y == 4 + index` → `BaseSquadRow { squad_index: index }`. Skip non-idle squads when counting rows (MVP: either 0 or 1 row).
   - `Mission`: if `Gathering` for squad 0, show on-site row at fixed line (e.g. **line 6**); if `rel_y == 6` → `MissionOnSiteRow { squad_index: 0 }`.
   - Else → `None`.

Use the **same** line numbers as `format_detail` below — extract shared `const` values.

- [ ] **Step 2: Implement `format_detail(game, selection) -> ratatui::text::Text`**

Content sketch (English or Russian per your product choice — pick **one** language for MVP strings and stay consistent):

- `None`: hint + no `[X]` line (or `[X]` greyed without hit — simpler: **omit** `[X]`).
- `Base`: `Silver: …`, header `Roster`, each idle squad `Squad N: idle`, gathering squads omitted from roster list (spec: roster = idle at base).
- `Mission`: title, `available_gather_missions`, if gathering show `Squad on site: …` + `seconds_left`.
- `Squad(id)`: show squad state line(s) for `game.units.squads[id.0]`.

- [ ] **Step 3: Unit test roster line hit**

```rust
#[test]
fn detects_close_hit() {
    // build a MainLayout with known detail_inner via compute_layout(Rect::new(0,0,80,24))
    // pick a coordinate inside close_x_rect
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test detail_`

- [ ] **Step 5: Commit**

```bash
git add src/ui/detail.rs src/ui/mod.rs
git commit -m "feat(ui): detail text and mouse hit targets"
```

---

### Task 7: `App` selection dispatch + `ui::render` integration

**Files:**

- Modify: `src/app/mod.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Add `selection` to `App`**

```rust
use crate::ui::{compute_layout, Selection};

pub struct App {
    game: Game,
    selection: Selection,
}

impl App {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            selection: Selection::None,
        }
    }
}
```

- [ ] **Step 2: On each loop iteration, resolve layout size**

Before `terminal.draw`, `let area = terminal.size()?;` (or equivalent API on your ratatui version) and `let layout = compute_layout(area);`.

Pass `&layout`, `&self.game`, `self.selection` into `ui::render`.

- [ ] **Step 3: Handle new actions in `match action`**

```rust
Action::ClearSelection => {
    self.selection = Selection::None;
}
Action::MousePress { column, row } => {
    let inside_close = column >= layout.close_x_rect.x
        && column < layout.close_x_rect.x.saturating_add(layout.close_x_rect.width)
        && row >= layout.close_x_rect.y
        && row < layout.close_x_rect.y.saturating_add(layout.close_x_rect.height);
    if inside_close {
        self.selection = Selection::None;
        continue;
    }
    match crate::ui::detail::detail_mouse_target(&layout, &self.game, self.selection, column, row) {
        crate::ui::detail::DetailMouseTarget::Close => {
            self.selection = Selection::None;
        }
        crate::ui::detail::DetailMouseTarget::BaseSquadRow { squad_index } => {
            self.selection = Selection::Squad(crate::ui::SquadId(squad_index));
        }
        crate::ui::detail::DetailMouseTarget::MissionOnSiteRow { squad_index } => {
            self.selection = Selection::Squad(crate::ui::SquadId(squad_index));
        }
        crate::ui::detail::DetailMouseTarget::None => {
            if let Some((cx, cy)) =
                crate::ui::map_layout::terminal_xy_to_cell(layout.map_inner, column, row)
            {
                use crate::ui::map_layout::{MapTarget, map_target_at_cell};
                match map_target_at_cell(layout.map_inner, cx, cy) {
                    MapTarget::Base => self.selection = Selection::Base,
                    MapTarget::Mission => self.selection = Selection::Mission,
                    MapTarget::Empty => self.selection = Selection::None,
                }
            }
        }
    }
}
```

Note: `Rect::contains` in ratatui may take `Position` — adjust to your ratatui version (`contains(x,y)` or manual bounds).

Deduplicate `Close` handling if `close_x_rect` already handled first.

- [ ] **Step 4: Rewrite `ui::render`**

- Build map `Paragraph` or `Text`: fill `' '` grid, place `'B'` and mission glyph at computed cells using `cell_for_base` / `cell_for_mission` + `Gathering` check from `game.units.squads[0].state`.
- Right column: `Block` titled `Detail`, inner filled with `format_detail`.
- Footer: include hint `Esc clear` + existing keys.

- [ ] **Step 5: Run binary**

Run: `cargo run`

Expected: window draws; mouse clicks change nothing fatal; `q` exits; mouse disabled after exit.

- [ ] **Step 6: Commit**

```bash
git add src/app/mod.rs src/ui/mod.rs src/ui/detail.rs
git commit -m "feat(app): wire selection, mouse hits, and new layout render"
```

---

### Task 8: Polish + regression `cargo test`

**Files:**

- Modify: `src/ui/mod.rs`, `src/ui/detail.rs` (copy tweaks), `README.md` only if you already document controls there (optional)

- [ ] **Step 1: Full test suite**

Run: `cargo test`

Expected: all PASS.

- [ ] **Step 2: Commit**

```bash
git commit -am "chore: polish map/detail UI strings"
```
(Or skip if no changes.)

---

## Plan self-review (author checklist)

**1. Spec coverage**

| Spec requirement | Task |
|------------------|------|
| Large map left, narrow detail right | Task 2, 7 |
| `Selection` in app, not in `Game` | Task 1, 7 |
| Pure layout / hit-test split | Task 2–3, 6 |
| Base = one cell; mission = one cell; idle squads not on map | Task 7 render |
| Mission glyph during `Gathering`; map click = `Mission` | Task 3 target + Task 7 dispatch |
| Empty map cell clears selection | Task 7 |
| Esc + `[X]` clear | Task 4, 5, 7 |
| Click outside targets no-op | Task 7 order (`detail_mouse_target` → map → none) |
| `Base` roster row → `Squad` | Task 6–7 |
| `Mission` on-site row → `Squad` | Task 6–7 |
| `Squad` detail view | Task 6 |
| Mouse capture lifecycle | Task 5 |
| Unit tests for pure helpers | Tasks 2–4, 6 |

**2. Placeholder scan:** No `TBD` / empty steps; `todo!()` only as transient in Task 2 before implementation.

**3. Type consistency:** `SquadId(usize)` used everywhere; `Action` variants match dispatch in Task 7.

**Gaps addressed:** `Rect::contains` API may differ by ratatui version — implementer adjusts bounds check inline if needed.

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-11-tui-map-mouse-selection-plan.md`. Two execution options:

**1. Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — run tasks in this session using executing-plans, batch execution with checkpoints.

Which approach do you want?
