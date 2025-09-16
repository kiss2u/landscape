use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PtyError {
    #[error("failed to open PTY: {0}")]
    OpenPty(#[from] io::Error),

    #[error("failed to spawn command: {0}")]
    SpawnCommand(String),

    #[error("{0}")]
    AnyErr(#[from] anyhow::Error),
}
