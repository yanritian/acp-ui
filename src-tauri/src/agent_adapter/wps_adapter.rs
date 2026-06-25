// WPS Adapter - WPS AI office automation
//
// Phase 4: Marketing & Finance Agents
// Supports document generation, PPT generation, Excel analysis

use crate::agent_adapter::{
    AgentAdapter,
    types::{AdapterType, Capability, AgentConfig, AgentTask, AgentResult, AgentError,
            TaskInput, TaskOutput, ResultStatus, ActualCost, TokenUsage, HealthMetrics,
            CostEstimate, TokenEstimate},
    health_tracker::HealthTracker,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// WPS API Adapter (金山办公 AI)
pub struct WPSAdapter {
    id: String,
    name: String,
    config: Option<WPSConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    client: Client,
}

/// WPS API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WPSConfig {
    /// API key (WPS AI)
    pub api_key: String,
    /// Base URL
    pub base_url: String,
    /// Default document format
    pub default_format: DocumentFormat,
}

impl Default for WPSConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.wps.cn/v1".to_string(),
            default_format: DocumentFormat::Docx,
        }
    }
}

/// Document format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentFormat {
    Docx,
    Pdf,
    Pptx,
    Xlsx,
    Txt,
}

impl DocumentFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentFormat::Docx => "docx",
            DocumentFormat::Pdf => "pdf",
            DocumentFormat::Pptx => "pptx",
            DocumentFormat::Xlsx => "xlsx",
            DocumentFormat::Txt => "txt",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "docx" | "doc" => DocumentFormat::Docx,
            "pdf" => DocumentFormat::Pdf,
            "pptx" | "ppt" => DocumentFormat::Pptx,
            "xlsx" | "xls" => DocumentFormat::Xlsx,
            "txt" => DocumentFormat::Txt,
            _ => DocumentFormat::Docx,
        }
    }
}

/// Document type for generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    Report,
    Contract,
    Proposal,
    Resume,
    Letter,
    Summary,
    Plan,
    Article,
}

impl DocumentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentType::Report => "report",
            DocumentType::Contract => "contract",
            DocumentType::Proposal => "proposal",
            DocumentType::Resume => "resume",
            DocumentType::Letter => "letter",
            DocumentType::Summary => "summary",
            DocumentType::Plan => "plan",
            DocumentType::Article => "article",
        }
    }
}

/// WPS API request
#[derive(Debug, Serialize)]
struct WPSRequest {
    prompt: String,
    document_type: String,
    format: String,
    language: String,
    style: Option<String>,
}

/// WPS API response
#[derive(Debug, Deserialize)]
struct WPSResponse {
    document_id: String,
    url: String,
    format: String,
    cost: f32,
}

/// Excel analysis request
#[derive(Debug, Serialize)]
struct ExcelAnalysisRequest {
    file_url: String,
    analysis_type: String,
    output_format: String,
}

/// Excel analysis response
#[derive(Debug, Deserialize)]
struct ExcelAnalysisResponse {
    summary: String,
    charts: Vec<String>,
    insights: Vec<String>,
    cost: f32,
}

impl WPSAdapter {
    pub fn new() -> Self {
        Self {
            id: "wps-office".to_string(),
            name: "WPS Office Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            client: Client::new(),
        }
    }

    pub fn with_config(config: WPSConfig) -> Self {
        Self {
            id: "wps-office".to_string(),
            name: "WPS Office Adapter".to_string(),
            config: Some(config),
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            client: Client::new(),
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "doc-gen".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.02, // $0.02 per document
                latency_ms: 10000,
            },
            Capability {
                name: "ppt-gen".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.05, // $0.05 per PPT
                latency_ms: 15000,
            },
            Capability {
                name: "excel-analysis".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.03,
                latency_ms: 8000,
            },
            Capability {
                name: "pdf-convert".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.01,
                latency_ms: 3000,
            },
        ]
    }

    /// Generate document via WPS API
    pub async fn generate_document(
        &self,
        prompt: &str,
        doc_type: DocumentType,
        format: DocumentFormat,
    ) -> Result<(String, f32), AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "WPS API key not configured".to_string(),
        })?;

        let request = WPSRequest {
            prompt: prompt.to_string(),
            document_type: doc_type.as_str().to_string(),
            format: format.as_str().to_string(),
            language: "zh-CN".to_string(),
            style: None,
        };

        let url = format!("{}/documents/generate", config.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let wps_response: WPSResponse = resp.json().await.map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to parse WPS response: {}", e),
                    retryable: false,
                })?;

                Ok((wps_response.url, wps_response.cost))
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                Err(AgentError::ExecutionError {
                    message: format!("WPS API error {}: {}", status, body),
                    retryable: status.is_server_error(),
                })
            }
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("WPS API request failed: {}", e),
                retryable: e.is_timeout() || e.is_connect(),
            }),
        }
    }

    /// Analyze Excel file
    pub async fn analyze_excel(
        &self,
        file_url: &str,
        analysis_type: &str,
    ) -> Result<(String, Vec<String>, Vec<String>, f32), AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "WPS API key not configured".to_string(),
        })?;

        let request = ExcelAnalysisRequest {
            file_url: file_url.to_string(),
            analysis_type: analysis_type.to_string(),
            output_format: "json".to_string(),
        };

        let url = format!("{}/excel/analyze", config.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let analysis: ExcelAnalysisResponse = resp.json().await.map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to parse Excel analysis: {}", e),
                    retryable: false,
                })?;

                Ok((analysis.summary, analysis.charts, analysis.insights, analysis.cost))
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                Err(AgentError::ExecutionError {
                    message: format!("WPS API error {}: {}", status, body),
                    retryable: status.is_server_error(),
                })
            }
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("WPS API request failed: {}", e),
                retryable: e.is_timeout() || e.is_connect(),
            }),
        }
    }

    /// Parse document type from description
    fn parse_doc_type(description: &str) -> DocumentType {
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("报告") || desc_lower.contains("report") {
            DocumentType::Report
        } else if desc_lower.contains("合同") || desc_lower.contains("contract") {
            DocumentType::Contract
        } else if desc_lower.contains("提案") || desc_lower.contains("proposal") {
            DocumentType::Proposal
        } else if desc_lower.contains("简历") || desc_lower.contains("resume") {
            DocumentType::Resume
        } else if desc_lower.contains("信函") || desc_lower.contains("letter") {
            DocumentType::Letter
        } else if desc_lower.contains("总结") || desc_lower.contains("summary") {
            DocumentType::Summary
        } else if desc_lower.contains("计划") || desc_lower.contains("plan") {
            DocumentType::Plan
        } else {
            DocumentType::Article
        }
    }
}

impl Default for WPSAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for WPSAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Api
    }

    fn capabilities(&self) -> Vec<Capability> {
        Self::get_capabilities()
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        let wps_config = if let Some(api_key) = config.metadata.get("api_key") {
            WPSConfig {
                api_key: api_key.clone(),
                base_url: config.metadata.get("base_url")
                    .cloned()
                    .unwrap_or_else(|| "https://api.wps.cn/v1".to_string()),
                default_format: config.metadata.get("format")
                    .map(|f| DocumentFormat::from_str(f))
                    .unwrap_or(DocumentFormat::Docx),
            }
        } else {
            return Err(AgentError::ConfigurationError {
                message: "WPS API key required in metadata".to_string(),
            });
        };

        self.config = Some(wps_config);
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "WPS not configured".to_string(),
        })?;

        if config.api_key.is_empty() {
            return Err(AgentError::ConfigurationError {
                message: "WPS API key is empty".to_string(),
            });
        }

        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();

        // Parse prompt and determine document type
        let prompt = match &task.input {
            TaskInput::Text(text) => text.clone(),
            TaskInput::Data(data) => data.clone(),
            TaskInput::Url(url) => format!("根据 {} 内容生成文档", url),
            _ => "生成一份标准文档".to_string(),
        };

        let doc_type = Self::parse_doc_type(&task.description);
        let config = self.config.as_ref().unwrap();
        let format = config.default_format;

        // Check if this is Excel analysis
        if task.description.to_lowercase().contains("excel") || task.description.to_lowercase().contains("表格") {
            if let TaskInput::Url(url) = &task.input {
                let (summary, charts, insights, cost) = self.analyze_excel(url, "comprehensive").await?;
                let duration_ms = start.elapsed().as_millis() as u64;

                return Ok(AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Data(serde_json::json!({
                        "summary": summary,
                        "charts": charts,
                        "insights": insights,
                    }).to_string()),
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                    cost: ActualCost {
                        amount: cost,
                        currency: "USD".to_string(),
                        token_usage: TokenUsage {
                            input_tokens: 0,
                            output_tokens: 0,
                            total_tokens: 0,
                        },
                    },
                    duration_ms,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    metadata: HashMap::from([
                        ("analysis_type".to_string(), "excel".to_string()),
                        ("provider".to_string(), "wps".to_string()),
                    ]),
                });
            }
        }

        // Generate document
        let (url, cost) = self.generate_document(&prompt, doc_type, format).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(AgentResult {
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Url(url),
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cost: ActualCost {
                amount: cost,
                currency: "USD".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                },
            },
            duration_ms,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            metadata: HashMap::from([
                ("document_type".to_string(), doc_type.as_str().to_string()),
                ("format".to_string(), format.as_str().to_string()),
                ("provider".to_string(), "wps".to_string()),
            ]),
        })
    }

    async fn cancel(&self, _task_id: &str) -> Result<(), AgentError> {
        Ok(())
    }

    fn status(&self) -> crate::agent_adapter::types::AgentStatus {
        crate::agent_adapter::types::AgentStatus::Idle
    }

    async fn health(&self) -> HealthMetrics {
        self.health_tracker.get_metrics()
    }

    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate {
        let doc_type = Self::parse_doc_type(&task.description);

        let base_cost = match doc_type {
            DocumentType::Report => 0.02,
            DocumentType::Contract => 0.03,
            DocumentType::Proposal => 0.04,
            DocumentType::Resume => 0.02,
            DocumentType::Summary => 0.02,
            DocumentType::Plan => 0.03,
            _ => 0.02,
        };

        CostEstimate {
            min_cost: base_cost * 0.8,
            max_cost: base_cost * 1.5,
            currency: "USD".to_string(),
            breakdown: HashMap::from([
                ("document_generation".to_string(), base_cost),
            ]),
            token_estimate: TokenEstimate {
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
            },
        }
    }

    fn actual_cost(&self, _task_id: &str) -> Option<ActualCost> {
        None
    }

    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage> {
        self.token_history.iter().rev().take(last_n as usize).cloned().collect()
    }

    async fn update_health(&mut self, result: &AgentResult) {
        match result.status {
            ResultStatus::Success => {
                self.health_tracker.record_success(result.duration_ms);
            }
            _ => {
                self.health_tracker.record_error();
            }
        }
    }

    fn is_circuit_breaker_allowed(&self) -> bool {
        self.health_tracker.is_circuit_breaker_closed()
    }

    async fn reset_circuit_breaker(&mut self) {
        self.health_tracker.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wps_adapter_new() {
        let adapter = WPSAdapter::new();
        assert_eq!(adapter.id(), "wps-office");
        assert_eq!(adapter.adapter_type(), AdapterType::Api);
    }

    #[test]
    fn test_wps_config_default() {
        let config = WPSConfig::default();
        assert_eq!(config.base_url, "https://api.wps.cn/v1");
        assert_eq!(config.default_format, DocumentFormat::Docx);
    }

    #[test]
    fn test_document_format_as_str() {
        assert_eq!(DocumentFormat::Docx.as_str(), "docx");
        assert_eq!(DocumentFormat::Pptx.as_str(), "pptx");
        assert_eq!(DocumentFormat::Pdf.as_str(), "pdf");
    }

    #[test]
    fn test_document_format_from_str() {
        assert_eq!(DocumentFormat::from_str("docx"), DocumentFormat::Docx);
        assert_eq!(DocumentFormat::from_str("pdf"), DocumentFormat::Pdf);
        assert_eq!(DocumentFormat::from_str("unknown"), DocumentFormat::Docx);
    }

    #[test]
    fn test_document_type_as_str() {
        assert_eq!(DocumentType::Report.as_str(), "report");
        assert_eq!(DocumentType::Contract.as_str(), "contract");
        assert_eq!(DocumentType::Summary.as_str(), "summary");
    }

    #[test]
    fn test_capabilities() {
        let adapter = WPSAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.iter().any(|c| c.name == "doc-gen"));
        assert!(caps.iter().any(|c| c.name == "ppt-gen"));
        assert!(caps.iter().any(|c| c.name == "excel-analysis"));
    }

    #[test]
    fn test_cost_estimate() {
        let adapter = WPSAdapter::new();
        let task = AgentTask {
            id: "test".to_string(),
            description: "生成一份报告".to_string(),
            input: TaskInput::Text("test".to_string()),
            constraints: Default::default(),
            scene_type: None,
            platform: None,
        };

        let estimate = adapter.cost_estimate(&task);
        assert!(estimate.min_cost > 0.0);
        assert!(estimate.max_cost > estimate.min_cost);
    }

    #[test]
    fn test_parse_doc_type() {
        assert_eq!(WPSAdapter::parse_doc_type("生成报告"), DocumentType::Report);
        assert_eq!(WPSAdapter::parse_doc_type("起草合同"), DocumentType::Contract);
        assert_eq!(WPSAdapter::parse_doc_type("make a proposal"), DocumentType::Proposal);
        assert_eq!(WPSAdapter::parse_doc_type("普通文档"), DocumentType::Article);
    }
}