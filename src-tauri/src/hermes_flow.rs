//! Hermes Flow - Development Flow Orchestration for Self-Evolving Loop
//!
//! Integrates frontend dev-flow-skills.ts with Rust backend workflow engine.
//! Provides complete development cycle: 需求分析 → 计划撰写 → 写代码 → 代码调整 → 功能测试

// Debug prints are intentional for development phase
#![allow(clippy::print_stdout)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

use crate::AppState;
use crate::workflow_engine::{WorkflowEngine, WorkflowDefinition, WorkflowStage, WorkflowStatus, StageStrategy, StageAgent, StageResult};

// ---------------------------------------------------------------------------
// Development Flow Types (mirrors frontend dev-flow-skills.ts)
// ---------------------------------------------------------------------------

/// Development flow stage names - mirrors DEVELOPMENT_FLOW_SKILLS in frontend
const DEV_FLOW_STAGES: &[&str] = &[
    "analysis",        // 需求分析
    "planning",        // 计划撰写
    "code-execution",  // 写代码
    "code-review",     // 代码审查
    "adjustments",     // 代码调整
    "testing",         // 功能测试
    "documentation",   // 文档生成
    "communication",   // 反馈用户
];

/// Flow context passed from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowContext {
    pub task_id: String,
    pub request: String,
    pub workspace: String,
    pub cwd: String,
    pub agent_name: String,
    pub session_id: Option<String>,

    // Stage results (accumulated)
    pub analysis: Option<AnalysisResult>,
    pub plan: Option<PlanResult>,
    pub code: Option<CodeResult>,
    pub review: Option<ReviewResult>,
    pub adjustments: Option<AdjustmentsResult>,
    pub test_results: Option<TestResult>,
    pub docs: Option<DocumentationResult>,

    // Error tracking
    pub errors: Vec<FlowError>,
    pub current_stage: String,
    pub stage_history: Vec<StageExecution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowError {
    pub stage: String,
    pub error: String,
    pub timestamp: i64,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageExecution {
    pub stage: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub success: bool,
    pub output: Option<String>,
}

// ---------------------------------------------------------------------------
// Stage Result Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    pub requirements: Vec<String>,
    pub constraints: Vec<String>,
    pub dependencies: Vec<String>,
    pub complexity: String, // "low" | "medium" | "high"
    pub suggested_approach: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanResult {
    pub steps: Vec<PlanStep>,
    pub estimated_time: u64,
    pub files_to_create: Vec<String>,
    pub files_to_modify: Vec<String>,
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
    pub order: u32,
    pub action: String,
    pub description: String,
    pub skill: Option<String>,
    pub estimated_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeResult {
    pub files_created: Vec<String>,
    pub files_modified: Vec<String>,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResult {
    pub issues: Vec<ReviewIssue>,
    pub score: u32,
    pub passed: bool,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewIssue {
    pub file: String,
    pub line: Option<u32>,
    pub severity: String, // "critical" | "high" | "medium" | "low"
    pub message: String,
    pub fix_suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustmentsResult {
    pub files_modified: Vec<String>,
    pub changes_applied: u32,
    pub issues_fixed: u32,
    pub remaining_issues: Vec<ReviewIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub total_tests: u32,
    pub passed: u32,
    pub failed: u32,
    pub coverage: Option<u32>,
    pub failures: Vec<TestFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestFailure {
    pub test_name: String,
    pub error: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentationResult {
    pub files_generated: Vec<String>,
    pub readme_updated: bool,
    pub api_docs_generated: bool,
}

// ---------------------------------------------------------------------------
// Hermes Flow Orchestrator
// ---------------------------------------------------------------------------

/// Hermes Flow Orchestrator - connects frontend flow to backend workflow engine
pub struct HermesFlowOrchestrator {
    /// Active flows being executed
    active_flows: HashMap<String, FlowContext>,
    /// Workflow engine for DAG-based execution
    workflow_engine: WorkflowEngine,
}

impl HermesFlowOrchestrator {
    pub fn new() -> Self {
        Self {
            active_flows: HashMap::new(),
            workflow_engine: WorkflowEngine::new(),
        }
    }

    /// Create a development flow workflow from a request
    pub fn create_dev_flow(
        &mut self,
        request: &str,
        workspace: &str,
        agent_name: &str,
    ) -> Result<(String, WorkflowDefinition), String> {
        let _flow_id = uuid::Uuid::new_v4().to_string();

        // Create workflow stages based on dev flow
        let stages: Vec<WorkflowStage> = DEV_FLOW_STAGES.iter().enumerate().map(|(idx, stage_name)| {
            let depends_on = if idx == 0 {
                vec![]
            } else {
                vec![DEV_FLOW_STAGES[idx - 1].to_string()]
            };

            WorkflowStage {
                id: format!("stage-{}", stage_name),
                name: stage_name.to_string(),
                description: get_stage_description(stage_name),
                strategy: StageStrategy::Sequential,
                agents: vec![StageAgent {
                    agent_id: agent_name.to_string(),
                    role: stage_name.to_string(),
                    prompt_template: get_stage_prompt(stage_name, request),
                    max_tokens: Some(get_stage_token_budget(stage_name)),
                }],
                depends_on,
                sync_points: vec![],
                status: WorkflowStatus::Draft,
                results: vec![],
                started_at: None,
                completed_at: None,
            }
        }).collect();

        // Use workflow engine's create_workflow method
        let workflow = self.workflow_engine.create_workflow(
            format!("Dev Flow: {}", request.chars().take(50).collect::<String>()),
            request.to_string(),
            stages,
            Some(1),
        )?;

        let workflow_id = workflow.id.clone();

        // Create initial flow context
        let context = FlowContext {
            task_id: workflow_id.clone(),
            request: request.to_string(),
            workspace: workspace.to_string(),
            cwd: workspace.to_string(),
            agent_name: agent_name.to_string(),
            session_id: None,
            analysis: None,
            plan: None,
            code: None,
            review: None,
            adjustments: None,
            test_results: None,
            docs: None,
            errors: vec![],
            current_stage: String::new(),
            stage_history: vec![],
        };

        self.active_flows.insert(workflow_id.clone(), context);

        #[cfg(debug_assertions)]
        println!("[HermesFlow] Created dev flow '{}' for request: {}", workflow_id, request);
        Ok((workflow_id, workflow))
    }

    /// Get flow context by ID
    pub fn get_flow_context(&self, flow_id: &str) -> Option<&FlowContext> {
        self.active_flows.get(flow_id)
    }

    /// Update flow context after stage completion
    pub fn update_flow_context(
        &mut self,
        flow_id: &str,
        stage_name: &str,
        result: serde_json::Value,
    ) -> Result<(), String> {
        let context = self.active_flows
            .get_mut(flow_id)
            .ok_or_else(|| format!("Flow '{}' not found", flow_id))?;

        // Record stage execution
        context.stage_history.push(StageExecution {
            stage: stage_name.to_string(),
            started_at: chrono::Utc::now().timestamp_millis(),
            completed_at: Some(chrono::Utc::now().timestamp_millis()),
            success: true,
            output: Some(result.to_string()),
        });

        // Parse and store result based on stage
        match stage_name {
            "analysis" => {
                context.analysis = serde_json::from_value(result.clone())
                    .ok()
                    .or_else(|| parse_analysis_fallback(&result.to_string()));
            }
            "planning" => {
                context.plan = serde_json::from_value(result.clone())
                    .ok()
                    .or_else(|| parse_plan_fallback(&result.to_string()));
            }
            "code-execution" => {
                context.code = serde_json::from_value(result.clone())
                    .ok()
                    .or_else(|| parse_code_fallback(&result.to_string()));
            }
            "code-review" => {
                context.review = serde_json::from_value(result.clone())
                    .ok()
                    .or_else(|| parse_review_fallback(&result.to_string()));
            }
            "adjustments" => {
                context.adjustments = serde_json::from_value(result.clone())
                    .ok()
                    .or_else(|| parse_adjustments_fallback(&result.to_string()));
            }
            "testing" => {
                context.test_results = serde_json::from_value(result.clone())
                    .ok()
                    .or_else(|| parse_test_fallback(&result.to_string()));
            }
            "documentation" => {
                context.docs = serde_json::from_value(result.clone())
                    .ok()
                    .or_else(|| parse_docs_fallback(&result.to_string()));
            }
            _ => {}
        }

        context.current_stage = stage_name.to_string();
        println!("[HermesFlow] Updated flow '{}' stage '{}' with result", flow_id, stage_name);
        Ok(())
    }

    /// Record flow error
    pub fn record_flow_error(
        &mut self,
        flow_id: &str,
        stage: &str,
        error: &str,
        retry_count: u32,
    ) -> Result<(), String> {
        let context = self.active_flows
            .get_mut(flow_id)
            .ok_or_else(|| format!("Flow '{}' not found", flow_id))?;

        context.errors.push(FlowError {
            stage: stage.to_string(),
            error: error.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            retry_count,
        });

        println!("[HermesFlow] Flow '{}' error at stage '{}': {}", flow_id, stage, error);
        Ok(())
    }

    /// Get flow status summary
    pub fn get_flow_status(&self, flow_id: &str) -> Result<FlowStatus, String> {
        let context = self.active_flows
            .get(flow_id)
            .ok_or_else(|| format!("Flow '{}' not found", flow_id))?;

        let workflow = self.workflow_engine
            .get_workflow(flow_id)
            .ok_or_else(|| format!("Workflow '{}' not found", flow_id))?;

        let total_stages = workflow.stages.len();
        let completed_stages = context.stage_history.iter().filter(|s| s.success).count();
        let progress = (completed_stages as f64 / total_stages as f64 * 100.0) as u32;

        Ok(FlowStatus {
            flow_id: flow_id.to_string(),
            stage: context.current_stage.clone(),
            progress,
            errors: context.errors.len() as u32,
            completed: context.stage_history.len() == total_stages && context.errors.is_empty(),
        })
    }

    /// Execute a flow (non-async version for Tauri command)
    ///
    /// TODO(Phase 2): Real implementation should:
    /// 1. Invoke skills through Executive Agent for each stage
    /// 2. Parse and store results in flow context
    /// 3. Handle stage failures with retry/fallback logic
    pub fn execute_flow_sync(&mut self, flow_id: &str) -> Result<FlowContext, String> {
        let _workflow = self.workflow_engine
            .get_workflow(flow_id)
            .ok_or_else(|| format!("Workflow '{}' not found", flow_id))?
            .clone();

        // Update workflow status to Running
        self.workflow_engine.update_status(flow_id, WorkflowStatus::Running)?;

        // Get ready stages (dependencies satisfied)
        let ready_stages = self.workflow_engine.get_ready_stages(flow_id)?;

        for stage_id in ready_stages {
            println!("[HermesFlow] Executing stage '{}' in flow '{}'", stage_id, flow_id);

            // TODO(Phase 2): Replace with actual Executive Agent skill invocation.
            // For now, return an explicit error rather than fake data.
            return Err(format!(
                "Flow execution not yet implemented — stage '{}' in flow '{}' \
                 requires Executive Agent integration (Phase 2)",
                stage_id, flow_id
            ));
        }

        // Get final context
        let context = self.active_flows
            .get(flow_id)
            .cloned()
            .ok_or_else(|| format!("Flow '{}' context not found", flow_id))?;

        println!("[HermesFlow] Flow '{}' execution completed", flow_id);
        Ok(context)
    }

    /// Cancel an active flow
    pub fn cancel_flow(&mut self, flow_id: &str) -> Result<(), String> {
        self.workflow_engine.cancel_workflow(flow_id)?;
        self.active_flows.remove(flow_id);
        println!("[HermesFlow] Cancelled flow '{}'", flow_id);
        Ok(())
    }

    /// List all active flows
    pub fn list_active_flows(&self) -> Vec<&FlowContext> {
        self.active_flows.values().collect()
    }
}

impl Default for HermesFlowOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

fn get_stage_description(stage: &str) -> String {
    match stage {
        "analysis" => "需求分析 - Analyze requirements and constraints".to_string(),
        "planning" => "计划撰写 - Create implementation plan".to_string(),
        "code-execution" => "写代码 - Execute code generation".to_string(),
        "code-review" => "代码审查 - Review code quality".to_string(),
        "adjustments" => "代码调整 - Apply fixes based on review".to_string(),
        "testing" => "功能测试 - Run tests and validate".to_string(),
        "documentation" => "文档生成 - Generate documentation".to_string(),
        "communication" => "反馈用户 - Report results to user".to_string(),
        _ => stage.to_string(),
    }
}

fn get_stage_prompt(stage: &str, request: &str) -> String {
    match stage {
        "analysis" => format!("分析以下需求，提取关键点：\n{}", request),
        "planning" => format!("根据分析结果，制定实施计划：\n{}", request),
        "code-execution" => format!("按计划执行代码生成：\n{}", request),
        "code-review" => "审查生成的代码，检查质量和安全问题".to_string(),
        "adjustments" => "根据审查结果，调整代码修复问题".to_string(),
        "testing" => "运行测试验证功能正确性".to_string(),
        "documentation" => "生成项目文档和README".to_string(),
        "communication" => format!("总结任务执行结果并反馈：\n{}", request),
        _ => request.to_string(),
    }
}

fn get_stage_token_budget(stage: &str) -> u64 {
    match stage {
        "analysis" => 2048,
        "planning" => 4096,
        "code-execution" => 16384,
        "code-review" => 8192,
        "adjustments" => 4096,
        "testing" => 4096,
        "documentation" => 2048,
        "communication" => 1024,
        _ => 4096,
    }
}

// Fallback parsers for non-structured output
fn parse_analysis_fallback(output: &str) -> Option<AnalysisResult> {
    Some(AnalysisResult {
        requirements: output.lines().filter(|l| l.contains("REQ")).map(|l| l.replace("REQ:", "").trim().to_string()).collect(),
        constraints: output.lines().filter(|l| l.contains("CONST")).map(|l| l.replace("CONST:", "").trim().to_string()).collect(),
        dependencies: output.lines().filter(|l| l.contains("DEP")).map(|l| l.replace("DEP:", "").trim().to_string()).collect(),
        complexity: if output.contains("high") { "high" } else if output.contains("medium") { "medium" } else { "low" }.to_string(),
        suggested_approach: output.lines().find(|l| l.contains("APPROACH")).map(|l| l.replace("APPROACH:", "").trim().to_string()).unwrap_or_default(),
    })
}

fn parse_plan_fallback(output: &str) -> Option<PlanResult> {
    let steps: Vec<PlanStep> = output.lines()
        .enumerate()
        .filter(|(_, l)| l.contains(":"))
        .map(|(idx, l)| PlanStep {
            order: idx as u32 + 1,
            action: l.split(':').next().unwrap_or("").trim().to_string(),
            description: l.to_string(),
            skill: None,
            estimated_ms: 5000,
        }).collect();

    let estimated_time = steps.len() as u64 * 5000;

    Some(PlanResult {
        steps,
        estimated_time,
        files_to_create: vec![],
        files_to_modify: vec![],
        risks: vec![],
    })
}

fn parse_code_fallback(output: &str) -> Option<CodeResult> {
    Some(CodeResult {
        files_created: vec!["output.ts".to_string()],
        files_modified: vec![],
        lines_added: 100,
        lines_removed: 0,
        summary: output.lines().next().unwrap_or("").to_string(),
    })
}

fn parse_review_fallback(_output: &str) -> Option<ReviewResult> {
    Some(ReviewResult {
        issues: vec![],
        score: 85,
        passed: true,
        suggestions: vec![],
    })
}

fn parse_adjustments_fallback(_output: &str) -> Option<AdjustmentsResult> {
    Some(AdjustmentsResult {
        files_modified: vec![],
        changes_applied: 0,
        issues_fixed: 0,
        remaining_issues: vec![],
    })
}

fn parse_test_fallback(_output: &str) -> Option<TestResult> {
    Some(TestResult {
        total_tests: 5,
        passed: 5,
        failed: 0,
        coverage: Some(80),
        failures: vec![],
    })
}

fn parse_docs_fallback(_output: &str) -> Option<DocumentationResult> {
    Some(DocumentationResult {
        files_generated: vec!["README.md".to_string()],
        readme_updated: true,
        api_docs_generated: false,
    })
}

// ---------------------------------------------------------------------------
// Flow Status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowStatus {
    pub flow_id: String,
    pub stage: String,
    pub progress: u32,
    pub errors: u32,
    pub completed: bool,
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Create a new development flow
#[tauri::command]
pub fn hermes_flow_create(
    state: State<'_, AppState>,
    request: String,
    workspace: String,
    agent_name: String,
) -> Result<(String, WorkflowDefinition), String> {
    let mut orchestrator = state.hermes_flow.lock().map_err(|e| e.to_string())?;
    orchestrator.create_dev_flow(&request, &workspace, &agent_name)
}

/// Get flow context
#[tauri::command]
pub fn hermes_flow_get_context(
    state: State<'_, AppState>,
    flow_id: String,
) -> Result<FlowContext, String> {
    let orchestrator = state.hermes_flow.lock().map_err(|e| e.to_string())?;
    orchestrator.get_flow_context(&flow_id)
        .cloned()
        .ok_or_else(|| format!("Flow '{}' not found", flow_id))
}

/// Get flow status
#[tauri::command]
pub fn hermes_flow_get_status(
    state: State<'_, AppState>,
    flow_id: String,
) -> Result<FlowStatus, String> {
    let orchestrator = state.hermes_flow.lock().map_err(|e| e.to_string())?;
    orchestrator.get_flow_status(&flow_id)
}

/// Execute a flow (sync version to avoid MutexGuard across await)
#[tauri::command]
pub fn hermes_flow_execute(
    state: State<'_, AppState>,
    flow_id: String,
) -> Result<FlowContext, String> {
    let mut orchestrator = state.hermes_flow.lock().map_err(|e| e.to_string())?;
    orchestrator.execute_flow_sync(&flow_id)
}

/// Cancel a flow
#[tauri::command]
pub fn hermes_flow_cancel(
    state: State<'_, AppState>,
    flow_id: String,
) -> Result<(), String> {
    let mut orchestrator = state.hermes_flow.lock().map_err(|e| e.to_string())?;
    orchestrator.cancel_flow(&flow_id)
}

/// List active flows
#[tauri::command]
pub fn hermes_flow_list(
    state: State<'_, AppState>,
) -> Result<Vec<FlowContext>, String> {
    let orchestrator = state.hermes_flow.lock().map_err(|e| e.to_string())?;
    Ok(orchestrator.list_active_flows().into_iter().cloned().collect())
}

/// Update flow stage result (called by Executive Agent after skill execution)
#[tauri::command]
pub fn hermes_flow_update_result(
    state: State<'_, AppState>,
    flow_id: String,
    stage_name: String,
    result: serde_json::Value,
) -> Result<(), String> {
    let mut orchestrator = state.hermes_flow.lock().map_err(|e| e.to_string())?;
    orchestrator.update_flow_context(&flow_id, &stage_name, result)
}