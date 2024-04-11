use axum::extract::Path;
use axum::Json;
use axum::{extract::State, http::StatusCode};

use bollard::network::{ListNetworksOptions, PruneNetworksOptions};
use serde_json::json;

use crate::SharedAppState;

use super::CrudReturn;

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

pub async fn prune_networks(State(state): State<SharedAppState>) -> CrudReturn {
    let ret = state
        .docker
        .lock_owned()
        .await
        .prune_networks(None::<PruneNetworksOptions<String>>)
        .await?;
    Ok((StatusCode::OK, Json(json!(&ret))))
}

pub async fn remove_network(
    State(state): State<SharedAppState>,
    Path(name): Path<String>,
) -> CrudReturn {
    state
        .docker
        .lock_owned()
        .await
        .remove_network(&name)
        .await?;
    Ok((StatusCode::OK, Json(json!({}))))
}
