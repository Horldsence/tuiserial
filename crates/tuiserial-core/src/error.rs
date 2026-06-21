use thiserror::Error;

/// Core error types for tuiserial.
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("{0}")]
    Validation(String),

    #[error("Config directory not found")]
    ConfigDirNotFound,

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization: {0}")]
    Serde(#[from] serde_json::Error),
}
