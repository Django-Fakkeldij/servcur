use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path as FsPath, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::Json;
use axum::{extract::Query, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};

use crate::config::DATA_FOLDER;
use crate::util::{error_from_stdoutput, run_bash};
use crate::SharedAppState;
use const_format::concatcp;

pub const PROJECT_FOLDER: &str = concatcp!(DATA_FOLDER, "/projects");

pub const WEBHOOK_PATH: &str = "/projects/webhook";

#[derive(Debug, Deserialize)]
pub struct NewProject {
    name: String,
    branch: String,
    https_url: String,
    auth: Auth,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Auth {
    None,
    Token(String),
}

impl Auth {
    pub fn is_none(&self) -> bool {
        matches!(self, Auth::None)
    }
}

pub async fn new_project(project: &NewProject) -> anyhow::Result<()> {
    // Name invalid
    if project.name.contains('/') || project.name.contains('\\') {
        return Err(anyhow::Error::msg("invalid project name"));
    }

    // Invalid url
    if !project.https_url.starts_with("https://") {
        return Err(anyhow::Error::msg("not an https git url"));
    }

    let project_root_folder = format!("{PROJECT_FOLDER}/{}", project.name);
    let project_branch_folder = format!("{PROJECT_FOLDER}/{}/{}", project.name, project.branch);
    // Exists already
    if tokio::fs::try_exists(&project_branch_folder).await? {
        return Err(anyhow::Error::msg("Project/ branch already exists"));
    }

    // Create folder with projectname
    tokio::fs::create_dir_all(&project_root_folder).await?;

    // Create folder with branch name
    tokio::fs::create_dir_all(&project_branch_folder).await?;

    // Clone git repo (with a insecure remote, :0 )
    let output = tokio::process::Command::new("git")
        .arg("clone")
        .arg(&create_auth_url(&project.https_url, &project.auth))
        .arg("-b")
        .arg(&project.branch)
        // Makes it so that it doesn't create a folder within the current work-dir
        .arg(".")
        .current_dir(project_branch_folder)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .await?;
    if !output.success() {
        return Err(anyhow::anyhow!(output.to_string()));
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookLocation {
    uri: String,
    project_name: String,
    branch: String,
}

pub fn format_webhook_url(name: &str, branch: &str, absolute: bool) -> String {
    if absolute {
        return format!("{WEBHOOK_PATH}/{name}/{branch}");
    }
    format!("{name}/{branch}")
}

pub async fn new_project_route(
    State(state): State<SharedAppState>,
    Json(project): Json<NewProject>,
) -> (StatusCode, Json<Value>) {
    match new_project(&project).await {
        Ok(_) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        }
    };
    info!(?project.name, ?project.branch, "created project / branch");

    let location = WebhookLocation {
        uri: format_webhook_url(&project.name, &project.branch, true),
        project_name: project.name.to_owned(),
        branch: project.branch.to_owned(),
    };

    if let Err(e) = state
        .lock_owned()
        .await
        .store
        .insert(&location.uri, serde_json::to_value(&location).unwrap())
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        );
    }
    info!(?project.name, ?project.branch, webhook=location.uri, "created project / branch webhook");
    (StatusCode::CREATED, Json(json!({"webhook": location.uri})))
}

pub async fn pull_project(name: &str, branch: &str) -> anyhow::Result<()> {
    // Name invalid
    if name.contains('/') || name.contains('\\') {
        return Err(anyhow::Error::msg("Invalid project name"));
    }

    let project_branch_folder = format!("{PROJECT_FOLDER}/{}/{}", name, branch);
    // Exists already
    if !tokio::fs::try_exists(&project_branch_folder).await? {
        return Err(anyhow::Error::msg("Project/ branch doesn't exist"));
    }

    // Fetch git repo
    let output = tokio::process::Command::new("git")
        .arg("pull")
        .current_dir(project_branch_folder)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .await?;
    if !output.success() {
        return Err(anyhow::anyhow!(output.to_string()));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct BaseProject {
    name: String,
    branch: String,
}

pub async fn pull_project_route(Query(project): Query<BaseProject>) -> (StatusCode, Json<Value>) {
    match pull_project(&project.name, &project.branch).await {
        Ok(_) => (StatusCode::CREATED, Json(json!({}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
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

    let val = match serde_json::from_value::<WebhookLocation>(val)
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

pub fn create_auth_url(https_url: &str, auth: &Auth) -> String {
    if auth.is_none() {
        return https_url.to_owned();
    }

    // Prepare url based on auth used
    let url_end = https_url.replacen("https://", "", 1);
    let url_auth: &str = match auth {
        Auth::Token(t) => t,
        _ => panic!(),
    };

    let url = format!("https://{url_auth}@{url_end}");

    url
}

pub trait Actions {
    type R: std::fmt::Debug + Display;
    fn start(
        &mut self,
        dir: &FsPath,
        project: &BaseProject,
    ) -> impl std::future::Future<Output = anyhow::Result<Self::R>> + Send;
    fn stop(
        &mut self,
        dir: &FsPath,
        project: &BaseProject,
    ) -> impl std::future::Future<Output = anyhow::Result<Self::R>> + Send;
    fn restart(
        &mut self,
        dir: &FsPath,
        project: &BaseProject,
    ) -> impl std::future::Future<Output = anyhow::Result<Self::R>> + Send;
}

pub struct CustomActions {
    start: String,
    stop: String,
    restart: String,
}

impl Actions for CustomActions {
    type R = String;
    async fn start(&mut self, dir: &FsPath, project: &BaseProject) -> Result<String> {
        let filename = PathBuf::from(format!("start-{}-{}.sh", &project.name, &project.branch));
        run_bash(&self.start, &filename, dir).await
    }
    async fn stop(&mut self, dir: &FsPath, project: &BaseProject) -> Result<String> {
        let filename = PathBuf::from(format!("stop-{}-{}.sh", &project.name, &project.branch));
        run_bash(&self.stop, &filename, dir).await
    }
    async fn restart(&mut self, dir: &FsPath, project: &BaseProject) -> Result<String> {
        let filename = PathBuf::from(format!("restart-{}-{}.sh", &project.name, &project.branch));
        run_bash(&self.restart, &filename, dir).await
    }
}

pub struct ComposeActions;
impl Actions for ComposeActions {
    type R = String;
    async fn start(&mut self, dir: &FsPath, _project: &BaseProject) -> Result<String> {
        let output = tokio::process::Command::new("docker")
            .arg("compose")
            .arg("up")
            .current_dir(dir)
            .output()
            .await?;
        if !output.status.success() {
            return Err(error_from_stdoutput(output)?);
        }
        Ok(String::from_utf8(output.stdout)?)
    }
    async fn stop(&mut self, dir: &FsPath, _project: &BaseProject) -> Result<String> {
        let output = tokio::process::Command::new("docker")
            .arg("compose")
            .arg("stop")
            .current_dir(dir)
            .output()
            .await?;
        if !output.status.success() {
            return Err(error_from_stdoutput(output)?);
        }
        Ok(String::from_utf8(output.stdout)?)
    }
    async fn restart(&mut self, dir: &FsPath, _project: &BaseProject) -> Result<String> {
        let output = tokio::process::Command::new("docker")
            .arg("compose")
            .arg("up")
            .arg("--force-recreate")
            .current_dir(dir)
            .output()
            .await?;
        if !output.status.success() {
            return Err(error_from_stdoutput(output)?);
        }
        Ok(String::from_utf8(output.stdout)?)
    }
}

pub struct DockerfileActions {
    build_id: u32,
}
impl Actions for DockerfileActions {
    type R = String;
    async fn start(&mut self, dir: &FsPath, project: &BaseProject) -> Result<String> {
        self.build_id += 1;

        let build_output = tokio::process::Command::new("docker")
            .arg("build")
            .arg("-t")
            .arg(format!(
                "{}-{}:{}",
                project.name, project.branch, self.build_id
            ))
            .current_dir(dir)
            .output()
            .await?;
        if !build_output.status.success() {
            return Err(error_from_stdoutput(build_output)?);
        }
        let start_output = tokio::process::Command::new("docker")
            .arg("start")
            .arg(format!(
                "{}-{}:{}",
                project.name, project.branch, self.build_id
            ))
            .current_dir(dir)
            .output()
            .await?;
        if !start_output.status.success() {
            return Err(error_from_stdoutput(start_output)?);
        }
        Ok(String::from_utf8(build_output.stdout)?)
    }
    async fn stop(&mut self, dir: &FsPath, project: &BaseProject) -> Result<String> {
        let output = tokio::process::Command::new("docker")
            .arg("stop")
            .arg(format!(
                "{}-{}:{}",
                project.name, project.branch, self.build_id
            ))
            .current_dir(dir)
            .output()
            .await?;
        if !output.status.success() {
            return Err(error_from_stdoutput(output)?);
        }
        Ok(String::from_utf8(output.stdout)?)
    }
    async fn restart(&mut self, dir: &FsPath, project: &BaseProject) -> Result<String> {
        let output = tokio::process::Command::new("docker")
            .arg("restart")
            .arg(format!(
                "{}-{}:{}",
                project.name, project.branch, self.build_id
            ))
            .current_dir(dir)
            .output()
            .await?;
        if !output.status.success() {
            return Err(error_from_stdoutput(output)?);
        }
        Ok(String::from_utf8(output.stdout)?)
    }
}

pub enum ProjectActions {
    Start,
    Stop,
    Restart,
}
