use landscape_common::error::LandscapeErrRespTrait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Missing Authorization header")]
    MissingAuthorizationHeader,

    #[error("Invalid Authorization header format")]
    InvalidAuthorizationHeaderFormat,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Invalid token")]
    UnauthorizedUser,

    #[error("Invalid username or password")]
    InvalidUsernameOrPassword,

    #[error("Token creation failed: {0}")]
    JwtCreationFailed(#[from] jsonwebtoken::errors::Error),
}

impl LandscapeErrRespTrait for AuthError {
    fn get_code(&self) -> u32 {
        match self {
            AuthError::MissingAuthorizationHeader => 401_401,
            AuthError::InvalidAuthorizationHeaderFormat => 402_401,
            AuthError::InvalidToken => 403_401,
            AuthError::UnauthorizedUser => 404_401,
            AuthError::InvalidUsernameOrPassword => 405_401,
            AuthError::JwtCreationFailed(_) => 406_500,
        }
    }
}
