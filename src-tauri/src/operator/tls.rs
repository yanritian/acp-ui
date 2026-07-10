// TLS Configuration Module
// Implements TLS/HTTPS support for secure communication

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TLS Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub ca_path: Option<PathBuf>,
    pub min_version: TlsVersion,
    pub cipher_suites: Vec<String>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            ca_path: None,
            min_version: TlsVersion::Tls12,
            cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_AES_128_GCM_SHA256".to_string(),
            ],
        }
    }
}

/// TLS Version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

impl TlsConfig {
    pub fn validate(&self) -> Result<(), TlsError> {
        if !self.enabled {
            return Ok(());
        }

        let cert_path = self.cert_path.as_ref().ok_or(TlsError::MissingCertPath)?;
        let key_path = self.key_path.as_ref().ok_or(TlsError::MissingKeyPath)?;

        if !cert_path.exists() {
            return Err(TlsError::CertNotFound(cert_path.clone()));
        }

        if !key_path.exists() {
            return Err(TlsError::KeyNotFound(key_path.clone()));
        }

        if let Some(ca_path) = &self.ca_path {
            if !ca_path.exists() {
                return Err(TlsError::CaNotFound(ca_path.clone()));
            }
        }

        Ok(())
    }
}

/// TLS Errors
#[derive(Debug)]
pub enum TlsError {
    MissingCertPath,
    MissingKeyPath,
    CertNotFound(PathBuf),
    KeyNotFound(PathBuf),
    CaNotFound(PathBuf),
    InvalidCert(String),
    InvalidKey(String),
    HandshakeFailed(String),
}

impl std::fmt::Display for TlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCertPath => write!(f, "TLS certificate path not configured"),
            Self::MissingKeyPath => write!(f, "TLS private key path not configured"),
            Self::CertNotFound(path) => write!(f, "TLS certificate not found: {}", path.display()),
            Self::KeyNotFound(path) => write!(f, "TLS private key not found: {}", path.display()),
            Self::CaNotFound(path) => write!(f, "CA certificate not found: {}", path.display()),
            Self::InvalidCert(msg) => write!(f, "Invalid TLS certificate: {}", msg),
            Self::InvalidKey(msg) => write!(f, "Invalid TLS private key: {}", msg),
            Self::HandshakeFailed(msg) => write!(f, "TLS handshake failed: {}", msg),
        }
    }
}

impl std::error::Error for TlsError {}

/// Rate Limit Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub window_size_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_second: 100,
            burst_size: 200,
            window_size_seconds: 60,
        }
    }
}

/// Rate Limiter
pub struct RateLimiter {
    config: RateLimitConfig,
    tokens: f64,
    last_check: std::time::Instant,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: config.burst_size as f64,
            last_check: std::time::Instant::now(),
            config,
        }
    }

    pub fn allow_request(&mut self) -> bool {
        if !self.config.enabled {
            return true;
        }

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_check).as_secs_f64();
        self.last_check = now;

        // Refill tokens based on elapsed time
        self.tokens += elapsed * self.config.requests_per_second as f64;
        self.tokens = self.tokens.min(self.config.burst_size as f64);

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn get_remaining_tokens(&self) -> f64 {
        self.tokens
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

/// CSRF Protection
#[derive(Debug, Clone)]
pub struct CsrfConfig {
    pub enabled: bool,
    pub token_length: usize,
    pub cookie_name: String,
    pub header_name: String,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            token_length: 32,
            cookie_name: "csrf_token".to_string(),
            header_name: "X-CSRF-Token".to_string(),
        }
    }
}

/// CSP (Content Security Policy) Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CspConfig {
    pub enabled: bool,
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub connect_src: Vec<String>,
    pub font_src: Vec<String>,
    pub object_src: Vec<String>,
    pub media_src: Vec<String>,
    pub frame_src: Vec<String>,
}

impl Default for CspConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string()],
            connect_src: vec!["'self'".to_string()],
            font_src: vec!["'self'".to_string()],
            object_src: vec!["'none'".to_string()],
            media_src: vec!["'self'".to_string()],
            frame_src: vec!["'none'".to_string()],
        }
    }
}

impl CspConfig {
    pub fn to_header_value(&self) -> String {
        if !self.enabled {
            return String::new();
        }

        let mut parts = Vec::new();

        if !self.default_src.is_empty() {
            parts.push(format!("default-src {}", self.default_src.join(" ")));
        }

        if !self.script_src.is_empty() {
            parts.push(format!("script-src {}", self.script_src.join(" ")));
        }

        if !self.style_src.is_empty() {
            parts.push(format!("style-src {}", self.style_src.join(" ")));
        }

        if !self.img_src.is_empty() {
            parts.push(format!("img-src {}", self.img_src.join(" ")));
        }

        if !self.connect_src.is_empty() {
            parts.push(format!("connect-src {}", self.connect_src.join(" ")));
        }

        if !self.font_src.is_empty() {
            parts.push(format!("font-src {}", self.font_src.join(" ")));
        }

        if !self.object_src.is_empty() {
            parts.push(format!("object-src {}", self.object_src.join(" ")));
        }

        if !self.media_src.is_empty() {
            parts.push(format!("media-src {}", self.media_src.join(" ")));
        }

        if !self.frame_src.is_empty() {
            parts.push(format!("frame-src {}", self.frame_src.join(" ")));
        }

        parts.join("; ")
    }
}

/// Security Headers
#[derive(Debug, Clone)]
pub struct SecurityHeaders {
    pub csp: CspConfig,
    pub hsts_enabled: bool,
    pub hsts_max_age: u64,
    pub x_frame_options: String,
    pub x_content_type_options: String,
    pub referrer_policy: String,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            csp: CspConfig::default(),
            hsts_enabled: true,
            hsts_max_age: 31536000, // 1 year
            x_frame_options: "DENY".to_string(),
            x_content_type_options: "nosniff".to_string(),
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
        }
    }
}

impl SecurityHeaders {
    pub fn get_headers(&self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        let csp_value = self.csp.to_header_value();
        if !csp_value.is_empty() {
            headers.push(("Content-Security-Policy".to_string(), csp_value));
        }

        if self.hsts_enabled {
            headers.push((
                "Strict-Transport-Security".to_string(),
                format!("max-age={}; includeSubDomains", self.hsts_max_age),
            ));
        }

        headers.push(("X-Frame-Options".to_string(), self.x_frame_options.clone()));
        headers.push(("X-Content-Type-Options".to_string(), self.x_content_type_options.clone()));
        headers.push(("Referrer-Policy".to_string(), self.referrer_policy.clone()));

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.min_version, TlsVersion::Tls12);
    }

    #[test]
    fn test_rate_limiter() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_second: 10,
            burst_size: 20,
            window_size_seconds: 60,
        };
        let mut limiter = RateLimiter::new(config);

        // Should allow initial burst
        for _ in 0..20 {
            assert!(limiter.allow_request());
        }

        // Should deny after burst
        assert!(!limiter.allow_request());
    }

    #[test]
    fn test_csp_header() {
        let config = CspConfig::default();
        let header = config.to_header_value();
        assert!(header.contains("default-src 'self'"));
        assert!(header.contains("script-src 'self'"));
    }

    #[test]
    fn test_security_headers() {
        let headers = SecurityHeaders::default();
        let header_list = headers.get_headers();
        assert!(header_list.iter().any(|(k, _)| k == "Content-Security-Policy"));
        assert!(header_list.iter().any(|(k, _)| k == "Strict-Transport-Security"));
        assert!(header_list.iter().any(|(k, _)| k == "X-Frame-Options"));
    }
}