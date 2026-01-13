use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Database error")]
    Db(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unexpected error: {0}")]
    Other(String),
}
