use super::{action::Action, screen::Screen, state::AppState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn map_key(state: &AppState, key: KeyEvent) -> Option<Action> {
    // global quit
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Quit);
    }
    if key.code == KeyCode::Char('q') {
        return Some(Action::Quit);
    }

    // --- Modal: Add Connection (takes priority over everything else) ---
    if state.screen == Screen::Connections && state.connections.adding.is_some() {
        return match key.code {
            KeyCode::Esc => Some(Action::CancelModal),
            KeyCode::Enter => Some(Action::Confirm),
            KeyCode::Tab => Some(Action::NextField),
            KeyCode::BackTab => Some(Action::PrevField),
            KeyCode::Backspace => Some(Action::Backspace),
            KeyCode::Char(c) => Some(Action::InputChar(c)),
            _ => None,
        };
    }

    // --- Connections screen shortcuts ---
    if state.screen == Screen::Connections {
        match key.code {
            KeyCode::Char('a') => return Some(Action::OpenAddConnection),
            _ => {}
        }
    }

    // screen shortcuts
    match key.code {
        KeyCode::Char('1') => return Some(Action::GoConnections),
        KeyCode::Char('2') => return Some(Action::GoSchema),
        KeyCode::Char('3') => return Some(Action::GoData),
        KeyCode::Char('4') => return Some(Action::GoQueries),
        KeyCode::Char('5') => return Some(Action::GoRunner),
        KeyCode::Char('t') => {
            if state.screen == Screen::Schema {
                return Some(Action::GoData);
            }
        }
        _ => {}
    }

    // universal nav
    match key.code {
        KeyCode::Up => Some(Action::Up),
        KeyCode::Down => Some(Action::Down),
        KeyCode::Left => Some(Action::Left),
        KeyCode::Right => Some(Action::Right),
        KeyCode::Enter => Some(Action::Confirm),
        KeyCode::Esc => Some(Action::Back),
        _ => None,
    }
}
