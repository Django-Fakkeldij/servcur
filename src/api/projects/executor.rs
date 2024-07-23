use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Output, Stdio},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};
use async_recursion::async_recursion;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{
        broadcast::{self},
        mpsc, RwLock,
    },
    task::JoinHandle,
    time::Instant,
};
use tracing::{debug, error, info, info_span, instrument, trace, warn, Instrument};
use ulid::Ulid;

use crate::{config::IO_LOG_FOLDER, util::upsert_file};

use super::BaseProject;

#[derive(Debug, Deserialize, Serialize)]
pub struct IoLog {
    pub status: usize,
    pub project: BaseProject,
    pub tag: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub child: Option<Box<Self>>,
}

impl IoLog {
    pub fn new(
        status: usize,
        project: BaseProject,
        tag: Option<String>,
        stdout: String,
        stderr: String,
    ) -> Self {
        Self {
            status,
            project,
            tag,
            stdout,
            stderr,
            child: None,
        }
    }

    pub fn from_output(project: BaseProject, tag: Option<String>, output: Output) -> Result<Self> {
        Ok(Self {
            status: output.status.code().unwrap_or(1) as usize,
            project,
            tag,
            stdout: String::from_utf8(output.stdout)?,
            stderr: String::from_utf8(output.stderr)?,
            child: None,
        })
    }

    pub async fn direct_to_file(&self, folder: &Path, file: &Path) -> Result<PathBuf> {
        upsert_file(folder, file, &serde_json::to_string_pretty(&self)?).await
    }

    pub fn set_child(mut self, child: Box<IoLog>) -> Self {
        self.child = Some(child);
        self
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

pub type IoHandleID = Ulid;

#[derive(Debug)]
pub struct ProjectIoExecutor {
    exec_tx: Arc<mpsc::Sender<(IoHandleID, ProjectIoHandle)>>,
    output_handles: Arc<RwLock<BTreeMap<IoHandleID, (OutputHandle, BaseProject)>>>,
    _exec_handle: JoinHandle<()>,
}

impl ProjectIoExecutor {
    pub fn new(size: usize) -> Self {
        let (tx, mut rx) = mpsc::channel::<(IoHandleID, ProjectIoHandle)>(size);

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
                        output_handles_clone_clone
                            .write()
                            .await
                            .insert(id, (output, handle.project.clone()));
                        let _ = execute_handle_manager(id, handle, output_sender).await;
                        output_handles_clone_clone.write().await.remove(&id);
                    });
                }
            }
            .instrument(info_span!("ProjectIoExecutor")),
        );

        Self {
            _exec_handle,
            exec_tx: Arc::new(tx),
            output_handles,
        }
    }

    pub async fn exec(&self, handle: ProjectIoHandle) -> Result<IoHandleID> {
        let id = Ulid::new();
        self.exec_tx
            .send((id, handle))
            .await
            .context("could not send handle to executor")?;
        Ok(id)
    }

    pub async fn get_handles(
        &self,
    ) -> tokio::sync::RwLockReadGuard<'_, BTreeMap<IoHandleID, (OutputHandle, BaseProject)>> {
        self.output_handles.read().await
    }

    pub async fn get_handle_by_id(&self, id: IoHandleID) -> Option<(OutputHandle, BaseProject)> {
        self.output_handles.read().await.get(&id).cloned()
    }
}

#[async_recursion]
async fn execute_handle(
    mut handle: ProjectIoHandle,
    output_handle: OutputSendHandle,
) -> Result<Box<IoLog>> {
    let mut child = None;
    if let Some(child_handle) = handle.depends_on {
        child = Some(execute_handle(*child_handle, output_handle.clone()).await?);
    }

    let mut command_handle = handle
        .command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawning child process failed")?;

    let mut stdout_collecter = None;
    if let Some(v) = command_handle.stdout.take() {
        let reader = BufReader::new(v);
        stdout_collecter = Some(tokio::spawn(async move {
            let mut total = String::new();
            let mut lines = reader.lines();
            while let Ok(Some(l)) = lines.next_line().await {
                total.push_str(&l);
                total.push('\n');
                if let Err(e) = output_handle.stdout.send(l) {
                    trace!(?e, "stdout sending error");
                };
            }
            total
        }));
    }

    let mut stderr_collecter = None;
    if let Some(v) = command_handle.stderr.take() {
        let reader = BufReader::new(v);
        stderr_collecter = Some(tokio::spawn(async move {
            let mut total = String::new();
            let mut lines = reader.lines();
            while let Ok(Some(l)) = lines.next_line().await {
                total.push_str(&l);
                total.push('\n');
                if let Err(e) = output_handle.stderr.send(l) {
                    trace!(?e, "stdout sending error");
                };
            }
            total
        }));
    }

    // Does not have any lines left because all are read by the streamers
    let out = command_handle
        .wait_with_output()
        .await
        .context("error while executing command")?;

    let mut stdout = String::new();
    let mut stderr = String::new();

    if let Some(v) = stdout_collecter {
        match tokio::try_join!(v) {
            Ok((s,)) => {
                stdout = s;
            }
            Err(e) => {
                error!(%e, "stdout join error")
            }
        };
    }

    if let Some(v) = stderr_collecter {
        match tokio::try_join!(v) {
            Ok((s,)) => {
                stderr = s;
            }
            Err(e) => {
                error!(%e, "stdout join error")
            }
        };
    }

    let mut io = IoLog::new(
        out.status.code().unwrap_or(0) as usize,
        handle.project,
        handle.tag,
        stdout,
        stderr,
    );

    if let Some(v) = child {
        io = io.set_child(v);
    }
    Ok(Box::new(io))
}

#[instrument(err(Debug), name = "IoHandleExecute", level = "info")]
#[allow(clippy::blocks_in_conditions)]
async fn execute_handle_manager(
    id: IoHandleID,
    handle: ProjectIoHandle,
    output_handle: OutputSendHandle,
) -> Result<Box<IoLog>> {
    info!("started IoHandle");
    let t0 = Instant::now();
    // execute
    let ret = execute_handle(handle, output_handle).await;
    // write to file
    let filename = format!("{id}.json");
    if let Ok(v) = &ret {
        v.direct_to_file(&PathBuf::from(IO_LOG_FOLDER), &PathBuf::from(filename))
            .await
            .context("error while writing to file")?;
    }
    let d = t0.elapsed();
    info!(duration=?d, "finished IoHandle");
    ret
}
