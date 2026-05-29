//! Embedding providers for text vectorization

use crate::MemoryError;
use reqwest::Client;

/// Embedding level for tiered processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingLevel {
    L1Local,   // Local ONNX model (384-dim, fast, free)
    L2Cloud,   // Cloud API (1024-dim, accurate, paid)
}

/// Embedding result
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub id: String,
    pub embedding: Vec<f32>,
    pub dimension: usize,
    pub level: EmbeddingLevel,
    pub model: String,
}

/// Embedding provider trait
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for text
    fn embed(&self, text: &str) -> Result<EmbeddingResult, MemoryError>;

    /// Get embedding dimension
    fn dimension(&self) -> usize;

    /// Get provider name
    fn name(&self) -> &str;
}

/// Local ONNX embedding provider (L1)
pub struct LocalEmbeddingProvider {
    _model_path: String,
    dimension: usize,
}

impl LocalEmbeddingProvider {
    pub fn new(model_path: String) -> Result<Self, MemoryError> {
        Ok(Self {
            _model_path: model_path,
            dimension: 384, // Standard small model dimension
        })
    }

    /// Simple embedding using hash-based approach (placeholder for actual ONNX)
    fn hash_embed(&self, text: &str) -> Vec<f32> {
        // Placeholder: In production, use actual ONNX model
        // For now, use a simple hash-based approach for testing
        let mut embedding = vec![0.0f32; self.dimension];

        for (i, c) in text.chars().cycle().take(self.dimension * 10).enumerate() {
            let idx = i % self.dimension;
            embedding[idx] += (c as u32 as f32) / 255.0;
        }

        // Normalize
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        embedding
    }
}

impl EmbeddingProvider for LocalEmbeddingProvider {
    fn embed(&self, text: &str) -> Result<EmbeddingResult, MemoryError> {
        let embedding = self.hash_embed(text);

        Ok(EmbeddingResult {
            id: uuid::Uuid::new_v4().to_string(),
            embedding,
            dimension: self.dimension,
            level: EmbeddingLevel::L1Local,
            model: "local-onnx".to_string(),
        })
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn name(&self) -> &str {
        "local-embedding"
    }
}

/// Cloud API embedding provider (L2)
pub struct CloudEmbeddingProvider {
    _client: Client,
    _api_url: String,
    _api_key: String,
    dimension: usize,
    model: String,
}

impl CloudEmbeddingProvider {
    pub fn new(api_url: String, api_key: String, model: String) -> Result<Self, MemoryError> {
        let client = Client::new();

        Ok(Self {
            _client: client,
            _api_url: api_url,
            _api_key: api_key,
            dimension: 1024,
            model,
        })
    }
}

impl EmbeddingProvider for CloudEmbeddingProvider {
    fn embed(&self, text: &str) -> Result<EmbeddingResult, MemoryError> {
        // Placeholder: In production, call actual embedding API
        // For testing, use local embedding
        let local = LocalEmbeddingProvider::new("placeholder".to_string())
            .map_err(|e| MemoryError::Embedding(e.to_string()))?;

        let result = local.embed(text);

        // Adjust to indicate L2 level
        result.map(|r| EmbeddingResult {
            level: EmbeddingLevel::L2Cloud,
            model: self.model.clone(),
            ..r
        })
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn name(&self) -> &str {
        "cloud-embedding"
    }
}

/// Tiered embedding provider that chooses between L1 and L2
pub struct TieredEmbeddingProvider {
    l1: LocalEmbeddingProvider,
    l2: Option<CloudEmbeddingProvider>,
    threshold_importance: f64,
}

impl TieredEmbeddingProvider {
    pub fn new(l1_model_path: String, l2_config: Option<(String, String, String)>) -> Result<Self, MemoryError> {
        let l1 = LocalEmbeddingProvider::new(l1_model_path)?;

        let l2 = l2_config.map(|(url, key, model)| {
            CloudEmbeddingProvider::new(url, key, model).unwrap()
        });

        Ok(Self {
            l1,
            l2,
            threshold_importance: 0.7, // Use L2 for important memories
        })
    }

    /// Choose embedding level based on importance
    pub fn embed_with_level(&self, text: &str, importance: f64) -> Result<EmbeddingResult, MemoryError> {
        if importance >= self.threshold_importance && self.l2.is_some() {
            self.l2.as_ref().unwrap().embed(text)
        } else {
            self.l1.embed(text)
        }
    }
}

impl EmbeddingProvider for TieredEmbeddingProvider {
    fn embed(&self, text: &str) -> Result<EmbeddingResult, MemoryError> {
        self.l1.embed(text)
    }

    fn dimension(&self) -> usize {
        self.l1.dimension()
    }

    fn name(&self) -> &str {
        "tiered-embedding"
    }
}