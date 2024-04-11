use std::path::PathBuf;

use anyhow::Result;
use futures::executor::block_on;
use serde::{de::DeserializeOwned, Serialize};
use tokio::fs;

use crate::util::upsert_file;

#[derive(Debug, Clone)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn new(folder: PathBuf, file: PathBuf) -> Result<Self> {
        let copy = block_on(upsert_file(&folder, &file, ""))?;
        Ok(Self { path: copy })
    }

    pub fn new_str(folder: &str, file: &str) -> Result<Self> {
        Self::new(PathBuf::from(folder), PathBuf::from(file))
    }

    pub async fn write<T: Serialize>(&self, inp: &T) -> Result<()> {
        let contents = serde_json::to_string_pretty(&inp)?;
        Ok(fs::write(&self.path, contents).await?)
    }

    pub async fn read<T: DeserializeOwned>(&self) -> Result<T> {
        let read = fs::read_to_string(&self.path).await?;
        let val = serde_json::from_str(&read)?;
        Ok(val)
    }
}
