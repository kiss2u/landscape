use serde::{Deserialize, Serialize};

/// This file is to prepare for the future migration
/// of the docker api library to avoid large-scale modification of the API
///
///
pub mod network;

pub const DOCKER_NETWORK_BRIDGE_NAME_OPTION_KEY: &str = "com.docker.network.bridge.name";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DockerTargetEnroll {
    pub id: String,
    pub ifindex: u32,
}
