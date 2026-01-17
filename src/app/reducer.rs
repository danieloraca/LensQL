use super::{
    action::Action,
    command::{Command, DbCommand, StorageCommand},
    event::{DbEvent, Event, StorageEvent},
    screen::Screen,
    state::{AppState, DeleteConnectionConfirm, NewConnectionDraft},
};

fn draft_field_mut(d: &mut NewConnectionDraft) -> &mut String {
    match d.field {
        0 => &mut d.name,
        1 => &mut d.host,
        2 => &mut d.port,
        3 => &mut d.user,
        4 => &mut d.password,
        _ => &mut d.database, // 5
    }
}

fn build_profiles(
    items: &[crate::app::state::ConnectionItem],
) -> Vec<crate::storage::model::ConnectionProfile> {
    items
        .iter()
        .map(|i| crate::storage::model::ConnectionProfile {
            id: i.id.to_string(), // <-- FIX: Ulid -> String
            name: i.name.clone(),
            host: i.host.clone(),
            port: i.port,
            user: i.user.clone(),
            database: i.db.clone(),
        })
        .collect()
}

pub fn reduce_action(state: &mut AppState, action: Action) -> Vec<Command> {
    // --- Modal first: if add/edit connection modal is open, most actions operate on it ---
    if state.screen == Screen::Connections && state.connections.adding.is_some() {
        match action {
            Action::CancelModal | Action::Back => {
                state.connections.adding = None;
                state.status.message = "Cancelled".to_string();
                return vec![];
            }

            Action::NextField => {
                if let Some(d) = state.connections.adding.as_mut() {
                    d.field = (d.field + 1) % 6;
                }
                return vec![];
            }

            Action::PrevField => {
                if let Some(d) = state.connections.adding.as_mut() {
                    d.field = (d.field + 5) % 6;
                }
                return vec![];
            }

            Action::Backspace => {
                if let Some(d) = state.connections.adding.as_mut() {
                    draft_field_mut(d).pop();
                }
                return vec![];
            }

            Action::InputChar(c) => {
                if let Some(d) = state.connections.adding.as_mut() {
                    if !c.is_control() {
                        draft_field_mut(d).push(c);
                    }
                }
                return vec![];
            }

            Action::Confirm => {
                // Enter = Save (add or edit)
                let Some(d) = state.connections.adding.take() else {
                    return vec![];
                };

                let name = d.name.trim().to_string();
                let host = d.host.trim().to_string();
                let user = d.user.trim().to_string();
                let db = d.database.trim().to_string();

                if name.is_empty() || host.is_empty() || user.is_empty() || db.is_empty() {
                    state.status.message =
                        "Missing required fields (name/host/user/database)".to_string();
                    state.connections.adding = Some(d);
                    return vec![];
                }

                let port: u16 = match d.port.trim().parse() {
                    Ok(p) => p,
                    Err(_) => {
                        state.status.message = "Port must be a number (e.g. 3306)".to_string();
                        state.connections.adding = Some(d);
                        return vec![];
                    }
                };

                let item = crate::app::state::ConnectionItem::new_with_id(
                    d.id,
                    &name,
                    &host,
                    port,
                    &user,
                    &d.password,
                    &db,
                );

                // Store secret (password) in keyring; only persist non-secret fields to disk.
                if let Err(e) = crate::storage::secrets::ConnectionSecrets::default()
                    .set_password(&item.id.to_string(), &item.password)
                {
                    state.status.message = format!("Failed to store password in keyring: {}", e);
                    return vec![];
                }

                // Upsert by id: if exists, replace; else append.
                if let Some(idx) = state.connections.items.iter().position(|c| c.id == item.id) {
                    state.connections.items[idx] = item;
                    state.connections.selected = idx;
                    state.status.message = "Connection updated (saving…)".to_string();
                } else {
                    state.connections.items.push(item);
                    state.connections.selected = state.connections.items.len().saturating_sub(1);
                    state.status.message = "Connection added (saving…)".to_string();
                }

                let profiles = build_profiles(&state.connections.items);
                return vec![Command::Storage(StorageCommand::SaveConnections {
                    connections: profiles,
                })];
            }

            // Ignore other actions while modal is open
            _ => return vec![],
        }
    }

    // --- Normal (non-modal) reducer ---
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

        Action::OpenAddConnection => {
            state.connections.adding = Some(NewConnectionDraft::new());
            state.status.message =
                "Add connection: Tab/Shift+Tab move • Enter save • Esc cancel".to_string();
            vec![]
        }

        Action::EditSelectedConnection => {
            let Some(item) = state
                .connections
                .items
                .get(state.connections.selected)
            else {
                state.status.message = "No connection selected".to_string();
                return vec![];
            };

            state.connections.adding = Some(NewConnectionDraft::edit_from(item));
            state.status.message =
                "Edit connection: Tab/Shift+Tab move • Enter save • Esc cancel".to_string();
            vec![]
        }

        Action::DeleteSelectedConnection => {
            if state.connections.items.is_empty() {
                state.status.message = "No connection to delete".to_string();
                return vec![];
            }

            if state.connections.selected >= state.connections.items.len() {
                state.connections.selected = state.connections.items.len().saturating_sub(1);
            }

            let Some(item) = state.connections.items.get(state.connections.selected).cloned() else {
                state.status.message = "No connection to delete".to_string();
                return vec![];
            };

            state.connections.delete_confirm = Some(DeleteConnectionConfirm {
                id: item.id,
                name: item.name.clone(),
            });

            state.status.message = format!(
                "Delete connection '{}'? (y/Enter confirm, n/Esc cancel)",
                item.name
            );

            vec![]
        }

        Action::ConfirmDeleteConnection => {
            let Some(confirm) = state.connections.delete_confirm.take() else {
                return vec![];
            };

            // remove by id (safer than relying on selected index)
            if let Some(idx) = state.connections.items.iter().position(|c| c.id == confirm.id) {
                let removed = state.connections.items.remove(idx);

                // keep selection in range
                if state.connections.selected >= state.connections.items.len() {
                    state.connections.selected = state.connections.items.len().saturating_sub(1);
                }

                // Best-effort: clear secret from keyring (implemented as overwrite with empty string).
                if let Err(e) = crate::storage::secrets::ConnectionSecrets::default()
                    .delete_password(&removed.id.to_string())
                {
                    state.status.message = format!(
                        "Deleted connection '{}', but failed to clear keyring password: {}",
                        removed.name, e
                    );
                } else {
                    state.status.message = format!("Deleted connection '{}'", removed.name);
                }

                let profiles = build_profiles(&state.connections.items);
                vec![Command::Storage(StorageCommand::SaveConnections {
                    connections: profiles,
                })]
            } else {
                state.status.message = "Connection already deleted".to_string();
                vec![]
            }
        }

        Action::CancelDeleteConnection => {
            state.connections.delete_confirm = None;
            state.status.message = "Delete cancelled".to_string();
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

        Action::Confirm => match state.screen {
            Screen::Connections => {
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
            Screen::Schema => vec![],
            _ => vec![],
        },

        Action::Back => {
            match state.screen {
                Screen::Connections => { /* no-op */ }
                _ => {
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

        // These are modal-only, ignore when not in modal
        Action::CancelModal
        | Action::NextField
        | Action::PrevField
        | Action::Backspace
        | Action::InputChar(_) => vec![],

        // delete-confirm modal actions are handled above; ignore elsewhere
        Action::ConfirmDeleteConnection | Action::CancelDeleteConnection => vec![],

        Action::Left | Action::Right | Action::ConnectSelected => vec![],
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
                state.schema.columns.clear();
                state.schema.columns_table = None;
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
                state.status.message = format!("DB error: {}", message);
                vec![]
            }
        },

        Event::Storage(se) => match se {
            StorageEvent::ConnectionsSaved => {
                state.status.message = "Connections saved".to_string();
                vec![]
            }
            StorageEvent::Error { message } => {
                state.status.message = format!("Storage error: {}", message);
                vec![]
            }
        },
    }
}
