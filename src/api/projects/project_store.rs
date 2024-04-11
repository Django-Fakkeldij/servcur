use std::sync::Arc;
use tokio::sync::RwLock;

use super::{Project, Projects};
use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct ProjectStore {
    inner: Arc<RwLock<Projects>>,
}

impl ProjectStore {
    pub async fn get(&self, name: &str, branch: &str) -> Option<Project> {
        self.inner.read().await.get(name, branch)
    }

    pub async fn insert(&self, project: Project) -> Result<()> {
        let mut store = self.inner.write().await;

        store.insert(project)
    }

    pub async fn get_all(&self) -> Projects {
        self.inner.read().await.clone()
    }
}
