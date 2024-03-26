use std::{
    path::{Path, PathBuf},
    process::{Output, Stdio},
    sync::Arc,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};
use async_recursion::async_recursion;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{broadcast, mpsc, RwLock},
    task::JoinHandle,
    time::Instant,
};
use tracing::{debug, info, info_span, instrument, warn, Instrument};

use crate::{
    config::BUILD_LOG_FOLDER,
    util::{format_time_iso8601, upsert_file},
};

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
    pub tag: Option<String>,
    pub command: Command,
    pub depends_on: Option<Box<ProjectIoHandle>>,
}

impl ProjectIoHandle {
    pub fn new(project: BaseProject, command: Command) -> Self {
        Self {
            project,
            command,
            tag: None,
            depends_on: None,
        }
    }

    pub fn with_tag(mut self, t: String) -> Self {
        self.tag = Some(t);
        self
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

    pub fn depends_on_same_tagged(mut self, depends_on: Command, tag: String) -> Self {
        let self_clone = Self::new(self.project.clone(), depends_on).with_tag(tag);
        self.depends_on = Some(Box::new(self_clone));
        self
    }
}

#[derive(Debug)]
pub struct OutputHandle {
    stdout: broadcast::Receiver<String>,
    stderr: broadcast::Receiver<String>,
}

impl OutputHandle {
    /// returns stdout, stderr, self
    pub fn new() -> (broadcast::Sender<String>, broadcast::Sender<String>, Self) {
        let (stdout_sender, stdout_recv) = broadcast::channel(128);
        let (stderr_sender, stderr_recv) = broadcast::channel(128);

        (
            stdout_sender,
            stderr_sender,
            Self {
                stdout: stdout_recv,
                stderr: stderr_recv,
            },
        )
    }
}

impl Clone for OutputHandle {
    fn clone(&self) -> Self {
        Self {
            stdout: self.stdout.resubscribe(),
            stderr: self.stderr.resubscribe(),
        }
    }
}

#[derive(Debug)]
pub struct ProjectIoExecutor {
    exec_tx: Arc<mpsc::Sender<ProjectIoHandle>>,
    output_handles: Arc<RwLock<OutputHandle>>,
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
                    tokio::spawn(async move {
                        info!("started executing");
                        let t = Instant::now();
                        let _ = traced_execute_handle(handle).await;
                        let duration = t.elapsed().as_millis();
                        info!(duration_milisecs = duration, "finished executing")
                    });
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
        self.exec_tx.send(handle).await.map_err(|e| e.into())
    }
}

#[async_recursion]
async fn execute_handle(mut handle: ProjectIoHandle) -> Result<()> {
    if let Some(child) = handle.depends_on {
        execute_handle(*child).await?;
    }

    let mut command_handle = handle
        .command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawning child process failed")?;

    let _guard1;
    if let Some(v) = command_handle.stdout.take() {
        let reader = BufReader::new(v);
        _guard1 = tokio::spawn(async move {
            let mut lines = reader.lines();
            while let Ok(Some(l)) = lines.next_line().await {
                println!("stdout reader: {}", l);
            }
        });
    }

    let _guard2;
    if let Some(v) = command_handle.stderr.take() {
        let reader = BufReader::new(v);
        _guard2 = tokio::spawn(async move {
            let mut lines = reader.lines();
            while let Ok(Some(l)) = lines.next_line().await {
                println!("stderr reader: {}", l);
            }
        });
    }

    let out = command_handle
        .wait_with_output()
        .await
        .context("error while executing command")?;
    let v = IoLog::from_output(out).context("error while converting to log")?;
    let time_str = format_time_iso8601(Utc::now());
    let filename = match handle.tag {
        Some(v) => format!(
            "{}-{}-{}-{}.json",
            handle.project.name, handle.project.branch, v, time_str
        ),
        None => format!(
            "{}-{}-{}.json",
            handle.project.name, handle.project.branch, time_str
        ),
    };
    v.direct_to_file(&PathBuf::from(BUILD_LOG_FOLDER), &PathBuf::from(filename))
        .await
        .context("error while writing to file")?;

    Ok(())
}

#[instrument(err, name = "IoHandleExecute", level = "info")]
#[allow(clippy::blocks_in_conditions)]
async fn traced_execute_handle(handle: ProjectIoHandle) -> Result<()> {
    let t0 = Instant::now();
    info!("started IoHandle");
    let ret = execute_handle(handle).await;
    let d = t0.elapsed();
    info!(duration=?d, "finished IoHandle");
    ret
}
