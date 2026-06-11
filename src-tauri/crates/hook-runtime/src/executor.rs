//! Hook executor - Hook执行器

use crate::types::{Hook, HookResult, HookType};
use std::collections::HashMap;
use std::process::Command;

/// HookExecutor - 执行注册的Hooks
pub struct HookExecutor {
    hooks: HashMap<String, Hook>,
}

impl HookExecutor {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// 注册Hook
    pub fn register(&mut self, hook: Hook) {
        self.hooks.insert(hook.id.clone(), hook);
    }

    /// 取消注册Hook
    pub fn unregister(&mut self, hook_id: &str) {
        self.hooks.remove(hook_id);
    }

    /// 执行特定类型的Hooks
    pub fn execute_hooks(&self, hook_type: HookType, context: &HookContext) -> Vec<HookResult> {
        let mut results = Vec::new();

        // 按优先级排序执行
        let mut hooks_to_execute: Vec<&Hook> = self.hooks
            .values()
            .filter(|h| h.hook_type == hook_type && h.enabled)
            .collect();

        // 按优先级排序
        hooks_to_execute.sort_by(|a, b| a.priority.cmp(&b.priority));

        for hook in hooks_to_execute {
            // 检查条件是否匹配
            if !self.matches_conditions(hook, context) {
                continue;
            }

            let result = self.execute_single_hook(hook, context);
            let should_block = result.should_block;
            results.push(result);

            // 如果Hook要求阻止后续操作，停止执行
            if should_block {
                break;
            }
        }

        results
    }

    /// 执行单个Hook
    fn execute_single_hook(&self, hook: &Hook, _context: &HookContext) -> HookResult {
        let start = std::time::Instant::now();

        let output = Command::new(&hook.command)
            .output();

        let duration_ms = start.elapsed().as_millis() as u64;

        match output {
            Ok(o) => {
                if o.status.success() {
                    HookResult {
                        hook_id: hook.id.clone(),
                        success: true,
                        output: Some(String::from_utf8_lossy(&o.stdout).to_string()),
                        error: None,
                        duration_ms,
                        should_block: false,
                    }
                } else {
                    HookResult {
                        hook_id: hook.id.clone(),
                        success: false,
                        output: None,
                        error: Some(String::from_utf8_lossy(&o.stderr).to_string()),
                        duration_ms,
                        should_block: false,
                    }
                }
            }
            Err(e) => HookResult::failure(hook.id.clone(), e.to_string()),
        }
    }

    /// 检查条件是否匹配
    fn matches_conditions(&self, hook: &Hook, context: &HookContext) -> bool {
        if let Some(conditions) = &hook.conditions {
            // 检查工具名称
            if let Some(tool_names) = &conditions.tool_names {
                if !tool_names.contains(&context.tool_name) {
                    return false;
                }
            }

            // 检查Goal ID
            if let Some(goal_ids) = &conditions.goal_ids {
                if let Some(goal_id) = &context.goal_id {
                    if !goal_ids.contains(goal_id) {
                        return false;
                    }
                }
            }

            // 检查Worker ID
            if let Some(worker_ids) = &conditions.worker_ids {
                if let Some(worker_id) = &context.worker_id {
                    if !worker_ids.contains(worker_id) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// 获取所有Hooks
    pub fn list_hooks(&self) -> Vec<&Hook> {
        self.hooks.values().collect()
    }

    /// 获取特定类型的Hooks
    pub fn get_hooks_by_type(&self, hook_type: HookType) -> Vec<&Hook> {
        self.hooks
            .values()
            .filter(|h| h.hook_type == hook_type)
            .collect()
    }
}

impl Default for HookExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// HookContext - Hook执行上下文
#[derive(Debug, Clone)]
pub struct HookContext {
    pub tool_name: String,
    pub goal_id: Option<String>,
    pub worker_id: Option<String>,
    pub file_path: Option<String>,
    pub command: Option<String>,
}

impl HookContext {
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            goal_id: None,
            worker_id: None,
            file_path: None,
            command: None,
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
}