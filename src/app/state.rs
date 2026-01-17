use ulid::Ulid;

use super::screen::Screen;

#[derive(Debug)]
pub struct AppState {
    pub screen: Screen,
    pub status: StatusState,

    pub connections: ConnectionsState,
    pub schema: SchemaState,
    pub data: DataState,
    pub queries: QueriesState,
    pub runner: RunnerState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            screen: Screen::Connections,
            status: StatusState::default(),
            connections: ConnectionsState::default(),
            schema: SchemaState::default(),
            data: DataState::default(),
            queries: QueriesState::default(),
            runner: RunnerState::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct StatusState {
    pub connection_label: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct ConnectionsState {
    pub selected: usize,
    pub items: Vec<ConnectionItem>,
    pub adding: Option<NewConnectionDraft>,
    pub delete_confirm: Option<DeleteConnectionConfirm>,
}

#[derive(Debug, Clone)]
pub struct ConnectionItem {
    pub id: Ulid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub db: String,
}

#[derive(Debug, Default, Clone)]
pub struct NewConnectionDraft {
    pub id: Ulid,
    pub name: String,
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub field: usize, // 0..5
    pub is_edit: bool,
}

#[derive(Debug, Clone)]
pub struct DeleteConnectionConfirm {
    pub id: Ulid,
    pub name: String,
}

impl NewConnectionDraft {
    pub fn new() -> Self {
        Self {
            id: Ulid::new(),
            port: "3306".to_string(),
            is_edit: false,
            ..Default::default()
        }
    }

    pub fn edit_from(item: &ConnectionItem) -> Self {
        Self {
            id: item.id,
            name: item.name.clone(),
            host: item.host.clone(),
            port: item.port.to_string(),
            user: item.user.clone(),
            password: item.password.clone(),
            database: item.db.clone(),
            field: 0,
            is_edit: true,
        }
    }
}

impl ConnectionItem {
    pub fn new(name: &str, host: &str, port: u16, user: &str, password: &str, db: &str) -> Self {
        Self {
            id: Ulid::new(),
            name: name.to_string(),
            host: host.to_string(),
            port,
            user: user.to_string(),
            password: password.to_string(),
            db: db.to_string(),
        }
    }

    pub fn new_with_id(
        id: Ulid,
        name: &str,
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        db: &str,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            host: host.to_string(),
            port,
            user: user.to_string(),
            password: password.to_string(),
            db: db.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_key: Option<String>, // "PRI", "MUL", "UNI", etc.
}

#[derive(Debug, Default)]
pub struct SchemaState {
    pub tables: Vec<String>,
    pub selected_table: usize,

    // new:
    pub columns_table: Option<String>, // which table these columns belong to
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Default)]
pub struct DataState {
    pub title: String,
}

#[derive(Debug, Default)]
pub struct QueriesState {
    pub title: String,
}

#[derive(Debug, Default)]
pub struct RunnerState {
    pub title: String,
}
