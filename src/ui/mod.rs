mod detail;
mod layout;
mod map_layout;
mod selection;

pub use detail::{detail_mouse_target, format_detail, DetailMouseTarget};
pub use layout::{compute_layout, MainLayout};
pub use map_layout::{map_view_origin_for_points, MapTarget, MAP_HEIGHT, MAP_WIDTH};
pub use selection::{Selection, SquadId};

/// Squad drawn at `(col, row)` on the logical map, if any (`S` glyph in the map widget).
pub fn squad_index_at_map_cell(game: &Game, col: u16, row: u16) -> Option<usize> {
    for (squad_index, squad) in game.units.squads.iter().enumerate() {
        if let Some((sc, sr)) = squad_cell_on_map(game, squad) {
            if sc == col && sr == row {
                return Some(squad_index);
            }
        }
    }
    None
}

use ratatui::{
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::core::{Game, Squad, SquadState};

/// Logical cells used to fit the viewport (base, active missions, squad).
fn map_viewport_points(game: &Game) -> Vec<(u16, u16)> {
    let mut pts = Vec::with_capacity(2 + game.world.active_missions.len());
    pts.push(game.world.base_cell);
    pts.extend_from_slice(&game.world.active_missions);
    if let Some(squad) = game.units.squads.first() {
        if let Some(c) = squad_cell_on_map(game, squad) {
            pts.push(c);
        }
    }
    pts
}

pub fn map_target_at_cell(game: &Game, col: u16, row: u16) -> MapTarget {
    if (col, row) == game.world.base_cell {
        MapTarget::Base
    } else if game
        .world
        .active_missions
        .iter()
        .any(|&m| m == (col, row))
    {
        MapTarget::Mission
    } else {
        MapTarget::Empty
    }
}

pub fn terminal_xy_to_map_cell_with_game(
    inner: ratatui::layout::Rect,
    column: u16,
    row: u16,
    game: &Game,
) -> Option<(u16, u16)> {
    let pts = map_viewport_points(game);
    map_layout::terminal_xy_to_map_cell_for_points(inner, column, row, &pts)
}

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
    let vw = usize::from(inner.width);
    let vh = usize::from(inner.height);
    if vw == 0 || vh == 0 {
        return Text::default();
    }

    let mw = usize::from(MAP_WIDTH);
    let mh = usize::from(MAP_HEIGHT);
    let mut logical = vec![vec!['.'; mw]; mh];

    let (base_col, base_row) = game.world.base_cell;
    logical[usize::from(base_row)][usize::from(base_col)] = 'B';

    let urgent_mission = game.units.squads.first().and_then(|squad| match squad.state {
        SquadState::MovingToMission | SquadState::Gathering { .. } => {
            game.route_to_mission.last().copied()
        }
        _ => None,
    });

    for &(mc, mr) in &game.world.active_missions {
        let glyph = if urgent_mission == Some((mc, mr)) {
            '!'
        } else {
            'M'
        };
        logical[usize::from(mr)][usize::from(mc)] = glyph;
    }

    for squad in &game.units.squads {
        if let Some((sc, sr)) = squad_cell_on_map(game, squad) {
            let uc = usize::from(sc);
            let ur = usize::from(sr);
            if uc < mw && ur < mh {
                logical[ur][uc] = 'S';
            }
        }
    }

    let pts = map_viewport_points(game);
    let (ox, oy) = map_view_origin_for_points(inner, &pts);
    let lines: Vec<Line> = (0..vh)
        .map(|dy| {
            let my = usize::from(oy).saturating_add(dy);
            let row: String = (0..vw)
                .map(|dx| {
                    let mx = usize::from(ox).saturating_add(dx);
                    if mx < mw && my < mh {
                        logical[my][mx]
                    } else {
                        ' '
                    }
                })
                .collect();
            Line::from(row)
        })
        .collect();

    Text::from(lines)
}

fn squad_cell_on_map(game: &Game, squad: &Squad) -> Option<(u16, u16)> {
    match squad.state {
        // Garrison: squads at base are not drawn on the map (roster in detail only).
        SquadState::IdleAtBase => None,
        SquadState::MovingToMission => game.route_to_mission.get(squad.path_index).copied(),
        SquadState::Gathering { .. } => game.route_to_mission.last().copied(),
        SquadState::MovingToBase => game.route_to_mission.get(squad.path_index).copied(),
    }
}

#[cfg(test)]
mod render_tests {
    use ratatui::{backend::TestBackend, buffer::Buffer, layout::Rect, Terminal};

    use super::*;

    #[test]
    fn render_uses_provided_layout_for_map_detail_and_footer(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Large enough that map_inner fits the full 100×100 logical map (no viewport crop).
        let backend = TestBackend::new(140, 120);
        let mut terminal = Terminal::new(backend)?;
        let game = Game::new_from_layout_for_test(
            (1, 50),
            vec![(75, 50), (80, 55), (70, 45)],
        );
        let layout = compute_layout(Rect::new(0, 0, 140, 120));

        let frame = terminal.draw(|f| render(f, &layout, &game, "paused", Selection::Mission))?;

        let (base_col, base_row) = game.world.base_cell;
        let (mission_col, mission_row) = game.world.active_missions[0];
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

    #[test]
    fn squad_not_on_map_when_idle_at_base_shows_when_deployed() {
        let mut game = Game::new_from_layout_for_test((10, 10), vec![(20, 10)]);
        let (bc, br) = game.world.base_cell;
        assert_eq!(squad_index_at_map_cell(&game, bc, br), None);

        game.tick(crate::core::SIMULATED_SECOND_MS);
        let pos = game.route_to_mission[game.units.squads[0].path_index];
        assert_eq!(squad_index_at_map_cell(&game, pos.0, pos.1), Some(0));
        assert_eq!(squad_index_at_map_cell(&game, 0, 0), None);
    }

    fn row_text(buffer: &Buffer, y: u16) -> String {
        (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect()
    }
}
