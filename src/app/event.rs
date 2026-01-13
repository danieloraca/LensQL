#[derive(Debug, Clone)]
pub enum Event {
    Db(DbEvent),
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
