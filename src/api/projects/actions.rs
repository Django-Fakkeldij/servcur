use std::path::{Path as FsPath, PathBuf};
use std::process::Stdio;

use anyhow::Result;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::util::run_bash;

use super::{executor::ProjectIoHandle, BaseProject};

pub trait Actions {
    fn start(
        &mut self,
        dir: &FsPath,
        project: &BaseProject,
    ) -> impl std::future::Future<Output = anyhow::Result<ProjectIoHandle>> + Send;
    fn stop(
        &mut self,
        dir: &FsPath,
        project: &BaseProject,
    ) -> impl std::future::Future<Output = anyhow::Result<ProjectIoHandle>> + Send;
    fn restart(
        &mut self,
        dir: &FsPath,
        project: &BaseProject,
    ) -> impl std::future::Future<Output = anyhow::Result<ProjectIoHandle>> + Send;
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CustomActions {
    start: String,
    stop: String,
    restart: String,
}

impl Actions for CustomActions {
    async fn start(&mut self, dir: &FsPath, project: &BaseProject) -> Result<ProjectIoHandle> {
        let filename = PathBuf::from(format!("start-{}-{}.sh", &project.name, &project.branch));
        let command = run_bash(&self.start, &filename, dir).await?;

        Ok(ProjectIoHandle::new(project.clone(), command))
    }
    async fn stop(&mut self, dir: &FsPath, project: &BaseProject) -> Result<ProjectIoHandle> {
        let filename = PathBuf::from(format!("stop-{}-{}.sh", &project.name, &project.branch));
        let command = run_bash(&self.start, &filename, dir).await?;

        Ok(ProjectIoHandle::new(project.clone(), command))
    }
    async fn restart(&mut self, dir: &FsPath, project: &BaseProject) -> Result<ProjectIoHandle> {
        let filename = PathBuf::from(format!("restart-{}-{}.sh", &project.name, &project.branch));
        let command = run_bash(&self.start, &filename, dir).await?;

        Ok(ProjectIoHandle::new(project.clone(), command))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComposeActions;
impl Actions for ComposeActions {
    async fn start(&mut self, dir: &FsPath, project: &BaseProject) -> Result<ProjectIoHandle> {
        let mut command = tokio::process::Command::new("docker");
        command
            .arg("compose")
            .arg("up")
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped());

        Ok(ProjectIoHandle::new(project.clone(), command))
    }
    async fn stop(&mut self, dir: &FsPath, project: &BaseProject) -> Result<ProjectIoHandle> {
        let mut command = tokio::process::Command::new("docker");
        command
            .arg("compose")
            .arg("stop")
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped());

        Ok(ProjectIoHandle::new(project.clone(), command))
    }
    async fn restart(&mut self, dir: &FsPath, project: &BaseProject) -> Result<ProjectIoHandle> {
        let mut command = tokio::process::Command::new("docker");
        command
            .arg("compose")
            .arg("up")
            .arg("--force-recreate")
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped());

        Ok(ProjectIoHandle::new(project.clone(), command))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DockerfileActions {
    image_version: u32,
}
impl Actions for DockerfileActions {
    async fn start(&mut self, dir: &FsPath, project: &BaseProject) -> Result<ProjectIoHandle> {
        self.image_version += 1;

        let mut build_command = Command::new("docker");
        build_command
            .arg("build")
            .arg(".")
            .arg("-t")
            .arg(format!(
                "{}-{}:{}",
                project.name, project.branch, self.image_version
            ))
            .current_dir(dir);

        let mut start_command = Command::new("docker");
        start_command
            .arg("run")
            .arg("-d")
            .arg("--name")
            .arg(format!(
                "{}-{}-{}",
                project.name, project.branch, self.image_version
            ))
            .arg("-t")
            .arg(format!(
                "{}-{}:{}",
                project.name, project.branch, self.image_version
            ))
            .current_dir(dir);

        let out = ProjectIoHandle::new(project.clone(), start_command)
            .depends_on_same_tagged(build_command, "build_step".into());
        Ok(out)
    }
    async fn stop(&mut self, dir: &FsPath, project: &BaseProject) -> Result<ProjectIoHandle> {
        let mut command = Command::new("docker");
        command
            .arg("stop")
            .arg(format!(
                "{}-{}-{}",
                project.name, project.branch, self.image_version
            ))
            .current_dir(dir)
            .output()
            .await?;
        Ok(ProjectIoHandle::new(project.clone(), command))
    }
    async fn restart(&mut self, dir: &FsPath, project: &BaseProject) -> Result<ProjectIoHandle> {
        let mut command = tokio::process::Command::new("docker");
        command
            .arg("restart")
            .arg(format!(
                "{}-{}:{}",
                project.name, project.branch, self.image_version
            ))
            .current_dir(dir)
            .output()
            .await?;
        Ok(ProjectIoHandle::new(project.clone(), command))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Dockerfile(DockerfileActions),
    Compose(ComposeActions),
    Custom(CustomActions),
}

impl Actions for ProjectKind {
    async fn start(
        &mut self,
        dir: &FsPath,
        project: &BaseProject,
    ) -> anyhow::Result<ProjectIoHandle> {
        match self {
            ProjectKind::Dockerfile(a) => a.start(dir, project).await,
            ProjectKind::Compose(a) => a.start(dir, project).await,
            ProjectKind::Custom(a) => a.start(dir, project).await,
        }
    }

    async fn stop(
        &mut self,
        dir: &FsPath,
        project: &BaseProject,
    ) -> anyhow::Result<ProjectIoHandle> {
        match self {
            ProjectKind::Dockerfile(a) => a.stop(dir, project).await,
            ProjectKind::Compose(a) => a.stop(dir, project).await,
            ProjectKind::Custom(a) => a.stop(dir, project).await,
        }
    }

    async fn restart(
        &mut self,
        dir: &FsPath,
        project: &BaseProject,
    ) -> anyhow::Result<ProjectIoHandle> {
        match self {
            ProjectKind::Dockerfile(a) => a.restart(dir, project).await,
            ProjectKind::Compose(a) => a.restart(dir, project).await,
            ProjectKind::Custom(a) => a.restart(dir, project).await,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectActions {
    Start,
    Stop,
    Restart,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectActionBody {
    pub action: ProjectActions,
}
