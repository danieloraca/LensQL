use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: String, // uuid later
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub database: String,
}
