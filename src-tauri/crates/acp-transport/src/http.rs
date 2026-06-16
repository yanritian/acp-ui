//! HTTP transport - HTTP传输层

use crate::adapters::TransportAdapter;

/// HttpAdapter - HTTP请求适配器
pub struct HttpAdapter {
    base_url: String,
}

impl HttpAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub fn get(&self, path: &str) -> Result<String, String> {
        let url = format!("{}{}", self.base_url, path);
        ureq::get(&url)
            .call()
            .map_err(|e| format!("HTTP GET failed: {}", e))?
            .into_string()
            .map_err(|e| format!("Failed to read response: {}", e))
    }

    pub fn post(&self, path: &str, body: &str) -> Result<String, String> {
        let url = format!("{}{}", self.base_url, path);
        ureq::post(&url)
            .send_string(body)
            .map_err(|e| format!("HTTP POST failed: {}", e))?
            .into_string()
            .map_err(|e| format!("Failed to read response: {}", e))
    }
}

impl TransportAdapter for HttpAdapter {
    fn send(&self, message: &str) -> Result<(), String> {
        self.post("/", message)?;
        Ok(())
    }

    fn receive(&self) -> Result<String, String> {
        self.get("/")
    }

    fn close(&self) -> Result<(), String> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        true
    }
}

impl Default for HttpAdapter {
    fn default() -> Self {
        Self::new("http://localhost:1421")
    }
}