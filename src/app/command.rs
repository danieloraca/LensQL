#[derive(Debug, Clone)]
pub enum Command {
    Db(DbCommand),
    Storage(StorageCommand),
}

#[derive(Debug, Clone)]
pub enum StorageCommand {
    SaveConnections {
        connections: Vec<crate::storage::model::ConnectionProfile>,
    },
}

#[derive(Debug, Clone)]
pub enum DbCommand {
    Connect {
        name: String,
        host: String,
        port: u16,
        user: String,
        password: String,
        db: String,
    },
    Disconnect,
    LoadTables,
    LoadColumns {
        table: String,
    },
}
