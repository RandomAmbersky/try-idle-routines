use ratatui::layout::Rect;

use crate::core::SquadState;

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

/// Integer lerp from `a` to `b` with `num/den` in \[0, den\], clamped to `inner`.
pub fn cell_lerp(inner: Rect, a: (u16, u16), b: (u16, u16), num: u32, den: u32) -> (u16, u16) {
    let den = den.max(1);
    let max_c = inner.width.saturating_sub(1);
    let max_r = inner.height.saturating_sub(1);
    let ac = u32::from(a.0.min(max_c));
    let ar = u32::from(a.1.min(max_r));
    let bc = u32::from(b.0.min(max_c));
    let br = u32::from(b.1.min(max_r));
    let cc = ac + (bc.saturating_sub(ac)).saturating_mul(num) / den;
    let cr = ar + (br.saturating_sub(ar)).saturating_mul(num) / den;
    ((cc as u16).min(max_c), (cr as u16).min(max_r))
}

/// Cell for the squad token `S` so travel home and to the mission are visible on the map.
pub fn squad_marker_cell(inner: Rect, state: SquadState) -> Option<(u16, u16)> {
    if inner.width == 0 || inner.height == 0 {
        return None;
    }
    let base = cell_for_base(inner);
    let mission = cell_for_mission(inner);
    let raw = match state {
        SquadState::IdleAtBase => cell_step_toward(inner, base, mission),
        SquadState::TravelingToMission { .. } => cell_lerp(inner, base, mission, 1, 2),
        SquadState::Gathering { .. } => cell_step_toward(inner, mission, base),
        SquadState::ReturningToBase { .. } => cell_lerp(inner, mission, base, 1, 2),
    };
    let (bc, br) = base;
    let (mc, mr) = mission;
    let (c, r) = raw;
    if (c, r) == (bc, br) || (c, r) == (mc, mr) {
        let toward = if (c, r) == (bc, br) {
            mission
        } else {
            base
        };
        Some(cell_step_toward(inner, raw, toward))
    } else {
        Some((c, r))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::SquadState;

    use super::*;

    #[test]
    fn squad_marker_not_on_base_cell_while_traveling() {
        let inner = Rect::new(0, 0, 50, 12);
        let (bc, br) = cell_for_base(inner);
        assert_ne!(
            squad_marker_cell(inner, SquadState::TravelingToMission { seconds_left: 1 }).unwrap(),
            (bc, br),
            "travel marker should leave the base cell"
        );
    }

    #[test]
    fn squad_marker_not_on_mission_cell_while_gathering() {
        let inner = Rect::new(0, 0, 50, 12);
        let (mc, mr) = cell_for_mission(inner);
        assert_ne!(
            squad_marker_cell(inner, SquadState::Gathering { seconds_left: 1 }).unwrap(),
            (mc, mr)
        );
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
