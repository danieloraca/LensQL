use crate::errors::AppError;
use keyring::Entry;
use tracing::{debug, warn};

pub struct ConnectionSecrets {
    service: String,
}

impl ConnectionSecrets {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    pub fn default() -> Self {
        Self::new("lensql")
    }

    fn entry(&self, connection_id: &str) -> Result<Entry, AppError> {
        if connection_id.trim().is_empty() {
            return Err(AppError::Config(
                "connection_id cannot be empty".to_string(),
            ));
        }

        debug!(
            service = %self.service,
            connection_id = %connection_id,
            "keyring: creating entry"
        );
        Entry::new(&self.service, connection_id)
            .map_err(|e| AppError::Other(format!("keyring entry error: {}", e)))
    }

    pub fn set_password(&self, connection_id: &str, password: &str) -> Result<(), AppError> {
        let entry = self.entry(connection_id)?;

        debug!(
            service = %self.service,
            connection_id = %connection_id,
            password_len = password.len(),
            "keyring: set_password"
        );

        entry.set_password(password).map_err(|e| {
            warn!(
                service = %self.service,
                connection_id = %connection_id,
                error = %e,
                "keyring: set_password failed"
            );
            AppError::Other(format!("keyring set_password error: {}", e))
        })
    }

    pub fn get_password(&self, connection_id: &str) -> Result<Option<String>, AppError> {
        let entry = self.entry(connection_id)?;

        debug!(
            service = %self.service,
            connection_id = %connection_id,
            "keyring: get_password"
        );

        match entry.get_password() {
            Ok(pw) => {
                debug!(
                    service = %self.service,
                    connection_id = %connection_id,
                    password_len = pw.len(),
                    "keyring: get_password hit"
                );
                Ok(Some(pw))
            }
            Err(e) => {
                let msg: String = e.to_string().to_lowercase();

                let is_missing = msg.contains("no matching entry found in secure storage")
                    || msg.contains("no matching entry found")
                    || msg.contains("no matching entry")
                    || msg.contains("not found")
                    || msg.contains("no entry")
                    || msg.contains("no password");

                if is_missing {
                    debug!(
                        service = %self.service,
                        connection_id = %connection_id,
                        "keyring: get_password miss (no entry)"
                    );
                    Ok(None)
                } else {
                    warn!(
                        service = %self.service,
                        connection_id = %connection_id,
                        error = %e,
                        "keyring: get_password failed"
                    );
                    Err(AppError::Other(format!("keyring get_password error: {}", e)))
                }
            }
        }
    }

    pub fn delete_password(&self, connection_id: &str) -> Result<(), AppError> {
        self.set_password(connection_id, "")
    }
}
