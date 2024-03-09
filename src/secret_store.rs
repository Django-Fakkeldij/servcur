use std::{fs::create_dir_all, path::PathBuf};

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct SecretStore {
    path: PathBuf,
}

impl SecretStore {
    pub fn new(path: PathBuf) -> Result<Self> {
        create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub async fn set(&self, inp: &impl Serialize) -> Result<()> {
        let contents = serde_json::to_string_pretty(inp)?;
        Ok(fs::write(&self.path, contents).await?)
    }

    pub async fn get<'a, T: DeserializeOwned>(&self) -> Result<T> {
        let read = fs::read_to_string(&self.path).await?;
        let val: T = serde_json::from_str(&read)?;
        Ok(val)
    }
}

impl Default for SecretStore {
    fn default() -> Self {
        SecretStore::new(PathBuf::from("./store/secrets.json")).unwrap()
    }
}
