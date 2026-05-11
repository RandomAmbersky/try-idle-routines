mod detail;
mod layout;
mod map_layout;
mod selection;

pub use detail::{detail_mouse_target, format_detail, DetailMouseTarget};
pub use layout::{compute_layout, MainLayout};
pub use map_layout::{
    cell_for_base, cell_for_mission, map_target_at_cell, squad_marker_cell, terminal_xy_to_cell,
    MapTarget,
};
pub use selection::{Selection, SquadId};

use ratatui::{
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::core::{Game, SquadState};

pub fn render(
    frame: &mut Frame,
    layout: &MainLayout,
    game: &Game,
    mode: &str,
    selection: Selection,
) {
    let map = Paragraph::new(map_text(layout.map_inner, game))
        .block(Block::default().title("Map").borders(Borders::ALL));
    let detail = Paragraph::new(format_detail(game, selection))
        .block(Block::default().title("Detail").borders(Borders::ALL));

    frame.render_widget(map, layout.map_block);
    frame.render_widget(detail, layout.detail_block);

    let help = Paragraph::new(Line::from(format!(
        "mode: {} | ticks: {} | q quit | p pause | n step | Esc clear",
        mode, game.ticks
    )))
    .style(Style::default());
    frame.render_widget(help, layout.footer_block);
}

fn map_text(inner: ratatui::layout::Rect, game: &Game) -> Text<'static> {
    let width = usize::from(inner.width);
    let height = usize::from(inner.height);
    let mut grid = vec![vec![' '; width]; height];

    if inner.width > 0 && inner.height > 0 {
        let (base_col, base_row) = cell_for_base(inner);
        grid[usize::from(base_row)][usize::from(base_col)] = 'B';

        let (mission_col, mission_row) = cell_for_mission(inner);
        let mission_glyph = if game
            .units
            .squads
            .iter()
            .any(|squad| !matches!(squad.state, SquadState::IdleAtBase))
        {
            '!'
        } else {
            'M'
        };
        grid[usize::from(mission_row)][usize::from(mission_col)] = mission_glyph;

        for squad in &game.units.squads {
            if let Some((sc, sr)) = squad_marker_cell(inner, squad.state) {
                let uc = usize::from(sc);
                let ur = usize::from(sr);
                if uc < width && ur < height {
                    grid[ur][uc] = 'S';
                }
            }
        }
    }

    Text::from(
        grid.into_iter()
            .map(|row| Line::from(row.into_iter().collect::<String>()))
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod render_tests {
    use ratatui::{backend::TestBackend, buffer::Buffer, layout::Rect, Terminal};

    use super::*;

    #[test]
    fn render_uses_provided_layout_for_map_detail_and_footer(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend)?;
        let game = Game::new();
        let layout = compute_layout(Rect::new(0, 0, 80, 24));

        let frame = terminal.draw(|f| render(f, &layout, &game, "paused", Selection::Mission))?;

        let (base_col, base_row) = cell_for_base(layout.map_inner);
        let (mission_col, mission_row) = cell_for_mission(layout.map_inner);
        assert_eq!(
            frame.buffer[(layout.map_inner.x + base_col, layout.map_inner.y + base_row)].symbol(),
            "B"
        );
        assert_eq!(
            frame.buffer[(
                layout.map_inner.x + mission_col,
                layout.map_inner.y + mission_row
            )]
                .symbol(),
            "M"
        );

        assert!(row_text(frame.buffer, layout.detail_inner.y + 1).contains("Mission"));
        assert!(row_text(frame.buffer, layout.footer_block.y).contains("Esc clear"));

        Ok(())
    }

    fn row_text(buffer: &Buffer, y: u16) -> String {
        (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect()
    }
}
