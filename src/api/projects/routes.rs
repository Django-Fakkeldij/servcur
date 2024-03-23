use std::collections::HashMap;

use anyhow::Result;
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
    Json(project_init): Json<NewProject>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let path = match new_project(&project_init).await {
        Ok(v) => v,
        Err(e) => return Err(ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e)),
    };
    info!(?project_init.name, ?project_init.branch, "created project / branch");

    let project = Project {
        uri: format_webhook_url(&project_init.name, &project_init.branch, true),
        project_name: project_init.name.to_owned(),
        branch: project_init.branch.to_owned(),
        project_kind: project_init.project_kind,
        path,
        history: Default::default(),
    };

    let uri = project.uri.clone();
    info!(?project_init.name, ?project_init.branch, webhook=project.uri, "created project / branch webhook");
    state
        .lock_owned()
        .await
        .projects
        .insert(uri.clone(), project);

    Ok((StatusCode::CREATED, Json(json!({"webhook": uri}))))
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
        info!(?webhook_body, "received an non relevant webhook");
        return StatusCode::OK;
    };

    let key = format_webhook_url(&name, &branch, true);

    let val = match state.lock_owned().await.projects.get(&key) {
        Some(a) => a.clone(),
        None => {
            error!(?name, ?branch, "no webhook registered");
            return StatusCode::NOT_FOUND;
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

    let mut val = match state.lock_owned().await.projects.get(&key) {
        Some(a) => a.clone(),
        None => {
            error!(?name, ?branch, "no project registered");
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                anyhow::anyhow!("no project registred"),
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
