use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, RwLockWriteGuard};

use crate::store::Store;

use super::{BaseProject, Project, Projects};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ProjectStore {
    inner: Arc<RwLock<Projects>>,
    fs_store: Arc<Mutex<Store>>,
}

impl ProjectStore {
    pub async fn new(store: Store) -> Self {
        let mut content = Projects::default();
        if let Ok(store_content) = store.read::<Vec<Project>>().await {
            content.0 = store_content;
        };
        Self {
            fs_store: Arc::new(Mutex::new(store)),
            inner: Arc::new(RwLock::new(content)),
        }
    }

    pub async fn get_owned(&self, name: &str, branch: &str) -> Option<Project> {
        self.inner.read().await.get_owned(name, branch)
    }

    pub async fn get_mut(&mut self) -> RwLockWriteGuard<Projects> {
        self.inner.write().await
    }

    pub async fn insert(&self, project: Project) -> Result<()> {
        let mut store = self.inner.write().await;

        store.insert(project)?;
        self.fs_store.lock().await.write(&store.0).await?;
        Ok(())
    }

    pub async fn remove(&self, project: &BaseProject) -> Result<()> {
        let mut store = self.inner.write().await;

        store.remove(project)?;
        self.fs_store.lock().await.write(&store.0).await?;

        Ok(())
    }

    pub async fn get_all(&self) -> Projects {
        self.inner.read().await.clone()
    }
}
