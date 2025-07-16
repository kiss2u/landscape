use landscape_common::error::LandscapeErrRespTrait;

#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error("create container error")]
    CreateContainerError,

    #[error("run container error")]
    StartContainerError,

    #[error("stop container error")]
    StopContainerError,

    #[error("remove container error")]
    FailToRemoveContainer,

    #[error("run container error")]
    FailToRunContainerByCmd,
}

impl LandscapeErrRespTrait for DockerError {
    fn get_code(&self) -> u32 {
        match self {
            DockerError::CreateContainerError => 501_500,
            DockerError::StartContainerError => 502_500,
            DockerError::StopContainerError => 503_500,
            DockerError::FailToRemoveContainer => 504_500,
            DockerError::FailToRunContainerByCmd => 505_500,
        }
    }
}
