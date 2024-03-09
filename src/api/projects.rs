use std::process::Stdio;

use axum::Json;
use axum::{extract::Query, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PROJECT_FOLDER: &str = "./projects";

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

pub async fn new_project_route(Json(project): Json<NewProject>) -> (StatusCode, Json<Value>) {
    match new_project(&project).await {
        Ok(_) => (StatusCode::CREATED, Json(json!({}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn fetch_project(name: &str, branch: &str) -> anyhow::Result<()> {
    // Name invalid
    if name.contains('/') || name.contains('\\') {
        return Err(anyhow::Error::msg("Invalid project name"));
    }

    let project_branch_folder = format!("{PROJECT_FOLDER}/{}/{}", name, branch);
    // Exists already
    if !tokio::fs::try_exists(&project_branch_folder).await? {
        return Err(anyhow::Error::msg("Project/ branch doesn't exist"));
    }

    // Clone git repo
    let output = tokio::process::Command::new("git")
        .arg("fetch")
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
pub struct FetchProject {
    name: String,
    branch: String,
}

pub async fn fetch_project_route(Query(project): Query<FetchProject>) -> (StatusCode, Json<Value>) {
    match fetch_project(&project.name, &project.branch).await {
        Ok(_) => (StatusCode::CREATED, Json(json!({}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
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
