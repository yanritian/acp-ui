// Flutter SDK Adapter - Flutter 移动端开发深度集成
//
// 集成 Flutter SDK，支持：
// - 项目创建与初始化
// - 多平台编译（iOS/Android/Web/Desktop）
// - 热重载开发
// - 测试运行
// - 发布构建

use crate::agent_adapter::{
    health_tracker::HealthTracker,
    types::{
        ActualCost, AdapterType, AgentConfig, AgentError, AgentResult, AgentStatus, AgentTask,
        Capability, CostEstimate, HealthMetrics, Platform, ResultStatus, SceneType, TaskInput,
        TaskOutput, TokenEstimate, TokenUsage,
    },
    AgentAdapter,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Flutter SDK Adapter
pub struct FlutterAdapter {
    id: String,
    name: String,
    config: Option<FlutterConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    flutter_sdk_path: Option<PathBuf>,
}

/// Flutter 配置
#[derive(Debug, Clone)]
pub struct FlutterConfig {
    /// Flutter SDK 路径
    pub sdk_path: Option<PathBuf>,
    /// 项目路径
    pub project_path: PathBuf,
    /// 目标平台
    pub target_platform: FlutterPlatform,
    /// 构建模式
    pub build_mode: FlutterBuildMode,
    /// 是否启用热重载
    pub hot_reload: bool,
}

/// Flutter 目标平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlutterPlatform {
    Android,
    Ios,
    Web,
    Windows,
    MacOS,
    Linux,
    All,
}

impl FlutterPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlutterPlatform::Android => "android",
            FlutterPlatform::Ios => "ios",
            FlutterPlatform::Web => "web",
            FlutterPlatform::Windows => "windows",
            FlutterPlatform::MacOS => "macos",
            FlutterPlatform::Linux => "linux",
            FlutterPlatform::All => "all",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "android" | "apk" | "appbundle" => Ok(FlutterPlatform::Android),
            "ios" | "ipa" => Ok(FlutterPlatform::Ios),
            "web" => Ok(FlutterPlatform::Web),
            "windows" => Ok(FlutterPlatform::Windows),
            "macos" => Ok(FlutterPlatform::MacOS),
            "linux" => Ok(FlutterPlatform::Linux),
            "all" | "multi" => Ok(FlutterPlatform::All),
            other => Err(format!("Unknown Flutter platform: {}", other)),
        }
    }
}

/// Flutter 构建模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlutterBuildMode {
    Debug,
    Profile,
    Release,
}

impl FlutterBuildMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlutterBuildMode::Debug => "debug",
            FlutterBuildMode::Profile => "profile",
            FlutterBuildMode::Release => "release",
        }
    }
}

/// Flutter 操作类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlutterAction {
    /// 创建项目
    Create,
    /// 运行开发服务器
    Run,
    /// 构建发布版本
    Build,
    /// 运行测试
    Test,
    /// 分析代码
    Analyze,
    /// 清理项目
    Clean,
    /// 获取依赖
    PubGet,
    /// 格式化代码
    Format,
    /// 添加依赖
    AddDependency,
}

impl FlutterAdapter {
    pub fn new() -> Self {
        Self {
            id: "flutter-sdk".to_string(),
            name: "Flutter SDK".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            flutter_sdk_path: Self::detect_flutter_path(),
        }
    }

    /// 自动检测 Flutter SDK 路径
    fn detect_flutter_path() -> Option<PathBuf> {
        // 尝试通过 flutter 命令获取路径
        if let Ok(output) = Command::new("flutter").arg("sdk-path").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Some(PathBuf::from(path));
            }
        }

        // Windows 默认路径
        let windows_paths = [
            "C:/flutter/bin/flutter.bat",
            "D:/flutter/bin/flutter.bat",
            "C:/src/flutter/bin/flutter.bat",
            "D:/src/flutter/bin/flutter.bat",
        ];

        for path in windows_paths {
            let pb = PathBuf::from(path);
            if pb.exists() {
                return Some(pb.parent().unwrap().parent().unwrap().to_path_buf());
            }
        }

        None
    }

    /// 执行 Flutter CLI 命令
    async fn execute_flutter(
        &self,
        args: &[String],
        cwd: Option<&PathBuf>,
    ) -> Result<String, AgentError> {
        let flutter_cmd = if cfg!(target_os = "windows") {
            "flutter.bat"
        } else {
            "flutter"
        };

        let mut cmd = Command::new(flutter_cmd);
        cmd.args(args);

        if let Some(working_dir) = cwd {
            cmd.current_dir(working_dir);
        }

        let output = cmd.output().map_err(|e| AgentError::ExecutionError {
            message: format!("执行 Flutter 命令失败: {}", e),
            retryable: false,
        })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AgentError::ExecutionError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
                retryable: true,
            })
        }
    }

    /// 创建 Flutter 项目
    async fn create_project(
        &self,
        name: &str,
        path: &str,
        template: Option<&str>,
    ) -> Result<AgentResult, AgentError> {
        let mut args = vec!["create".to_string()];

        if let Some(tpl) = template {
            args.push("--template".to_string());
            args.push(tpl.to_string());
        }

        args.push(path.to_string());

        let output = self.execute_flutter(&args, None).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("Flutter 项目创建成功: {}\n{}", name, output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::from([
                ("project_name".to_string(), name.to_string()),
                ("project_path".to_string(), path.to_string()),
            ]),
        })
    }

    /// 运行 Flutter 应用（开发模式）
    async fn run_app(
        &self,
        project_path: &PathBuf,
        platform: FlutterPlatform,
        hot_reload: bool,
    ) -> Result<AgentResult, AgentError> {
        let mut args = vec!["run".to_string()];

        args.push("-d".to_string());
        args.push(platform.as_str().to_string());

        if hot_reload {
            args.push("--hot".to_string());
        }

        let output = self.execute_flutter(&args, Some(project_path)).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("Flutter 应用运行成功\n{}", output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::from([
                ("platform".to_string(), platform.as_str().to_string()),
                ("hot_reload".to_string(), hot_reload.to_string()),
            ]),
        })
    }

    /// 构建 Flutter 应用
    async fn build_app(
        &self,
        project_path: &PathBuf,
        platform: FlutterPlatform,
        mode: FlutterBuildMode,
    ) -> Result<AgentResult, AgentError> {
        let mut args = vec!["build".to_string()];

        // 平台特定构建命令
        match platform {
            FlutterPlatform::Android => {
                args.push("apk".to_string());
            }
            FlutterPlatform::Ios => {
                args.push("ipa".to_string());
            }
            FlutterPlatform::Web => {
                args.push("web".to_string());
            }
            FlutterPlatform::Windows => {
                args.push("windows".to_string());
            }
            FlutterPlatform::MacOS => {
                args.push("macos".to_string());
            }
            FlutterPlatform::Linux => {
                args.push("linux".to_string());
            }
            FlutterPlatform::All => {
                // 构建 Android APK（默认）
                args.push("apk".to_string());
            }
        }

        // 构建模式
        match mode {
            FlutterBuildMode::Release => {
                args.push("--release".to_string());
            }
            FlutterBuildMode::Profile => {
                args.push("--profile".to_string());
            }
            FlutterBuildMode::Debug => {
                args.push("--debug".to_string());
            }
        }

        let output = self.execute_flutter(&args, Some(project_path)).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!(
                "构建成功: {} {}\n{}",
                platform.as_str(),
                mode.as_str(),
                output
            )),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::from([
                ("platform".to_string(), platform.as_str().to_string()),
                ("build_mode".to_string(), mode.as_str().to_string()),
            ]),
        })
    }

    /// 运行 Flutter 测试
    async fn run_tests(&self, project_path: &PathBuf) -> Result<AgentResult, AgentError> {
        let args = vec!["test".to_string(), "--coverage".to_string()];

        let output = self.execute_flutter(&args, Some(project_path)).await?;

        // 解析测试结果
        let passed = output.matches("passed").count();
        let failed = output.matches("failed").count();
        let all_passed = output.contains("All tests passed") || (passed > 0 && failed == 0);

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: if all_passed {
                ResultStatus::Success
            } else {
                ResultStatus::Failed {
                    error: format!("{} tests failed", failed),
                    retryable: true,
                }
            },
            output: TaskOutput::Text(format!(
                "测试结果: passed={}, failed={}\n{}",
                passed, failed, output
            )),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::from([
                ("tests_passed".to_string(), passed.to_string()),
                ("tests_failed".to_string(), failed.to_string()),
                ("coverage".to_string(), "generated".to_string()),
            ]),
        })
    }

    /// 分析 Flutter 代码
    async fn analyze_code(&self, project_path: &PathBuf) -> Result<AgentResult, AgentError> {
        let args = vec!["analyze".to_string()];

        let output = self.execute_flutter(&args, Some(project_path)).await?;

        // 解析分析结果
        let issues = output.matches("error").count() + output.matches("warning").count();

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: if issues > 0 {
                ResultStatus::Success // 分析完成，但有问题
            } else {
                ResultStatus::Success
            },
            output: TaskOutput::Text(format!("代码分析: {} issues\n{}", issues, output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::from([("issues_count".to_string(), issues.to_string())]),
        })
    }

    /// 添加依赖包
    async fn add_dependency(
        &self,
        project_path: &PathBuf,
        package: &str,
        version: Option<&str>,
    ) -> Result<AgentResult, AgentError> {
        let mut args = vec!["pub".to_string(), "add".to_string()];

        if let Some(ver) = version {
            args.push(format!("{}:{}", package, ver));
        } else {
            args.push(package.to_string());
        }

        let output = self.execute_flutter(&args, Some(project_path)).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("添加依赖成功: {}\n{}", package, output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::from([
                ("package".to_string(), package.to_string()),
                (
                    "version".to_string(),
                    version.unwrap_or("latest").to_string(),
                ),
            ]),
        })
    }
}

#[async_trait]
impl AgentAdapter for FlutterAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Cli
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "flutter_create".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 30000,
            },
            Capability {
                name: "flutter_run".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 15000,
            },
            Capability {
                name: "flutter_build_android".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 120000,
            },
            Capability {
                name: "flutter_build_ios".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.0,
                latency_ms: 180000,
            },
            Capability {
                name: "flutter_build_web".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 60000,
            },
            Capability {
                name: "flutter_test".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 60000,
            },
            Capability {
                name: "flutter_analyze".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 10000,
            },
            Capability {
                name: "flutter_pub".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 30000,
            },
        ]
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        let project_path = config
            .metadata
            .get("project_path")
            .map(PathBuf::from)
            .unwrap_or_default();

        let target_platform = config
            .metadata
            .get("platform")
            .map(|p| FlutterPlatform::from_str(p).unwrap_or(FlutterPlatform::Android))
            .unwrap_or(FlutterPlatform::Android);

        let build_mode = config
            .metadata
            .get("mode")
            .map(|m| match m.as_str() {
                "release" => FlutterBuildMode::Release,
                "profile" => FlutterBuildMode::Profile,
                _ => FlutterBuildMode::Debug,
            })
            .unwrap_or(FlutterBuildMode::Debug);

        let hot_reload = config
            .metadata
            .get("hot_reload")
            .map(|h| h == "true")
            .unwrap_or(true);

        self.config = Some(FlutterConfig {
            sdk_path: self.flutter_sdk_path.clone(),
            project_path,
            target_platform,
            build_mode,
            hot_reload,
        });

        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        // 检查 Flutter 是否安装
        let flutter_cmd = if cfg!(target_os = "windows") {
            "flutter.bat"
        } else {
            "flutter"
        };

        let output = Command::new(flutter_cmd)
            .arg("--version")
            .output()
            .map_err(|e| AgentError::ConfigurationError {
                message: format!("Flutter SDK 未安装: {}", e),
            })?;

        if !output.status.success() {
            return Err(AgentError::ConfigurationError {
                message: "Flutter SDK 未正确安装".to_string(),
            });
        }

        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();

        // 解析操作类型
        let action = match task.description.as_str() {
            s if s.contains("创建") || s.contains("create") => FlutterAction::Create,
            s if s.contains("运行") || s.contains("run") || s.contains("dev") => {
                FlutterAction::Run
            }
            s if s.contains("构建") || s.contains("build") => FlutterAction::Build,
            s if s.contains("测试") || s.contains("test") => FlutterAction::Test,
            s if s.contains("分析") || s.contains("analyze") => FlutterAction::Analyze,
            s if s.contains("清理") || s.contains("clean") => FlutterAction::Clean,
            s if s.contains("依赖") || s.contains("pub") || s.contains("get") => {
                FlutterAction::PubGet
            }
            s if s.contains("格式化") || s.contains("format") => FlutterAction::Format,
            s if s.contains("添加") || s.contains("add") => FlutterAction::AddDependency,
            _ => FlutterAction::Build,
        };

        // 获取项目路径
        let project_path = self
            .config
            .as_ref()
            .map(|c| &c.project_path)
            .cloned()
            .unwrap_or(PathBuf::from("."));

        // 执行操作
        let result = match action {
            FlutterAction::Create => {
                let (name, path, template) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        let n = parts.get(0).unwrap_or(&"flutter_app");
                        let p = parts.get(1).unwrap_or(&"./");
                        let t = parts.get(2);
                        (n.to_string(), p.to_string(), t.map(|s| s.to_string()))
                    }
                    _ => ("flutter_app".to_string(), "./".to_string(), None),
                };
                self.create_project(&name, &path, template.as_deref())
                    .await?
            }
            FlutterAction::Run => {
                let platform = self
                    .config
                    .as_ref()
                    .map(|c| c.target_platform)
                    .unwrap_or(FlutterPlatform::Android);
                let hot_reload = self.config.as_ref().map(|c| c.hot_reload).unwrap_or(true);
                self.run_app(&project_path, platform, hot_reload).await?
            }
            FlutterAction::Build => {
                let platform = self
                    .config
                    .as_ref()
                    .map(|c| c.target_platform)
                    .unwrap_or(FlutterPlatform::Android);
                let mode = self
                    .config
                    .as_ref()
                    .map(|c| c.build_mode)
                    .unwrap_or(FlutterBuildMode::Release);
                self.build_app(&project_path, platform, mode).await?
            }
            FlutterAction::Test => self.run_tests(&project_path).await?,
            FlutterAction::Analyze => self.analyze_code(&project_path).await?,
            FlutterAction::Clean => {
                let args = vec!["clean".to_string()];
                let output = self.execute_flutter(&args, Some(&project_path)).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("清理完成\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost {
                        amount: 0.0,
                        currency: "CNY".to_string(),
                        token_usage: TokenUsage {
                            input_tokens: 0,
                            output_tokens: output.len() as u64 / 4,
                            total_tokens: output.len() as u64 / 4,
                        },
                    },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                }
            }
            FlutterAction::PubGet => {
                let args = vec!["pub".to_string(), "get".to_string()];
                let output = self.execute_flutter(&args, Some(&project_path)).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("依赖获取成功\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost {
                        amount: 0.0,
                        currency: "CNY".to_string(),
                        token_usage: TokenUsage {
                            input_tokens: 0,
                            output_tokens: output.len() as u64 / 4,
                            total_tokens: output.len() as u64 / 4,
                        },
                    },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                }
            }
            FlutterAction::Format => {
                let args = vec!["format".to_string(), ".".to_string()];
                let output = self.execute_flutter(&args, Some(&project_path)).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("代码格式化完成\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost {
                        amount: 0.0,
                        currency: "CNY".to_string(),
                        token_usage: TokenUsage {
                            input_tokens: 0,
                            output_tokens: output.len() as u64 / 4,
                            total_tokens: output.len() as u64 / 4,
                        },
                    },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                }
            }
            FlutterAction::AddDependency => {
                let (package, version) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        let pkg = parts.get(0).unwrap_or(&"flutter_riverpod");
                        let ver = parts.get(1);
                        (pkg.to_string(), ver.map(|s| s.to_string()))
                    }
                    _ => ("flutter_riverpod".to_string(), None),
                };
                self.add_dependency(&project_path, &package, version.as_deref())
                    .await?
            }
        };

        Ok(result)
    }

    async fn cancel(&self, task_id: &str) -> Result<(), AgentError> {
        // Flutter CLI 不支持取消操作
        Ok(())
    }

    fn status(&self) -> AgentStatus {
        let metrics = self.health_tracker.get_metrics();
        if metrics.health_score > 80.0 {
            AgentStatus::Idle
        } else {
            AgentStatus::Error {
                message: "健康分数过低".to_string(),
                error_count: metrics.error_count,
            }
        }
    }

    async fn health(&self) -> HealthMetrics {
        self.health_tracker.get_metrics()
    }

    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate {
        CostEstimate {
            min_cost: 0.0,
            max_cost: 0.0,
            currency: "CNY".to_string(),
            breakdown: HashMap::new(),
            token_estimate: TokenEstimate {
                input_tokens: 0,
                output_tokens: 1000,
                total_tokens: 1000,
            },
        }
    }

    fn actual_cost(&self, task_id: &str) -> Option<ActualCost> {
        None
    }

    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage> {
        self.token_history
            .iter()
            .rev()
            .take(last_n as usize)
            .cloned()
            .collect()
    }

    async fn update_health(&mut self, result: &AgentResult) {
        if result.status == ResultStatus::Success {
            self.health_tracker.record_success(result.duration_ms);
        } else {
            self.health_tracker.record_error();
        }
        self.token_history.push(TokenUsage {
            input_tokens: result.input_tokens,
            output_tokens: result.output_tokens,
            total_tokens: result.total_tokens,
        });

        if self.token_history.len() > 100 {
            self.token_history.remove(0);
        }
    }

    fn is_circuit_breaker_allowed(&self) -> bool {
        let metrics = self.health_tracker.get_metrics();
        matches!(
            metrics.circuit_breaker_state,
            crate::agent_adapter::types::CircuitBreakerState::Closed
                | crate::agent_adapter::types::CircuitBreakerState::HalfOpen { .. }
        )
    }

    async fn reset_circuit_breaker(&mut self) {
        self.health_tracker.reset();
    }
}

impl Default for FlutterAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flutter_adapter_creation() {
        let adapter = FlutterAdapter::new();
        assert_eq!(adapter.id(), "flutter-sdk");
        assert_eq!(adapter.name(), "Flutter SDK");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_flutter_capabilities() {
        let adapter = FlutterAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.len() >= 8);
        assert!(caps.iter().any(|c| c.name == "flutter_create"));
        assert!(caps.iter().any(|c| c.name == "flutter_build_android"));
        assert!(caps.iter().any(|c| c.name == "flutter_test"));
    }

    #[test]
    fn test_flutter_platform() {
        assert_eq!(FlutterPlatform::Android.as_str(), "android");
        assert_eq!(FlutterPlatform::Ios.as_str(), "ios");
        assert_eq!(FlutterPlatform::Web.as_str(), "web");
        assert!(FlutterPlatform::from_str("android").is_ok());
    }

    #[test]
    fn test_flutter_build_mode() {
        assert_eq!(FlutterBuildMode::Debug.as_str(), "debug");
        assert_eq!(FlutterBuildMode::Release.as_str(), "release");
    }

    #[test]
    fn test_health_tracker() {
        let adapter = FlutterAdapter::new();
        let health = adapter.health_tracker.get_metrics();
        assert_eq!(health.health_score, 100.0);
    }
}
