use std::path::Path as FsPath;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::api::projects::{executor::ProjectIoHandle, BaseProject};

use super::Action;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockerFileActions {
    Build,
    Start,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockerFile {
    image_version: usize,
}

impl Action for DockerFile {
    type W = DockerFileActions;
    async fn exec(
        &mut self,
        which: &Self::W,
        dir: &FsPath,
        project: &BaseProject,
    ) -> anyhow::Result<ProjectIoHandle> {
        match which {
            DockerFileActions::Build => {
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

                let out = ProjectIoHandle::new(project.clone(), build_command);
                Ok(out)
            }
            DockerFileActions::Start => {
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

                Ok(ProjectIoHandle::new(project.clone(), start_command))
            }
            DockerFileActions::Stop => {
                let mut command = Command::new("docker");
                command
                    .arg("stop")
                    .arg(format!(
                        "{}-{}-{}",
                        project.name, project.branch, self.image_version
                    ))
                    .current_dir(dir);

                Ok(ProjectIoHandle::new(project.clone(), command))
            }
        }
    }
}
