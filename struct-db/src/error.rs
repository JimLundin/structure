use thiserror::Error;

/// Error types for struct-db operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("Record not found: {0}")]
    RecordNotFound(u64),

    #[error("Type '{0}' is not registered with the database. Call .register::<{0}>() when opening the database.")]
    TypeNotRegistered(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("WAL error: {0}")]
    Wal(String),

    #[error("Database error: {0}")]
    Other(String),
}

/// Result type for database operations
pub type Result<T> = std::result::Result<T, Error>;

// Allow conversion from anyhow::Error for backward compatibility during transition
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err.to_string())
    }
}
