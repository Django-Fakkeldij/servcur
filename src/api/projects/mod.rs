use std::path::{Path as FsPath, PathBuf};
use std::process::{Output, Stdio};

use anyhow::Result;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::config::DATA_FOLDER;
use crate::util::{create_git_auth_url, error_from_stdoutput, run_bash};

use const_format::concatcp;

pub mod routes;

pub const PROJECT_FOLDER: &str = concatcp!(DATA_FOLDER, "/projects");

pub const WEBHOOK_PATH: &str = "/projects/webhook";

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

pub async fn new_project(project: &NewProject) -> anyhow::Result<PathBuf> {
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
        .arg(&create_git_auth_url(&project.https_url, &project.auth))
        .arg("-b")
        .arg(&project.branch)
        // Makes it so that it doesn't create a folder within the current work-dir
        .arg(".")
        .current_dir(&project_branch_folder)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .await?;
    if !output.success() {
        return Err(anyhow::anyhow!(output.to_string()));
    }
    Ok(PathBuf::from(project_branch_folder))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    uri: String,
    path: PathBuf,
    project_name: String,
    branch: String,
    project_kind: ProjectKind,
    history: ProjectBuildHistory,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProjectBuildHistory {
    inner: Vec<ProjectBuild>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProjectBuild {
    status: usize,
    date_unix_s: usize,
}

pub fn format_webhook_url(name: &str, branch: &str, absolute: bool) -> String {
    if absolute {
        return format!("{WEBHOOK_PATH}/{name}/{branch}");
    }
    format!("{name}/{branch}")
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

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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
    action: ProjectActions,
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
