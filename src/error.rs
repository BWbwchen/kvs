use std::io;
use thiserror::Error;

/// Error type for kvs.
#[derive(Error, Debug)]
pub enum KvsError {
    /// IO error
    #[error("IO error with {0}")]
    IO(#[from] io::Error),
    /// Serialization or deserialization error.
    #[error("serde error with {0}")]
    Serde(#[from] serde_json::Error),
    /// Removing an non-existent key error
    #[error("Key not found")]
    KeyNotFound,
    /// Unexpected command error
    #[error("Unexpected command type")]
    UnexpectedCommandType,
}

/// Result type for kvs
pub type Result<T> = anyhow::Result<T>;
