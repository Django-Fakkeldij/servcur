use axum::extract::Path;
use axum::Json;
use axum::{extract::State, http::StatusCode};

use bollard::volume::{ListVolumesOptions, PruneVolumesOptions, RemoveVolumeOptions};
use serde_json::json;

use crate::SharedAppState;

use super::CrudReturn;

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

pub async fn remove_volume(
    State(state): State<SharedAppState>,
    Path(name): Path<String>,
) -> CrudReturn {
    state
        .docker
        .lock_owned()
        .await
        .remove_volume(&name, Some(RemoveVolumeOptions { force: true }))
        .await?;
    Ok((StatusCode::OK, Json(json!({}))))
}

pub async fn prune_volumes(State(state): State<SharedAppState>) -> CrudReturn {
    state
        .docker
        .lock_owned()
        .await
        .prune_volumes(None::<PruneVolumesOptions<String>>)
        .await?;
    Ok((StatusCode::OK, Json(json!({}))))
}
