use std::error::Error;
use std::io;
use std::io::IsTerminal;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

fn main() -> Result<(), Box<dyn Error>> {
    if !io::stdout().is_terminal() {
        println!("Hello, world!");
        return Ok(());
    }

    let mut terminal = setup_terminal()?;

    let result = run_app(&mut terminal);

    restore_terminal(&mut terminal)?;

    result?;
    Ok(())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)])
                .split(area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title("try_idle_routines")
                .title_alignment(Alignment::Center);

            let text = Paragraph::new("Hello, world!\n\nPress q to quit.")
                .block(block)
                .alignment(Alignment::Center)
                .style(Style::default())
                .wrap(Wrap { trim: true });

            frame.render_widget(text, chunks[0]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(())
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
