use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::core::{Game, SquadState};

fn squad_primary_line(game: &Game) -> String {
    match game.units.squads[0].state {
        SquadState::IdleAtBase => String::from("Squad A: idle at base"),
        SquadState::Gathering { seconds_left } => {
            format!("Squad A: gathering ({seconds_left} s left on site)")
        }
    }
}

fn format_map_panel(game: &Game) -> String {
    format!(
        "Gather missions available: {}\n{}",
        game.world.available_gather_missions,
        squad_primary_line(game)
    )
}

fn format_base_panel(game: &Game) -> String {
    format!("Silver: {}", game.base.silver)
}

pub fn render(frame: &mut Frame, game: &Game, mode: &str) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let body = chunks[0];
    let footer = chunks[1];

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(body);

    let map = Paragraph::new(format_map_panel(game))
        .block(Block::default().title("Map").borders(Borders::ALL));
    let base = Paragraph::new(format_base_panel(game))
        .block(Block::default().title("Base").borders(Borders::ALL));
    let units = Paragraph::new(squad_primary_line(game))
        .block(Block::default().title("Units").borders(Borders::ALL));

    frame.render_widget(map, cols[0]);
    frame.render_widget(base, cols[1]);
    frame.render_widget(units, cols[2]);

    let help = Paragraph::new(Line::from(format!(
        "mode: {} | ticks: {} | q quit | p pause | n step",
        mode, game.ticks
    )))
    .style(Style::default());
    frame.render_widget(help, footer);
}
