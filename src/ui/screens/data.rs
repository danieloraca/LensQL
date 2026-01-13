use crate::app::state::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

pub fn render(f: &mut Frame, area: Rect, _state: &AppState) {
    let w = Paragraph::new(
        "Data view placeholder.\n\nNext: table rows grid + paging + column jump (g c).",
    )
    .block(Block::default().title("Data").borders(Borders::ALL));
    f.render_widget(w, area);
}
