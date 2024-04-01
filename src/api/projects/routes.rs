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
use super::{BaseProject, NewProject};

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
    };

    let uri = project.uri.clone();
    info!(?project_init.name, ?project_init.branch, webhook=project.uri, "created project / branch webhook");

    state
        .projects
        .lock_owned()
        .await
        .insert(project)
        .map_err(|e| ApiError::new(StatusCode::CONFLICT, e))?;

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

    let val = match state.projects.lock_owned().await.get(&name, &branch) {
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
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let mut val = match state.projects.lock_owned().await.get(&name, &branch) {
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

    let handle = match match body.action {
        ProjectActions::Start => val.project_kind.start(&dir, &project).await,
        ProjectActions::Stop => val.project_kind.stop(&dir, &project).await,
        ProjectActions::Restart => val.project_kind.restart(&dir, &project).await,
    } {
        Ok(out) => out,
        Err(e) => return Err(ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e)),
    };

    let project = handle.project.clone();
    let id = state
        .io_executor
        .exec(handle)
        .await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "project": project,
            "io_handle_id": id,
        })),
    ))
}

pub async fn list_projects_route(
    State(state): State<SharedAppState>,
) -> Result<(StatusCode, Json<Vec<Project>>), ApiError> {
    let projects = state.projects.lock_owned().await.to_owned().0;

    Ok((StatusCode::OK, Json(projects)))
}
