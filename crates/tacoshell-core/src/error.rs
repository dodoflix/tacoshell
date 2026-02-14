//! Error types for Tacoshell

use thiserror::Error;

/// Core error type for Tacoshell operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Secret error: {0}")]
    Secret(String),

    #[error("Transfer error: {0}")]
    Transfer(String),

    #[error("Kubernetes error: {0}")]
    Kubernetes(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Result type alias using Tacoshell's Error type
pub type Result<T> = std::result::Result<T, Error>;

