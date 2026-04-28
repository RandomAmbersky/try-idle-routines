use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};

#[derive(Default, Debug)]
pub struct App {
    pub should_quit: bool,
    last_mouse: Option<(u16, u16)>,
    last_resize: Option<(u16, u16)>,
}

impl App {
    pub fn update(&mut self, ev: Event) {
        match ev {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                _ => {}
            },
            Event::Mouse(m) => {
                self.last_mouse = Some((m.column, m.row));
            }
            Event::Resize(w, h) => {
                self.last_resize = Some((w, h));
            }
            _ => {}
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let lines = vec![
            Line::from(vec![
                Span::raw("Press "),
                Span::styled("q", Style::new().add_modifier(Modifier::BOLD)),
                Span::raw(" or "),
                Span::styled("Esc", Style::new().add_modifier(Modifier::BOLD)),
                Span::raw(" to quit."),
            ]),
            Line::raw(format!(
                "Mouse: {}",
                self.last_mouse
                    .map(|(x, y)| format!("col={x} row={y}"))
                    .unwrap_or_else(|| "-".into())
            )),
            Line::raw(format!(
                "Last resize: {}",
                self.last_resize
                    .map(|(w, h)| format!("{w}x{h}"))
                    .unwrap_or_else(|| "-".into())
            )),
        ];

        let block = Block::new().borders(Borders::ALL).title("try-idle-routines");
        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }
}

