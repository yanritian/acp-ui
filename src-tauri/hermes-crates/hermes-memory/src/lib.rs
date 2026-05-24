//! # hermes-memory
//!
//! Hybrid memory storage system combining SQLite for structured data
//! and Chroma for vector semantic search.

pub mod sqlite_provider;
pub mod chroma_provider;
pub mod embedding;
pub mod hybrid_search;
pub mod memory_injection;

// Re-export main types
pub use sqlite_provider::{SqliteMemoryProvider, MemoryRecord, MemoryScope};
pub use chroma_provider::{ChromaProvider, ChromaCollection};
pub use embedding::{EmbeddingProvider, EmbeddingLevel, EmbeddingResult};
pub use hybrid_search::{HybridSearchEngine, SearchResult, SearchMode};
pub use memory_injection::{MemoryInjector, InjectionTiming, InjectionContext};

use async_trait::async_trait;
use thiserror::Error;

/// Memory provider trait for storage abstraction
#[async_trait]
pub trait MemoryProvider: Send + Sync {
    /// Store a memory record
    async fn store(&self, record: &MemoryRecord) -> Result<String, MemoryError>;

    /// Retrieve memories by scope
    async fn retrieve_by_scope(&self, scope: MemoryScope, limit: usize) -> Result<Vec<MemoryRecord>, MemoryError>;

    /// Search memories by content (keyword or semantic)
    async fn search(&self, query: &str, mode: SearchMode, limit: usize) -> Result<Vec<SearchResult>, MemoryError>;

    /// Delete memories by ID or scope
    async fn delete(&self, id: &str) -> Result<(), MemoryError>;

    /// Update memory access count and last_accessed
    async fn touch(&self, id: &str) -> Result<(), MemoryError>;
}

/// Memory errors
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("SQLite error: {0}")]
    Sqlite(String),

    #[error("Chroma error: {0}")]
    Chroma(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("Memory not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Connection error: {0}")]
    Connection(String),
}