use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapTarget {
    Base,
    Mission,
    Empty,
}

pub fn cell_for_base(inner: Rect) -> (u16, u16) {
    let col = 1u16.min(inner.width.saturating_sub(1));
    let row = inner.height / 2;
    (col, row.min(inner.height.saturating_sub(1)))
}

pub fn cell_for_mission(inner: Rect) -> (u16, u16) {
    let row = inner.height / 2;
    let col = (inner.width.saturating_mul(3) / 4)
        .max(2)
        .min(inner.width.saturating_sub(1));
    let (bc, br) = cell_for_base(inner);
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

/// One grid step from `from` toward `to`, clamped inside `inner`.
pub fn cell_step_toward(inner: Rect, from: (u16, u16), to: (u16, u16)) -> (u16, u16) {
    let max_c = inner.width.saturating_sub(1);
    let max_r = inner.height.saturating_sub(1);
    let (fc, fr) = (i32::from(from.0.min(max_c)), i32::from(from.1.min(max_r)));
    let (tc, tr) = (i32::from(to.0.min(max_c)), i32::from(to.1.min(max_r)));
    let dc = (tc - fc).signum();
    let dr = (tr - fr).signum();
    let mc = i32::from(max_c);
    let mr = i32::from(max_r);
    if dc != 0 && (dr == 0 || dc.unsigned_abs() >= dr.unsigned_abs()) {
        let nc = (fc + dc).clamp(0, mc) as u16;
        (nc, from.1.min(max_r))
    } else if dr != 0 {
        let nr = (fr + dr).clamp(0, mr) as u16;
        (from.0.min(max_c), nr)
    } else {
        (from.0.min(max_c), from.1.min(max_r))
    }
}

/// Cells from the first step off-base through the mission site (inclusive), in travel order.
pub fn route_outbound_cells(inner: Rect) -> Vec<(u16, u16)> {
    if inner.width == 0 || inner.height == 0 {
        return Vec::new();
    }
    let base = cell_for_base(inner);
    let mission = cell_for_mission(inner);
    let start = cell_step_toward(inner, base, mission);
    bresenham_inclusive(start, mission, inner.width, inner.height)
}

fn bresenham_inclusive(
    start: (u16, u16),
    end: (u16, u16),
    max_w: u16,
    max_h: u16,
) -> Vec<(u16, u16)> {
    let max_x = i32::from(max_w.saturating_sub(1));
    let max_y = i32::from(max_h.saturating_sub(1));
    let mut x0 = i32::from(start.0).clamp(0, max_x);
    let mut y0 = i32::from(start.1).clamp(0, max_y);
    let x1 = i32::from(end.0).clamp(0, max_x);
    let y1 = i32::from(end.1).clamp(0, max_y);
    let mut out = Vec::new();
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        out.push((x0 as u16, y0 as u16));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    out
}

/// Consecutive cells along `route_outbound_cells` are king-adjacent (one map cell per step).
pub fn route_steps_are_one_cell_apart(inner: Rect) -> bool {
    let r = route_outbound_cells(inner);
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
        let inner = Rect::new(0, 0, 56, 21);
        assert!(route_steps_are_one_cell_apart(inner));
    }

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
}
