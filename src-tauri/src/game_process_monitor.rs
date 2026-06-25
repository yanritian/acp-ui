// Game Process Monitor
// Phase 3 Day 9: Enhanced process monitoring and metrics

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ===== Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetrics {
    pub process_id: u32,
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub thread_count: u32,
    pub handle_count: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSummary {
    pub avg_memory_mb: f64,
    pub max_memory_mb: f64,
    pub avg_cpu_percent: f64,
    pub max_cpu_percent: f64,
    pub samples: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    HighMemory,
    HighCpu,
    LowFps,
    ProcessCrash,
}

// ===== Process Monitor =====

pub struct ProcessMonitor {
    metrics_history: VecDeque<ProcessMetrics>,
    max_samples: usize,
    alerts: Vec<PerformanceAlert>,
    memory_threshold_mb: f64,
    cpu_threshold_percent: f64,
}

impl ProcessMonitor {
    pub fn new(max_samples: usize) -> Self {
        Self {
            metrics_history: VecDeque::with_capacity(max_samples),
            max_samples,
            alerts: Vec::new(),
            memory_threshold_mb: 2048.0,  // 2GB default
            cpu_threshold_percent: 90.0,   // 90% default
        }
    }

    /// Set memory threshold for alerts
    pub fn set_memory_threshold(&mut self, threshold_mb: f64) {
        self.memory_threshold_mb = threshold_mb;
    }

    /// Set CPU threshold for alerts
    pub fn set_cpu_threshold(&mut self, threshold_percent: f64) {
        self.cpu_threshold_percent = threshold_percent;
    }

    /// Record a new metrics sample
    pub fn record_metrics(&mut self, metrics: ProcessMetrics) {
        // Check for alerts
        if metrics.memory_mb > self.memory_threshold_mb {
            self.alerts.push(PerformanceAlert {
                alert_type: AlertType::HighMemory,
                message: format!("High memory usage: {:.2} MB", metrics.memory_mb),
                value: metrics.memory_mb,
                threshold: self.memory_threshold_mb,
                timestamp: metrics.timestamp,
            });
        }

        if metrics.cpu_percent > self.cpu_threshold_percent {
            self.alerts.push(PerformanceAlert {
                alert_type: AlertType::HighCpu,
                message: format!("High CPU usage: {:.2}%", metrics.cpu_percent),
                value: metrics.cpu_percent,
                threshold: self.cpu_threshold_percent,
                timestamp: metrics.timestamp,
            });
        }

        // Add to history
        if self.metrics_history.len() >= self.max_samples {
            self.metrics_history.pop_front();
        }
        self.metrics_history.push_back(metrics);
    }

    /// Get performance summary
    pub fn get_summary(&self) -> Option<PerformanceSummary> {
        if self.metrics_history.is_empty() {
            return None;
        }

        let samples = self.metrics_history.len() as u32;

        let avg_memory = self.metrics_history.iter()
            .map(|m| m.memory_mb)
            .sum::<f64>() / samples as f64;

        let max_memory = self.metrics_history.iter()
            .map(|m| m.memory_mb)
            .fold(0.0_f64, |a, b| a.max(b));

        let avg_cpu = self.metrics_history.iter()
            .map(|m| m.cpu_percent)
            .sum::<f64>() / samples as f64;

        let max_cpu = self.metrics_history.iter()
            .map(|m| m.cpu_percent)
            .fold(0.0_f64, |a, b| a.max(b));

        let duration = if samples > 1 {
            let first = self.metrics_history.front().unwrap().timestamp;
            let last = self.metrics_history.back().unwrap().timestamp;
            last - first
        } else {
            0
        };

        Some(PerformanceSummary {
            avg_memory_mb: avg_memory,
            max_memory_mb: max_memory,
            avg_cpu_percent: avg_cpu,
            max_cpu_percent: max_cpu,
            samples,
            duration_ms: duration,
        })
    }

    /// Get all alerts
    pub fn get_alerts(&self) -> &[PerformanceAlert] {
        &self.alerts
    }

    /// Clear alerts
    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    /// Get recent metrics
    pub fn get_recent_metrics(&self, count: usize) -> Vec<ProcessMetrics> {
        self.metrics_history.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    /// Get current metrics (latest sample)
    pub fn get_current_metrics(&self) -> Option<ProcessMetrics> {
        self.metrics_history.back().cloned()
    }
}

// ===== Platform-Specific Process Info =====

/// Get process metrics (platform-specific implementation)
pub fn get_process_metrics(process_id: u32) -> Option<ProcessMetrics> {
    #[cfg(target_os = "windows")]
    {
        get_process_metrics_windows(process_id)
    }

    #[cfg(target_os = "macos")]
    {
        get_process_metrics_macos(process_id)
    }

    #[cfg(target_os = "linux")]
    {
        get_process_metrics_linux(process_id)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn get_process_metrics_windows(process_id: u32) -> Option<ProcessMetrics> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // Use PowerShell to get process info
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            &format!(
                "Get-Process -Id {} | Select-Object WorkingSet64, CPU, Threads, HandleCount | ConvertTo-Json",
                process_id
            ),
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON (simplified - in production use serde_json)
    let memory_mb = extract_json_f64(&stdout, "WorkingSet64")
        .map(|bytes| bytes / 1024.0 / 1024.0)?;

    let cpu_percent = extract_json_f64(&stdout, "CPU").unwrap_or(0.0);
    let thread_count = extract_json_u32(&stdout, "Threads").unwrap_or(0);
    let handle_count = extract_json_u32(&stdout, "HandleCount").unwrap_or(0);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    Some(ProcessMetrics {
        process_id,
        memory_mb,
        cpu_percent,
        thread_count,
        handle_count,
        timestamp,
    })
}

#[cfg(target_os = "macos")]
fn get_process_metrics_macos(process_id: u32) -> Option<ProcessMetrics> {
    use std::process::Command;

    // Use ps to get process info
    let output = Command::new("ps")
        .args(&["-p", &process_id.to_string(), "-o", "rss,%cpu,ncmds"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() < 2 {
        return None;
    }

    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let memory_mb = parts[0].parse::<f64>().ok()? / 1024.0;
    let cpu_percent = parts[1].parse::<f64>().ok()?;
    let thread_count = parts[2].parse::<u32>().ok()?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    Some(ProcessMetrics {
        process_id,
        memory_mb,
        cpu_percent,
        thread_count,
        handle_count: 0,  // Not available on macOS via ps
        timestamp,
    })
}

#[cfg(target_os = "linux")]
fn get_process_metrics_linux(process_id: u32) -> Option<ProcessMetrics> {
    use std::fs;

    // Read /proc/[pid]/status
    let status_path = format!("/proc/{}/status", process_id);
    let status = fs::read_to_string(&status_path).ok()?;

    let mut memory_kb = 0u64;
    let mut threads = 0u32;

    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                memory_kb = parts[1].parse().ok()?;
            }
        } else if line.starts_with("Threads:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                threads = parts[1].parse().ok()?;
            }
        }
    }

    let memory_mb = memory_kb as f64 / 1024.0;

    // Read CPU usage from /proc/[pid]/stat
    let stat_path = format!("/proc/{}/stat", process_id);
    let stat = fs::read_to_string(&stat_path).ok()?;
    let cpu_percent = 0.0;  // Simplified - would need to calculate over time

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    Some(ProcessMetrics {
        process_id,
        memory_mb,
        cpu_percent,
        thread_count: threads,
        handle_count: 0,  // Not available on Linux
        timestamp,
    })
}

// ===== Helper Functions =====

#[cfg(target_os = "windows")]
fn extract_json_f64(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    if let Some(start) = json.find(&pattern) {
        let value_start = start + pattern.len();
        let value_str = &json[value_start..];
        let end = value_str.find(|c: char| !c.is_digit(10) && c != '.' && c != '-')?;
        value_str[..end].parse().ok()
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn extract_json_u32(json: &str, key: &str) -> Option<u32> {
    extract_json_f64(json, key).map(|v| v as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_monitor() {
        let mut monitor = ProcessMonitor::new(100);
        monitor.set_memory_threshold(1000.0);
        monitor.set_cpu_threshold(80.0);

        let metrics = ProcessMetrics {
            process_id: 1234,
            memory_mb: 500.0,
            cpu_percent: 50.0,
            thread_count: 4,
            handle_count: 100,
            timestamp: 1000,
        };

        monitor.record_metrics(metrics);

        let summary = monitor.get_summary().unwrap();
        assert_eq!(summary.avg_memory_mb, 500.0);
        assert_eq!(summary.samples, 1);
    }

    #[test]
    fn test_memory_alert() {
        let mut monitor = ProcessMonitor::new(100);
        monitor.set_memory_threshold(100.0);

        let metrics = ProcessMetrics {
            process_id: 1234,
            memory_mb: 200.0,  // Above threshold
            cpu_percent: 50.0,
            thread_count: 4,
            handle_count: 100,
            timestamp: 1000,
        };

        monitor.record_metrics(metrics);

        let alerts = monitor.get_alerts();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, AlertType::HighMemory);
    }

    #[test]
    fn test_cpu_alert() {
        let mut monitor = ProcessMonitor::new(100);
        monitor.set_cpu_threshold(80.0);

        let metrics = ProcessMetrics {
            process_id: 1234,
            memory_mb: 50.0,
            cpu_percent: 90.0,  // Above threshold
            thread_count: 4,
            handle_count: 100,
            timestamp: 1000,
        };

        monitor.record_metrics(metrics);

        let alerts = monitor.get_alerts();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, AlertType::HighCpu);
    }
}
