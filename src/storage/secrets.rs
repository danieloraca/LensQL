use crate::errors::AppError;
use keyring::Entry;

/// Keyring helper for storing per-connection passwords.
///
/// Design:
/// - We store secrets under a stable `service` name for this app.
/// - The `username` slot is the connection id (ULID as string).
///
/// This allows:
/// - `set_password(conn_id, password)`
/// - `get_password(conn_id)`
/// - `delete_password(conn_id)`
///
/// Note: keyring backend behavior varies by OS.
/// - macOS: stored in Keychain
/// - Linux: Secret Service (GNOME Keyring / KWallet)
/// - Windows: Credential Manager
pub struct ConnectionSecrets {
    service: String,
}

impl ConnectionSecrets {
    /// Create a new secret store. Typically you want the default.
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    /// Default service name used by this app.
    pub fn default() -> Self {
        Self::new("lensql")
    }

    fn entry(&self, connection_id: &str) -> Result<Entry, AppError> {
        if connection_id.trim().is_empty() {
            return Err(AppError::Config(
                "connection_id cannot be empty".to_string(),
            ));
        }

        Entry::new(&self.service, connection_id)
            .map_err(|e| AppError::Other(format!("keyring entry error: {}", e)))
    }

    /// Store/overwrite the password for a connection id.
    pub fn set_password(&self, connection_id: &str, password: &str) -> Result<(), AppError> {
        // Allow empty password (some DBs allow it); but reject missing id.
        let entry = self.entry(connection_id)?;
        entry
            .set_password(password)
            .map_err(|e| AppError::Other(format!("keyring set_password error: {}", e)))
    }

    /// Fetch the stored password for a connection id.
    ///
    /// Returns:
    /// - `Ok(Some(password))` when present
    /// - `Ok(None)` when not found / not set
    /// - `Err` for other keyring errors
    pub fn get_password(&self, connection_id: &str) -> Result<Option<String>, AppError> {
        let entry = self.entry(connection_id)?;

        match entry.get_password() {
               Ok(pw) => Ok(Some(pw)),
               Err(e) => {
                   // Detect "not found" via message (keyring's error surface differs by backend).
                   let msg: String = e.to_string().to_lowercase();
                   if msg.contains("not found")
                       || msg.contains("no entry")
                       || msg.contains("no password")
                       || msg.contains("missing")
                       || msg.contains("no matching entry")
                       || msg.contains("no matching entry found")
                       || msg.contains("secure storage")
                   {
                       Ok(None)
                   } else {
                       Err(AppError::Other(format!("keyring get_password error: {}", e)))
                   }
               }
        }
    }

    /// Delete the stored password for a connection id.
    ///
    /// `keyring` v3 does not expose a cross-platform delete API on `Entry`,
    /// so we approximate deletion by overwriting the secret with an empty string.
    ///
    /// If you need true deletion semantics per-platform, we can revisit this with
    /// backend-specific handling.
    pub fn delete_password(&self, connection_id: &str) -> Result<(), AppError> {
        self.set_password(connection_id, "")
    }
}
