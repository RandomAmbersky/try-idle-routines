use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    TogglePause,
    Step,
    Tick,
    ClearSelection,
    MousePress { column: u16, row: u16 },
    None,
}

pub fn action_from_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('p') => Action::TogglePause,
        KeyCode::Char('n') => Action::Step,
        KeyCode::Esc => Action::ClearSelection,
        _ => Action::None,
    }
}

pub fn action_from_mouse(mouse: MouseEvent) -> Action {
    if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
        Action::MousePress {
            column: mouse.column,
            row: mouse.row,
        }
    } else {
        Action::None
    }
}

pub fn read_action_blocking() -> std::io::Result<Action> {
    match event::read()? {
        Event::Key(key) => Ok(action_from_key(key)),
        Event::Mouse(m) => Ok(action_from_mouse(m)),
        _ => Ok(Action::None),
    }
}

pub fn read_action_tick_aware(timeout_ms: u64) -> std::io::Result<Action> {
    if event::poll(Duration::from_millis(timeout_ms))? {
        match event::read()? {
            Event::Key(key) => Ok(action_from_key(key)),
            Event::Mouse(m) => Ok(action_from_mouse(m)),
            _ => Ok(Action::None),
        }
    } else {
        Ok(Action::Tick)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

    use super::{Action, action_from_key, action_from_mouse};

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
    fn maps_esc_to_clear_selection() {
        assert_eq!(
            action_from_key(KeyEvent::from(KeyCode::Esc)),
            Action::ClearSelection
        );
    }

    #[test]
    fn maps_left_mouse_down_to_press() {
        let m = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 12,
            row: 7,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(
            action_from_mouse(m),
            Action::MousePress { column: 12, row: 7 }
        );
    }
}
