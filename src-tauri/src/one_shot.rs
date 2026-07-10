// One-Shot Interface - Unified single entry point for all agent operations
//
// Phase 5: One-Shot Interface + User Role Detection
// Automatically detects user intent and routes to appropriate agent

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One-Shot request - single unified input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneShotRequest {
    /// User input (text, file path, URL, etc.)
    pub input: String,
    /// Optional context hint
    pub context_hint: Option<String>,
    /// Optional preferred agent
    pub preferred_agent: Option<String>,
    /// Budget constraint
    pub max_cost: Option<f32>,
    /// Timeout constraint
    pub timeout_ms: Option<u64>,
}

/// One-Shot response - unified output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneShotResponse {
    /// Detected user role
    pub detected_role: UserRole,
    /// Detected scene
    pub detected_scene: String,
    /// Selected agent
    pub selected_agent: String,
    /// Selection reason
    pub selection_reason: String,
    /// Execution result
    pub result: Option<OneShotResult>,
    /// Cost
    pub cost: f32,
    /// Duration
    pub duration_ms: u64,
    /// Transparency info
    pub transparency: TransparencyInfo,
}

/// Detected user role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    Developer,
    Marketer,
    Designer,
    Finance,
    Gamer,
    Writer,
    Analyst,
    General,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Developer => "developer",
            UserRole::Marketer => "marketer",
            UserRole::Designer => "designer",
            UserRole::Finance => "finance",
            UserRole::Gamer => "gamer",
            UserRole::Writer => "writer",
            UserRole::Analyst => "analyst",
            UserRole::General => "general",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "developer" | "程序员" | "dev" => UserRole::Developer,
            "marketer" | "营销" | "marketing" => UserRole::Marketer,
            "designer" | "设计师" | "设计" => UserRole::Designer,
            "finance" | "财务" | "会计" => UserRole::Finance,
            "gamer" | "游戏" | "游戏开发" => UserRole::Gamer,
            "writer" | "作者" | "文案" => UserRole::Writer,
            "analyst" | "分析" | "分析师" => UserRole::Analyst,
            _ => UserRole::General,
        }
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneShotResult {
    /// Output type
    pub output_type: OutputType,
    /// Output content
    pub content: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputType {
    Text,
    File,
    Url,
    Image,
    Video,
    Document,
    Data,
}

/// Transparency info for decision explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyInfo {
    /// Agent selection DAG visualization
    pub dag_visualization: String,
    /// Execution预估
    pub estimated_cost: f32,
    /// Estimated duration
    pub estimated_duration_ms: u64,
    /// Agent proficiency scores considered
    pub proficiency_scores: HashMap<String, f32>,
    /// Privacy decision
    pub privacy_level: String,
    /// Budget status
    pub budget_status: String,
}

/// User Role Detector
pub struct UserRoleDetector {
    /// Keyword patterns per role
    patterns: HashMap<UserRole, Vec<String>>,
}

impl UserRoleDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    fn default_patterns() -> HashMap<UserRole, Vec<String>> {
        HashMap::from([
            (
                UserRole::Developer,
                vec![
                    "代码".to_string(),
                    "编程".to_string(),
                    "bug".to_string(),
                    "函数".to_string(),
                    "模块".to_string(),
                    "API".to_string(),
                    "框架".to_string(),
                    "开发".to_string(),
                    "实现".to_string(),
                    "重构".to_string(),
                    "测试".to_string(),
                    "部署".to_string(),
                    "build".to_string(),
                    "compile".to_string(),
                    "rust".to_string(),
                    "typescript".to_string(),
                    "python".to_string(),
                    "go".to_string(),
                    "java".to_string(),
                    "vue".to_string(),
                    "react".to_string(),
                ],
            ),
            (
                UserRole::Marketer,
                vec![
                    "文案".to_string(),
                    "营销".to_string(),
                    "推广".to_string(),
                    "广告".to_string(),
                    "宣传".to_string(),
                    "品牌".to_string(),
                    "活动".to_string(),
                    "抖音".to_string(),
                    "快手".to_string(),
                    "小红书".to_string(),
                    "微信".to_string(),
                    "视频".to_string(),
                    "直播".to_string(),
                    "内容".to_string(),
                    "copywriting".to_string(),
                    "marketing".to_string(),
                    "campaign".to_string(),
                    "promotion".to_string(),
                ],
            ),
            (
                UserRole::Designer,
                vec![
                    "设计".to_string(),
                    "UI".to_string(),
                    "UX".to_string(),
                    "界面".to_string(),
                    "图标".to_string(),
                    "logo".to_string(),
                    "海报".to_string(),
                    "图片".to_string(),
                    "配色".to_string(),
                    "布局".to_string(),
                    "样式".to_string(),
                    "原型".to_string(),
                    "mockup".to_string(),
                    "design".to_string(),
                    "figma".to_string(),
                    "sketch".to_string(),
                    "photoshop".to_string(),
                ],
            ),
            (
                UserRole::Finance,
                vec![
                    "财务".to_string(),
                    "报表".to_string(),
                    "账目".to_string(),
                    "预算".to_string(),
                    "成本".to_string(),
                    "利润".to_string(),
                    "收入".to_string(),
                    "Excel".to_string(),
                    "表格".to_string(),
                    "数据".to_string(),
                    "统计".to_string(),
                    "分析".to_string(),
                    "审计".to_string(),
                    "WPS".to_string(),
                    "文档".to_string(),
                    "合同".to_string(),
                    "invoice".to_string(),
                    "budget".to_string(),
                ],
            ),
            (
                UserRole::Gamer,
                vec![
                    "游戏".to_string(),
                    "Unity".to_string(),
                    "Godot".to_string(),
                    "Unreal".to_string(),
                    "场景".to_string(),
                    "角色".to_string(),
                    "动画".to_string(),
                    "资源".to_string(),
                    "精灵".to_string(),
                    "模型".to_string(),
                    "关卡".to_string(),
                    "游戏开发".to_string(),
                    "game".to_string(),
                    "sprite".to_string(),
                    "asset".to_string(),
                    "level".to_string(),
                    "character".to_string(),
                ],
            ),
            (
                UserRole::Writer,
                vec![
                    "写作".to_string(),
                    "文章".to_string(),
                    "小说".to_string(),
                    "故事".to_string(),
                    "博客".to_string(),
                    "翻译".to_string(),
                    "摘要".to_string(),
                    "编辑".to_string(),
                    "润色".to_string(),
                    "改写".to_string(),
                    "story".to_string(),
                    "article".to_string(),
                    "blog".to_string(),
                    "translate".to_string(),
                    "summary".to_string(),
                ],
            ),
            (
                UserRole::Analyst,
                vec![
                    "分析".to_string(),
                    "数据".to_string(),
                    "统计".to_string(),
                    "图表".to_string(),
                    "报告".to_string(),
                    "趋势".to_string(),
                    "预测".to_string(),
                    "KPI".to_string(),
                    "指标".to_string(),
                    "dashboard".to_string(),
                    "analytics".to_string(),
                    "data".to_string(),
                    "visualization".to_string(),
                    "chart".to_string(),
                    "report".to_string(),
                ],
            ),
        ])
    }

    /// Detect user role from input
    pub fn detect(&self, input: &str) -> UserRole {
        let input_lower = input.to_lowercase();
        let mut scores: HashMap<UserRole, u32> = HashMap::new();

        for (role, patterns) in &self.patterns {
            let count = patterns
                .iter()
                .filter(|p| input_lower.contains(&p.to_lowercase()))
                .count() as u32;
            scores.insert(*role, count);
        }

        // Find role with highest score
        let max_score = scores.values().copied().max().unwrap_or(0);

        // If no patterns matched, return General
        if max_score == 0 {
            return UserRole::General;
        }

        scores
            .iter()
            .filter(|(_, count)| **count == max_score)
            .map(|(role, _)| *role)
            .next()
            .unwrap_or(UserRole::General)
    }

    /// Get confidence score for detected role
    pub fn confidence(&self, input: &str, role: UserRole) -> f32 {
        let input_lower = input.to_lowercase();
        let empty_patterns = vec![];
        let patterns = self.patterns.get(&role).unwrap_or(&empty_patterns);

        let matches = patterns
            .iter()
            .filter(|p| input_lower.contains(&p.to_lowercase()))
            .count();

        if patterns.is_empty() {
            0.0
        } else {
            matches as f32 / patterns.len() as f32
        }
    }
}

impl Default for UserRoleDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Scene Detector for specific operation type
pub struct SceneDetector {
    /// Scene patterns
    patterns: HashMap<String, Vec<String>>,
}

impl SceneDetector {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    fn default_patterns() -> HashMap<String, Vec<String>> {
        HashMap::from([
            (
                "code_generation".to_string(),
                vec![
                    "生成代码".to_string(),
                    "写代码".to_string(),
                    "实现".to_string(),
                    "create".to_string(),
                    "generate".to_string(),
                    "implement".to_string(),
                ],
            ),
            (
                "code_review".to_string(),
                vec![
                    "审查".to_string(),
                    "review".to_string(),
                    "检查代码".to_string(),
                    "优化".to_string(),
                    "refactor".to_string(),
                ],
            ),
            (
                "bug_fix".to_string(),
                vec![
                    "修复".to_string(),
                    "bug".to_string(),
                    "错误".to_string(),
                    "fix".to_string(),
                    "error".to_string(),
                    "issue".to_string(),
                ],
            ),
            (
                "documentation".to_string(),
                vec![
                    "文档".to_string(),
                    "说明".to_string(),
                    "README".to_string(),
                    "document".to_string(),
                    "docs".to_string(),
                ],
            ),
            (
                "image_generation".to_string(),
                vec![
                    "生成图片".to_string(),
                    "画图".to_string(),
                    "设计图".to_string(),
                    "generate image".to_string(),
                    "create visual".to_string(),
                ],
            ),
            (
                "video_generation".to_string(),
                vec![
                    "生成视频".to_string(),
                    "制作视频".to_string(),
                    "视频".to_string(),
                    "generate video".to_string(),
                    "create video".to_string(),
                ],
            ),
            (
                "copywriting".to_string(),
                vec![
                    "文案".to_string(),
                    "广告文案".to_string(),
                    "标题".to_string(),
                    "copywriting".to_string(),
                    "write copy".to_string(),
                ],
            ),
            (
                "translation".to_string(),
                vec![
                    "翻译".to_string(),
                    "translate".to_string(),
                    "转换语言".to_string(),
                ],
            ),
            (
                "document_generation".to_string(),
                vec![
                    "生成文档".to_string(),
                    "写报告".to_string(),
                    "制作PPT".to_string(),
                    "create document".to_string(),
                    "generate report".to_string(),
                ],
            ),
            (
                "excel_analysis".to_string(),
                vec![
                    "分析表格".to_string(),
                    "Excel分析".to_string(),
                    "数据分析".to_string(),
                    "analyze excel".to_string(),
                ],
            ),
            (
                "game_build".to_string(),
                vec![
                    "构建游戏".to_string(),
                    "编译游戏".to_string(),
                    "build game".to_string(),
                    "export game".to_string(),
                ],
            ),
            (
                "asset_generation".to_string(),
                vec![
                    "生成资源".to_string(),
                    "创建素材".to_string(),
                    "generate asset".to_string(),
                ],
            ),
        ])
    }

    /// Detect scene from input
    pub fn detect(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();

        for (scene, patterns) in &self.patterns {
            if patterns
                .iter()
                .any(|p| input_lower.contains(&p.to_lowercase()))
            {
                return scene.clone();
            }
        }

        "general".to_string()
    }
}

impl Default for SceneDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent Selector - selects best agent based on role and scene
pub struct AgentSelector {
    /// Agent mapping per role
    role_mapping: HashMap<UserRole, Vec<String>>,
    /// Scene-specific agent preferences
    scene_mapping: HashMap<String, String>,
}

impl AgentSelector {
    pub fn new() -> Self {
        Self {
            role_mapping: Self::default_role_mapping(),
            scene_mapping: Self::default_scene_mapping(),
        }
    }

    fn default_role_mapping() -> HashMap<UserRole, Vec<String>> {
        HashMap::from([
            (
                UserRole::Developer,
                vec![
                    // CLI Agents
                    "claude-code".to_string(),
                    "codex".to_string(),
                    // Desktop Development
                    "tauri-desktop".to_string(),
                    "electron-desktop".to_string(),
                    // Mobile Development
                    "flutter-sdk".to_string(),
                    // Mini Program Development
                    "wechat-miniprogram".to_string(),
                    // Containerized Deployment
                    "docker".to_string(),
                    "kubernetes".to_string(),
                ],
            ),
            (
                UserRole::Marketer,
                vec![
                    // Marketing Content Generation
                    "kimi-marketing".to_string(),
                    "jimeng-marketing".to_string(),
                    "kling-marketing".to_string(),
                    // Social Media Platforms
                    "douyin-open".to_string(),
                    "kuaishou-open".to_string(),
                ],
            ),
            (
                UserRole::Designer,
                vec![
                    // Image/Design Generation
                    "jimeng-marketing".to_string(),
                    "kimi-marketing".to_string(),
                ],
            ),
            (
                UserRole::Finance,
                vec![
                    // Office Automation
                    "wps-office".to_string(),
                    "kimi-marketing".to_string(),
                    // Enterprise Platforms
                    "dingtalk-open".to_string(),
                    "feishu-open".to_string(),
                ],
            ),
            (
                UserRole::Gamer,
                vec![
                    // Game Development
                    "unity-game".to_string(),
                    "godot-game".to_string(),
                ],
            ),
            (
                UserRole::Writer,
                vec![
                    // Content Creation
                    "kimi-marketing".to_string(),
                    "claude-code".to_string(),
                    // Document Generation
                    "wps-office".to_string(),
                    "feishu-open".to_string(),
                ],
            ),
            (
                UserRole::Analyst,
                vec![
                    // Data Analysis
                    "wps-office".to_string(),
                    "kimi-marketing".to_string(),
                    // Enterprise Analytics
                    "dingtalk-open".to_string(),
                    "feishu-open".to_string(),
                ],
            ),
            (
                UserRole::General,
                vec![
                    // General-purpose Agents
                    "claude-code".to_string(),
                    "kimi-marketing".to_string(),
                ],
            ),
        ])
    }

    fn default_scene_mapping() -> HashMap<String, String> {
        HashMap::from([
            // Marketing & Content
            (
                "image_generation".to_string(),
                "jimeng-marketing".to_string(),
            ),
            (
                "video_generation".to_string(),
                "kling-marketing".to_string(),
            ),
            ("copywriting".to_string(), "kimi-marketing".to_string()),
            ("translation".to_string(), "kimi-marketing".to_string()),
            ("document_generation".to_string(), "wps-office".to_string()),
            ("excel_analysis".to_string(), "wps-office".to_string()),
            // Game Development
            ("game_build".to_string(), "unity-game".to_string()),
            ("game_asset".to_string(), "godot-game".to_string()),
            // Code Development
            ("code_generation".to_string(), "claude-code".to_string()),
            ("bug_fix".to_string(), "codex".to_string()),
            ("code_review".to_string(), "claude-code".to_string()),
            // Mobile Development
            ("mobile_build".to_string(), "flutter-sdk".to_string()),
            ("mobile_run".to_string(), "flutter-sdk".to_string()),
            ("mobile_test".to_string(), "flutter-sdk".to_string()),
            // Mini Program Development
            (
                "miniprogram_create".to_string(),
                "wechat-miniprogram".to_string(),
            ),
            (
                "miniprogram_build".to_string(),
                "wechat-miniprogram".to_string(),
            ),
            (
                "miniprogram_publish".to_string(),
                "wechat-miniprogram".to_string(),
            ),
            // Desktop Development
            ("desktop_build".to_string(), "tauri-desktop".to_string()),
            ("desktop_run".to_string(), "tauri-desktop".to_string()),
            // Containerized Deployment
            ("dockerfile_generate".to_string(), "docker".to_string()),
            ("docker_build".to_string(), "docker".to_string()),
            ("docker_run".to_string(), "docker".to_string()),
            ("k8s_deploy".to_string(), "kubernetes".to_string()),
            ("k8s_scale".to_string(), "kubernetes".to_string()),
            // Enterprise Platforms
            ("douyin_publish".to_string(), "douyin-open".to_string()),
            ("douyin_analytics".to_string(), "douyin-open".to_string()),
            ("kuaishou_publish".to_string(), "kuaishou-open".to_string()),
            ("dingtalk_message".to_string(), "dingtalk-open".to_string()),
            ("dingtalk_approval".to_string(), "dingtalk-open".to_string()),
            ("feishu_document".to_string(), "feishu-open".to_string()),
            ("feishu_bitable".to_string(), "feishu-open".to_string()),
        ])
    }

    /// Select best agent
    pub fn select(&self, role: UserRole, scene: &str) -> (String, String) {
        // First check scene-specific mapping
        if let Some(agent) = self.scene_mapping.get(scene) {
            return (
                agent.clone(),
                format!("Selected {} for scene: {}", agent, scene),
            );
        }

        // Then use role mapping
        let default_agents = vec!["claude-code".to_string()];
        let agents = self.role_mapping.get(&role).unwrap_or(&default_agents);
        let agent = agents.first().unwrap();

        (
            agent.clone(),
            format!("Selected {} for role: {}", agent, role.as_str()),
        )
    }
}

impl Default for AgentSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// One-Shot Interface - unified entry point
pub struct OneShotInterface {
    role_detector: UserRoleDetector,
    scene_detector: SceneDetector,
    agent_selector: AgentSelector,
}

impl OneShotInterface {
    pub fn new() -> Self {
        Self {
            role_detector: UserRoleDetector::new(),
            scene_detector: SceneDetector::new(),
            agent_selector: AgentSelector::new(),
        }
    }

    /// Process one-shot request
    pub fn process(&self, request: OneShotRequest) -> OneShotResponse {
        // 1. Detect user role
        let role = self.role_detector.detect(&request.input);

        // 2. Detect scene
        let scene = self.scene_detector.detect(&request.input);

        // 3. Select agent
        let (agent, reason) = if let Some(preferred) = &request.preferred_agent {
            (preferred.clone(), format!("User preferred: {}", preferred))
        } else {
            self.agent_selector.select(role, &scene)
        };

        // 4. Build transparency info
        let transparency = TransparencyInfo {
            dag_visualization: format!(
                "Input -> RoleDetection({}) -> SceneDetection({}) -> AgentSelection({})",
                role.as_str(),
                scene,
                agent
            ),
            estimated_cost: 0.05,
            estimated_duration_ms: 30000,
            proficiency_scores: HashMap::from([(agent.clone(), 0.85)]),
            privacy_level: "standard".to_string(),
            budget_status: "ok".to_string(),
        };

        OneShotResponse {
            detected_role: role,
            detected_scene: scene,
            selected_agent: agent,
            selection_reason: reason,
            result: None, // Would be filled after execution
            cost: 0.0,
            duration_ms: 0,
            transparency,
        }
    }
}

impl Default for OneShotInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_as_str() {
        assert_eq!(UserRole::Developer.as_str(), "developer");
        assert_eq!(UserRole::Marketer.as_str(), "marketer");
    }

    #[test]
    fn test_user_role_from_str() {
        assert_eq!(UserRole::from_str("程序员"), UserRole::Developer);
        assert_eq!(UserRole::from_str("营销"), UserRole::Marketer);
    }

    #[test]
    fn test_role_detector_detect_developer() {
        let detector = UserRoleDetector::new();
        let role = detector.detect("帮我写一个 Rust 函数");
        assert_eq!(role, UserRole::Developer);
    }

    #[test]
    fn test_role_detector_detect_marketer() {
        let detector = UserRoleDetector::new();
        let role = detector.detect("写一段抖音文案");
        assert_eq!(role, UserRole::Marketer);
    }

    #[test]
    fn test_role_detector_detect_gamer() {
        let detector = UserRoleDetector::new();
        let role = detector.detect("帮我构建 Unity 游戏");
        assert_eq!(role, UserRole::Gamer);
    }

    #[test]
    fn test_role_detector_detect_finance() {
        let detector = UserRoleDetector::new();
        let role = detector.detect("分析 Excel 数据");
        assert_eq!(role, UserRole::Finance);
    }

    #[test]
    fn test_role_detector_detect_general() {
        let detector = UserRoleDetector::new();
        let role = detector.detect("随便说点什么");
        assert_eq!(role, UserRole::General);
    }

    #[test]
    fn test_scene_detector_detect() {
        let detector = SceneDetector::new();
        assert_eq!(detector.detect("生成图片"), "image_generation");
        assert_eq!(detector.detect("生成视频"), "video_generation");
        assert_eq!(detector.detect("写文案"), "copywriting");
        assert_eq!(detector.detect("翻译这段文字"), "translation");
    }

    #[test]
    fn test_agent_selector_select() {
        let selector = AgentSelector::new();
        let (agent, _) = selector.select(UserRole::Marketer, "copywriting");
        assert_eq!(agent, "kimi-marketing");

        let (agent, _) = selector.select(UserRole::Developer, "code_generation");
        assert_eq!(agent, "claude-code");

        let (agent, _) = selector.select(UserRole::Gamer, "game_build");
        assert_eq!(agent, "unity-game");
    }

    #[test]
    fn test_one_shot_interface_process() {
        let interface = OneShotInterface::new();
        let request = OneShotRequest {
            input: "帮我写一段抖音广告文案".to_string(),
            context_hint: None,
            preferred_agent: None,
            max_cost: None,
            timeout_ms: None,
        };

        let response = interface.process(request);
        assert_eq!(response.detected_role, UserRole::Marketer);
        assert_eq!(response.detected_scene, "copywriting");
        assert_eq!(response.selected_agent, "kimi-marketing");
    }

    #[test]
    fn test_one_shot_interface_with_preferred_agent() {
        let interface = OneShotInterface::new();
        let request = OneShotRequest {
            input: "写代码".to_string(),
            context_hint: None,
            preferred_agent: Some("codex".to_string()),
            max_cost: None,
            timeout_ms: None,
        };

        let response = interface.process(request);
        assert_eq!(response.selected_agent, "codex");
        assert!(response.selection_reason.contains("preferred"));
    }
}
