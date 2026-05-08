use crossterm::event::{self, Event, KeyCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    None,
}

pub fn read_action_blocking() -> std::io::Result<Action> {
    match event::read()? {
        Event::Key(key) if key.code == KeyCode::Char('q') => Ok(Action::Quit),
        _ => Ok(Action::None),
    }
}
