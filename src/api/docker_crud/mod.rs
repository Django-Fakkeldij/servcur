use axum::Json;
use axum::{extract::State, http::StatusCode};

use serde_json::{json, Value};

use crate::SharedAppState;

use super::error::ApiError;

pub type CrudReturn = Result<(StatusCode, Json<Value>), ApiError>;

pub mod container;
pub mod image;
pub mod network;
pub mod volume;

pub async fn docker_sys_info(State(state): State<SharedAppState>) -> CrudReturn {
    let ret = state.docker.lock_owned().await.info().await?;
    Ok((StatusCode::OK, Json(json!(&ret))))
}
