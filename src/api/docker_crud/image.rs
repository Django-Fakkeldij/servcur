use axum::extract::Path;
use axum::Json;
use axum::{extract::State, http::StatusCode};

use bollard::image::{ListImagesOptions, PruneImagesOptions, RemoveImageOptions};
use serde_json::json;

use crate::SharedAppState;

use super::CrudReturn;

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

pub async fn prune_images(State(state): State<SharedAppState>) -> CrudReturn {
    let ret = state
        .docker
        .lock_owned()
        .await
        .prune_images(None::<PruneImagesOptions<String>>)
        .await?;
    Ok((StatusCode::OK, Json(json!(&ret))))
}

pub async fn remove_images(
    State(state): State<SharedAppState>,
    Path(name): Path<String>,
) -> CrudReturn {
    let ret = state
        .docker
        .lock_owned()
        .await
        .remove_image(
            &name,
            Some(RemoveImageOptions {
                force: true,
                ..Default::default()
            }),
            None,
        )
        .await?;
    Ok((StatusCode::OK, Json(json!(&ret))))
}
