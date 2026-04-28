use color_eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
};
use ratatui::{prelude::*, widgets::*, DefaultTerminal, Frame};
use std::{io::stdout, time::Duration};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

#[derive(Default, Debug)]
struct App {
    should_quit: bool,
    last_mouse: Option<(u16, u16)>,
    last_resize: Option<(u16, u16)>,
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // Enable mouse tracking for terminals that support it.
    execute!(stdout(), EnableMouseCapture)?;
    let mut app = App::default();

    let tick_rate = Duration::from_millis(250);
    while !app.should_quit {
        terminal.draw(|f| render(f, &app))?;

        // Wait for input up to tick_rate; on timeout, loop to draw again.
        if event::poll(tick_rate)? {
            let ev = event::read()?;
            update(&mut app, ev);
        }
    }

    execute!(stdout(), DisableMouseCapture)?;
    Ok(())
}

fn update(app: &mut App, ev: Event) {
    match ev {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            _ => {}
        },
        Event::Mouse(m) => {
            app.last_mouse = Some((m.column, m.row));
        }
        Event::Resize(w, h) => {
            app.last_resize = Some((w, h));
        }
        _ => {}
    }
}

fn render(frame: &mut Frame, app: &App) {
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
            app.last_mouse
                .map(|(x, y)| format!("col={x} row={y}"))
                .unwrap_or_else(|| "-".into())
        )),
        Line::raw(format!(
            "Last resize: {}",
            app.last_resize
                .map(|(w, h)| format!("{w}x{h}"))
                .unwrap_or_else(|| "-".into())
        )),
    ];

    let block = Block::new().borders(Borders::ALL).title("try-idle-routines");
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
