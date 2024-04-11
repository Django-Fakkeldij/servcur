use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::store::Store;

use super::{Project, Projects};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ProjectStore {
    inner: Arc<RwLock<Projects>>,
    store: Arc<Mutex<Store>>,
}

impl ProjectStore {
    pub async fn new(store: Store) -> Self {
        let mut content = Projects::default();
        if let Ok(store_content) = store.read::<Vec<Project>>().await {
            content.0 = store_content;
        };
        Self {
            store: Arc::new(Mutex::new(store)),
            inner: Arc::new(RwLock::new(content)),
        }
    }

    pub async fn get(&self, name: &str, branch: &str) -> Option<Project> {
        self.inner.read().await.get(name, branch)
    }

    pub async fn insert(&self, project: Project) -> Result<()> {
        let mut store = self.inner.write().await;

        store.insert(project)?;
        self.store.lock().await.write(&store.0).await?;

        Ok(())
    }

    pub async fn get_all(&self) -> Projects {
        self.inner.read().await.clone()
    }
}
