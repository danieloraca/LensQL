#[derive(Debug, Clone)]
pub enum Command {
    Db(DbCommand),
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
