use landscape_common::LdApiError;

#[derive(Debug, thiserror::Error, LdApiError)]
pub enum AuthError {
    #[error("Missing Authorization header")]
    #[api_error(id = "auth.missing_header", status = 401)]
    MissingAuthorizationHeader,

    #[error("Invalid Authorization header format")]
    #[api_error(id = "auth.invalid_format", status = 401)]
    InvalidAuthorizationHeaderFormat,

    #[error("Invalid token")]
    #[api_error(id = "auth.invalid_token", status = 401)]
    InvalidToken,

    #[error("Unauthorized user")]
    #[api_error(id = "auth.unauthorized", status = 401)]
    UnauthorizedUser,

    #[error("Invalid username or password")]
    #[api_error(id = "auth.invalid_credentials", status = 401)]
    InvalidUsernameOrPassword,

    #[error("Token creation failed: {0}")]
    #[api_error(id = "auth.token_creation_failed", status = 500)]
    JwtCreationFailed(#[from] jsonwebtoken::errors::Error),

    #[error("Current password is incorrect")]
    #[api_error(id = "auth.current_password_incorrect", status = 400)]
    CurrentPasswordIncorrect,

    #[error("New password does not meet complexity requirements")]
    #[api_error(id = "auth.password_too_weak", status = 400)]
    PasswordTooWeak,

    #[error("New password and confirm password do not match")]
    #[api_error(id = "auth.password_mismatch", status = 400)]
    PasswordMismatch,

    #[error("New password cannot be the same as current password")]
    #[api_error(id = "auth.password_same_as_old", status = 400)]
    SamePassword,
}
