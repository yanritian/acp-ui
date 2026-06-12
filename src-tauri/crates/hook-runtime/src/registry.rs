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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Hook;

    #[test]
    fn registry_new_is_empty() {
        let registry = HookRegistry::new();
        assert!(registry.get_global_hooks().is_empty());
        assert!(registry.list().is_empty());
    }

    #[test]
    fn registry_register_global() {
        let mut registry = HookRegistry::new();
        let hook = Hook::new("hook-1", HookType::PreToolUse, "echo");
        registry.register_global(hook);
        assert_eq!(registry.get_global_hooks().len(), 1);
    }

    #[test]
    fn registry_get_hooks_by_type() {
        let mut registry = HookRegistry::new();
        let pre_hook = Hook::new("pre-1", HookType::PreToolUse, "echo");
        let post_hook = Hook::new("post-1", HookType::PostToolUse, "echo");
        registry.register_global(pre_hook);
        registry.register_global(post_hook);

        let pre_hooks = registry.get_hooks_by_type(HookType::PreToolUse, None);
        assert_eq!(pre_hooks.len(), 1);

        let post_hooks = registry.get_hooks_by_type(HookType::PostToolUse, None);
        assert_eq!(post_hooks.len(), 1);
    }

    #[test]
    fn registry_register_for_agent() {
        let mut registry = HookRegistry::new();
        let hook = Hook::new("agent-hook", HookType::PreToolUse, "echo");
        registry.register_for_agent("agent-1", hook);
        assert_eq!(registry.get_agent_hooks("agent-1").len(), 1);
    }

    #[test]
    fn registry_set_enabled() {
        let mut registry = HookRegistry::new();
        let hook = Hook::new("hook-1", HookType::PreToolUse, "echo");
        registry.register_global(hook);
        registry.set_enabled("hook-1", false);
        let hooks = registry.get_global_hooks();
        assert!(!hooks[0].enabled);
    }
}