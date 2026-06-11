//! Hook registry - Hook注册表管理

use crate::types::{Hook, HookType};
use std::collections::HashMap;

/// HookRegistry - 全局Hook注册表
pub struct HookRegistry {
    /// 全局Hooks（对所有Agent生效）
    global_hooks: HashMap<String, Hook>,
    /// Agent专属Hooks
    agent_hooks: HashMap<String, HashMap<String, Hook>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            global_hooks: HashMap::new(),
            agent_hooks: HashMap::new(),
        }
    }

    /// 注册全局Hook
    pub fn register_global(&mut self, hook: Hook) {
        self.global_hooks.insert(hook.id.clone(), hook);
    }

    /// 取消注册全局Hook
    pub fn unregister_global(&mut self, hook_id: &str) {
        self.global_hooks.remove(hook_id);
    }

    /// 为特定Agent注册Hook
    pub fn register_for_agent(&mut self, agent_id: &str, hook: Hook) {
        self.agent_hooks
            .entry(agent_id.to_string())
            .or_default()
            .insert(hook.id.clone(), hook);
    }

    /// 取消Agent的Hook
    pub fn unregister_agent_hook(&mut self, agent_id: &str, hook_id: &str) {
        if let Some(hooks) = self.agent_hooks.get_mut(agent_id) {
            hooks.remove(hook_id);
        }
    }

    /// 取消Agent的所有Hooks
    pub fn unregister_agent_hooks(&mut self, agent_id: &str) {
        self.agent_hooks.remove(agent_id);
    }

    /// 获取全局Hooks
    pub fn get_global_hooks(&self) -> Vec<&Hook> {
        self.global_hooks.values().collect()
    }

    /// 获取Agent的Hooks
    pub fn get_agent_hooks(&self, agent_id: &str) -> Vec<&Hook> {
        self.agent_hooks
            .get(agent_id)
            .map(|h| h.values().collect())
            .unwrap_or_default()
    }

    /// 获取特定类型的所有Hooks（全局+Agent）
    pub fn get_hooks_by_type(&self, hook_type: HookType, agent_id: Option<&str>) -> Vec<&Hook> {
        let mut hooks: Vec<&Hook> = self.global_hooks
            .values()
            .filter(|h| h.hook_type == hook_type)
            .collect();

        if let Some(id) = agent_id {
            if let Some(agent_hooks) = self.agent_hooks.get(id) {
                hooks.extend(
                    agent_hooks
                        .values()
                        .filter(|h| h.hook_type == hook_type)
                );
            }
        }

        hooks
    }

    /// 列出所有Hooks
    pub fn list(&self) -> Vec<&Hook> {
        let mut hooks: Vec<&Hook> = self.global_hooks.values().collect();
        for agent_hooks in self.agent_hooks.values() {
            hooks.extend(agent_hooks.values());
        }
        hooks
    }

    /// 启用/禁用Hook
    pub fn set_enabled(&mut self, hook_id: &str, enabled: bool) {
        if let Some(hook) = self.global_hooks.get_mut(hook_id) {
            hook.enabled = enabled;
        }
        for agent_hooks in self.agent_hooks.values_mut() {
            if let Some(hook) = agent_hooks.get_mut(hook_id) {
                hook.enabled = enabled;
            }
        }
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}