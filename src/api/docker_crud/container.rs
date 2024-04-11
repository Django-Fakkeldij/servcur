use axum::extract::Path;
use axum::Json;
use axum::{extract::State, http::StatusCode};

use bollard::container::{
    ListContainersOptions, RemoveContainerOptions, RestartContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use serde_json::json;

use crate::SharedAppState;

use super::CrudReturn;

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

pub async fn start_container(
    State(state): State<SharedAppState>,
    Path(name): Path<String>,
) -> CrudReturn {
    state
        .docker
        .lock_owned()
        .await
        .start_container(&name, None::<StartContainerOptions<String>>)
        .await?;
    Ok((StatusCode::OK, Json(json!({}))))
}

pub async fn stop_container(
    State(state): State<SharedAppState>,
    Path(name): Path<String>,
) -> CrudReturn {
    state
        .docker
        .lock_owned()
        .await
        .stop_container(&name, None::<StopContainerOptions>)
        .await?;
    Ok((StatusCode::OK, Json(json!({}))))
}

pub async fn restart_container(
    State(state): State<SharedAppState>,
    Path(name): Path<String>,
) -> CrudReturn {
    state
        .docker
        .lock_owned()
        .await
        .restart_container(&name, None::<RestartContainerOptions>)
        .await?;
    Ok((StatusCode::OK, Json(json!({}))))
}

pub async fn remove_container(
    State(state): State<SharedAppState>,
    Path(name): Path<String>,
) -> CrudReturn {
    state
        .docker
        .lock_owned()
        .await
        .remove_container(
            &name,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await?;
    Ok((StatusCode::OK, Json(json!({}))))
}
