//! Hook types - Hook扩展类型定义
//!
//! 根据执行计划定义的12种Hook类型：
//! - PreToolUse: Tool执行前
//! - PostToolUse: Tool执行后
//! - PostToolUseFailure: Tool执行失败后
//! - Stop: 会话结束时
//! - 以及更多扩展类型

use serde::{Deserialize, Serialize};

/// Hook类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookType {
    // === 基础Hook类型 ===
    /// Tool执行前
    PreToolUse,
    /// Tool执行后
    PostToolUse,
    /// Tool执行失败后
    PostToolUseFailure,
    /// 会话结束
    Stop,

    // === 扩展Hook类型（12种） ===
    /// Goal提交时
    GoalSubmitted,
    /// Goal激活时
    GoalActive,
    /// Goal评估时
    GoalEvaluating,
    /// Goal收敛时
    GoalConverged,
    /// Goal失败时
    GoalFailed,
    /// Worker注册时
    WorkerRegistered,
    /// Worker断开时
    WorkerDisconnected,
    /// Queen选举时
    QueenElected,
    /// 命令批准时
    CommandApproved,
    /// 命令拒绝时
    CommandRejected,
    /// 文件修改时
    FileModified,
    /// 测试运行时
    TestRun,
}

/// Hook定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    /// Hook ID
    pub id: String,
    /// Hook类型
    pub hook_type: HookType,
    /// Hook名称
    pub name: String,
    /// 执行命令
    pub command: String,
    /// 是否启用
    pub enabled: bool,
    /// 优先级（数字越小优先级越高）
    pub priority: u32,
    /// 条件过滤
    pub conditions: Option<HookConditions>,
}

/// Hook条件过滤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConditions {
    /// 匹配的工具名称
    pub tool_names: Option<Vec<String>>,
    /// 匹配的Goal ID
    pub goal_ids: Option<Vec<String>>,
    /// 匹配的Worker ID
    pub worker_ids: Option<Vec<String>>,
    /// 匹配的文件路径模式
    pub file_patterns: Option<Vec<String>>,
}

impl Hook {
    pub fn new(id: impl Into<String>, hook_type: HookType, command: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            hook_type,
            name: String::new(),
            command: command.into(),
            enabled: true,
            priority: 100,
            conditions: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_conditions(mut self, conditions: HookConditions) -> Self {
        self.conditions = Some(conditions);
        self
    }
}

/// Hook执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Hook ID
    pub hook_id: String,
    /// 是否成功
    pub success: bool,
    /// 输出内容
    pub output: Option<String>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
    /// 是否应该阻止后续操作
    pub should_block: bool,
}

impl HookResult {
    pub fn success(hook_id: impl Into<String>, output: Option<String>) -> Self {
        Self {
            hook_id: hook_id.into(),
            success: true,
            output,
            error: None,
            duration_ms: 0,
            should_block: false,
        }
    }

    pub fn failure(hook_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            hook_id: hook_id.into(),
            success: false,
            output: None,
            error: Some(error.into()),
            duration_ms: 0,
            should_block: false,
        }
    }

    pub fn block(hook_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            hook_id: hook_id.into(),
            success: false,
            output: None,
            error: Some(reason.into()),
            duration_ms: 0,
            should_block: true,
        }
    }
}