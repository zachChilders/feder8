use thiserror::Error;

/// Main error type for the feder8 library
#[derive(Error, Debug)]
pub enum Feder8Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Delivery error: {0}")]
    Delivery(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Embedded constraint error: {0}")]
    EmbeddedConstraint(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Feder8Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Feder8Error::Other(err.to_string())
    }
}
