use std::collections::HashMap;

use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::Json;
use axum::{extract::Query, http::StatusCode};
use serde_json::{json, Value};
use tracing::{error, info};

use crate::api::error::ApiError;
use crate::api::projects::project_management::new_project;
use crate::api::projects::Project;
use crate::util::format_webhook_url;
use crate::SharedAppState;

use super::project_management::pull_project;
use super::{BaseProject, BuildLog, NewProject};

use super::actions::{Actions, ProjectActionBody, ProjectActions};

pub async fn pull_project_route(
    Query(project): Query<BaseProject>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    Ok(pull_project(&project.name, &project.branch)
        .await
        .map(|_| (StatusCode::CREATED, Json(json!({}))))?)
}

pub async fn new_project_route(
    State(state): State<SharedAppState>,
    Json(project): Json<NewProject>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let path = match new_project(&project).await {
        Ok(v) => v,
        Err(e) => return Err(ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e)),
    };
    info!(?project.name, ?project.branch, "created project / branch");

    let location = Project {
        uri: format_webhook_url(&project.name, &project.branch, true),
        project_name: project.name.to_owned(),
        branch: project.branch.to_owned(),
        project_kind: project.project_kind,
        path,
        history: Default::default(),
    };

    if let Err(e) = state
        .lock_owned()
        .await
        .store
        .insert(&location.uri, serde_json::to_value(&location).unwrap())
        .await
    {
        return Err(ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e));
    }
    info!(?project.name, ?project.branch, webhook=location.uri, "created project / branch webhook");
    Ok((StatusCode::CREATED, Json(json!({"webhook": location.uri}))))
}

pub async fn webhook_route(
    Path((name, branch)): Path<(String, String)>,
    State(state): State<SharedAppState>,
    Json(webhook_body): Json<HashMap<String, Value>>,
) -> StatusCode {
    if !(webhook_body.contains_key("before")
        && webhook_body.contains_key("after")
        && webhook_body.contains_key("compare"))
    {
        return StatusCode::OK;
    };

    let key = format_webhook_url(&name, &branch, true);

    let val = match state.lock_owned().await.store.get(&key).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            error!(?name, ?branch, "no webhook registered");
            return StatusCode::NOT_FOUND;
        }
        Err(e) => {
            error!(?e, ?name, ?branch, "error while receiving webhook");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let val = match serde_json::from_value::<Project>(val)
        .context("json value did not have the right type")
    {
        Ok(a) => a,
        Err(e) => {
            error!(?e, ?name, ?branch, "reading store error");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    match pull_project(&val.project_name, &val.branch).await {
        Ok(_) => info!(
            name = &val.project_name,
            branch = &val.branch,
            "fetched on webhook"
        ),
        Err(e) => {
            error!(
                ?e,
                name = &val.project_name,
                branch = &val.branch,
                "fetch error"
            );
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::OK
}

pub async fn project_action_route(
    Path((name, branch)): Path<(String, String)>,
    State(state): State<SharedAppState>,
    Json(body): Json<ProjectActionBody>,
) -> Result<(StatusCode, Json<BuildLog>), ApiError> {
    let key = format_webhook_url(&name, &branch, true);

    let val = match state.lock_owned().await.store.get(&key).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            error!(?name, ?branch, "no project registered");
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                anyhow::anyhow!("no project registred"),
            ));
        }
        Err(e) => {
            error!(?e, ?name, ?branch, "error while receiving location");
            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow::anyhow!("error while receiving location"),
            ));
        }
    };

    let mut val = match serde_json::from_value::<Project>(val)
        .context("json value did not have the right type")
    {
        Ok(a) => a,
        Err(e) => {
            error!(?e, ?name, ?branch, "reading store error");
            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow::anyhow!("reading store error"),
            ));
        }
    };

    let project = BaseProject {
        name: val.project_name,
        branch: val.branch,
    };
    let dir = val.path;

    match match body.action {
        ProjectActions::Start => val.project_kind.start(&dir, &project).await,
        ProjectActions::Stop => val.project_kind.stop(&dir, &project).await,
        ProjectActions::Restart => val.project_kind.restart(&dir, &project).await,
    } {
        Ok(out) => Ok((StatusCode::OK, Json(out))),
        Err(e) => Err(ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
