use std::{
    collections::BTreeMap,
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
    sync::{broadcast, mpsc, Mutex, RwLock},
    task::JoinHandle,
    time::Instant,
};
use tracing::{debug, info, info_span, instrument, trace, warn, Instrument};

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

#[derive(Debug, Clone)]
struct OutputSendHandle {
    stdout: broadcast::Sender<String>,
    stderr: broadcast::Sender<String>,
}

#[derive(Debug)]
pub struct OutputHandle {
    pub stdout: broadcast::Receiver<String>,
    pub stderr: broadcast::Receiver<String>,
}

impl OutputHandle {
    /// returns stdout, stderr, self
    fn new() -> (OutputSendHandle, Self) {
        let (stdout_sender, stdout_recv) = broadcast::channel(128);
        let (stderr_sender, stderr_recv) = broadcast::channel(128);

        (
            OutputSendHandle {
                stdout: stdout_sender,
                stderr: stderr_sender,
            },
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
    build_id_counter: Arc<Mutex<usize>>,
    exec_tx: Arc<mpsc::Sender<(usize, ProjectIoHandle)>>,
    output_handles: Arc<RwLock<BTreeMap<usize, OutputHandle>>>,
    _exec_handle: JoinHandle<()>,
}

impl ProjectIoExecutor {
    pub fn new(size: usize) -> Self {
        let (tx, mut rx) = mpsc::channel::<(usize, ProjectIoHandle)>(size);

        let output_handles = Arc::new(RwLock::new(BTreeMap::new()));

        let output_handles_clone = output_handles.clone();
        let _exec_handle = tokio::spawn(
            async move {
                debug!("started executor");
                loop {
                    let (id, handle) = if let Some((i, v)) = rx.recv().await {
                        debug!(?v.project, "received io_handle");
                        (i, v)
                    } else {
                        warn!("emtpy recv");
                        continue;
                    };
                    let (output_sender, output) = OutputHandle::new();
                    let output_handles_clone_clone = output_handles_clone.clone();
                    tokio::spawn(async move {
                        output_handles_clone_clone.write().await.insert(id, output);
                        let _ = execute_handle_manager(handle, output_sender).await;
                        output_handles_clone_clone.write().await.remove(&id);
                    });
                }
            }
            .instrument(info_span!("ProjectIoExecutor")),
        );

        Self {
            build_id_counter: Arc::new(Mutex::new(0)),
            _exec_handle,
            exec_tx: Arc::new(tx),
            output_handles,
        }
    }

    pub async fn exec(&self, handle: ProjectIoHandle) -> Result<usize> {
        let id = {
            let mut id = self.build_id_counter.lock().await;
            *id += 1;
            *id
        };
        self.exec_tx
            .send((id, handle))
            .await
            .context("could not send handle to executor")?;
        Ok(id)
    }

    pub async fn get_handles(
        &self,
    ) -> tokio::sync::RwLockReadGuard<'_, BTreeMap<usize, OutputHandle>> {
        self.output_handles.read().await
    }

    pub async fn get_handle_by_id(&self, id: usize) -> Option<OutputHandle> {
        self.output_handles.read().await.get(&id).cloned()
    }
}

#[async_recursion]
async fn execute_handle(
    mut handle: ProjectIoHandle,
    output_handle: OutputSendHandle,
) -> Result<()> {
    if let Some(child) = handle.depends_on {
        execute_handle(*child, output_handle.clone()).await?;
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
                if let Err(e) = output_handle.stdout.send(l) {
                    trace!(?e, "stdout sending error");
                };
            }
        });
    }

    let _guard2;
    if let Some(v) = command_handle.stderr.take() {
        let reader = BufReader::new(v);
        _guard2 = tokio::spawn(async move {
            let mut lines = reader.lines();
            while let Ok(Some(l)) = lines.next_line().await {
                if let Err(e) = output_handle.stderr.send(l) {
                    trace!(?e, "stdout sending error");
                };
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
async fn execute_handle_manager(
    handle: ProjectIoHandle,
    output_handle: OutputSendHandle,
) -> Result<()> {
    let t0 = Instant::now();
    info!("started IoHandle");
    let ret = execute_handle(handle, output_handle).await;
    let d = t0.elapsed();
    info!(duration=?d, "finished IoHandle");
    ret
}
