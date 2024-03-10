use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use const_format::concatcp;
use futures::executor::block_on;
use serde_json::Value;
use tokio::fs;

use crate::{config::DATA_FOLDER, util::upsert_file};

const STORE_LOCATION: &str = concatcp!(DATA_FOLDER, "/store/");
const STORE_FILE: &str = "store.json";

#[derive(Debug, Clone)]
pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn new(folder: PathBuf, file: PathBuf) -> Result<Self> {
        let copy = block_on(upsert_file(&folder, &file, "{}"))?;
        Ok(Self { path: copy })
    }

    pub async fn write(&self, inp: HashMap<String, Value>) -> Result<()> {
        let contents = serde_json::to_string_pretty(&inp)?;
        Ok(fs::write(&self.path, contents).await?)
    }

    pub async fn read(&self) -> Result<HashMap<String, Value>> {
        let read = fs::read_to_string(&self.path).await?;
        let val = serde_json::from_str(&read)?;
        Ok(val)
    }

    pub async fn insert(&self, key: &str, val: Value) -> Result<()> {
        let mut read = self.read().await?;
        read.insert(key.to_owned(), val);
        self.write(read).await?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<Value>> {
        let read = self.read().await?;
        Ok(read.get(key).cloned())
    }
}

impl Default for Store {
    fn default() -> Self {
        Store::new(PathBuf::from(STORE_LOCATION), PathBuf::from(STORE_FILE)).unwrap()
    }
}
