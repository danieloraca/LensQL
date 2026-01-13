use crate::app::state::AppState;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
        .split(area);

    // ---- Left: Tables list ----
    let table_items: Vec<ListItem> = state
        .schema
        .tables
        .iter()
        .map(|t| ListItem::new(t.clone()))
        .collect();

    let tables = List::new(table_items)
        .block(Block::default().title("Tables").borders(Borders::ALL))
        .highlight_symbol("> ");

    let mut ls = ListState::default();
    if !state.schema.tables.is_empty() {
        ls.select(Some(state.schema.selected_table));
    }

    f.render_stateful_widget(tables, chunks[0], &mut ls);

    // ---- Right: Columns / details ----
    let selected_table = state
        .schema
        .tables
        .get(state.schema.selected_table)
        .cloned()
        .unwrap_or_default();

    let title = if selected_table.is_empty() {
        "Details".to_string()
    } else {
        format!("Table: {}", selected_table)
    };

    let body = if selected_table.is_empty() {
        "Connect to a DB to load tables.\n\nEnter on Connections connects.\n".to_string()
    } else if state.schema.columns_table.as_deref() != Some(&selected_table) {
        "Loading columns…".to_string()
    } else if state.schema.columns.is_empty() {
        "(No columns)".to_string()
    } else {
        let mut lines = Vec::new();
        for c in &state.schema.columns {
            let nullability = if c.is_nullable { "NULL" } else { "NOT NULL" };
            let key = c.column_key.as_deref().unwrap_or("");

            if key.is_empty() {
                lines.push(format!("{}  ({}, {})", c.name, c.data_type, nullability));
            } else {
                // PRI / MUL / UNI etc.
                lines.push(format!(
                    "{}  ({}, {}, {})",
                    c.name, c.data_type, nullability, key
                ));
            }
        }
        lines.join("\n")
    };

    let detail = Paragraph::new(body).block(Block::default().title(title).borders(Borders::ALL));

    f.render_widget(detail, chunks[1]);
}
