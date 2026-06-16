//! In-memory storage for testing

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Generic in-memory store
pub struct InMemoryStore<T> {
    data: Arc<RwLock<HashMap<i64, T>>>,
    next_id: Arc<RwLock<i64>>,
}

impl<T> InMemoryStore<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn insert(&self, item: T) -> i64 {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;

        let mut data = self.data.write().await;
        data.insert(id, item);

        id
    }

    pub async fn get(&self, id: i64) -> Option<T>
    where
        T: Clone,
    {
        let data = self.data.read().await;
        data.get(&id).cloned()
    }

    pub async fn list(&self) -> Vec<T>
    where
        T: Clone,
    {
        let data = self.data.read().await;
        data.values().cloned().collect()
    }

    pub async fn remove(&self, id: i64) -> Option<T>
    where
        T: Clone,
    {
        let mut data = self.data.write().await;
        data.remove(&id)
    }

    pub async fn count(&self) -> usize {
        let data = self.data.read().await;
        data.len()
    }
}

impl<T> Default for InMemoryStore<T> {
    fn default() -> Self {
        Self::new()
    }
}