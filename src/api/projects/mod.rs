use std::path::PathBuf;

use anyhow::{bail, Result};

use serde::{Deserialize, Serialize};

use self::actions::ProjectKind;
use crate::config::PROJECT_FOLDER;

pub mod actions;
pub mod executor;
pub mod iohandle_ws;
pub mod project_management;
pub mod project_store;
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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Projects(Vec<Project>);

impl Projects {
    pub fn get(&self, name: &str, branch: &str) -> Option<Project> {
        self.0
            .iter()
            .find(|v| v.project_name == name && v.branch == branch)
            .cloned()
    }

    pub fn insert(&mut self, project: Project) -> Result<()> {
        if self.get(&project.project_name, &project.branch).is_some() {
            bail!("project already exists");
        }
        self.0.push(project);
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Project {
    uri: String,
    path: PathBuf,
    project_name: String,
    branch: String,
    project_kind: ProjectKind,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct BaseProject {
    name: String,
    branch: String,
}
