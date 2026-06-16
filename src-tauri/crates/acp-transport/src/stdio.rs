//! Stdio transport - 标准输入输出传输层

use crate::adapters::TransportAdapter;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

/// StdioAdapter - 进程标准输入输出适配器
pub struct StdioAdapter {
    process: Arc<Mutex<Option<Child>>>,
}

impl StdioAdapter {
    pub fn new() -> Self {
        Self { process: Arc::new(Mutex::new(None)) }
    }

    pub fn spawn(&self, command: &str, args: &[String]) -> Result<(), String> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let mut process_guard = self.process.lock().map_err(|e| e.to_string())?;
        *process_guard = Some(child);
        Ok(())
    }
}

impl TransportAdapter for StdioAdapter {
    fn send(&self, message: &str) -> Result<(), String> {
        let mut process_guard = self.process.lock().map_err(|e| e.to_string())?;
        if let Some(ref mut child) = *process_guard {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(message.as_bytes())
                    .map_err(|e| format!("Failed to write to stdin: {}", e))?;
                stdin.flush()
                    .map_err(|e| format!("Failed to flush stdin: {}", e))?;
                Ok(())
            } else {
                Err("stdin not available".to_string())
            }
        } else {
            Err("Process not running".to_string())
        }
    }

    fn receive(&self) -> Result<String, String> {
        // 简化实现：返回空字符串
        // 完整实现需要在后台线程读取stdout
        Ok(String::new())
    }

    fn close(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().map_err(|e| e.to_string())?;
        if let Some(ref mut child) = *process_guard {
            let _ = child.kill();
            let _ = child.wait();
            *process_guard = None;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        match self.process.lock() {
            Ok(guard) => guard.is_some(),
            Err(_) => false,
        }
    }
}

impl Default for StdioAdapter {
    fn default() -> Self {
        Self::new()
    }
}