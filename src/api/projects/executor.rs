use std::{
    path::{Path, PathBuf},
    process::Output,
    sync::Arc,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};
use async_recursion::async_recursion;
use tokio::{process::Command, sync::mpsc, task::JoinHandle, time::Instant};
use tracing::{debug, error, info, info_span, warn, Instrument};

use crate::{config::BUILD_LOG_FOLDER, util::upsert_file};

use super::BaseProject;

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
    pub command: Command,
    pub depends_on: Option<Box<ProjectIoHandle>>,
}

impl ProjectIoHandle {
    pub fn new(project: BaseProject, command: Command) -> Self {
        Self {
            project,
            command,
            depends_on: None,
        }
    }

    pub fn depends_on(mut self, depends_on: ProjectIoHandle) -> Self {
        self.depends_on = Some(Box::new(depends_on));
        self
    }

    pub fn depends_on_same(mut self, depends_on: Command) -> Self {
        let self_clone = Self::new(self.project.clone(), depends_on);
        self.depends_on = Some(Box::new(self_clone));
        self
    }
}

#[derive(Debug)]
pub struct ProjectIoExecutor {
    exec_tx: Arc<mpsc::Sender<ProjectIoHandle>>,
    _exec_handle: JoinHandle<()>,
}

impl ProjectIoExecutor {
    pub fn new(size: usize) -> Self {
        let (tx, mut rx) = mpsc::channel::<ProjectIoHandle>(size);

        let _exec_handle = tokio::spawn(
            async move {
                debug!("started executor");
                loop {
                    let handle = if let Some(v) = rx.recv().await {
                        debug!(?v.project, "received io_handle");
                        v
                    } else {
                        warn!("emtpy recv");
                        continue;
                    };
                    let project = handle.project.clone();
                    tokio::spawn(
                        async move {
                            info!("started executing");
                            let t = Instant::now();
                            if let Err(e) = execute_handle(handle).await {
                                error!(?e);
                            }
                            let duration = t.elapsed().as_millis();
                            info!(duration_milisecs = duration, "finished executing")
                        }
                        .instrument(info_span!("IoHandleExecute", ?project)),
                    );
                }
            }
            .instrument(info_span!("ProjectIoExecutor")),
        );

        Self {
            _exec_handle,
            exec_tx: Arc::new(tx),
        }
    }

    pub async fn exec(&self, handle: ProjectIoHandle) -> Result<()> {
        self.exec_tx
            .clone()
            .send(handle)
            .await
            .map_err(|e| e.into())
    }
}

#[async_recursion]
async fn execute_handle(mut handle: ProjectIoHandle) -> Result<()> {
    if let Some(child) = handle.depends_on {
        debug!("executing child started");
        execute_handle(*child).await?;
        debug!("executing child finished");
    }

    let out = handle
        .command
        .output()
        .await
        .context("error while executing command")?;
    let v = IoLog::from_output(out).context("error while converting to log")?;
    let time_str = Utc::now().timestamp();
    let filename = format!(
        "{}-{}_{}.json",
        handle.project.name, handle.project.branch, time_str
    );
    v.direct_to_file(&PathBuf::from(BUILD_LOG_FOLDER), &PathBuf::from(filename))
        .await
        .context("error while writing to file")?;

    Ok(())
}
