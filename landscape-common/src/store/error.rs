use serde_json;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Invalid file: {0}")]
    InvalidFile(String),

    #[error("Directory contains invalid files: {0}")]
    DirectoryNotEmpty(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Index corrupted: {0}")]
    IndexCorrupted(String),
}

pub type StoreResult<T> = Result<T, StoreError>;
