//! Hybrid search engine combining FTS and vector search

use crate::{MemoryError, MemoryProvider};
use crate::sqlite_provider::SqliteMemoryProvider;
use crate::chroma_provider::ChromaProvider;
use crate::embedding::{EmbeddingProvider, TieredEmbeddingProvider};
use serde::{Deserialize, Serialize};

/// Search result with relevance scoring
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub relevance_score: f64,
    pub source: String, // "fts", "chroma", "hybrid"
}

/// Search mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    Keyword,    // FTS5 full-text search only
    Semantic,   // Vector similarity search only
    Hybrid,     // Combine FTS + Vector + scoring
}

/// Hybrid search engine
pub struct HybridSearchEngine {
    sqlite: SqliteMemoryProvider,
    chroma: Option<ChromaProvider>,
    embedding: TieredEmbeddingProvider,
}

impl HybridSearchEngine {
    /// Create a new hybrid search engine
    pub fn new(
        sqlite: SqliteMemoryProvider,
        chroma: Option<ChromaProvider>,
        embedding: TieredEmbeddingProvider,
    ) -> Self {
        Self {
            sqlite,
            chroma,
            embedding,
        }
    }

    /// Perform hybrid search
    pub async fn search(
        &self,
        query: &str,
        mode: SearchMode,
        limit: usize,
    ) -> Result<Vec<SearchResult>, MemoryError> {
        match mode {
            SearchMode::Keyword => {
                self.sqlite.search(query, mode, limit).await
            }
            SearchMode::Semantic => {
                if let Some(chroma) = &self.chroma {
                    let embedding_result = self.embedding.embed(query)?;
                    chroma.query_similar("acp_memory_global_v1", embedding_result.embedding, limit).await
                } else {
                    Ok(vec![])
                }
            }
            SearchMode::Hybrid => {
                self.hybrid_search(query, limit).await
            }
        }
    }

    /// Hybrid search combining FTS and vector
    async fn hybrid_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, MemoryError> {
        // Get FTS results
        let fts_results = self.sqlite.search_fts(query, limit)?;

        // Get vector results if Chroma available
        let vector_results = if let Some(chroma) = &self.chroma {
            let embedding_result = self.embedding.embed(query)?;
            chroma.query_similar("acp_memory_global_v1", embedding_result.embedding, limit).await?
        } else {
            vec![]
        };

        // Combine and score results
        let combined = self.combine_results(fts_results, vector_results, limit);

        Ok(combined)
    }

    /// Combine FTS and vector results with relevance scoring
    fn combine_results(
        &self,
        fts_results: Vec<SearchResult>,
        vector_results: Vec<SearchResult>,
        limit: usize,
    ) -> Vec<SearchResult> {
        let mut combined: Vec<SearchResult> = vec![];
        let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Add FTS results with score boost
        for result in fts_results {
            if !seen_ids.contains(&result.id) {
                seen_ids.insert(result.id.clone());
                combined.push(SearchResult {
                    relevance_score: result.relevance_score * 0.6, // FTS weight
                    source: "hybrid-fts".to_string(),
                    ..result
                });
            }
        }

        // Add vector results with score boost
        for result in vector_results {
            if seen_ids.contains(&result.id) {
                // Boost score if already in FTS results (exact match)
                for existing in &mut combined {
                    if existing.id == result.id {
                        existing.relevance_score += result.relevance_score * 0.4; // Vector boost
                        existing.source = "hybrid-exact".to_string();
                    }
                }
            } else {
                seen_ids.insert(result.id.clone());
                combined.push(SearchResult {
                    relevance_score: result.relevance_score * 0.4, // Vector weight
                    source: "hybrid-vector".to_string(),
                    ..result
                });
            }
        }

        // Sort by relevance score
        combined.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Limit results
        combined.truncate(limit);

        combined
    }

    /// Search with context injection
    pub async fn search_for_injection(
        &self,
        context: &str,
        limit: usize,
        min_relevance: f64,
    ) -> Result<Vec<SearchResult>, MemoryError> {
        let results = self.search(context, SearchMode::Hybrid, limit * 2).await?;

        // Filter by minimum relevance
        let filtered = results.into_iter()
            .filter(|r| r.relevance_score >= min_relevance)
            .take(limit)
            .collect();

        Ok(filtered)
    }
}