use std::path::PathBuf;
use std::process::Stdio;

use tokio::fs;

use crate::util::create_git_auth_url;
use crate::util::format_project_folder;
use crate::util::format_project_root_folder;

use super::NewProject;
use super::PROJECT_FOLDER;

pub async fn new_project(project: &NewProject) -> anyhow::Result<PathBuf> {
    // Name invalid
    if project.name.contains('/') || project.name.contains('\\') {
        return Err(anyhow::Error::msg("invalid project name"));
    }

    // Invalid url
    if !project.https_url.starts_with("https://") {
        return Err(anyhow::Error::msg("not an https git url"));
    }

    let project_root_folder = format_project_root_folder(&project.name);
    let project_branch_folder = format_project_folder(&project.name, &project.branch);
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

pub async fn remove_project(name: &str, branch: &str) -> anyhow::Result<()> {
    fs::remove_dir_all(format_project_folder(name, branch)).await?;
    Ok(())
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
