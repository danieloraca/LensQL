pub mod screens;
pub mod terminal;

use crate::app::{screen::Screen, state::AppState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)].as_ref())
        .split(f.size());

    // main content
    match state.screen {
        Screen::Connections => screens::connections::render(f, chunks[0], state),
        Screen::Schema => screens::schema::render(f, chunks[0], state),
        Screen::Data => screens::data::render(f, chunks[0], state),
        Screen::Queries => screens::queries::render(f, chunks[0], state),
        Screen::Runner => screens::runner::render(f, chunks[0], state),
    }

    // status bar
    let status = format!(
        " {}  |  {}  |  (1)Conn (2)Schema (3)Data (4)Queries (5)Runner | q:quit ",
        if state.status.connection_label.is_empty() {
            "Not connected".to_string()
        } else {
            format!("Connected: {}", state.status.connection_label)
        },
        state.status.message
    );

    let bar = Paragraph::new(status).block(Block::default().borders(Borders::TOP));
    f.render_widget(bar, chunks[1]);
}
