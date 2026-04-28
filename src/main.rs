use color_eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
};
use ratatui::DefaultTerminal;
use std::{io::stdout, time::Duration};

mod app;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // Enable mouse tracking for terminals that support it.
    execute!(stdout(), EnableMouseCapture)?;
    let mut app = app::App::default();

    let tick_rate = Duration::from_millis(250);
    while !app.should_quit {
        terminal.draw(|f| app.render(f))?;

        // Wait for input up to tick_rate; on timeout, loop to draw again.
        if event::poll(tick_rate)? {
            let ev = event::read()?;
            app.update(ev);
        }
    }

    execute!(stdout(), DisableMouseCapture)?;
    Ok(())
}
