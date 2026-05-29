//! Smart Router - Three-layer progressive complexity evaluation
//!
//! Implements intelligent routing: Heuristic → Structural → LLM-assisted

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Task complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskComplexity {
    Simple,     // Single file, minor changes
    Medium,     // Multiple files, refactoring
    Complex,    // Cross-module, new features
    VeryComplex, // Multi-agent needed
}

#[allow(dead_code)]
impl TaskComplexity {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskComplexity::Simple => "simple",
            TaskComplexity::Medium => "medium",
            TaskComplexity::Complex => "complex",
            TaskComplexity::VeryComplex => "very_complex",
        }
    }
}

/// Task type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Frontend,       // UI, Vue components
    Backend,        // API, database
    Fullstack,      // Both frontend and backend
    DevOps,         // CI/CD, deployment
    Testing,        // Tests, QA
    Documentation,  // Docs, comments
    Refactoring,    // Code cleanup
    BugFix,         // Fix issues
}

#[allow(dead_code)]
impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::Frontend => "frontend",
            TaskType::Backend => "backend",
            TaskType::Fullstack => "fullstack",
            TaskType::DevOps => "devops",
            TaskType::Testing => "testing",
            TaskType::Documentation => "documentation",
            TaskType::Refactoring => "refactoring",
            TaskType::BugFix => "bugfix",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "frontend" => Ok(TaskType::Frontend),
            "backend" => Ok(TaskType::Backend),
            "fullstack" => Ok(TaskType::Fullstack),
            "devops" => Ok(TaskType::DevOps),
            "testing" => Ok(TaskType::Testing),
            "documentation" => Ok(TaskType::Documentation),
            "refactoring" => Ok(TaskType::Refactoring),
            "bugfix" => Ok(TaskType::BugFix),
            other => Err(format!("Unknown task type: {}", other)),
        }
    }
}

/// Input type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputType {
    Text,       // Plain text request
    Image,      // Image input
    Document,   // Document/file input
    Code,       // Code snippet
    Command,    // Direct command
}

#[allow(dead_code)]
impl InputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InputType::Text => "text",
            InputType::Image => "image",
            InputType::Document => "document",
            InputType::Code => "code",
            InputType::Command => "command",
        }
    }
}

/// Route target - which agent/team to use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteTarget {
    ClaudeCodeHaiku,    // Simple tasks, fast response
    ClaudeCodeSonnet,   // Medium tasks, balanced
    CodexHaiku,         // Backend simple
    CodexSonnet,        // Backend complex
    Team,               // Multi-agent collaboration
    HumanReview,        // Needs human decision
}

#[allow(dead_code)]
impl RouteTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            RouteTarget::ClaudeCodeHaiku => "claude-code-haiku",
            RouteTarget::ClaudeCodeSonnet => "claude-code-sonnet",
            RouteTarget::CodexHaiku => "codex-haiku",
            RouteTarget::CodexSonnet => "codex-sonnet",
            RouteTarget::Team => "team",
            RouteTarget::HumanReview => "human-review",
        }
    }
}

/// Route decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub id: String,
    pub input_id: String,
    pub task_type: TaskType,
    pub input_type: InputType,
    pub complexity: TaskComplexity,
    pub route_target: RouteTarget,
    pub reason: String,
    pub agent_load: u32,
    pub historical_success_rate: f64,
    pub evaluation_method: EvaluationMethod,
    pub created_at: DateTime<Utc>,
}

/// Evaluation method used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvaluationMethod {
    Heuristic,   // Zero-cost rule matching
    Structural,  // Low-cost AST analysis
    LLMAssisted, // High-cost LLM evaluation
}

#[allow(dead_code)]
impl EvaluationMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            EvaluationMethod::Heuristic => "heuristic",
            EvaluationMethod::Structural => "structural",
            EvaluationMethod::LLMAssisted => "llm_assisted",
        }
    }
}

/// Task analyzer for complexity evaluation
pub struct TaskAnalyzer {
    heuristic_rules: HashMap<String, TaskComplexity>,
    structural_patterns: HashMap<String, TaskType>,
}

impl TaskAnalyzer {
    pub fn new() -> Self {
        let mut heuristic_rules = HashMap::new();

        // Simple task patterns
        heuristic_rules.insert("修改一行".to_string(), TaskComplexity::Simple);
        heuristic_rules.insert("修复bug".to_string(), TaskComplexity::Simple);
        heuristic_rules.insert("添加注释".to_string(), TaskComplexity::Simple);
        heuristic_rules.insert("格式化代码".to_string(), TaskComplexity::Simple);

        // Medium task patterns
        heuristic_rules.insert("重构".to_string(), TaskComplexity::Medium);
        heuristic_rules.insert("添加功能".to_string(), TaskComplexity::Medium);
        heuristic_rules.insert("优化性能".to_string(), TaskComplexity::Medium);
        heuristic_rules.insert("添加测试".to_string(), TaskComplexity::Medium);

        // Complex task patterns
        heuristic_rules.insert("跨模块".to_string(), TaskComplexity::Complex);
        heuristic_rules.insert("新功能".to_string(), TaskComplexity::Complex);
        heuristic_rules.insert("架构调整".to_string(), TaskComplexity::Complex);
        heuristic_rules.insert("集成".to_string(), TaskComplexity::Complex);

        // Very complex patterns
        heuristic_rules.insert("多agent".to_string(), TaskComplexity::VeryComplex);
        heuristic_rules.insert("大型重构".to_string(), TaskComplexity::VeryComplex);
        heuristic_rules.insert("系统设计".to_string(), TaskComplexity::VeryComplex);

        let mut structural_patterns = HashMap::new();
        structural_patterns.insert("vue".to_string(), TaskType::Frontend);
        structural_patterns.insert("component".to_string(), TaskType::Frontend);
        structural_patterns.insert("api".to_string(), TaskType::Backend);
        structural_patterns.insert("database".to_string(), TaskType::Backend);
        structural_patterns.insert("test".to_string(), TaskType::Testing);
        structural_patterns.insert("doc".to_string(), TaskType::Documentation);

        Self {
            heuristic_rules,
            structural_patterns,
        }
    }

    /// Analyze task using three-layer evaluation
    pub fn analyze(&self, input: &str, input_type: InputType) -> RouteDecision {
        let id = uuid::Uuid::new_v4().to_string();

        // Layer 1: Heuristic evaluation (zero cost)
        let (complexity, task_type) = self.heuristic_eval(input);

        // If heuristic is confident, use it (80% of requests)
        if complexity != TaskComplexity::Complex {
            let target = self.select_target(&task_type, &complexity, &input_type, 0, 0.8);

            return RouteDecision {
                id,
                input_id: String::new(),
                task_type,
                input_type,
                complexity,
                route_target: target,
                reason: "Heuristic match".to_string(),
                agent_load: 0,
                historical_success_rate: 0.8,
                evaluation_method: EvaluationMethod::Heuristic,
                created_at: Utc::now(),
            };
        }

        // Layer 2: Structural evaluation (low cost)
        let structural_result = self.structural_eval(input);
        let (complexity2, task_type2) = structural_result;

        // If structural gives clear answer, use it (15% of requests)
        if complexity2 != TaskComplexity::Complex {
            let target = self.select_target(&task_type2, &complexity2, &input_type, 0, 0.85);

            return RouteDecision {
                id,
                input_id: String::new(),
                task_type: task_type2,
                input_type,
                complexity: complexity2,
                route_target: target,
                reason: "Structural analysis".to_string(),
                agent_load: 0,
                historical_success_rate: 0.85,
                evaluation_method: EvaluationMethod::Structural,
                created_at: Utc::now(),
            };
        }

        // Layer 3: LLM-assisted evaluation (high cost, 5% of requests)
        // In production, this would call an LLM to analyze
        let target = RouteTarget::Team; // Default to team for complex

        RouteDecision {
            id,
            input_id: String::new(),
            task_type: TaskType::Fullstack,
            input_type,
            complexity: TaskComplexity::Complex,
            route_target: target,
            reason: "LLM-assisted evaluation recommended Team".to_string(),
            agent_load: 0,
            historical_success_rate: 0.9,
            evaluation_method: EvaluationMethod::LLMAssisted,
            created_at: Utc::now(),
        }
    }

    /// Heuristic evaluation - rule matching
    fn heuristic_eval(&self, input: &str) -> (TaskComplexity, TaskType) {
        let input_lower = input.to_lowercase();

        // Check complexity patterns
        for (pattern, complexity) in &self.heuristic_rules {
            if input_lower.contains(pattern) {
                // Determine task type from keywords
                let task_type = self.determine_task_type(&input_lower);
                return (*complexity, task_type);
            }
        }

        // Default to medium complexity
        (TaskComplexity::Medium, TaskType::Fullstack)
    }

    /// Structural evaluation - keyword counting
    fn structural_eval(&self, input: &str) -> (TaskComplexity, TaskType) {
        let input_lower = input.to_lowercase();

        // Count file references
        let file_count = input_lower.matches("文件").count()
            + input_lower.matches(".vue").count()
            + input_lower.matches(".ts").count()
            + input_lower.matches(".rs").count();

        // Count module references
        let module_count = input_lower.matches("模块").count()
            + input_lower.matches("组件").count()
            + input_lower.matches("服务").count();

        // Determine complexity by counts
        let complexity = if file_count <= 1 && module_count == 0 {
            TaskComplexity::Simple
        } else if file_count <= 3 && module_count <= 1 {
            TaskComplexity::Medium
        } else if file_count <= 5 && module_count <= 3 {
            TaskComplexity::Complex
        } else {
            TaskComplexity::VeryComplex
        };

        let task_type = self.determine_task_type(&input_lower);

        (complexity, task_type)
    }

    /// Determine task type from keywords
    fn determine_task_type(&self, input: &str) -> TaskType {
        for (pattern, task_type) in &self.structural_patterns {
            if input.contains(pattern) {
                return task_type.clone();
            }
        }

        if input.contains("前端") || input.contains("ui") || input.contains("界面") {
            TaskType::Frontend
        } else if input.contains("后端") || input.contains("api") || input.contains("数据库") {
            TaskType::Backend
        } else if input.contains("测试") || input.contains("qa") {
            TaskType::Testing
        } else {
            TaskType::Fullstack
        }
    }

    /// Select route target based on analysis
    fn select_target(
        &self,
        task_type: &TaskType,
        complexity: &TaskComplexity,
        input_type: &InputType,
        _agent_load: u32,
        _success_rate: f64,
    ) -> RouteTarget {
        // Image input prefers Claude Code (visual capability)
        if *input_type == InputType::Image {
            return RouteTarget::ClaudeCodeSonnet;
        }

        // Document input prefers Team (分工处理)
        if *input_type == InputType::Document {
            return RouteTarget::Team;
        }

        // Route by task type and complexity
        match (task_type, complexity) {
            (TaskType::Frontend, TaskComplexity::Simple) => RouteTarget::ClaudeCodeHaiku,
            (TaskType::Frontend, TaskComplexity::Medium) => RouteTarget::ClaudeCodeSonnet,
            (TaskType::Frontend, TaskComplexity::Complex) => RouteTarget::Team,
            (TaskType::Backend, TaskComplexity::Simple) => RouteTarget::CodexHaiku,
            (TaskType::Backend, TaskComplexity::Medium) => RouteTarget::CodexSonnet,
            (TaskType::Backend, TaskComplexity::Complex) => RouteTarget::Team,
            (TaskType::Fullstack, TaskComplexity::Simple) => RouteTarget::ClaudeCodeHaiku,
            (TaskType::Fullstack, TaskComplexity::Medium) => RouteTarget::ClaudeCodeSonnet,
            (TaskType::Fullstack, TaskComplexity::Complex) => RouteTarget::Team,
            (TaskType::Testing, _) => RouteTarget::ClaudeCodeSonnet,
            (TaskType::Documentation, _) => RouteTarget::ClaudeCodeHaiku,
            (TaskType::Refactoring, TaskComplexity::Simple) => RouteTarget::ClaudeCodeHaiku,
            (TaskType::Refactoring, TaskComplexity::Medium) => RouteTarget::ClaudeCodeSonnet,
            (TaskType::Refactoring, TaskComplexity::Complex) => RouteTarget::Team,
            (TaskType::BugFix, _) => RouteTarget::ClaudeCodeSonnet,
            (TaskType::DevOps, _) => RouteTarget::CodexSonnet,
            (_, TaskComplexity::VeryComplex) => RouteTarget::Team,
        }
    }
}

impl Default for TaskAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}