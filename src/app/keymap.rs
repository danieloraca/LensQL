use super::{action::Action, screen::Screen, state::AppState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn map_key(state: &AppState, key: KeyEvent) -> Option<Action> {
    // global
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Quit);
    }
    if key.code == KeyCode::Char('q') {
        return Some(Action::Quit);
    }

    // universal nav
    match key.code {
        KeyCode::Up => return Some(Action::Up),
        KeyCode::Down => return Some(Action::Down),
        KeyCode::Left => return Some(Action::Left),
        KeyCode::Right => return Some(Action::Right),
        KeyCode::Enter => return Some(Action::Confirm),
        KeyCode::Esc => return Some(Action::Back),
        _ => {}
    }

    // screen shortcuts (brainstorm defaults)
    match key.code {
        KeyCode::Char('1') => Some(Action::GoConnections),
        KeyCode::Char('2') => Some(Action::GoSchema),
        KeyCode::Char('3') => Some(Action::GoData),
        KeyCode::Char('4') => Some(Action::GoQueries),
        KeyCode::Char('5') => Some(Action::GoRunner),
        KeyCode::Char('t') => {
            if state.screen == Screen::Schema {
                Some(Action::GoData)
            } else {
                None
            }
        }
        KeyCode::Char('q') => {
            // already handled as Quit globally, but left for future per-screen behavior
            None
        }
        _ => None,
    }
}
