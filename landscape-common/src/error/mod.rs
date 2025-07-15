use sea_orm::DbErr;
use thiserror::Error;

/// 仅定义当前 common 错误
#[derive(Error, Debug)]
pub enum LdError {
    #[error("Lnadscape boot error: {0}")]
    Boot(String),
    // OpenFileError
    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("homedir error occurred: {0}")]
    HomeError(#[from] homedir::GetHomeError),

    #[error("setting cpu balance error: {0}")]
    SettingCpuBalanceError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DbErr),

    #[error("data is expired")]
    DataIsExpired,

    #[error("Database error: {0}")]
    DbMsg(String),
}

pub type LdResult<T> = Result<T, LdError>;
