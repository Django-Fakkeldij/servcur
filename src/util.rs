use std::{
    path::{Path, PathBuf},
    process::Output,
};

use tokio::{
    fs::{create_dir_all, File},
    io::AsyncWriteExt,
};

use crate::{
    api::projects::{BuildLog, GitAuth},
    config::{TEMP_SCRIPT_FOLDER, WEBHOOK_URL_PATH},
};

pub async fn upsert_file(folder: &Path, file: &Path, default: &str) -> anyhow::Result<PathBuf> {
    create_dir_all(&folder).await?;
    let mut copy: PathBuf = folder.into();
    copy.push(file);
    if !copy.exists() {
        let mut f = File::create(&copy).await?;
        f.write_all(default.as_bytes()).await?;
    }
    Ok(copy)
}

pub fn create_git_auth_url(https_url: &str, auth: &GitAuth) -> String {
    if auth.is_none() {
        return https_url.to_owned();
    }

    // Prepare url based on auth used
    let url_end = https_url.replacen("https://", "", 1);
    let url_auth: &str = match auth {
        GitAuth::Token(t) => t,
        _ => panic!(),
    };

    let url = format!("https://{url_auth}@{url_end}");

    url
}

pub fn format_webhook_url(name: &str, branch: &str, absolute: bool) -> String {
    if absolute {
        return format!("{WEBHOOK_URL_PATH}/{name}/{branch}");
    }
    format!("{name}/{branch}")
}

pub fn error_from_stdoutput(output: Output) -> anyhow::Result<anyhow::Error> {
    Err(anyhow::anyhow!(
        "{}\n\nSTDOUT \n\n{}\n\nSTDERR\n\n{}",
        output.status,
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?
    ))
}

pub async fn run_bash(script: &str, filename: &Path, workdir: &Path) -> anyhow::Result<BuildLog> {
    let folder = TEMP_SCRIPT_FOLDER;
    let file_path = upsert_file(&PathBuf::from(&folder), &PathBuf::from(&filename), script).await?;

    // Clone git repo (with a insecure remote, :0 )
    let output = tokio::process::Command::new("/bin/bash")
        .arg(&file_path)
        .current_dir(workdir)
        .output()
        .await?;
    if !output.status.success() {
        return Err(error_from_stdoutput(output)?);
    }
    BuildLog::from_output(output)
}
