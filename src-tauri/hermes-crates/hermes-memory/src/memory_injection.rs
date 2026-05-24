//! Memory injection system for LLM context enhancement

use crate::{MemoryError, SearchResult};
use crate::hybrid_search::HybridSearchEngine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Injection timing points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjectionTiming {
    PreTask,    // Before task execution starts
    PreTool,    // Before tool execution
    PreTurn,    // Before each LLM turn
    PreCompress, // Before context compression
}

impl InjectionTiming {
    pub fn as_str(&self) -> &'static str {
        match self {
            InjectionTiming::PreTask => "pre-task",
            InjectionTiming::PreTool => "pre-tool",
            InjectionTiming::PreTurn => "pre-turn",
            InjectionTiming::PreCompress => "pre-compress",
        }
    }
}

/// Injection context containing task and agent information
#[derive(Debug, Clone)]
pub struct InjectionContext {
    pub task_id: String,
    pub agent_id: String,
    pub session_id: String,
    pub task_type: String,
    pub task_description: String,
    pub timing: InjectionTiming,
    pub context_length: usize,
    pub max_tokens_for_injection: usize,
}

/// Memory injector for context enhancement
pub struct MemoryInjector {
    search_engine: HybridSearchEngine,
    injection_threshold: f64,
    max_injection_tokens: usize,
}

impl MemoryInjector {
    /// Create a new memory injector
    pub fn new(search_engine: HybridSearchEngine) -> Self {
        Self {
            search_engine,
            injection_threshold: 0.5,
            max_injection_tokens: 4096,
        }
    }

    /// Retrieve relevant memories for injection
    pub async fn retrieve_relevant_memories(
        &self,
        context: &InjectionContext,
        limit: usize,
    ) -> Result<Vec<SearchResult>, MemoryError> {
        let query = format!(
            "{} {}",
            context.task_type,
            context.task_description
        );

        self.search_engine.search_for_injection(
            &query,
            limit,
            self.injection_threshold,
        ).await
    }

    /// Format memories for injection into LLM context
    pub fn format_for_injection(
        &self,
        memories: Vec<SearchResult>,
        timing: InjectionTiming,
    ) -> String {
        if memories.is_empty() {
            return String::new();
        }

        let header = match timing {
            InjectionTiming::PreTask => "## Relevant Previous Experiences",
            InjectionTiming::PreTool => "## Tool Usage Patterns",
            InjectionTiming::PreTurn => "## Contextual Memory",
            InjectionTiming::PreCompress => "## Key Memories Summary",
        };

        let mut formatted = format!("{}\n\n", header);

        for (i, memory) in memories.iter().enumerate() {
            formatted.push_str(&format!(
                "{}. [Relevance: {:.2}] {}\n",
                i + 1,
                memory.relevance_score,
                memory.content
            ));
        }

        formatted.push_str("\n---\n");
        formatted
    }

    /// Inject memories into context if space available
    pub async fn inject_into_context(
        &self,
        context: &InjectionContext,
        current_context: &str,
    ) -> Result<String, MemoryError> {
        // Check if there's space for injection
        let current_tokens = current_context.len() / 4; // Rough estimate
        let available_tokens = context.max_tokens_for_injection.saturating_sub(current_tokens);

        if available_tokens < 100 {
            return Ok(current_context.to_string()); // No space for injection
        }

        // Calculate how many memories we can fit
        let estimated_memory_tokens = 200; // Average memory size
        let max_memories = available_tokens / estimated_memory_tokens;
        let limit = std::cmp::min(max_memories, 10);

        // Retrieve relevant memories
        let memories = self.retrieve_relevant_memories(context, limit).await?;

        if memories.is_empty() {
            return Ok(current_context.to_string());
        }

        // Format memories
        let injection_text = self.format_for_injection(memories, context.timing);

        // Inject into context based on timing
        let injected = match context.timing {
            InjectionTiming::PreTask => {
                format!("{}\n\n{}", injection_text, current_context)
            }
            InjectionTiming::PreTool => {
                format!("{}\n\n{}", current_context, injection_text)
            }
            InjectionTiming::PreTurn => {
                // Insert before last message
                format!("{}\n\n{}", injection_text, current_context)
            }
            InjectionTiming::PreCompress => {
                // Add summary at end
                format!("{}\n\n{}", current_context, injection_text)
            }
        };

        Ok(injected)
    }

    /// Calculate injection token budget
    pub fn calculate_budget(&self, context_length: usize, max_context: usize) -> usize {
        let remaining = max_context.saturating_sub(context_length);
        let injection_budget = remaining / 10; // 10% of remaining space for injection
        std::cmp::min(injection_budget, self.max_injection_tokens)
    }
}

/// Memory injection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionStats {
    pub timing: InjectionTiming,
    pub memories_injected: usize,
    pub tokens_used: usize,
    pub average_relevance: f64,
    pub injection_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}