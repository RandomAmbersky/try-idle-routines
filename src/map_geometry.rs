//! King-adjacent grid routing; depends only on `constants`.

use crate::constants::{MAP_HEIGHT, MAP_WIDTH};

/// One grid step from `from` toward `to`, clamped inside the fixed map.
pub fn cell_step_toward(from: (u16, u16), to: (u16, u16)) -> (u16, u16) {
    let max_c = MAP_WIDTH.saturating_sub(1);
    let max_r = MAP_HEIGHT.saturating_sub(1);
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
