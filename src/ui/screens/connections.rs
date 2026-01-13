use crate::app::state::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = state
        .connections
        .items
        .iter()
        .map(|c| ListItem::new(format!("{}   {}   {}", c.name, c.host, c.db)))
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Connections").borders(Borders::ALL))
        .highlight_symbol("> ");

    let mut ls = ListState::default();
    if !state.connections.items.is_empty() {
        ls.select(Some(state.connections.selected));
    }

    f.render_stateful_widget(list, area, &mut ls);
}
