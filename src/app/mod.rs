use std::io;

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{core::Game, input::Action, tui::Tui, ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Running,
    Paused,
}

pub struct App {
    game: Game,
}

impl App {
    pub fn new() -> Self {
        Self { game: Game::new() }
    }

    pub fn run(mut self) -> io::Result<()> {
        let mut tui = Tui::enter()?;
        let backend = CrosstermBackend::new(tui.stdout());
        let mut terminal = Terminal::new(backend)?;
        let mut mode = RunMode::Running;

        loop {
            let mode_label = match mode {
                RunMode::Running => "running",
                RunMode::Paused => "paused",
            };
            terminal.draw(|f| ui::render(f, &self.game, mode_label))?;

            let action = match mode {
                RunMode::Running => crate::input::read_action_tick_aware(1000)?,
                RunMode::Paused => crate::input::read_action_blocking()?,
            };

            match action {
                Action::Quit => break,
                Action::TogglePause => {
                    mode = match mode {
                        RunMode::Running => RunMode::Paused,
                        RunMode::Paused => RunMode::Running,
                    };
                }
                Action::Tick => {
                    self.game.tick(1000);
                }
                Action::Step => {
                    if mode == RunMode::Paused {
                        self.game.tick(1000);
                    }
                }
                Action::ClearSelection => {}
                Action::MousePress { .. } => {}
                Action::None => {}
            }
        }

        Ok(())
    }
}
