use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    TogglePause,
    Step,
    Tick,
    None,
}

pub fn action_from_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('p') => Action::TogglePause,
        KeyCode::Char('n') => Action::Step,
        _ => Action::None,
    }
}

pub fn read_action_blocking() -> std::io::Result<Action> {
    match event::read()? {
        Event::Key(key) => Ok(action_from_key(key)),
        _ => Ok(Action::None),
    }
}

pub fn read_action_tick_aware(timeout_ms: u64) -> std::io::Result<Action> {
    if event::poll(Duration::from_millis(timeout_ms))? {
        match event::read()? {
            Event::Key(key) => Ok(action_from_key(key)),
            _ => Ok(Action::None),
        }
    } else {
        Ok(Action::Tick)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent};

    use super::{Action, action_from_key};

    #[test]
    fn maps_q_to_quit() {
        assert_eq!(
            action_from_key(KeyEvent::from(KeyCode::Char('q'))),
            Action::Quit
        );
    }

    #[test]
    fn maps_p_to_toggle_pause() {
        assert_eq!(
            action_from_key(KeyEvent::from(KeyCode::Char('p'))),
            Action::TogglePause
        );
    }

    #[test]
    fn maps_n_to_step() {
        assert_eq!(
            action_from_key(KeyEvent::from(KeyCode::Char('n'))),
            Action::Step
        );
    }

    #[test]
    fn maps_other_keys_to_none() {
        assert_eq!(action_from_key(KeyEvent::from(KeyCode::Esc)), Action::None);
    }
}
