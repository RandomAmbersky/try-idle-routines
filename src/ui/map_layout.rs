use ratatui::layout::Rect;

pub use crate::constants::{MAP_HEIGHT, MAP_WIDTH};

#[allow(unused_imports)]
pub use crate::map_geometry::{cell_step_toward, outbound_route_len, route_outbound_cells_from};

#[inline]
pub fn map_bounds() -> Rect {
    Rect::new(0, 0, MAP_WIDTH, MAP_HEIGHT)
}

/// Best top-left `origin` in `[0, map_len - view)` so the viewport overlaps `[lo, hi]` as much
/// as possible. Tie-breaks: more of `{a,b}` covered, then center closest to segment midpoint,
/// then smaller `origin`.
fn viewport_origin_1d(lo: u16, hi: u16, view: u16, map_len: u16, a: u16, b: u16) -> u16 {
    if map_len == 0 || view == 0 {
        return 0;
    }
    
    let view = view.min(map_len);
    let max_o = map_len.saturating_sub(view);
    let lo = lo.min(map_len.saturating_sub(1));
    let hi = hi.min(map_len.saturating_sub(1));
    let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };

    let mid = u32::from(lo.saturating_add(hi) / 2);
    let center_dist = |o: u16| {
        let c = u32::from(o).saturating_add(u32::from(view) / 2);
        if c > mid {
            c - mid
        } else {
            mid - c
        }
    };
    let covers = |o: u16, p: u16| o <= p && p < o.saturating_add(view);
    // Prefer mission (`b`) when the viewport cannot show both markers.
    let cover_rank = |o: u16| {
        u8::from(covers(o, b)).saturating_mul(2).saturating_add(u8::from(covers(o, a)))
    };

    let mut best_o = 0u16;
    let mut best_len = 0u32;
    let mut best_rank = 0u8;
    let mut best_dist = u32::MAX;
    for o in 0..=max_o {
        let left = u32::from(o.max(lo));
        let right = u32::from(o.saturating_add(view).min(hi.saturating_add(1)));
        let len = right.saturating_sub(left);
        let rank = cover_rank(o);
        let dist = center_dist(o);
        let better = len > best_len
            || (len == best_len && rank > best_rank)
            || (len == best_len && rank == best_rank && dist < best_dist)
            || (len == best_len && rank == best_rank && dist == best_dist && o < best_o);
        if better {
            best_len = len;
            best_rank = rank;
            best_dist = dist;
            best_o = o;
        }
    }
    best_o
}

/// Top-left of the visible map slice inside the map widget (viewport into the logical map).
/// Uses the legacy fixed demo base/mission cells (tests only).
pub fn map_view_origin(inner: Rect) -> (u16, u16) {
    let (bc, br) = cell_for_base();
    let (mc, mr) = cell_for_mission();
    map_view_origin_for_points(inner, &[(bc, br), (mc, mr)])
}

/// Viewport that covers every point in `points` (non-empty).
pub fn map_view_origin_for_points(inner: Rect, points: &[(u16, u16)]) -> (u16, u16) {
    if inner.width == 0 || inner.height == 0 || points.is_empty() {
        return (0, 0);
    }
    let mut min_c = u16::MAX;
    let mut max_c = 0u16;
    let mut min_r = u16::MAX;
    let mut max_r = 0u16;
    for &(c, r) in points {
        min_c = min_c.min(c);
        max_c = max_c.max(c);
        min_r = min_r.min(r);
        max_r = max_r.max(r);
    }
    let ox = if inner.width >= MAP_WIDTH {
        0
    } else {
        viewport_origin_1d(min_c, max_c, inner.width, MAP_WIDTH, min_c, max_c)
    };
    let oy = if inner.height >= MAP_HEIGHT {
        0
    } else {
        viewport_origin_1d(min_r, max_r, inner.height, MAP_HEIGHT, min_r, max_r)
    };
    (ox, oy)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapTarget {
    Base,
    Mission,
    Empty,
}

pub fn cell_for_base() -> (u16, u16) {
    let inner = map_bounds();
    let col = 1u16.min(inner.width.saturating_sub(1));
    let row = inner.height / 2;
    (col, row.min(inner.height.saturating_sub(1)))
}

pub fn cell_for_mission() -> (u16, u16) {
    let inner = map_bounds();
    let row = inner.height / 2;
    let col = (inner.width.saturating_mul(3) / 4)
        .max(2)
        .min(inner.width.saturating_sub(1));
    let (bc, br) = cell_for_base();
    let mr = row.min(inner.height.saturating_sub(1));
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

/// Terminal coordinates → logical map cell (viewport follows `points`).
pub fn terminal_xy_to_map_cell_for_points(
    inner: Rect,
    column: u16,
    row: u16,
    points: &[(u16, u16)],
) -> Option<(u16, u16)> {
    let (vx, vy) = terminal_xy_to_cell(inner, column, row)?;
    let (ox, oy) = map_view_origin_for_points(inner, points);
    let mx = ox.saturating_add(vx);
    let my = oy.saturating_add(vy);
    if mx < MAP_WIDTH && my < MAP_HEIGHT {
        Some((mx, my))
    } else {
        None
    }
}

/// Legacy: fixed demo base/mission viewport.
pub fn terminal_xy_to_map_cell(inner: Rect, column: u16, row: u16) -> Option<(u16, u16)> {
    let (bc, br) = cell_for_base();
    let (mc, mr) = cell_for_mission();
    terminal_xy_to_map_cell_for_points(inner, column, row, &[(bc, br), (mc, mr)])
}

/// Hit-test for the fixed demo base/mission layout (tests).
pub fn map_target_at_fixed_layout_cell(col: u16, row: u16) -> MapTarget {
    let (bc, br) = cell_for_base();
    let (mc, mr) = cell_for_mission();
    if col == bc && row == br {
        MapTarget::Base
    } else if col == mc && row == mr {
        MapTarget::Mission
    } else {
        MapTarget::Empty
    }
}

/// Cells from the first step off-base through the mission site (inclusive), in travel order.
pub fn route_outbound_cells() -> Vec<(u16, u16)> {
    route_outbound_cells_from(cell_for_base(), cell_for_mission())
}

/// Consecutive cells along `route_outbound_cells` are king-adjacent (one map cell per step).
pub fn route_steps_are_one_cell_apart() -> bool {
    let r = route_outbound_cells();
    r.windows(2).all(|w| {
        let (a, b) = (w[0], w[1]);
        let dc = a.0.abs_diff(b.0);
        let dr = a.1.abs_diff(b.1);
        dc <= 1 && dr <= 1 && (dc + dr > 0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_cells_are_one_step_each() {
        assert!(route_steps_are_one_cell_apart());
    }

    #[test]
    fn click_inside_maps_to_cell() {
        let inner = Rect::new(5, 3, 40, 10);
        assert_eq!(terminal_xy_to_cell(inner, 5, 3), Some((0, 0)));
        assert_eq!(terminal_xy_to_cell(inner, 44, 12), Some((39, 9)));
        assert_eq!(terminal_xy_to_cell(inner, 4, 3), None);
    }

    #[test]
    fn base_and_mission_targets_distinct_on_fixed_map() {
        let (bc, br) = cell_for_base();
        let (mc, mr) = cell_for_mission();
        assert_ne!((bc, br), (mc, mr));
        assert_eq!(map_target_at_fixed_layout_cell(bc, br), MapTarget::Base);
        assert_eq!(map_target_at_fixed_layout_cell(mc, mr), MapTarget::Mission);
        assert_eq!(map_target_at_fixed_layout_cell(0, 0), MapTarget::Empty);
    }

    #[test]
    fn terminal_xy_resolves_to_map_cell_through_viewport() {
        let inner = Rect::new(5, 3, 40, 10);
        let (ox, oy) = map_view_origin(inner);
        assert_eq!(terminal_xy_to_map_cell(inner, 5, 3), Some((ox, oy)));
        assert_eq!(
            terminal_xy_to_map_cell(inner, 44, 12),
            Some((ox.saturating_add(39), oy.saturating_add(9)))
        );
        assert_eq!(terminal_xy_to_map_cell(inner, 4, 3), None);
    }

    #[test]
    fn click_in_padding_when_inner_wider_than_map_returns_none() {
        let inner = Rect::new(0, 0, MAP_WIDTH + 10, 5);
        let x = MAP_WIDTH + 5;
        let y = 0;
        assert!(terminal_xy_to_cell(inner, x, y).is_some());
        assert_eq!(terminal_xy_to_map_cell(inner, x, y), None);
    }
}
