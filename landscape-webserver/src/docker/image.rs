use crate::LandscapeApp;
use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Router,
};
use bollard::{query_parameters::ListImagesOptions, secret::ImageSummary, Docker};
use landscape_common::docker::image::PullManagerInfo;

use crate::{api::LandscapeApiResp, error::LandscapeApiResult};

pub async fn get_docker_images_paths() -> Router<LandscapeApp> {
    Router::new()
        .route("/", get(get_all_images))
        .route("/{image_name}", post(pull_image_by_image_name))
        .route("/id/{image_id}", delete(delete_image_by_id))
        .route("/tasks", get(get_current_task))
}

async fn get_current_task(
    State(state): State<LandscapeApp>,
) -> LandscapeApiResult<PullManagerInfo> {
    LandscapeApiResp::success(state.docker_service.pull_manager.get_info().await)
}

async fn get_all_images() -> LandscapeApiResult<Vec<ImageSummary>> {
    let mut summarys: Vec<ImageSummary> = vec![];
    let docker = Docker::connect_with_socket_defaults();

    if let Ok(docker) = docker {
        let option = ListImagesOptions { all: true, ..Default::default() };
        if let Ok(images) = docker.list_images(Some(option)).await {
            summarys = images;
        }
    }
    LandscapeApiResp::success(summarys)
}

async fn pull_image_by_image_name(
    State(state): State<LandscapeApp>,
    Path(image_name): Path<String>,
) -> LandscapeApiResult<()> {
    state.docker_service.pull_manager.pull_img(image_name).await;
    LandscapeApiResp::success(())
}

async fn delete_image_by_id(Path(image_id): Path<String>) -> LandscapeApiResult<()> {
    let command = ["docker", "rmi", &image_id];
    if let Ok(status) = tokio::process::Command::new(&command[0]).args(&command[1..]).status().await
    {
        if status.success() {
            tracing::info!("Docker command executed successfully.");
        } else {
            tracing::error!("Docker command failed with status: {:?}", status);
        }
    }

    LandscapeApiResp::success(())
}
