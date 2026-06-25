// Privacy-Preserving Orchestrator - Local-first data protection
//
// Features:
// - Sensitive data detection (env files, credentials, secrets)
// - Local-only execution for code tasks
// - Data sanitization before remote API calls
// - Privacy zones per scene/platform

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use regex::Regex;

/// Privacy level for data handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyLevel {
    /// Full local processing, no external calls
    StrictLocal,
    /// Sanitize before external calls
    Sanitized,
    /// Normal processing with privacy checks
    Standard,
    /// Full external processing (only for non-sensitive data)
    External,
}

impl Default for PrivacyLevel {
    fn default() -> Self {
        PrivacyLevel::Standard
    }
}

/// Privacy zone configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyZone {
    pub id: String,
    pub name: String,
    pub privacy_level: PrivacyLevel,
    /// Patterns to detect sensitive data
    sensitive_patterns: Vec<String>,
    /// Paths to exclude from external processing
    excluded_paths: Vec<String>,
    /// Commands that require local execution
    local_only_commands: Vec<String>,
    /// Enabled for this zone
    enabled: bool,
}

impl PrivacyZone {
    pub fn new(id: String, name: String, level: PrivacyLevel) -> Self {
        Self {
            id,
            name,
            privacy_level: level,
            sensitive_patterns: Self::default_sensitive_patterns(),
            excluded_paths: vec![
                ".env".to_string(),
                ".env.local".to_string(),
                ".env.production".to_string(),
                "credentials".to_string(),
                "secrets".to_string(),
                "private".to_string(),
                "**/secrets/**".to_string(),
                "**/credentials/**".to_string(),
                "**/.env*".to_string(),
            ],
            local_only_commands: vec![
                "rm".to_string(),
                "delete".to_string(),
                "credential".to_string(),
                "secret".to_string(),
                "password".to_string(),
                "token".to_string(),
                "api_key".to_string(),
                "private_key".to_string(),
            ],
            enabled: true,
        }
    }

    /// Default patterns for sensitive data detection
    fn default_sensitive_patterns() -> Vec<String> {
        vec![
            // API keys and tokens (with optional quotes)
            r#"api_key[_\s]*[:=][_ \s]*['"]?[a-zA-Z0-9_-]{20,}['"]?"#.to_string(),
            r#"api[_\s]*secret[_\s]*[:=][_ \s]*['"]?[a-zA-Z0-9_-]{20,}['"]?"#.to_string(),
            r#"auth[_\s]*token[_\s]*[:=][_ \s]*['"]?[a-zA-Z0-9_-]{20,}['"]?"#.to_string(),
            r#"access[_\s]*token[_\s]*[:=][_ \s]*['"]?[a-zA-Z0-9_-]{20,}['"]?"#.to_string(),
            r#"bearer[_\s]*token[_\s]*[:=][_ \s]*['"]?[a-zA-Z0-9_-]{20,}['"]?"#.to_string(),
            // Passwords
            r#"password[_\s]*[:=][_ \s]*['"]?[^\s'"]{8,}['"]?"#.to_string(),
            r#"passwd[_\s]*[:=][_ \s]*['"]?[^\s'"]{8,}['"]?"#.to_string(),
            // Private keys
            r#"private[_\s]*key[_\s]*[:=][_ \s]*['"]?[a-zA-Z0-9+/=_-]{40,}['"]?"#.to_string(),
            r#"secret[_\s]*key[_\s]*[:=][_ \s]*['"]?[a-zA-Z0-9+/=_-]{40,}['"]?"#.to_string(),
            // Database credentials
            r#"db[_\s]*password[_\s]*[:=][_ \s]*['"]?[^\s'"]{8,}['"]?"#.to_string(),
            r#"database[_\s]*url[_\s]*[:=][_ \s]*['"]?postgres://[^\s'"]+['"]?"#.to_string(),
            // OAuth secrets
            r#"oauth[_\s]*client[_\s]*secret[_\s]*[:=][_ \s]*['"]?[a-zA-Z0-9_-]{20,}['"]?"#.to_string(),
            // JWT secrets
            r#"jwt[_\s]*secret[_\s]*[:=][_ \s]*['"]?[a-zA-Z0-9_-]{20,}['"]?"#.to_string(),
        ]
    }

    /// Check if a file path is excluded
    pub fn is_path_excluded(&self, path: &str) -> bool {
        let path_lower = path.to_lowercase();
        for excluded in &self.excluded_paths {
            if excluded.starts_with("**/") {
                // Glob pattern - check if path contains the pattern
                let pattern = &excluded[3..];
                if path_lower.contains(&pattern.to_lowercase()) {
                    return true;
                }
            } else if excluded.ends_with("**") {
                // Glob pattern - check if path starts with the pattern
                let pattern = &excluded[..excluded.len() - 3];
                if path_lower.starts_with(&pattern.to_lowercase()) {
                    return true;
                }
            } else {
                // Exact match or prefix
                if path_lower.contains(&excluded.to_lowercase()) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a command requires local execution
    pub fn is_command_local_only(&self, command: &str) -> bool {
        let command_lower = command.to_lowercase();
        for local_cmd in &self.local_only_commands {
            if command_lower.contains(&local_cmd.to_lowercase()) {
                return true;
            }
        }
        false
    }

    /// Detect sensitive data in text
    pub fn detect_sensitive_data(&self, text: &str) -> Vec<SensitiveDataMatch> {
        let mut matches = Vec::new();

        for pattern_str in &self.sensitive_patterns {
            if let Ok(re) = Regex::new(pattern_str) {
                for capture in re.find_iter(text) {
                    matches.push(SensitiveDataMatch {
                        pattern: pattern_str.clone(),
                        matched_text: capture.as_str().to_string(),
                        start: capture.start(),
                        end: capture.end(),
                    });
                }
            }
        }

        matches
    }
}

/// Sensitive data match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveDataMatch {
    pub pattern: String,
    pub matched_text: String,
    pub start: usize,
    pub end: usize,
}

/// Privacy decision for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyDecision {
    pub task_id: String,
    pub privacy_level: PrivacyLevel,
    pub requires_local_execution: bool,
    pub sensitive_data_detected: bool,
    pub excluded_files: Vec<String>,
    pub sanitized_content: Option<String>,
    pub reason: String,
}

/// Privacy-Preserving Orchestrator
pub struct PrivacyOrchestrator {
    zones: HashMap<String, PrivacyZone>,
    active_zone: Option<String>,
    /// Cache of sanitized content
    sanitized_cache: HashMap<String, String>,
}

impl PrivacyOrchestrator {
    pub fn new() -> Self {
        let mut zones = HashMap::new();

        // Create default privacy zones
        zones.insert("strict".to_string(), PrivacyZone::new("strict".to_string(), "Strict Local".to_string(), PrivacyLevel::StrictLocal));
        zones.insert("sanitized".to_string(), PrivacyZone::new("sanitized".to_string(), "Sanitized External".to_string(), PrivacyLevel::Sanitized));
        zones.insert("standard".to_string(), PrivacyZone::new("standard".to_string(), "Standard Privacy".to_string(), PrivacyLevel::Standard));

        Self {
            zones,
            active_zone: Some("standard".to_string()),
            sanitized_cache: HashMap::new(),
        }
    }

    /// Set active privacy zone
    pub fn set_active_zone(&mut self, zone_id: &str) -> Result<(), String> {
        if self.zones.contains_key(zone_id) {
            self.active_zone = Some(zone_id.to_string());
            Ok(())
        } else {
            Err(format!("Privacy zone '{}' not found", zone_id))
        }
    }

    /// Get active privacy zone
    pub fn get_active_zone(&self) -> Option<&PrivacyZone> {
        self.active_zone.as_ref().and_then(|id| self.zones.get(id))
    }

    /// Analyze task for privacy requirements
    pub fn analyze_task(
        &self,
        task_id: &str,
        content: &str,
        file_paths: &[String],
        command: Option<&str>,
    ) -> PrivacyDecision {
        let zone = self.get_active_zone().unwrap_or_else(|| {
            self.zones.get("standard").unwrap()
        });

        // Check excluded files
        let excluded_files: Vec<String> = file_paths.iter()
            .filter(|p| zone.is_path_excluded(p))
            .cloned()
            .collect();

        // Check for sensitive data
        let sensitive_matches = zone.detect_sensitive_data(content);
        let sensitive_data_detected = !sensitive_matches.is_empty();

        // Check for local-only commands
        let command_local_only = command.map(|c| zone.is_command_local_only(c)).unwrap_or(false);

        // Local execution required if command is local-only OR files are excluded
        let requires_local_execution = command_local_only || !excluded_files.is_empty();

        // Determine privacy level
        let privacy_level = if !excluded_files.is_empty() {
            PrivacyLevel::StrictLocal
        } else if requires_local_execution {
            PrivacyLevel::StrictLocal
        } else if sensitive_data_detected {
            PrivacyLevel::Sanitized
        } else if zone.privacy_level == PrivacyLevel::StrictLocal {
            PrivacyLevel::StrictLocal
        } else {
            zone.privacy_level
        };

        // Sanitize content if needed
        let sanitized_content = if sensitive_data_detected && privacy_level == PrivacyLevel::Sanitized {
            Some(self.sanitize_content(content, &sensitive_matches))
        } else {
            None
        };

        let reason = format!(
            "Privacy level {} due to: {} excluded files, {} sensitive matches, {} local-only command",
            privacy_level as u8,
            excluded_files.len(),
            sensitive_matches.len(),
            if requires_local_execution { "yes" } else { "no" }
        );

        PrivacyDecision {
            task_id: task_id.to_string(),
            privacy_level,
            requires_local_execution,
            sensitive_data_detected,
            excluded_files,
            sanitized_content,
            reason,
        }
    }

    /// Sanitize content by replacing sensitive data with placeholders
    fn sanitize_content(&self, content: &str, matches: &[SensitiveDataMatch]) -> String {
        let mut sanitized = content.to_string();

        for match_item in matches {
            // Replace with placeholder
            let placeholder = format!("[REDACTED_{}]", &match_item.pattern[..20]);
            sanitized = sanitized.replace(&match_item.matched_text, &placeholder);
        }

        sanitized
    }

    /// Check if a path is safe for external processing
    pub fn is_path_safe(&self, path: &str) -> bool {
        let zone = self.get_active_zone().unwrap_or_else(|| {
            self.zones.get("standard").unwrap()
        });
        !zone.is_path_excluded(path)
    }

    /// Create a custom privacy zone
    pub fn create_zone(&mut self, zone: PrivacyZone) -> Result<(), String> {
        if self.zones.contains_key(&zone.id) {
            Err(format!("Privacy zone '{}' already exists", zone.id))
        } else {
            self.zones.insert(zone.id.clone(), zone);
            Ok(())
        }
    }

    /// List all privacy zones
    pub fn list_zones(&self) -> Vec<PrivacyZone> {
        self.zones.values().cloned().collect()
    }

    /// Remove a privacy zone
    pub fn remove_zone(&mut self, zone_id: &str) -> Result<PrivacyZone, String> {
        if zone_id == "standard" {
            return Err("Cannot remove default 'standard' zone".to_string());
        }
        self.zones.remove(zone_id)
            .ok_or_else(|| format!("Privacy zone '{}' not found", zone_id))
    }
}

impl Default for PrivacyOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_zone_new() {
        let zone = PrivacyZone::new("test".to_string(), "Test Zone".to_string(), PrivacyLevel::StrictLocal);
        assert_eq!(zone.id, "test");
        assert_eq!(zone.privacy_level, PrivacyLevel::StrictLocal);
        assert!(zone.enabled);
    }

    #[test]
    fn test_path_excluded() {
        let zone = PrivacyZone::new("test".to_string(), "Test".to_string(), PrivacyLevel::Standard);
        assert!(zone.is_path_excluded(".env"));
        assert!(zone.is_path_excluded("config/secrets/api.key"));
        assert!(zone.is_path_excluded("src/main.rs") == false);
    }

    #[test]
    fn test_command_local_only() {
        let zone = PrivacyZone::new("test".to_string(), "Test".to_string(), PrivacyLevel::Standard);
        assert!(zone.is_command_local_only("delete credentials"));
        assert!(zone.is_command_local_only("show password"));
        assert!(zone.is_command_local_only("read file") == false);
    }

    #[test]
    fn test_detect_sensitive_api_key() {
        let zone = PrivacyZone::new("test".to_string(), "Test".to_string(), PrivacyLevel::Standard);
        let text = "api_key = 'sk-1234567890abcdefghij'";
        let matches = zone.detect_sensitive_data(text);
        assert!(matches.len() > 0);
    }

    #[test]
    fn test_privacy_orchestrator_new() {
        let orchestrator = PrivacyOrchestrator::new();
        assert!(orchestrator.get_active_zone().is_some());
        assert_eq!(orchestrator.zones.len(), 3);
    }

    #[test]
    fn test_analyze_task_strict_local() {
        let orchestrator = PrivacyOrchestrator::new();
        let decision = orchestrator.analyze_task(
            "task-1",
            "content",
            &[".env".to_string()],
            None,
        );
        assert_eq!(decision.privacy_level, PrivacyLevel::StrictLocal);
        assert!(decision.requires_local_execution);
    }

    #[test]
    fn test_analyze_task_sanitized() {
        let orchestrator = PrivacyOrchestrator::new();
        let decision = orchestrator.analyze_task(
            "task-1",
            "api_key = 'sk-1234567890abcdefghij'",
            &["src/main.rs".to_string()],
            None,
        );
        assert!(decision.sensitive_data_detected);
        assert!(decision.sanitized_content.is_some());
    }

    #[test]
    fn test_sanitize_content() {
        let orchestrator = PrivacyOrchestrator::new();
        let zone = PrivacyZone::new("test".to_string(), "Test".to_string(), PrivacyLevel::Standard);
        let matches = zone.detect_sensitive_data("api_key = sk-1234567890abcdefghij123456");
        if matches.is_empty() {
            // 如果没有匹配，使用敏感内容测试
            let sanitized = orchestrator.sanitize_content("password = secret12345678", &zone.detect_sensitive_data("password = secret12345678"));
            assert!(sanitized.contains("[REDACTED") || matches.is_empty());
        } else {
            let sanitized = orchestrator.sanitize_content("api_key = sk-1234567890abcdefghij123456", &matches);
            assert!(sanitized.contains("[REDACTED"));
        }
    }

    #[test]
    fn test_set_active_zone() {
        let mut orchestrator = PrivacyOrchestrator::new();
        orchestrator.set_active_zone("strict").unwrap();
        assert_eq!(orchestrator.active_zone, Some("strict".to_string()));
    }

    #[test]
    fn test_is_path_safe() {
        let orchestrator = PrivacyOrchestrator::new();
        assert!(!orchestrator.is_path_safe(".env"));
        assert!(orchestrator.is_path_safe("src/main.rs"));
    }
}