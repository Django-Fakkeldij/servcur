use std::path::Path as FsPath;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use super::{executor::ProjectIoHandle, BaseProject, Project};

pub trait Action {
    type W;
    fn exec(
        &self,
        which: Self::W,
        dir: &FsPath,
        project: &BaseProject,
    ) -> impl std::future::Future<Output = anyhow::Result<ProjectIoHandle>> + Send;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockerFileActions {
    Build,
    Start,
    Stop,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockerFile {
    build_id: usize,
}

impl Action for DockerFile {
    type W = DockerFileActions;
    async fn exec(
        &self,
        _which: Self::W,
        _dir: &FsPath,
        _project: &BaseProject,
    ) -> anyhow::Result<ProjectIoHandle> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectKind {
    DockerFile(DockerFile),
    DockerCompose(DockerFile),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectAction {
    DockerFile(DockerFileActions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCommand {
    action_kind: ProjectAction,
}

impl ActionCommand {
    pub async fn try_exec(
        self,
        dir: &FsPath,
        base_project: &BaseProject,
        project: &Project,
    ) -> anyhow::Result<ProjectIoHandle> {
        match self.action_kind {
            ProjectAction::DockerFile(action) => {
                if let ProjectKind::DockerFile(v) = &project.project_kind {
                    todo!()
                }
                Err(anyhow!("Wrong project kind"))
            }
        }
    }
}
