#[derive(Debug, Clone)]
pub enum Event {
    Db(DbEvent),
    Storage(StorageEvent),
}

#[derive(Debug, Clone)]
pub enum StorageEvent {
    ConnectionsSaved,
    Error { message: String },
}

#[derive(Debug, Clone)]
pub enum DbEvent {
    Connected {
        display: String,
    },
    Disconnected,
    TablesLoaded {
        tables: Vec<String>,
    },
    ColumnsLoaded {
        table: String,
        columns: Vec<crate::app::state::ColumnInfo>,
    },
    Error {
        message: String,
    },
}
