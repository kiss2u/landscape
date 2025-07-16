use axum::response::IntoResponse;
use landscape_common::error::{LandscapeErrRespTrait, LdError};

use crate::{api::LandscapeApiResp, auth::error::AuthError, docker::error::DockerError};

#[derive(thiserror::Error, Debug)]
pub enum LandscapeApiError {
    #[error("`{0}` not found")]
    NotFound(String),

    // #[error("Failed to parse configuration: {0}")]
    // JsonParseError(#[from] serde_json::Error),
    // #[error("IO error occurred: {0}")]
    // IoError(#[from] std::io::Error),
    #[error(transparent)]
    DockerError(#[from] DockerError),

    #[error(transparent)]
    AuthError(#[from] AuthError),

    #[error("Landscape Error: {0}")]
    LdError(#[from] LdError),

    #[error("DHCPConflict Error: {0}")]
    DHCPConflict(String),
}

impl LandscapeErrRespTrait for LandscapeApiError {
    fn get_code(&self) -> u32 {
        match self {
            LandscapeApiError::NotFound(_) => 201_404,
            LandscapeApiError::AuthError(err) => err.get_code(),
            LandscapeApiError::DockerError(err) => err.get_code(),
            LandscapeApiError::LdError(_) => 500,
            LandscapeApiError::DHCPConflict(_) => 301_400,
        }
    }
}

impl IntoResponse for LandscapeApiError {
    fn into_response(self) -> axum::response::Response {
        let resp: LandscapeApiResp<()> =
            LandscapeApiResp::error(self.get_code(), self.get_message());
        resp.into_response()
    }
}

pub type LandscapeApiResult<T> = Result<LandscapeApiResp<T>, LandscapeApiError>;
