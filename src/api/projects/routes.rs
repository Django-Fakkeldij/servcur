use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::Json;
use axum::{extract::Query, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::fs;
use tracing::{error, info};
use ulid::Ulid;

use crate::api::error::ApiError;
use crate::api::projects::project_management::new_project;
use crate::api::projects::Project;
use crate::config::IO_LOG_FOLDER;
use crate::util::format_webhook_url;
use crate::SharedAppState;

use super::actions::ActionCommand;
use super::executor::IoHandleID;
use super::project_management::{pull_project, remove_project};
use super::{BaseProject, NewProject};

use anyhow::anyhow;

pub async fn list_builds() -> Result<Json<Vec<String>>, ApiError> {
    let mut files_iter = fs::read_dir(&PathBuf::from(IO_LOG_FOLDER))
        .await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.into()))?;

    let mut files: Vec<String> = Vec::new();

    while let Ok(Some(f)) = files_iter.next_entry().await {
        files.push(f.file_name().into_string().map_err(|_| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow!("found malformed string"),
            )
        })?);
    }

    Ok(Json(files))
}
pub async fn list_current_builds(
    State(state): State<SharedAppState>,
) -> Result<Json<BTreeMap<Ulid, BaseProject>>, ApiError> {
    let s;

    {
        s = state.io_executor.get_handles().await.clone();
    }

    Ok(Json(
        s.into_iter()
            .map(|(k, v)| (k, (v.1).clone()))
            .collect::<BTreeMap<_, _>>(),
    ))
}

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
        .insert(project)
        .await
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

    let val = match state.projects.get_owned(&name, &branch).await {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectActionReturn {
    project: BaseProject,
    io_id: IoHandleID,
}

pub async fn project_action_route(
    Path((name, branch)): Path<(String, String)>,
    State(mut state): State<SharedAppState>,
    Json(body): Json<ActionCommand>,
) -> Result<(StatusCode, Json<ProjectActionReturn>), ApiError> {
    let mut all_projects = state.projects.get_mut().await;
    let project = match all_projects.get_mut(&name, &branch) {
        Some(a) => a,
        None => {
            error!(?name, ?branch, "no project registered");
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                anyhow::anyhow!("no project registred"),
            ));
        }
    };

    let base_project = BaseProject {
        name: project.project_name.clone(),
        branch: project.branch.clone(),
    };
    let dir = project.path.clone();

    let handle = match body.try_exec(&dir, &base_project, project).await {
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
        Json(ProjectActionReturn { project, io_id: id }),
    ))
}

pub async fn list_projects_route(
    State(state): State<SharedAppState>,
) -> Result<(StatusCode, Json<Vec<Project>>), ApiError> {
    let projects = state.projects.get_all().await.0;

    Ok((StatusCode::OK, Json(projects)))
}
pub async fn remove_project_route(
    State(state): State<SharedAppState>,
    Query(project): Query<BaseProject>,
) -> Result<StatusCode, ApiError> {
    remove_project(&project.name, &project.branch).await?;
    state.projects.remove(&project).await?;

    Ok(StatusCode::OK)
}
