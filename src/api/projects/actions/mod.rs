use std::path::Path as FsPath;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use self::docker::{DockerFile, DockerFileActions};

use super::{executor::ProjectIoHandle, BaseProject, Project};

pub mod docker;

pub trait Action {
    type W;
    fn exec(
        &mut self,
        which: &Self::W,
        dir: &FsPath,
        project: &BaseProject,
    ) -> impl std::future::Future<Output = anyhow::Result<ProjectIoHandle>> + Send;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProjectKind {
    DockerFile(DockerFile),
    DockerCompose(DockerFile),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "project_kind", content = "command")]
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
        project: &mut Project,
    ) -> anyhow::Result<ProjectIoHandle> {
        match &self.action_kind {
            ProjectAction::DockerFile(action) => {
                if let ProjectKind::DockerFile(v) = &mut project.project_kind {
                    return v.exec(action, dir, base_project).await;
                }
            }
        }
        Err(anyhow!(
            "wrong projectaction or kind. Got {:?}, expected {:?}",
            self.action_kind,
            project.project_kind
        ))
    }
}
