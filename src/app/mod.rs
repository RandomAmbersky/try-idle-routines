use std::io;

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{core::Game, input::Action, tui::Tui, ui};

pub struct App {
    game: Game,
}

impl App {
    pub fn new() -> Self {
        Self { game: Game::new() }
    }

    pub fn run(self) -> io::Result<()> {
        let mut tui = Tui::enter()?;
        let backend = CrosstermBackend::new(tui.stdout());
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| ui::render(f, &self.game))?;

            match crate::input::read_action_blocking()? {
                Action::Quit => break,
                Action::None => {}
            }
        }

        Ok(())
    }
}
