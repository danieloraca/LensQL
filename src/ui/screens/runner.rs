use crate::app::state::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

pub fn render(f: &mut Frame, area: Rect, _state: &AppState) {
    let w = Paragraph::new("Query runner placeholder.\n\nNext: editor + results view + save (s).")
        .block(Block::default().title("Runner").borders(Borders::ALL));
    f.render_widget(w, area);
}
