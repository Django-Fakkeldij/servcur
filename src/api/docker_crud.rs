use axum::Json;
use axum::{extract::State, http::StatusCode};

use bollard::{
    container::ListContainersOptions, image::ListImagesOptions, network::ListNetworksOptions,
    volume::ListVolumesOptions,
};
use serde_json::{json, Value};

use crate::SharedAppState;

use super::error::ApiError;

pub type CrudReturn = Result<(StatusCode, Json<Value>), ApiError>;

pub async fn docker_sys_info(State(state): State<SharedAppState>) -> CrudReturn {
    let ret = state.docker.lock_owned().await.info().await?;
    Ok((StatusCode::OK, Json(json!(&ret))))
}

pub async fn containers(State(state): State<SharedAppState>) -> CrudReturn {
    let ret = state
        .docker
        .lock_owned()
        .await
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;
    Ok((StatusCode::OK, Json(json!(&ret))))
}

pub async fn images(State(state): State<SharedAppState>) -> CrudReturn {
    let ret = state
        .docker
        .lock_owned()
        .await
        .list_images(Some(ListImagesOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;
    Ok((StatusCode::OK, Json(json!(&ret))))
}

pub async fn volumes(State(state): State<SharedAppState>) -> CrudReturn {
    let ret = state
        .docker
        .lock_owned()
        .await
        .list_volumes(Some(ListVolumesOptions::<String> {
            ..Default::default()
        }))
        .await?;
    Ok((StatusCode::OK, Json(json!(&ret))))
}

pub async fn networks(State(state): State<SharedAppState>) -> CrudReturn {
    let ret = state
        .docker
        .lock_owned()
        .await
        .list_networks(Some(ListNetworksOptions::<String> {
            ..Default::default()
        }))
        .await?;
    Ok((StatusCode::OK, Json(json!(&ret))))
}
