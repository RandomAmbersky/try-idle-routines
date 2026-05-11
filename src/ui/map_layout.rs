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

#[cfg(test)]
mod tests {
    use super::*;

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
