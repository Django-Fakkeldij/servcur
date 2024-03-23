use std::path::Path;
use std::process::Output;
use std::{path::PathBuf, process::Stdio};

use anyhow::{bail, Result};

use chrono::Local;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, warn};

use self::actions::ProjectKind;
use crate::config::{BUILD_LOG_FOLDER, PROJECT_FOLDER};
use crate::util::upsert_file;

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
        if self.get(&project.project_name, &project.branch).is_none() {
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

#[derive(Debug, Deserialize)]
pub struct BaseProject {
    name: String,
    branch: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct IoLog {
    pub status: usize,
    pub stdout: String,
    pub stderr: String,
}

impl IoLog {
    pub fn new(status: usize, stdout: &str, stderr: &str) -> Self {
        Self {
            status,
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        }
    }

    pub fn from_output(output: Output) -> Result<Self> {
        Ok(Self {
            status: output.status.code().unwrap_or(1) as usize,
            stdout: String::from_utf8(output.stdout)?,
            stderr: String::from_utf8(output.stderr)?,
        })
    }

    pub async fn direct_to_file(&self, folder: &Path, file: &Path) -> Result<PathBuf> {
        upsert_file(folder, file, &serde_json::to_string_pretty(&self)?).await
    }
}

#[derive(Debug)]
pub struct ProjectIoHandle {
    pub project: BaseProject,
    pub stdout: Stdio,
    pub stderr: Stdio,
    pub command: Command,
}

#[derive(Debug)]
pub struct ProjectIoExecutor {
    exec_tx: mpsc::Sender<ProjectIoHandle>,
    _exec_handle: JoinHandle<()>,
}

impl ProjectIoExecutor {
    pub fn new(size: usize) -> Self {
        let (tx, mut rx) = mpsc::channel::<ProjectIoHandle>(size);

        let _exec_handle = tokio::spawn(async move {
            loop {
                let mut handle = if let Some(v) = rx.recv().await {
                    v
                } else {
                    warn!("emtpy recv");
                    continue;
                };

                tokio::spawn(async move {
                    let out = match handle.command.output().await {
                        Ok(v) => v,
                        Err(e) => {
                            error!(?e, "error while executing command");
                            return;
                        }
                    };
                    match IoLog::from_output(out) {
                        Ok(v) => {
                            let time_str = Local::now().to_rfc3339();
                            let filename = format!(
                                "{}-{}_{}",
                                handle.project.name, handle.project.branch, time_str
                            );
                            if let Err(e) = v
                                .direct_to_file(
                                    &PathBuf::from(BUILD_LOG_FOLDER),
                                    &PathBuf::from(filename),
                                )
                                .await
                            {
                                error!(?e, "error while writing to file")
                            }
                        }
                        Err(e) => {
                            error!(?e, "error while converting to log")
                        }
                    };
                });
            }
        });

        Self {
            _exec_handle,
            exec_tx: tx,
        }
    }

    pub async fn exec(&mut self, handle: ProjectIoHandle) -> Result<()> {
        self.exec_tx.send(handle).await.map_err(|e| e.into())
    }
}
