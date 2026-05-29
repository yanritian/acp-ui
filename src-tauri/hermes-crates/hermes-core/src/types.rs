use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MessageRole
// ---------------------------------------------------------------------------

/// Role of a message participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

// ---------------------------------------------------------------------------
// CacheControl
// ---------------------------------------------------------------------------

/// Cache hint type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheType {
    Ephemeral,
    Persistent,
}

/// Cache control annotation for a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheControl {
    pub cache_type: CacheType,
}

// ---------------------------------------------------------------------------
// FunctionCall / ToolCall
// ---------------------------------------------------------------------------

/// A function call within a tool call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// A tool call emitted by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    #[serde(default)]
    pub id: String,
    #[serde(flatten)]
    pub function: FunctionCall,
}

// ---------------------------------------------------------------------------
// ReasoningContent
// ---------------------------------------------------------------------------

/// Format of the reasoning content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningFormat {
    Simple,
    Details,
}

/// Reasoning content parsed from the LLM response.
///
/// Supports multiple formats used by different providers:
/// - `reasoning_content` (simple string)
/// - `reasoning` (simple string)
/// - `reasoning_details` (structured array)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningContent {
    pub text: String,
    pub format: ReasoningFormat,
}

impl ReasoningContent {
    /// Parse reasoning content from a raw JSON value.
    ///
    /// Accepts:
    /// - A plain string (from `reasoning_content` or `reasoning`)
    /// - An object with a `text` field
    /// - An array of objects with `text` fields (from `reasoning_details`)
    pub fn from_value(value: &serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::String(s) => Some(ReasoningContent {
                text: s.clone(),
                format: ReasoningFormat::Simple,
            }),
            serde_json::Value::Object(map) => {
                map.get("text")
                    .and_then(|v| v.as_str())
                    .map(|text| ReasoningContent {
                        text: text.to_string(),
                        format: ReasoningFormat::Details,
                    })
            }
            serde_json::Value::Array(arr) => {
                let text = arr
                    .iter()
                    .filter_map(|item| item.get("text").and_then(|v| v.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n");
                if text.is_empty() {
                    None
                } else {
                    Some(ReasoningContent {
                        text,
                        format: ReasoningFormat::Details,
                    })
                }
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: MessageRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl Message {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            reasoning_content: None,
            cache_control: None,
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            reasoning_content: None,
            cache_control: None,
        }
    }

    /// Create an assistant message with text content.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            reasoning_content: None,
            cache_control: None,
        }
    }

    /// Create an assistant message with tool calls.
    pub fn assistant_with_tool_calls(content: Option<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            name: None,
            reasoning_content: None,
            cache_control: None,
        }
    }

    /// Create a tool result message.
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            name: None,
            reasoning_content: None,
            cache_control: None,
        }
    }
}

// ---------------------------------------------------------------------------
// ToolResult
// ---------------------------------------------------------------------------

/// Result from executing a tool call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful tool result.
    pub fn ok(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// Create an error tool result.
    pub fn err(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error: true,
        }
    }
}

// ---------------------------------------------------------------------------
// UsageStats / ToolErrorRecord / AgentResult
// ---------------------------------------------------------------------------

/// Token usage statistics from an LLM response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageStats {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cost: Option<f64>,
}

/// Record of a tool error during execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolErrorRecord {
    pub tool_name: String,
    pub error: String,
    pub turn: u32,
}

fn is_false(v: &bool) -> bool {
    !*v
}

/// Final result of an agent run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentResult {
    pub messages: Vec<Message>,
    pub finished_naturally: bool,
    pub total_turns: u32,
    #[serde(default)]
    pub tool_errors: Vec<ToolErrorRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageStats>,
    /// Set when the loop stopped due to [`crate::AgentError::Interrupted`] (Python parity).
    #[serde(default, skip_serializing_if = "is_false")]
    pub interrupted: bool,
    /// Estimated session spend in USD when cost tracking is active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_cost_usd: Option<f64>,
    /// Hook / plugin parity: `on_session_start` ran this run (new session, not restored prompt).
    #[serde(default, skip_serializing_if = "is_false")]
    pub session_started_hooks_fired: bool,
}

// ---------------------------------------------------------------------------
// BudgetConfig
// ---------------------------------------------------------------------------

/// Budget configuration for constraining output sizes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BudgetConfig {
    pub max_result_size_chars: usize,
    pub max_aggregate_chars: usize,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_result_size_chars: 100_000,
            max_aggregate_chars: 1_000_000,
        }
    }
}

// ---------------------------------------------------------------------------
// LlmResponse
// ---------------------------------------------------------------------------

/// Complete response from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmResponse {
    pub message: Message,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageStats>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Streaming types
// ---------------------------------------------------------------------------

/// Delta for a function call within a streaming tool call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCallDelta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Delta for a tool call within a streaming response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallDelta {
    pub index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionCallDelta>,
}

/// Delta content in a streaming chunk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamDelta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

/// A single chunk from a streaming LLM response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamChunk {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<StreamDelta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageStats>,
}

// ---------------------------------------------------------------------------
// Skill and Memory types (referenced by trait definitions)
// ---------------------------------------------------------------------------

/// Metadata for a skill listing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillMeta {
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub generation: Option<u32>,
}

/// Skill execution pattern - determines how invoke_skill orchestrates core tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillExecutionPattern {
    /// Sequential execution: call tools in order
    #[default]
    Sequential,
    /// Parallel execution: multiple independent steps at once
    Parallel,
    /// Conditional execution: decide next step based on previous result
    Conditional,
    /// Loop until condition satisfied
    LoopUntil,
    /// Interactive: requires user input for intermediate results
    Interactive,
}

/// Skill version tracking for self-evolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SkillVersion {
    /// Semantic version (e.g., "1.2.3")
    #[serde(default = "default_semver")]
    pub semver: String,
    /// Content hash for deduplication
    #[serde(default)]
    pub content_hash: String,
    /// Evolution generation (number of improvements since creation)
    #[serde(default)]
    pub generation: u32,
    /// Last evolution timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_evolved_at: Option<String>,
    /// Reason for last evolution
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_evolution_reason: Option<String>,
}

fn default_semver() -> String {
    "1.0.0".to_string()
}

/// Skill execution statistics for self-evolution decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SkillExecutionStats {
    /// Total execution count
    #[serde(default)]
    pub total_executions: u64,
    /// Success count
    #[serde(default)]
    pub success_count: u64,
    /// Failure count
    #[serde(default)]
    pub failure_count: u64,
    /// Average execution time in milliseconds
    #[serde(default)]
    pub avg_duration_ms: u64,
    /// Average user rating (1-5)
    #[serde(default)]
    pub avg_rating: f32,
    /// Common error patterns for evolution analysis
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub common_errors: Vec<ErrorPattern>,
    /// Last execution timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_executed_at: Option<String>,
}

/// Error pattern for identifying evolution opportunities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorPattern {
    /// Error description pattern
    pub pattern: String,
    /// Occurrence count
    #[serde(default)]
    pub count: u32,
    /// Suggested fix from evolution
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_fix: Option<String>,
}

/// A skill definition with self-evolution support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    /// Skill name (unique identifier, kebab-case)
    pub name: String,
    /// Skill content (Markdown execution instructions)
    pub content: String,
    /// Category (e.g., "development", "testing", "deployment")
    pub category: Option<String>,
    /// Short description
    pub description: Option<String>,
    /// Execution pattern
    #[serde(default)]
    pub execution_pattern: SkillExecutionPattern,
    /// Version tracking
    #[serde(default)]
    pub version: SkillVersion,
    /// Execution statistics
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_stats: Option<SkillExecutionStats>,
    /// Tags for search and categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Author/creator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// Context for skill execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SkillExecutionContext {
    /// Session ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Agent name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    /// Task ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Working directory
    #[serde(default)]
    pub cwd: String,
    /// Previous tool outputs for reference
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub previous_outputs: serde_json::Map<String, serde_json::Value>,
}

/// Result from skill execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SkillExecutionResult {
    /// Success status
    pub success: bool,
    /// Output content
    #[serde(default)]
    pub output: String,
    /// Error message if failed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Execution duration in milliseconds
    #[serde(default)]
    pub duration_ms: u64,
    /// Number of tool calls made
    #[serde(default)]
    pub tool_calls_count: u32,
    /// Whether self-evolution was triggered
    #[serde(default, skip_serializing_if = "is_false")]
    pub evolved: bool,
    /// Evolution summary if evolved
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolution_summary: Option<String>,
}

/// Invocation request for invoke_skill meta-tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillInvocation {
    /// Skill name to invoke
    pub skill_name: String,
    /// Parameters for skill execution
    #[serde(default)]
    pub parameters: serde_json::Map<String, serde_json::Value>,
    /// Execution context
    #[serde(default)]
    pub context: SkillExecutionContext,
}

impl Skill {
    /// Create a minimal skill with just name and content
    pub fn minimal(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            category: None,
            description: None,
            execution_pattern: SkillExecutionPattern::default(),
            version: SkillVersion::default(),
            execution_stats: None,
            tags: Vec::new(),
            author: None,
        }
    }

    /// Check if skill needs evolution based on execution stats
    pub fn should_evolve(&self, triggers: &EvolutionTriggers) -> Option<EvolutionReason> {
        let stats = self.execution_stats.as_ref()?;

        // Check minimum execution threshold
        if stats.total_executions < triggers.min_executions {
            return None;
        }

        // Check cooldown period
        if let Some(_last_evolved) = &self.version.last_evolved_at {
            // Parse timestamp and check cooldown (simplified)
            // In production, use proper datetime parsing
        }

        // Check failure rate
        if stats.total_executions > 0 {
            let failure_rate = stats.failure_count as f32 / stats.total_executions as f32;
            if failure_rate > triggers.failure_rate_threshold {
                return Some(EvolutionReason::HighFailureRate(failure_rate));
            }
        }

        // Check recurring errors
        for pattern in &stats.common_errors {
            if pattern.count >= triggers.error_pattern_threshold {
                return Some(EvolutionReason::RecurringError(pattern.clone()));
            }
        }

        // Check low rating
        if stats.avg_rating > 0.0 && stats.avg_rating < triggers.min_rating_threshold {
            return Some(EvolutionReason::LowRating(stats.avg_rating));
        }

        None
    }
}

/// Triggers for skill self-evolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvolutionTriggers {
    /// Failure rate threshold (trigger if above)
    pub failure_rate_threshold: f32,
    /// Minimum executions before considering evolution
    pub min_executions: u64,
    /// Error pattern occurrence threshold
    pub error_pattern_threshold: u32,
    /// Minimum acceptable rating
    pub min_rating_threshold: f32,
    /// Cooldown between evolutions in seconds
    pub cooldown_seconds: u64,
}

impl Default for EvolutionTriggers {
    fn default() -> Self {
        Self {
            failure_rate_threshold: 0.15,  // 15% failure rate
            min_executions: 10,             // at least 10 runs
            error_pattern_threshold: 3,     // same error 3 times
            min_rating_threshold: 2.5,      // rating below 2.5/5
            cooldown_seconds: 3600,         // 1 hour cooldown
        }
    }
}

/// Reason for triggering evolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvolutionReason {
    HighFailureRate(f32),
    RecurringError(ErrorPattern),
    LowRating(f32),
    UserRequested,
}

// ---------------------------------------------------------------------------
// CommandOutput (referenced by TerminalBackend trait)
// ---------------------------------------------------------------------------

/// Output from a terminal command execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_construction() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content.as_deref(), Some("Hello"));

        let msg = Message::system("You are helpful");
        assert_eq!(msg.role, MessageRole::System);
    }

    #[test]
    fn test_tool_result() {
        let ok = ToolResult::ok("call_1", "result text");
        assert!(!ok.is_error);

        let err = ToolResult::err("call_2", "failed");
        assert!(err.is_error);
    }

    #[test]
    fn test_reasoning_content_from_string() {
        let val = serde_json::Value::String("thinking...".to_string());
        let rc = ReasoningContent::from_value(&val).unwrap();
        assert_eq!(rc.text, "thinking...");
        assert_eq!(rc.format, ReasoningFormat::Simple);
    }

    #[test]
    fn test_reasoning_content_from_array() {
        let val = serde_json::json!([
            {"text": "step 1"},
            {"text": "step 2"}
        ]);
        let rc = ReasoningContent::from_value(&val).unwrap();
        assert_eq!(rc.text, "step 1\nstep 2");
        assert_eq!(rc.format, ReasoningFormat::Details);
    }

    #[test]
    fn test_serde_message_role() {
        let json = serde_json::to_string(&MessageRole::Assistant).unwrap();
        assert_eq!(json, "\"assistant\"");
        let role: MessageRole = serde_json::from_str(&json).unwrap();
        assert_eq!(role, MessageRole::Assistant);
    }

    #[test]
    fn test_budget_config_default() {
        let bc = BudgetConfig::default();
        assert_eq!(bc.max_result_size_chars, 100_000);
        assert_eq!(bc.max_aggregate_chars, 1_000_000);
    }
}
