use super::{model::ConnectionProfile, repo::ConnectionRepo};
use crate::errors::AppError;
use std::{fs, path::PathBuf};

pub struct FileConnectionRepo {
    path: PathBuf,
}

impl FileConnectionRepo {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ConnectionRepo for FileConnectionRepo {
    fn load_connections(&self) -> Result<Vec<ConnectionProfile>, AppError> {
        if !self.path.exists() {
            return Ok(vec![]);
        }

        let data = fs::read_to_string(&self.path)?;
        let conns = serde_json::from_str(&data).map_err(|e| AppError::Config(e.to_string()))?;
        Ok(conns)
    }

    fn save_connections(&self, conns: &[ConnectionProfile]) -> Result<(), AppError> {
        let data =
            serde_json::to_string_pretty(conns).map_err(|e| AppError::Config(e.to_string()))?;
        fs::write(&self.path, data)?;
        Ok(())
    }
}
