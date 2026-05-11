use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders};

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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let body = chunks[0];
    let footer_block = chunks[1];

    let body_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(area.width.saturating_sub(24)),
            Constraint::Length(24),
        ])
        .split(body);

    let map_block = body_cols[0];
    let detail_block = body_cols[1];

    let block = Block::default().borders(Borders::ALL);
    let map_inner = block.inner(map_block);
    let detail_inner = block.inner(detail_block);

    let close_w = 4u16;
    let close_x = detail_inner
        .x
        .saturating_add(detail_inner.width.saturating_sub(close_w));
    let close_x_rect = Rect::new(close_x, detail_inner.y, close_w, 1);

    MainLayout {
        map_block,
        detail_block,
        map_inner,
        detail_inner,
        close_x_rect,
        footer_block,
    }
}

#[cfg(test)]
mod tests {
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
}
