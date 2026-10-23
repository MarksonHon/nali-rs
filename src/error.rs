use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NalideError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database not found: {0}")]
    DatabaseNotFound(String),

    #[error("Database load error for '{path}': {reason}")]
    DatabaseLoad { path: PathBuf, reason: String },

    #[error("Database query error: {0}")]
    Query(String),

    #[error("Invalid IP address: {0}")]
    InvalidIp(String),

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Update error: {0}")]
    Update(String),

    #[error("Directory error: {0}")]
    Directory(String),
}

pub type Result<T> = std::result::Result<T, NalideError>;
