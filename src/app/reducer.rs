use super::{
    action::Action,
    command::{Command, DbCommand},
    event::{DbEvent, Event},
    screen::Screen,
    state::AppState,
};

pub fn reduce_action(state: &mut AppState, action: Action) -> Vec<Command> {
    match action {
        Action::GoConnections => {
            state.screen = Screen::Connections;
            vec![]
        }
        Action::GoSchema => {
            state.screen = Screen::Schema;
            vec![]
        }
        Action::GoData => {
            state.screen = Screen::Data;
            vec![]
        }
        Action::GoQueries => {
            state.screen = Screen::Queries;
            vec![]
        }
        Action::GoRunner => {
            state.screen = Screen::Runner;
            vec![]
        }

        Action::Up => {
            match state.screen {
                Screen::Connections => {
                    if state.connections.selected > 0 {
                        state.connections.selected -= 1;
                    }
                }
                Screen::Schema => {
                    let prev = state.schema.selected_table;
                    if prev > 0 {
                        state.schema.selected_table -= 1;
                    }

                    // trigger columns load if table changed
                    if state.schema.selected_table != prev {
                        if let Some(table) = state
                            .schema
                            .tables
                            .get(state.schema.selected_table)
                            .cloned()
                        {
                            state.status.message = format!("Loading columns for {}…", table);
                            return vec![Command::Db(DbCommand::LoadColumns { table })];
                        }
                    }
                }
                _ => {}
            }
            vec![]
        }

        Action::Down => {
            match state.screen {
                Screen::Connections => {
                    if state.connections.selected + 1 < state.connections.items.len() {
                        state.connections.selected += 1;
                    }
                }
                Screen::Schema => {
                    let prev = state.schema.selected_table;
                    if state.schema.selected_table + 1 < state.schema.tables.len() {
                        state.schema.selected_table += 1;
                    }

                    if state.schema.selected_table != prev {
                        if let Some(table) = state
                            .schema
                            .tables
                            .get(state.schema.selected_table)
                            .cloned()
                        {
                            state.status.message = format!("Loading columns for {}…", table);
                            return vec![Command::Db(DbCommand::LoadColumns { table })];
                        }
                    }
                }
                _ => {}
            }
            vec![]
        }

        Action::Confirm => {
            match state.screen {
                Screen::Connections => {
                    // Connect selected
                    let Some(item) = state
                        .connections
                        .items
                        .get(state.connections.selected)
                        .cloned()
                    else {
                        return vec![];
                    };
                    state.status.message = format!("Connecting to {}…", item.name);
                    vec![Command::Db(DbCommand::Connect {
                        name: item.name,
                        host: item.host,
                        port: item.port,
                        user: item.user,
                        password: item.password,
                        db: item.db,
                    })]
                }
                Screen::Schema => {
                    // in v1, Enter can load tables (already loaded post-connect),
                    // or later show table detail; keep no-op for now
                    vec![]
                }
                _ => vec![],
            }
        }

        Action::Back => {
            match state.screen {
                Screen::Connections => {
                    // back on connections does nothing
                }
                _ => {
                    // simple back behavior: go to schema if connected else connections
                    if !state.status.connection_label.is_empty() {
                        state.screen = Screen::Schema;
                    } else {
                        state.screen = Screen::Connections;
                    }
                }
            }
            vec![]
        }

        Action::Disconnect => vec![Command::Db(DbCommand::Disconnect)],

        Action::Quit => vec![],
        _ => vec![],
    }
}

pub fn reduce_event(state: &mut AppState, event: Event) -> Vec<Command> {
    match event {
        Event::Db(db) => match db {
            DbEvent::Connected { display } => {
                state.status.connection_label = display;
                state.status.message = "Connected".to_string();
                state.screen = Screen::Schema;
                vec![Command::Db(DbCommand::LoadTables)]
            }
            DbEvent::Disconnected => {
                state.status.connection_label.clear();
                state.status.message = "Disconnected".to_string();
                state.schema.tables.clear();
                state.schema.selected_table = 0;
                state.screen = Screen::Connections;
                vec![]
            }
            DbEvent::TablesLoaded { tables } => {
                state.schema.tables = tables;
                state.schema.selected_table = 0;
                state.schema.columns.clear();
                state.schema.columns_table = None;
                state.status.message = "Tables loaded".to_string();

                if let Some(table) = state.schema.tables.get(0).cloned() {
                    state.status.message = format!("Loading columns for {}…", table);
                    return vec![Command::Db(DbCommand::LoadColumns { table })];
                }

                vec![]
            }
            DbEvent::ColumnsLoaded { table, columns } => {
                state.schema.columns_table = Some(table);
                state.schema.columns = columns;
                state.status.message = "Columns loaded".to_string();
                vec![]
            }
            DbEvent::Error { message } => {
                state.status.message = format!("Error: {}", message);
                vec![]
            }
        },
    }
}
