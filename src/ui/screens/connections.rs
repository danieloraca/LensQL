use crate::app::state::AppState;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    // --- list ---
    let items: Vec<ListItem> = state
        .connections
        .items
        .iter()
        .map(|c| {
            ListItem::new(format!(
                "{}   {}:{}   {}   {}",
                c.name, c.host, c.port, c.user, c.db
            ))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Connections (a:add, e:edit, d:delete, Enter:connect)")
                .borders(Borders::ALL),
        )
        .highlight_symbol("> ");

    let mut ls = ListState::default();
    if !state.connections.items.is_empty() {
        ls.select(Some(state.connections.selected));
    }
    f.render_stateful_widget(list, area, &mut ls);

    // --- modal: add connection ---
    if let Some(d) = state.connections.adding.as_ref() {
        let popup = centered_rect(70, 60, area);
        f.render_widget(Clear, popup);

        let block = Block::default()
            .title(if d.is_edit { "Edit Connection" } else { "Add Connection" })
            .borders(Borders::ALL);
        f.render_widget(block, popup);

        let inner = Rect {
            x: popup.x + 2,
            y: popup.y + 2,
            width: popup.width - 4,
            height: popup.height - 4,
        };

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(inner);

        let field = |idx: usize, label: &str, value: String| {
            let is_active = d.field == idx;
            let mut v = value;
            if idx == 4 {
                // password masked
                v = "*".repeat(v.chars().count());
            }

            let line = format!("{:<10} {}", format!("{}:", label), v);
            let style = if is_active {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            Paragraph::new(line).style(style)
        };

        f.render_widget(field(0, "Name", d.name.clone()), rows[0]);
        f.render_widget(field(1, "Host", d.host.clone()), rows[1]);
        f.render_widget(field(2, "Port", d.port.clone()), rows[2]);
        f.render_widget(field(3, "User", d.user.clone()), rows[3]);
        f.render_widget(field(4, "Password", d.password.clone()), rows[4]);
        f.render_widget(field(5, "Database", d.database.clone()), rows[5]);

        let help = Paragraph::new("Tab/Shift+Tab: move • Enter: save • Esc: cancel")
            .alignment(Alignment::Left);
        f.render_widget(help, rows[6]);
    }

    // --- modal: delete confirm ---
    if let Some(c) = state.connections.delete_confirm.as_ref() {
        let popup = centered_rect(60, 25, area);
        f.render_widget(Clear, popup);

        let block = Block::default()
            .title("Delete Connection?")
            .borders(Borders::ALL);
        f.render_widget(block, popup);

        let inner = Rect {
            x: popup.x + 2,
            y: popup.y + 2,
            width: popup.width - 4,
            height: popup.height - 4,
        };

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(inner);

        let body = format!(
            "Are you sure you want to delete:\n\n{}\n\nThis cannot be undone.",
            c.name
        );
        f.render_widget(Paragraph::new(body), rows[0]);

        let help = Paragraph::new("y/Enter: delete  •  n/Esc: cancel")
            .alignment(Alignment::Left);
        f.render_widget(help, rows[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
