use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use bollard::{
    query_parameters::{
        CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
        StartContainerOptions, StopContainerOptions,
    },
    secret::{ContainerCreateBody, ContainerSummary},
    Docker,
};

use image::get_docker_images_paths;
use landscape_common::{docker::DockerCmd, service::DefaultWatchServiceStatus};
use network::get_docker_networks_paths;

use crate::{api::LandscapeApiResp, error::LandscapeApiResult};
use crate::{docker::error::DockerError, LandscapeApp};

pub mod error;
mod image;
mod network;

pub async fn get_docker_paths() -> Router<LandscapeApp> {
    Router::new()
        .route(
            "/status",
            get(get_docker_status).post(start_docker_status).delete(stop_docker_status),
        )
        .route("/container_summarys", get(get_all_container_summarys))
        .route("/run/{container_name}", post(run_container))
        .route("/run_cmd", post(run_cmd_container))
        .route("/start/{container_name}", post(start_container))
        .route("/stop/{container_name}", post(stop_container))
        .route("/remove/{container_name}", post(remove_container))
        .nest("/images", get_docker_images_paths().await)
        .nest("/networks", get_docker_networks_paths().await)
}

async fn get_docker_status(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<DefaultWatchServiceStatus> {
    LandscapeApiResp::success(state.docker_service.status)
}

async fn start_docker_status(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<DefaultWatchServiceStatus> {
    state.docker_service.start_to_listen_event().await;
    LandscapeApiResp::success(state.docker_service.status)
}

async fn stop_docker_status(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<DefaultWatchServiceStatus> {
    state.docker_service.status.wait_stop().await;
    LandscapeApiResp::success(state.docker_service.status)
}

async fn get_all_container_summarys() -> LandscapeApiResult<Vec<ContainerSummary>> {
    let mut container_summarys: Vec<ContainerSummary> = vec![];
    let docker = Docker::connect_with_socket_defaults();

    if let Ok(docker) = docker {
        let option = ListContainersOptions { all: true, ..Default::default() };
        if let Ok(containers) = docker.list_containers(Some(option)).await {
            container_summarys = containers;
        }
    }

    LandscapeApiResp::success(container_summarys)
}

async fn run_container(
    Path(container_name): Path<String>,
    Json(container_config): Json<ContainerCreateBody>,
) -> LandscapeApiResult<()> {
    let docker = Docker::connect_with_socket_defaults().unwrap();
    if let Err(e) = &docker
        .create_container(
            Some(CreateContainerOptions {
                name: Some(container_name.clone()),
                platform: "".to_string(),
            }),
            container_config,
        )
        .await
    {
        tracing::error!("{:?}", e);
        return Err(DockerError::CreateContainerError)?;
    } else {
        let query: Option<StartContainerOptions> = None;
        if let Err(e) = &docker.start_container(&container_name, query).await {
            tracing::error!("{:?}", e);
            return Err(DockerError::StartContainerError)?;
        }
    }
    LandscapeApiResp::success(())
}

async fn start_container(Path(container_name): Path<String>) -> LandscapeApiResult<()> {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    if let Err(e) = &docker.start_container(&container_name, None::<StartContainerOptions>).await {
        tracing::error!("{:?}", e);
        return Err(DockerError::StartContainerError)?;
    }

    LandscapeApiResp::success(())
}

async fn stop_container(Path(container_name): Path<String>) -> LandscapeApiResult<()> {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    if let Err(e) = &docker.stop_container(&container_name, None::<StopContainerOptions>).await {
        tracing::error!("{:?}", e);
        return Err(DockerError::StopContainerError)?;
    }

    LandscapeApiResp::success(())
}

async fn remove_container(Path(container_name): Path<String>) -> LandscapeApiResult<()> {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let config = RemoveContainerOptions { force: true, v: false, link: false };
    if let Err(e) = &docker.remove_container(&container_name, Some(config)).await {
        tracing::error!("{:?}", e);
        return Err(DockerError::FailToRemoveContainer)?;
    }

    LandscapeApiResp::success(())
}

async fn run_cmd_container(
    State(state): State<LandscapeApp>,
    Json(docker_cmd): Json<DockerCmd>,
) -> LandscapeApiResult<()> {
    if let Err(_) = docker_cmd.execute_docker_command(&state.home_path).await {
        return Err(DockerError::FailToRunContainerByCmd)?;
    }
    LandscapeApiResp::success(())
}
