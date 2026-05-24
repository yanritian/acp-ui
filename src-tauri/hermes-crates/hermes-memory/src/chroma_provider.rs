//! Chroma vector database provider for semantic search

use crate::{MemoryError, SearchResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Chroma collection for storing embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaCollection {
    pub name: String,
    pub metadata: HashMap<String, String>,
}

/// Chroma embedding record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromaEmbedding {
    pub id: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub document: Option<String>,
}

/// Chroma provider for vector storage
pub struct ChromaProvider {
    client: Client,
    base_url: String,
    collections: Vec<String>,
}

impl ChromaProvider {
    /// Create a new Chroma provider
    pub fn new(base_url: String) -> Result<Self, MemoryError> {
        let client = Client::new();

        Ok(Self {
            client,
            base_url,
            collections: vec![
                "acp_memory_global_v1".to_string(),
                "acp_memory_agent_v1".to_string(),
                "acp_memory_session_v1".to_string(),
                "acp_memory_task_v1".to_string(),
            ],
        })
    }

    /// Create collection if not exists
    pub async fn ensure_collections(&self) -> Result<(), MemoryError> {
        for collection_name in &self.collections {
            let url = format!("{}/collections", self.base_url);
            let body = serde_json::json!({
                "name": collection_name,
                "metadata": {
                    "description": format!("Memory collection for {}", collection_name),
                }
            });

            let response = self.client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| MemoryError::Connection(e.to_string()))?;

            // Collection may already exist, that's fine
            if !response.status().is_success() && response.status() != 409 {
                return Err(MemoryError::Chroma(format!(
                    "Failed to create collection {}: {}",
                    collection_name,
                    response.status()
                )));
            }
        }
        Ok(())
    }

    /// Add embedding to collection
    pub async fn add_embedding(
        &self,
        collection: &str,
        id: &str,
        embedding: Vec<f32>,
        document: &str,
        metadata: HashMap<String, String>,
    ) -> Result<(), MemoryError> {
        let url = format!("{}/collections/{}/add", self.base_url, collection);

        let body = serde_json::json!({
            "ids": [id],
            "embeddings": [embedding],
            "documents": [document],
            "metadatas": [metadata],
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| MemoryError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MemoryError::Chroma(format!(
                "Failed to add embedding: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Query similar embeddings
    pub async fn query_similar(
        &self,
        collection: &str,
        query_embedding: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, MemoryError> {
        let url = format!("{}/collections/{}/query", self.base_url, collection);

        let body = serde_json::json!({
            "query_embeddings": [query_embedding],
            "n_results": limit,
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| MemoryError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MemoryError::Chroma(format!(
                "Failed to query: {}",
                response.status()
            )));
        }

        let result: ChromaQueryResult = response
            .json()
            .await
            .map_err(|e| MemoryError::Chroma(e.to_string()))?;

        // Convert to SearchResult - handle nested arrays from Chroma API
        let results: Vec<SearchResult> = if result.ids.is_empty() || result.ids[0].is_empty() {
            vec![]
        } else {
            result.ids[0].iter()
                .zip(result.documents[0].iter())
                .zip(result.distances[0].iter())
                .map(|((id, doc), dist)| SearchResult {
                    id: id.clone(),
                    content: doc.clone().unwrap_or_default(),
                    relevance_score: 1.0 - *dist, // Convert distance to similarity
                    source: "chroma".to_string(),
                })
                .collect()
        };

        Ok(results)
    }

    /// Delete embedding by ID
    pub async fn delete_embedding(&self, collection: &str, id: &str) -> Result<(), MemoryError> {
        let url = format!("{}/collections/{}/delete", self.base_url, collection);

        let body = serde_json::json!({
            "ids": [id],
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| MemoryError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MemoryError::Chroma(format!(
                "Failed to delete: {}",
                response.status()
            )));
        }

        Ok(())
    }
}

/// Chroma query result
#[derive(Debug, Deserialize)]
struct ChromaQueryResult {
    ids: Vec<Vec<String>>,
    documents: Vec<Vec<Option<String>>>,
    distances: Vec<Vec<f64>>,
}