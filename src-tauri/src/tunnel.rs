use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use serde::{Serialize, Deserialize};

/// Ngrok tunnel manager
#[derive(Clone)]
pub struct NgrokManager {
    process: Arc<Mutex<Option<Child>>>,
    public_url: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub running: bool,
    pub public_url: Option<String>,
    pub error: Option<String>,
}

impl NgrokManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            public_url: Arc::new(Mutex::new(None)),
        }
    }

    /// Start ngrok tunnel and return public URL
    pub fn start(&self, token: &str, port: u16, region: &str, app_handle: AppHandle) -> Result<String, String> {
        // Check if already running
        if self.is_running() {
            return Err("Tunnel already running".to_string());
        }

        // Check ngrok binary exists
        let ngrok_path = self.find_ngrok_binary()?;

        println!("Starting ngrok on port {} with region {}", port, region);

        // Start ngrok process
        let mut cmd = Command::new(&ngrok_path);
        cmd.arg("http")
            .arg("--authtoken")
            .arg(token)
            .arg("--region")
            .arg(region)
            .arg(port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()
            .map_err(|e| format!("Failed to start ngrok: {}. Please ensure ngrok is installed and in PATH.", e))?;

        // Store process
        *self.process.lock().unwrap() = Some(child);

        // Wait for ngrok to start and get public URL from API
        let public_url = self.wait_for_url(port)?;

        // Store URL
        *self.public_url.lock().unwrap() = Some(public_url.clone());

        // Emit event
        let _ = app_handle.emit("tunnel-started", serde_json::json!({
            "provider": "ngrok",
            "public_url": public_url,
        }));

        Ok(public_url)
    }

    /// Stop ngrok tunnel
    pub fn stop(&self, app_handle: AppHandle) -> Result<(), String> {
        let mut process_guard = self.process.lock().unwrap();

        if let Some(mut child) = process_guard.take() {
            child.kill()
                .map_err(|e| format!("Failed to kill ngrok process: {}", e))?;

            // Wait for process to exit
            let _ = child.wait();

            *self.public_url.lock().unwrap() = None;

            // Emit event
            let _ = app_handle.emit("tunnel-stopped", ());

            println!("Ngrok tunnel stopped");
        }

        Ok(())
    }

    /// Check if tunnel is running
    pub fn is_running(&self) -> bool {
        self.process.lock().unwrap().is_some()
    }

    /// Get current public URL
    pub fn get_public_url(&self) -> Option<String> {
        self.public_url.lock().unwrap().clone()
    }

    /// Get tunnel status
    pub fn get_status(&self) -> TunnelStatus {
        TunnelStatus {
            running: self.is_running(),
            public_url: self.get_public_url(),
            error: None,
        }
    }

    /// Find ngrok binary in PATH or common locations
    fn find_ngrok_binary(&self) -> Result<String, String> {
        // Try ngrok in PATH first
        if Command::new("ngrok").arg("version").output().is_ok() {
            return Ok("ngrok".to_string());
        }

        // Check common Windows locations
        #[cfg(windows)]
        {
            let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
            let common_paths = [
                "C:\\Program Files\\ngrok\\ngrok.exe",
                "C:\\Program Files (x86)\\ngrok\\ngrok.exe",
                "C:\\ngrok\\ngrok.exe",
                &format!("{}\\.ngrok\\ngrok.exe", user_profile) as &str,
            ];

            for path in common_paths.iter() {
                if std::path::Path::new(path).exists() {
                    return Ok(path.to_string());
                }
            }
        }

        // Check common Unix locations
        #[cfg(not(windows))]
        {
            let common_paths = [
                "/usr/local/bin/ngrok",
                "/usr/bin/ngrok",
                "/opt/ngrok/ngrok",
            ];

            for path in common_paths.iter() {
                if std::path::Path::new(path).exists() {
                    return Ok(path.to_string());
                }
            }
        }

        Err("ngrok not found. Please install ngrok from https://ngrok.com/download and add it to PATH.".to_string())
    }

    /// Wait for ngrok to start and fetch public URL from local API
    fn wait_for_url(&self, port: u16) -> Result<String, String> {
        // Ngrok exposes a local API at http://127.0.0.1:4040
        let api_url = "http://127.0.0.1:4040/api/tunnels";

        // Wait up to 10 seconds for ngrok to start
        for attempt in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Try to fetch tunnel info from ngrok API
            let response = self.fetch_ngrok_api(api_url, port);

            if let Ok(json) = response {
                // Parse JSON and find public URL for our tunnel
                if let Some(url) = self.extract_public_url(&json, port) {
                    println!("Ngrok tunnel started: {}", url);
                    return Ok(url);
                }
            }

            println!("Waiting for ngrok to start... (attempt {})", attempt + 1);
        }

        Err("Ngrok failed to start within timeout. Check ngrok logs for errors.".to_string())
    }

    /// Fetch tunnel info from ngrok local API
    fn fetch_ngrok_api(&self, url: &str, _port: u16) -> Result<serde_json::Value, String> {
        // Use ureq for HTTP request (works on all platforms)
        let response = ureq::get(url)
            .timeout(std::time::Duration::from_secs(5))
            .call()
            .map_err(|e| format!("Failed to call ngrok API: {}", e))?;

        let json: serde_json::Value = response.into_json()
            .map_err(|e| format!("Failed to parse ngrok API response: {}", e))?;

        Ok(json)
    }

    /// Extract public URL from ngrok API response
    fn extract_public_url(&self, json: &serde_json::Value, expected_port: u16) -> Option<String> {
        if let Some(tunnels) = json.get("tunnels").and_then(|t| t.as_array()) {
            for tunnel in tunnels {
                // Check if this tunnel is for our port (exact match, not substring)
                let addr = tunnel.get("config")
                    .and_then(|c| c.get("addr"))
                    .and_then(|a| a.as_str());

                if let Some(addr_str) = addr {
                    // Parse port from addr (format: "127.0.0.1:PORT" or "PORT")
                    let tunnel_port = if addr_str.contains(':') {
                        // Split on ':' and parse the last part as port
                        addr_str.split(':').next_back()
                            .and_then(|s| s.parse::<u16>().ok())
                    } else {
                        // Try to parse the whole string as port
                        addr_str.parse::<u16>().ok()
                    };

                    // Exact port match (not substring)
                    if tunnel_port == Some(expected_port) {
                        // Get public URL
                        return tunnel.get("public_url")
                            .and_then(|u| u.as_str())
                            .map(|s| s.to_string());
                    }
                }

                // Fallback: just return first https tunnel
                if tunnel.get("proto").and_then(|p| p.as_str()) == Some("https") {
                    return tunnel.get("public_url")
                        .and_then(|u| u.as_str())
                        .map(|s| s.to_string());
                }
            }
        }
        None
    }
}

impl Default for NgrokManager {
    fn default() -> Self {
        Self::new()
    }
}