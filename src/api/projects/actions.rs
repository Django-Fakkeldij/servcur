use std::path::{Path as FsPath, PathBuf};

use anyhow::Result;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::util::{error_from_stdoutput, run_bash};

use super::{BaseProject, BuildLog};

pub trait Actions {
    type R: Serialize + DeserializeOwned;
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CustomActions {
    start: String,
    stop: String,
    restart: String,
}

impl Actions for CustomActions {
    type R = BuildLog;
    async fn start(&mut self, dir: &FsPath, project: &BaseProject) -> Result<Self::R> {
        let filename = PathBuf::from(format!("start-{}-{}.sh", &project.name, &project.branch));
        run_bash(&self.start, &filename, dir).await
    }
    async fn stop(&mut self, dir: &FsPath, project: &BaseProject) -> Result<Self::R> {
        let filename = PathBuf::from(format!("stop-{}-{}.sh", &project.name, &project.branch));
        run_bash(&self.stop, &filename, dir).await
    }
    async fn restart(&mut self, dir: &FsPath, project: &BaseProject) -> Result<Self::R> {
        let filename = PathBuf::from(format!("restart-{}-{}.sh", &project.name, &project.branch));
        run_bash(&self.restart, &filename, dir).await
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComposeActions;
impl Actions for ComposeActions {
    type R = BuildLog;
    async fn start(&mut self, dir: &FsPath, _project: &BaseProject) -> Result<Self::R> {
        let output = tokio::process::Command::new("docker")
            .arg("compose")
            .arg("up")
            .current_dir(dir)
            .output()
            .await?;
        if !output.status.success() {
            return Err(error_from_stdoutput(output)?);
        }
        BuildLog::from_output(output)
    }
    async fn stop(&mut self, dir: &FsPath, _project: &BaseProject) -> Result<Self::R> {
        let output = tokio::process::Command::new("docker")
            .arg("compose")
            .arg("stop")
            .current_dir(dir)
            .output()
            .await?;
        if !output.status.success() {
            return Err(error_from_stdoutput(output)?);
        }
        BuildLog::from_output(output)
    }
    async fn restart(&mut self, dir: &FsPath, _project: &BaseProject) -> Result<Self::R> {
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
        BuildLog::from_output(output)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DockerfileActions {
    build_id: u32,
}
impl Actions for DockerfileActions {
    type R = BuildLog;
    async fn start(&mut self, dir: &FsPath, project: &BaseProject) -> Result<Self::R> {
        self.build_id += 1;

        let build_output = tokio::process::Command::new("docker")
            .arg("build")
            .arg(".")
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
            .arg("run")
            .arg("-d")
            .arg("-t")
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
        BuildLog::from_output(build_output)
    }
    async fn stop(&mut self, dir: &FsPath, project: &BaseProject) -> Result<Self::R> {
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
        BuildLog::from_output(output)
    }
    async fn restart(&mut self, dir: &FsPath, project: &BaseProject) -> Result<Self::R> {
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
        BuildLog::from_output(output)
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
    type R = BuildLog;

    async fn start(&mut self, dir: &FsPath, project: &BaseProject) -> anyhow::Result<Self::R> {
        match self {
            ProjectKind::Dockerfile(a) => a.start(dir, project).await,
            ProjectKind::Compose(a) => a.start(dir, project).await,
            ProjectKind::Custom(a) => a.start(dir, project).await,
        }
    }

    async fn stop(&mut self, dir: &FsPath, project: &BaseProject) -> anyhow::Result<Self::R> {
        match self {
            ProjectKind::Dockerfile(a) => a.stop(dir, project).await,
            ProjectKind::Compose(a) => a.stop(dir, project).await,
            ProjectKind::Custom(a) => a.stop(dir, project).await,
        }
    }

    async fn restart(&mut self, dir: &FsPath, project: &BaseProject) -> anyhow::Result<Self::R> {
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
