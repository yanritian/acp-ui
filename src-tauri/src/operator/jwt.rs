// JWT Utilities Module
// Implements JWT parsing, validation, and signature verification

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// JWT Header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: Option<String>,
    pub kid: Option<String>,
}

/// JWT Payload (Claims)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtPayload {
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: Option<String>,
    pub exp: Option<u64>,
    pub iat: Option<u64>,
    pub nbf: Option<u64>,
    pub jti: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
}

/// JWT Token
#[derive(Debug, Clone)]
pub struct JwtToken {
    pub header: JwtHeader,
    pub payload: JwtPayload,
    pub signature: String,
}

impl JwtToken {
    /// Parse JWT token from string
    pub fn from_str(token: &str) -> Result<Self, JwtError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(JwtError::InvalidFormat("JWT must have 3 parts".to_string()));
        }

        // Decode header
        let header_bytes = BASE64
            .decode(parts[0])
            .map_err(|e| JwtError::InvalidFormat(format!("Invalid header: {}", e)))?;
        let header: JwtHeader = serde_json::from_slice(&header_bytes)
            .map_err(|e| JwtError::InvalidFormat(format!("Invalid header JSON: {}", e)))?;

        // Decode payload
        let payload_bytes = BASE64
            .decode(parts[1])
            .map_err(|e| JwtError::InvalidFormat(format!("Invalid payload: {}", e)))?;
        let payload: JwtPayload = serde_json::from_slice(&payload_bytes)
            .map_err(|e| JwtError::InvalidFormat(format!("Invalid payload JSON: {}", e)))?;

        // Signature (don't decode, just store as string)
        let signature = parts[2].to_string();

        Ok(Self {
            header,
            payload,
            signature,
        })
    }

    /// Validate JWT expiration
    pub fn validate_expiration(&self) -> Result<(), JwtError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| JwtError::ValidationError("Cannot get current time".to_string()))?
            .as_secs();

        if let Some(exp) = self.payload.exp {
            if now > exp {
                return Err(JwtError::Expired);
            }
        }

        if let Some(nbf) = self.payload.nbf {
            if now < nbf {
                return Err(JwtError::NotYetValid);
            }
        }

        Ok(())
    }

    /// Validate JWT issuer
    pub fn validate_issuer(&self, expected_issuer: &str) -> Result<(), JwtError> {
        match &self.payload.iss {
            Some(iss) if iss == expected_issuer => Ok(()),
            Some(iss) => Err(JwtError::ValidationError(format!(
                "Invalid issuer: expected {}, got {}",
                expected_issuer, iss
            ))),
            None => Err(JwtError::ValidationError("Missing issuer".to_string())),
        }
    }

    /// Validate JWT audience
    pub fn validate_audience(&self, expected_audience: &str) -> Result<(), JwtError> {
        match &self.payload.aud {
            Some(aud) if aud == expected_audience => Ok(()),
            Some(aud) => Err(JwtError::ValidationError(format!(
                "Invalid audience: expected {}, got {}",
                expected_audience, aud
            ))),
            None => Err(JwtError::ValidationError("Missing audience".to_string())),
        }
    }

    /// Verify signature (simplified - for testing only)
    pub fn verify_signature(&self, secret: &str) -> Result<bool, JwtError> {
        // TODO: Implement proper signature verification using the algorithm from header
        // For now, perform basic HMAC-SHA256 verification

        let header_b64 = BASE64.encode(serde_json::to_string(&self.header).unwrap());
        let payload_b64 = BASE64.encode(serde_json::to_string(&self.payload).unwrap());

        let signing_input = format!("{}.{}", header_b64, payload_b64);

        // Calculate HMAC-SHA256
        let mut hasher = Sha256::new();
        hasher.update(signing_input.as_bytes());
        hasher.update(secret.as_bytes());
        let expected_signature = BASE64.encode(hasher.finalize());

        Ok(self.signature == expected_signature)
    }
}

/// JWT Errors
#[derive(Debug)]
pub enum JwtError {
    InvalidFormat(String),
    Expired,
    NotYetValid,
    ValidationError(String),
    SignatureInvalid,
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(msg) => write!(f, "Invalid JWT format: {}", msg),
            Self::Expired => write!(f, "JWT token has expired"),
            Self::NotYetValid => write!(f, "JWT token is not yet valid"),
            Self::ValidationError(msg) => write!(f, "JWT validation error: {}", msg),
            Self::SignatureInvalid => write!(f, "JWT signature is invalid"),
        }
    }
}

impl std::error::Error for JwtError {}

/// JWKS Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwkKey {
    pub kty: String,
    pub kid: String,
    pub use_: Option<String>,
    pub alg: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
}

/// JWKS (JSON Web Key Set)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<JwkKey>,
}

impl Jwks {
    /// Find key by kid (Key ID)
    pub fn find_key(&self, kid: &str) -> Option<&JwkKey> {
        self.keys.iter().find(|k| k.kid == kid)
    }

    /// Get signing key (first RSA key)
    pub fn get_signing_key(&self) -> Option<&JwkKey> {
        self.keys.iter().find(|k| k.kty == "RSA")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_parsing() {
        // Create a simple JWT for testing
        let header = JwtHeader {
            alg: "HS256".to_string(),
            typ: Some("JWT".to_string()),
            kid: None,
        };
        let payload = JwtPayload {
            iss: Some("test-issuer".to_string()),
            sub: Some("user_123".to_string()),
            aud: Some("test-audience".to_string()),
            exp: Some(9999999999),
            iat: Some(1234567890),
            nbf: None,
            jti: None,
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            email_verified: Some(true),
        };

        let header_b64 = BASE64.encode(serde_json::to_string(&header).unwrap());
        let payload_b64 = BASE64.encode(serde_json::to_string(&payload).unwrap());
        let token_str = format!("{}.{}.signature", header_b64, payload_b64);

        let token = JwtToken::from_str(&token_str).unwrap();
        assert_eq!(token.header.alg, "HS256");
        assert_eq!(token.payload.sub, Some("user_123".to_string()));
    }

    #[test]
    fn test_jwt_expiration_validation() {
        let header = JwtHeader {
            alg: "HS256".to_string(),
            typ: None,
            kid: None,
        };
        let payload = JwtPayload {
            iss: None,
            sub: None,
            aud: None,
            exp: Some(9999999999), // Far future
            iat: None,
            nbf: None,
            jti: None,
            name: None,
            email: None,
            email_verified: None,
        };

        let header_b64 = BASE64.encode(serde_json::to_string(&header).unwrap());
        let payload_b64 = BASE64.encode(serde_json::to_string(&payload).unwrap());
        let token_str = format!("{}.{}.signature", header_b64, payload_b64);

        let token = JwtToken::from_str(&token_str).unwrap();
        assert!(token.validate_expiration().is_ok());
    }

    #[test]
    fn test_jwt_issuer_validation() {
        let header = JwtHeader {
            alg: "HS256".to_string(),
            typ: None,
            kid: None,
        };
        let payload = JwtPayload {
            iss: Some("test-issuer".to_string()),
            sub: None,
            aud: None,
            exp: None,
            iat: None,
            nbf: None,
            jti: None,
            name: None,
            email: None,
            email_verified: None,
        };

        let header_b64 = BASE64.encode(serde_json::to_string(&header).unwrap());
        let payload_b64 = BASE64.encode(serde_json::to_string(&payload).unwrap());
        let token_str = format!("{}.{}.signature", header_b64, payload_b64);

        let token = JwtToken::from_str(&token_str).unwrap();
        assert!(token.validate_issuer("test-issuer").is_ok());
        assert!(token.validate_issuer("wrong-issuer").is_err());
    }

    #[test]
    fn test_jwks_key_lookup() {
        let jwks = Jwks {
            keys: vec![
                JwkKey {
                    kty: "RSA".to_string(),
                    kid: "key1".to_string(),
                    use_: Some("sig".to_string()),
                    alg: Some("RS256".to_string()),
                    n: None,
                    e: None,
                },
                JwkKey {
                    kty: "RSA".to_string(),
                    kid: "key2".to_string(),
                    use_: Some("sig".to_string()),
                    alg: Some("RS256".to_string()),
                    n: None,
                    e: None,
                },
            ],
        };

        assert!(jwks.find_key("key1").is_some());
        assert!(jwks.find_key("key3").is_none());
        assert!(jwks.get_signing_key().is_some());
    }
}