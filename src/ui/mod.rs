use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::core::Game;

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

    let map = Paragraph::new(format!("Hello world\n\n{:?}", game.world))
        .block(Block::default().title("Map").borders(Borders::ALL));
    let base = Paragraph::new(format!("{:?}", game.base))
        .block(Block::default().title("Base").borders(Borders::ALL));
    let units = Paragraph::new(format!("{:?}", game.units))
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
