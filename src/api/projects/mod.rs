use std::path::PathBuf;
use std::process::Output;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use self::actions::ProjectKind;
use crate::config::PROJECT_FOLDER;

pub mod actions;
pub mod project_management;
pub mod routes;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewProject {
    name: String,
    branch: String,
    https_url: String,
    auth: GitAuth,
    project_kind: ProjectKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GitAuth {
    None,
    Token(String),
}

impl GitAuth {
    pub fn is_none(&self) -> bool {
        matches!(self, GitAuth::None)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Project {
    uri: String,
    path: PathBuf,
    project_name: String,
    branch: String,
    project_kind: ProjectKind,
    history: ProjectBuildHistory,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ProjectBuildHistory {
    inner: Vec<ProjectBuild>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ProjectBuild {
    status: usize,
    date_unix_s: usize,
}

#[derive(Debug, Deserialize)]
pub struct BaseProject {
    name: String,
    branch: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BuildLog {
    pub status: usize,
    pub output: String,
}

impl BuildLog {
    pub fn new(status: usize, body: &str) -> Self {
        Self {
            status,
            output: body.to_owned(),
        }
    }

    pub fn from_output(output: Output) -> Result<Self> {
        Ok(Self {
            status: output.status.code().unwrap_or(1) as usize,
            output: String::from_utf8(output.stdout)?,
        })
    }
}
