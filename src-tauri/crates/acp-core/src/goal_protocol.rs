//! Goal Protocol - Goal definition and submission
//!
//! Unified Goal types for both src-tauri/src/ and swarm-engine crate.
//!
//! **Type Unification Strategy (C-1)**:
//! - GoalSpec: Submission specification (input)
//! - GoalRuntime: Execution state (runtime)
//! - Both crates use these unified types from acp-core

use serde::{Deserialize, Serialize};

/// Goal specification (用于提交 Goal 的规格)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSpec {
    /// Goal ID
    pub id: String,
    /// Goal description
    #[serde(alias = "desc")]
    pub description: String,
    /// Completion condition
    #[serde(alias = "completionCondition")]
    pub completion_condition: CompletionConditionSpec,
    /// Evaluator type
    pub evaluator: EvaluatorSpec,
    /// Executor assignment
    pub executor: Option<String>,
    /// Dependencies
    #[serde(alias = "dependencies", alias = "depends_on")]
    pub depends_on: Vec<String>,
    /// Token budget
    #[serde(alias = "tokenBudget")]
    pub token_budget: u64,
    /// Maximum iterations
    #[serde(alias = "maxIterations")]
    pub max_iterations: u32,
    /// Per-iteration timeout (milliseconds)
    #[serde(default = "default_timeout", alias = "perIterationTimeoutMs")]
    pub per_iteration_timeout_ms: u64,
}

/// GoalRuntime - Goal execution state (unified runtime type for C-1)
///
/// This is the authoritative Goal type for execution.
/// Both src-tauri/src/ and swarm-engine should use this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GoalRuntime {
    // === Identity ===
    /// Goal unique identifier
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Parent task ID (for grouping)
    pub parent_task_id: Option<String>,
    /// Parent goal ID (for sub-goals)
    #[serde(alias = "parent_id", alias = "parent_goal_id")]
    pub parent_goal_id: Option<String>,

    // === Completion Condition ===
    /// Condition for convergence
    pub completion_condition: CompletionConditionSpec,

    // === Role Assignment ===
    /// Evaluator type
    pub evaluator: EvaluatorSpec,
    /// Assigned worker ID
    #[serde(alias = "assigned_worker")]
    pub executor: Option<String>,

    // === Dependencies ===
    /// Dependent goal IDs
    #[serde(alias = "dependencies")]
    pub depends_on: Vec<String>,

    // === Budget & Limits ===
    /// Token budget limit (None = unlimited, serialized as Option)
    #[serde(alias = "tokenBudget")]
    pub token_budget: Option<u64>,
    /// Tokens consumed
    #[serde(alias = "token_used")]
    pub tokens_used: u64,
    /// Maximum iterations
    pub max_iterations: u32,
    /// Current iteration count
    pub current_iteration: u32,
    /// Per-iteration timeout (milliseconds)
    pub per_iteration_timeout_ms: u64,

    // === State ===
    /// Execution status
    pub status: GoalStatus,
    /// Iteration history
    #[serde(alias = "iterations")]
    pub iteration_log: Vec<IterationRecord>,

    // === Metadata ===
    /// Creation timestamp (UNIX milliseconds)
    pub created_at: u64,
    /// When execution started (UNIX milliseconds)
    pub started_at: Option<u64>,
    /// Convergence timestamp
    pub converged_at: Option<u64>,
    /// Output file paths
    pub output_files: Vec<String>,
}

impl GoalRuntime {
    /// Create a new goal from spec
    pub fn from_spec(spec: GoalSpec) -> Self {
        Self {
            id: spec.id,
            description: spec.description,
            parent_task_id: None,
            parent_goal_id: None,
            completion_condition: spec.completion_condition,
            evaluator: spec.evaluator,
            executor: spec.executor,
            depends_on: spec.depends_on,
            token_budget: Some(spec.token_budget),
            tokens_used: 0,
            max_iterations: spec.max_iterations,
            current_iteration: 0,
            per_iteration_timeout_ms: spec.per_iteration_timeout_ms,
            status: GoalStatus::Pending,
            iteration_log: Vec::new(),
            created_at: current_timestamp(),
            started_at: None,
            converged_at: None,
            output_files: Vec::new(),
        }
    }

    /// Check if goal can start (no pending dependencies)
    pub fn is_ready(&self, converged_ids: &[&str]) -> bool {
        self.status == GoalStatus::Pending
            && self.depends_on.iter().all(|dep| converged_ids.contains(&dep.as_str()))
    }

    /// Check if budget exhausted
    pub fn budget_exhausted(&self) -> bool {
        self.token_budget.map(|b| self.tokens_used >= b).unwrap_or(false)
    }

    /// Check if max iterations reached
    pub fn max_iter_reached(&self) -> bool {
        self.current_iteration >= self.max_iterations
    }

    /// Get current iteration number (1-based)
    pub fn current_iteration_number(&self) -> u32 {
        self.iteration_log.len() as u32 + 1
    }

    /// Append iteration feedback (for reconcile loop)
    pub fn append_feedback(&mut self, iteration: u32, feedback: &str, tokens: u64) {
        let record = IterationRecord {
            iteration,
            worker_output: feedback.to_string(),
            evaluation: EvaluationResult {
                passed: false,
                explanation: feedback.to_string(),
                details: vec![],
            },
            feedback: Some(feedback.to_string()),
            timestamp: current_timestamp(),
            tokens_used: tokens,
            duration_ms: 0,
        };
        self.iteration_log.push(record);
        self.tokens_used += tokens;
        self.current_iteration = iteration;
    }

    /// Create a new goal with simple parameters (for testing convenience)
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        completion_condition: CompletionConditionSpec,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            parent_task_id: None,
            parent_goal_id: None,
            completion_condition,
            evaluator: EvaluatorSpec::Auto,
            executor: None,
            depends_on: vec![],
            token_budget: Some(100_000),
            tokens_used: 0,
            max_iterations: 5,
            current_iteration: 0,
            per_iteration_timeout_ms: 60_000,
            status: GoalStatus::Pending,
            iteration_log: Vec::new(),
            created_at: current_timestamp(),
            started_at: None,
            converged_at: None,
            output_files: Vec::new(),
        }
    }

    /// Create a new goal with executor assignment (for testing convenience)
    pub fn with_executor(
        id: impl Into<String>,
        description: impl Into<String>,
        completion_condition: CompletionConditionSpec,
        executor: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            parent_task_id: None,
            parent_goal_id: None,
            completion_condition,
            evaluator: EvaluatorSpec::Auto,
            executor: Some(executor.into()),
            depends_on: vec![],
            token_budget: Some(100_000),
            tokens_used: 0,
            max_iterations: 5,
            current_iteration: 0,
            per_iteration_timeout_ms: 60_000,
            status: GoalStatus::Pending,
            iteration_log: Vec::new(),
            created_at: current_timestamp(),
            started_at: None,
            converged_at: None,
            output_files: Vec::new(),
        }
    }
}

fn default_timeout() -> u64 {
    60_000
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Goal YAML file structure (完整的 .goal.yaml 文件格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalYamlFile {
    /// File name/identifier
    pub name: String,
    /// Overall description
    pub description: String,
    /// List of goals
    pub goals: Vec<GoalSpec>,
    /// Execution topology (chain, star, etc.)
    #[serde(default = "default_topology")]
    pub topology: String,
    /// Queen configuration
    #[serde(default)]
    pub queen: Option<QueenConfig>,
    /// Worker configurations
    #[serde(default)]
    pub workers: Vec<WorkerDefinition>,
}

fn default_topology() -> String {
    "chain".to_string()
}

/// Queen configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueenConfig {
    #[serde(default = "default_lease_ttl")]
    pub lease_ttl_seconds: u64,
    #[serde(default = "default_renew_interval")]
    pub renew_interval_seconds: u64,
    pub worker_id: String,
}

fn default_lease_ttl() -> u64 { 30 }
fn default_renew_interval() -> u64 { 10 }

/// Worker definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerDefinition {
    pub id: String,
    #[serde(rename = "type")]
    pub worker_type: String,
    #[serde(default)]
    pub skills: Vec<SkillDefinition>,
}

/// Skill definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub name: String,
    pub proficiency: f32,
    #[serde(default)]
    pub avg_duration_ms: u64,
}

impl GoalYamlFile {
    /// Parse a Goal YAML file from string
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Parse from file path
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        serde_yaml::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Convert to individual GoalSpecs
    pub fn to_goal_specs(&self) -> Vec<GoalSpec> {
        self.goals.clone()
    }
}

/// Completion condition specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum CompletionConditionSpec {
    #[serde(rename = "command_success")]
    CommandSuccess {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        cwd: Option<String>,
    },
    #[serde(rename = "file_check")]
    FileCheck {
        path: String,
        #[serde(default, alias = "content_contains")]
        content_contains: Option<String>,
        #[serde(default, alias = "max_size_bytes")]
        max_size_bytes: Option<u64>,
    },
    #[serde(rename = "output_contains")]
    OutputContains {
        command: String,
        #[serde(alias = "pattern")]
        pattern: String,
        #[serde(default)]
        case_sensitive: bool,
    },
    #[serde(rename = "output_matches")]
    OutputMatches {
        command: String,
        #[serde(alias = "regex")]
        regex: String,
    },
    #[serde(rename = "http_health_check")]
    HttpHealthCheck {
        #[serde(alias = "url")]
        url: String,
        #[serde(default = "default_get_method")]
        method: String,
        #[serde(default, alias = "expected_status")]
        expected_status: Option<u16>,
    },
    #[serde(rename = "queen_judgment")]
    QueenJudgment {
        #[serde(alias = "criteria")]
        criteria: String,
    },
    #[serde(rename = "all")]
    All { conditions: Vec<CompletionConditionSpec> },
    #[serde(rename = "any")]
    Any { conditions: Vec<CompletionConditionSpec> },
    #[serde(rename = "custom")]
    Custom { evaluator: String },
}

impl CompletionConditionSpec {
    /// Create a command success condition (helper for tests)
    pub fn command_success(command: impl Into<String>) -> Self {
        Self::CommandSuccess {
            command: command.into(),
            args: vec![],
            cwd: None,
        }
    }

    /// Create a file exists condition (helper for tests)
    pub fn file_exists(path: impl Into<String>) -> Self {
        Self::FileCheck {
            path: path.into(),
            content_contains: None,
            max_size_bytes: None,
        }
    }
}

fn default_get_method() -> String {
    "GET".to_string()
}

/// Intermediate type for deserializing EvaluatorSpec (string or object)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(dead_code, non_snake_case)]
enum EvaluatorRaw {
    String(String),
    QueenObj { queen_worker_id: String },
    AdversarialObj { primary: String, adversary: String },
    HybridObj { auto_conditions: Vec<CompletionConditionSpec>, queen_criteria: String },
    QueenWrapped { Queen: QueenInner },
    AdversarialWrapped { Adversarial: AdversarialInner },
    HybridWrapped { Hybrid: HybridInner },
    AutoWrapped { Auto: serde_json::Value },
}

#[derive(Debug, Clone, Deserialize)]
struct QueenInner {
    queen_worker_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AdversarialInner {
    primary: String,
    adversary: String,
}

#[derive(Debug, Clone, Deserialize)]
struct HybridInner {
    auto_conditions: Vec<CompletionConditionSpec>,
    queen_criteria: String,
}

impl From<EvaluatorRaw> for EvaluatorSpec {
    fn from(raw: EvaluatorRaw) -> Self {
        match raw {
            EvaluatorRaw::String(s) if s == "auto" => EvaluatorSpec::Auto,
            EvaluatorRaw::String(_) => EvaluatorSpec::Auto, // default fallback for unknown strings
            EvaluatorRaw::QueenObj { queen_worker_id } => EvaluatorSpec::Queen { queen_worker_id },
            EvaluatorRaw::AdversarialObj { primary, adversary } => EvaluatorSpec::Adversarial { primary, adversary },
            EvaluatorRaw::HybridObj { auto_conditions, queen_criteria } => EvaluatorSpec::Hybrid { auto_conditions, queen_criteria },
            EvaluatorRaw::QueenWrapped { Queen: inner } => EvaluatorSpec::Queen { queen_worker_id: inner.queen_worker_id },
            EvaluatorRaw::AdversarialWrapped { Adversarial: inner } => EvaluatorSpec::Adversarial { primary: inner.primary, adversary: inner.adversary },
            EvaluatorRaw::HybridWrapped { Hybrid: inner } => EvaluatorSpec::Hybrid { auto_conditions: inner.auto_conditions, queen_criteria: inner.queen_criteria },
            EvaluatorRaw::AutoWrapped { .. } => EvaluatorSpec::Auto,
        }
    }
}

/// Evaluator specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(from = "EvaluatorRaw")]
pub enum EvaluatorSpec {
    Auto,
    Queen { queen_worker_id: String },
    Adversarial { primary: String, adversary: String },
    Hybrid { auto_conditions: Vec<CompletionConditionSpec>, queen_criteria: String },
}

// ============================================================================
// 执行状态类型（用于 Goal 执行过程中）
// ============================================================================

/// Goal 执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GoalStatus {
    Pending,
    Active,
    Evaluating,
    Converged,
    Iterating { feedback: String },
    Failed { reason: String },
    BudgetExhausted,
    MaxIterReached,
    Cancelled,
}

impl GoalStatus {
    /// 检查是否为终止状态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            GoalStatus::Converged
                | GoalStatus::Failed { .. }
                | GoalStatus::BudgetExhausted
                | GoalStatus::MaxIterReached
                | GoalStatus::Cancelled
        )
    }

    /// 检查是否为成功状态
    pub fn is_success(&self) -> bool {
        matches!(self, GoalStatus::Converged)
    }
}

/// 迭代记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IterationRecord {
    /// 迭代编号
    pub iteration: u32,
    /// Worker 输出
    pub worker_output: String,
    /// 评估结果
    pub evaluation: EvaluationResult,
    /// 反馈信息（评估未通过时）
    #[serde(default)]
    pub feedback: Option<String>,
    /// 消耗的 token
    pub tokens_used: u64,
    /// 时间戳（UNIX 毫秒）
    pub timestamp: u64,
    /// 本次迭代耗时（毫秒）
    #[serde(default)]
    pub duration_ms: u64,
}

/// 评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EvaluationResult {
    /// 是否通过
    pub passed: bool,
    /// 解释说明
    pub explanation: String,
    /// 详细条件结果
    pub details: Vec<ConditionResult>,
}

/// 单个条件结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConditionResult {
    /// 条件描述
    pub description: String,
    /// 是否通过
    pub passed: bool,
    /// 证据/输出
    pub evidence: String,
}

/// Goal 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalOutcome {
    Converged { iterations: u32, tokens_used: u64 },
    Failed { reason: String, iterations: u32 },
    BudgetExhausted { tokens_used: u64 },
    MaxIterReached { iterations: u32 },
    Cancelled,
}

/// Goal 图摘要
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoalGraphSummary {
    pub total: u32,
    pub pending: u32,
    pub active: u32,
    pub evaluating: u32,
    pub converged: u32,
    pub iterating: u32,
    pub failed: u32,
    pub budget_exhausted: u32,
    pub max_iter_reached: u32,
    pub cancelled: u32,
}

// ============================================================================
// AcpEvent - Event Protocol 实现
// ============================================================================

/// ACP 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpEvent {
    /// 事件类型
    pub event_type: String,
    /// Goal ID
    pub goal_id: Option<String>,
    /// Worker ID
    pub worker_id: Option<String>,
    /// 事件数据
    pub data: serde_json::Value,
    /// 时间戳
    pub timestamp: u64,
}

impl AcpEvent {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            goal_id: None,
            worker_id: None,
            data: serde_json::Value::Null,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    pub fn with_goal(mut self, goal_id: impl Into<String>) -> Self {
        self.goal_id = Some(goal_id.into());
        self
    }

    pub fn with_worker(mut self, worker_id: impl Into<String>) -> Self {
        self.worker_id = Some(worker_id.into());
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_spec_new() {
        let spec = GoalSpec {
            id: "goal-001".to_string(),
            description: "Fix compilation errors".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "cargo build".to_string(),
                args: vec![],
                cwd: None,
            },
            evaluator: EvaluatorSpec::Auto,
            executor: Some("worker-001".to_string()),
            depends_on: vec![],
            token_budget: 50_000,
            max_iterations: 5,
            per_iteration_timeout_ms: 60_000,
        };

        assert_eq!(spec.id, "goal-001");
        assert_eq!(spec.token_budget, 50_000);
    }

    #[test]
    fn goal_status_is_terminal() {
        assert!(GoalStatus::Converged.is_terminal());
        assert!(GoalStatus::Failed { reason: "test".into() }.is_terminal());
        assert!(!GoalStatus::Pending.is_terminal());
        assert!(!GoalStatus::Active.is_terminal());
    }

    #[test]
    fn goal_status_is_success() {
        assert!(GoalStatus::Converged.is_success());
        assert!(!GoalStatus::Failed { reason: "test".into() }.is_success());
    }

    #[test]
    fn acp_event_builder() {
        let event = AcpEvent::new("goal.created")
            .with_goal("goal-001")
            .with_data(serde_json::json!({"description": "Test"}));

        assert_eq!(event.event_type, "goal.created");
        assert_eq!(event.goal_id, Some("goal-001".to_string()));
    }

    #[test]
    fn goal_spec_serde_roundtrip() {
        let spec = GoalSpec {
            id: "goal-002".to_string(),
            description: "Run tests".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "npm test".to_string(),
                args: vec!["--verbose".to_string()],
                cwd: Some("/tmp".to_string()),
            },
            evaluator: EvaluatorSpec::Auto,
            executor: None,
            depends_on: vec!["goal-001".to_string()],
            token_budget: 30_000,
            max_iterations: 3,
            per_iteration_timeout_ms: 120_000,
        };

        let json = serde_json::to_string(&spec).unwrap();
        let decoded: GoalSpec = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, spec.id);
        assert_eq!(decoded.depends_on.len(), 1);
    }

    #[test]
    fn completion_condition_command_success() {
        let cc = CompletionConditionSpec::CommandSuccess {
            command: "echo hello".to_string(),
            args: vec![],
            cwd: None,
        };

        let json = serde_json::to_string(&cc).unwrap();
        assert!(json.contains("command_success"));
        assert!(json.contains("echo hello"));

        let decoded: CompletionConditionSpec = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, CompletionConditionSpec::CommandSuccess { .. }));
    }

    #[test]
    fn completion_condition_file_check() {
        let cc = CompletionConditionSpec::FileCheck {
            path: "README.md".to_string(),
            content_contains: Some("hello".to_string()),
            max_size_bytes: None,
        };

        let json = serde_json::to_string(&cc).unwrap();
        let decoded: CompletionConditionSpec = serde_json::from_str(&json).unwrap();

        assert!(matches!(decoded, CompletionConditionSpec::FileCheck { .. }));
    }

    #[test]
    fn completion_condition_all() {
        let cc = CompletionConditionSpec::All {
            conditions: vec![
                CompletionConditionSpec::CommandSuccess {
                    command: "cargo test".to_string(),
                    args: vec![],
                    cwd: None,
                },
                CompletionConditionSpec::FileCheck {
                    path: "Cargo.toml".to_string(),
                    content_contains: None,
                    max_size_bytes: None,
                },
            ],
        };

        let json = serde_json::to_string(&cc).unwrap();
        let decoded: CompletionConditionSpec = serde_json::from_str(&json).unwrap();

        if let CompletionConditionSpec::All { conditions } = decoded {
            assert_eq!(conditions.len(), 2);
        } else {
            panic!("Expected All variant");
        }
    }

    #[test]
    fn completion_condition_yaml_parsing() {
        let yaml = r#"
type: command_success
command: "npx vue-tsc --noEmit"
args: []
cwd: null
"#;
        let cc: CompletionConditionSpec = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(cc, CompletionConditionSpec::CommandSuccess { .. }));
    }

    #[test]
    fn completion_condition_file_check_yaml() {
        let yaml = r#"
type: file_check
path: "README.md"
content_contains: "hello-swarm"
"#;
        let cc: CompletionConditionSpec = serde_yaml::from_str(yaml).unwrap();
        if let CompletionConditionSpec::FileCheck { path, content_contains, .. } = cc {
            assert_eq!(path, "README.md");
            assert_eq!(content_contains, Some("hello-swarm".to_string()));
        } else {
            panic!("Expected FileCheck variant");
        }
    }

    #[test]
    fn evaluator_spec_serde() {
        let eval = EvaluatorSpec::Queen { queen_worker_id: "queen-001".into() };
        let json = serde_json::to_string(&eval).unwrap();
        let decoded: EvaluatorSpec = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, EvaluatorSpec::Queen { .. }));
    }

    #[test]
    fn iteration_record() {
        let record = IterationRecord {
            iteration: 1,
            worker_output: "output".into(),
            evaluation: EvaluationResult {
                passed: true,
                explanation: "success".into(),
                details: vec![],
            },
            feedback: None,
            timestamp: 1000,
            tokens_used: 100,
            duration_ms: 0,
        };

        let json = serde_json::to_string(&record).unwrap();
        let decoded: IterationRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.iteration, 1);
    }

    #[test]
    fn goal_outcome_converged() {
        let outcome = GoalOutcome::Converged { iterations: 3, tokens_used: 1000 };
        let json = serde_json::to_string(&outcome).unwrap();
        assert!(json.contains("Converged"));
    }

    #[test]
    fn goal_graph_summary_default() {
        let summary = GoalGraphSummary::default();
        assert_eq!(summary.total, 0);
        assert_eq!(summary.pending, 0);
    }

    #[test]
    fn goal_yaml_file_hello_swarm() {
        // Simulate parsing hello-swarm.goal.yaml structure
        let yaml = r#"
name: hello-swarm
description: "Hello World Swarm Demo - Fix TypeScript compilation errors"

goals:
  - id: fix-typescript
    description: "Fix all TypeScript compilation errors in the project"
    completion_condition:
      type: command_success
      command: "npx vue-tsc --noEmit"
      args: []
      cwd: null
    evaluator: auto
    executor: claude-code-worker
    depends_on: []
    token_budget: 80000
    max_iterations: 5
    per_iteration_timeout_ms: 120000

  - id: run-tests
    description: "Run all tests to ensure no regressions"
    completion_condition:
      type: command_success
      command: "npm"
      args: ["test"]
      cwd: null
    evaluator: auto
    executor: codex-worker
    depends_on:
      - fix-typescript
    token_budget: 50000
    max_iterations: 3
    per_iteration_timeout_ms: 180000

topology: chain
"#;

        let goal_file: GoalYamlFile = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(goal_file.name, "hello-swarm");
        assert_eq!(goal_file.goals.len(), 2);
        assert_eq!(goal_file.topology, "chain");

        // Check first goal
        let goal1 = &goal_file.goals[0];
        assert_eq!(goal1.id, "fix-typescript");
        assert_eq!(goal1.token_budget, 80000);
        assert!(goal1.depends_on.is_empty());

        // Check second goal with dependency
        let goal2 = &goal_file.goals[1];
        assert_eq!(goal2.id, "run-tests");
        assert_eq!(goal2.depends_on, vec!["fix-typescript"]);
    }

    #[test]
    fn goal_yaml_file_with_workers() {
        let yaml = r#"
name: test
description: "Test workers"
goals:
  - id: g1
    description: "Task 1"
    completion_condition:
      type: command_success
      command: "echo"
    evaluator: auto
    depends_on: []
    token_budget: 10000
    max_iterations: 1
workers:
  - id: worker-001
    type: claude_code
    skills:
      - name: code-generation
        proficiency: 0.9
        avg_duration_ms: 30000
"#;

        let goal_file: GoalYamlFile = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(goal_file.workers.len(), 1);
        let worker = &goal_file.workers[0];
        assert_eq!(worker.id, "worker-001");
        assert_eq!(worker.worker_type, "claude_code");
        assert_eq!(worker.skills.len(), 1);
        assert_eq!(worker.skills[0].proficiency, 0.9);
    }

    #[test]
    fn goal_runtime_from_spec() {
        let spec = GoalSpec {
            id: "goal-001".to_string(),
            description: "Test goal".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "echo".to_string(),
                args: vec![],
                cwd: None,
            },
            evaluator: EvaluatorSpec::Auto,
            executor: Some("worker-001".to_string()),
            depends_on: vec!["goal-000".to_string()],
            token_budget: 50_000,
            max_iterations: 5,
            per_iteration_timeout_ms: 60_000,
        };

        let runtime = GoalRuntime::from_spec(spec);

        assert_eq!(runtime.id, "goal-001");
        assert_eq!(runtime.description, "Test goal");
        assert_eq!(runtime.status, GoalStatus::Pending);
        assert_eq!(runtime.tokens_used, 0);
        assert_eq!(runtime.current_iteration, 0);
        assert_eq!(runtime.executor, Some("worker-001".to_string()));
        assert_eq!(runtime.depends_on, vec!["goal-000"]);
        assert!(runtime.iteration_log.is_empty());
        assert!(runtime.output_files.is_empty());
    }

    #[test]
    fn goal_runtime_is_ready() {
        let spec = GoalSpec {
            id: "goal-002".to_string(),
            description: "Dependent goal".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "echo".to_string(),
                args: vec![],
                cwd: None,
            },
            evaluator: EvaluatorSpec::Auto,
            executor: None,
            depends_on: vec!["goal-001".to_string()],
            token_budget: 30_000,
            max_iterations: 3,
            per_iteration_timeout_ms: 60_000,
        };

        let runtime = GoalRuntime::from_spec(spec);

        // Not ready if dependencies not converged
        assert!(!runtime.is_ready(&[]));
        assert!(!runtime.is_ready(&["other-goal"]));

        // Ready when all dependencies converged
        assert!(runtime.is_ready(&["goal-001"]));
    }

    #[test]
    fn goal_runtime_budget_exhausted() {
        let spec = GoalSpec {
            id: "goal-003".to_string(),
            description: "Budget test".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "echo".to_string(),
                args: vec![],
                cwd: None,
            },
            evaluator: EvaluatorSpec::Auto,
            executor: None,
            depends_on: vec![],
            token_budget: 1000,
            max_iterations: 5,
            per_iteration_timeout_ms: 60_000,
        };

        let mut runtime = GoalRuntime::from_spec(spec);

        assert!(!runtime.budget_exhausted());

        runtime.tokens_used = 500;
        assert!(!runtime.budget_exhausted());

        runtime.tokens_used = 1000;
        assert!(runtime.budget_exhausted());
    }

    #[test]
    fn goal_runtime_max_iter_reached() {
        let spec = GoalSpec {
            id: "goal-004".to_string(),
            description: "Iter test".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "echo".to_string(),
                args: vec![],
                cwd: None,
            },
            evaluator: EvaluatorSpec::Auto,
            executor: None,
            depends_on: vec![],
            token_budget: 50_000,
            max_iterations: 3,
            per_iteration_timeout_ms: 60_000,
        };

        let mut runtime = GoalRuntime::from_spec(spec);

        assert!(!runtime.max_iter_reached());

        runtime.current_iteration = 2;
        assert!(!runtime.max_iter_reached());

        runtime.current_iteration = 3;
        assert!(runtime.max_iter_reached());
    }

    #[test]
    fn goal_runtime_serde_roundtrip() {
        let spec = GoalSpec {
            id: "goal-005".to_string(),
            description: "Serde test".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "cargo test".to_string(),
                args: vec![],
                cwd: None,
            },
            evaluator: EvaluatorSpec::Auto,
            executor: Some("claude-worker".to_string()),
            depends_on: vec!["goal-004".to_string()],
            token_budget: 80_000,
            max_iterations: 10,
            per_iteration_timeout_ms: 120_000,
        };

        let runtime = GoalRuntime::from_spec(spec);
        let json = serde_json::to_string(&runtime).unwrap();
        let decoded: GoalRuntime = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, runtime.id);
        assert_eq!(decoded.executor, runtime.executor);
        assert_eq!(decoded.depends_on, runtime.depends_on);
        assert_eq!(decoded.status, GoalStatus::Pending);
    }
}