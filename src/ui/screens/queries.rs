use crate::app::state::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

pub fn render(f: &mut Frame, area: Rect, _state: &AppState) {
    let w =
        Paragraph::new("Query library placeholder.\n\nNext: saved queries list + preview + run.")
            .block(Block::default().title("Queries").borders(Borders::ALL));
    f.render_widget(w, area);
}
