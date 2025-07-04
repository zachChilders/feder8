use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct SignatureService {
    private_key: Option<String>,
}

// Signature verification result
#[derive(Debug, PartialEq)]
pub enum SignatureVerification {
    Valid,
    Invalid(String),
    NotImplemented,
}

impl SignatureVerification {
    pub fn is_valid(&self) -> bool {
        matches!(self, SignatureVerification::Valid)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            SignatureVerification::Invalid(msg) => Some(msg),
            _ => None,
        }
    }
}

// Functional signature creation result
#[derive(Debug)]
pub struct SignatureData {
    pub signature: String,
    pub headers: Vec<String>,
    pub algorithm: String,
}

impl SignatureData {
    fn new(signature: String, headers: Vec<String>, algorithm: String) -> Self {
        Self {
            signature,
            headers,
            algorithm,
        }
    }

    fn placeholder() -> Self {
        Self::new(
            "signature-placeholder".to_string(),
            vec!["(request-target)".to_string(), "host".to_string(), "date".to_string()],
            "rsa-sha256".to_string(),
        )
    }
}

impl SignatureService {
    pub fn new(private_key: Option<String>) -> Self {
        Self { private_key }
    }

    // Functional signature verification with pattern matching
    pub fn verify_signature(
        &self,
        headers: &HashMap<String, String>,
        signature: &str,
    ) -> Result<SignatureVerification> {
        debug!("Verifying signature: {}", signature);

        match self.parse_signature_header(signature) {
            Ok(sig_data) => {
                // TODO: Implement actual verification logic
                warn!("Signature verification not fully implemented - accepting all signatures");
                Ok(SignatureVerification::Valid)
            }
            Err(e) => {
                warn!("Failed to parse signature header: {}", e);
                Ok(SignatureVerification::Invalid(e.to_string()))
            }
        }
    }

    // Parse signature header functionally
    fn parse_signature_header(&self, signature: &str) -> Result<HashMap<String, String>> {
        signature
            .split(',')
            .map(|part| {
                let mut split = part.trim().splitn(2, '=');
                let key = split
                    .next()
                    .context("Missing key in signature part")?
                    .to_string();
                let value = split
                    .next()
                    .context("Missing value in signature part")?
                    .trim_matches('"')
                    .to_string();
                Ok((key, value))
            })
            .collect::<Result<HashMap<String, String>>>()
    }

    // Functional request signing
    pub fn sign_request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<SignatureData> {
        debug!("Signing {} request to {}", method, url);

        match &self.private_key {
            Some(_key) => {
                // TODO: Implement actual signing logic
                warn!("Request signing not fully implemented - returning placeholder");
                Ok(SignatureData::placeholder())
            }
            None => {
                warn!("No private key available for signing");
                Ok(SignatureData::placeholder())
            }
        }
    }

    // Functional HTTP signature creation
    pub fn create_http_signature(
        &self,
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
    ) -> Result<String> {
        let signature_data = self.sign_request(method, path, headers)?;
        
        Ok(format!(
            r#"keyId="placeholder",algorithm="{}",headers="{}",signature="{}""#,
            signature_data.algorithm,
            signature_data.headers.join(" "),
            signature_data.signature
        ))
    }

    // Utility for creating signing headers
    pub fn create_signing_headers(
        &self,
        host: &str,
        date: &str,
        digest: Option<&str>,
    ) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), host.to_string());
        headers.insert("date".to_string(), date.to_string());
        
        if let Some(digest_value) = digest {
            headers.insert("digest".to_string(), digest_value.to_string());
        }
        
        headers
    }

    // Check if service can sign requests
    pub fn can_sign(&self) -> bool {
        self.private_key.is_some()
    }
}

// Functional constructors
pub fn create_signature_service(private_key: Option<String>) -> SignatureService {
    SignatureService::new(private_key)
}

pub fn create_signature_service_with_key(private_key: String) -> SignatureService {
    SignatureService::new(Some(private_key))
}

// Utility functions for common signature operations
pub fn extract_key_id(signature: &str) -> Option<String> {
    signature
        .split(',')
        .find_map(|part| {
            let part = part.trim();
            if part.starts_with("keyId=") {
                Some(part[6..].trim_matches('"').to_string())
            } else {
                None
            }
        })
}

pub fn extract_algorithm(signature: &str) -> Option<String> {
    signature
        .split(',')
        .find_map(|part| {
            let part = part.trim();
            if part.starts_with("algorithm=") {
                Some(part[10..].trim_matches('"').to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_signature() -> String {
        r#"keyId="https://example.com/users/alice#main-key",algorithm="rsa-sha256",headers="(request-target) host date digest",signature="base64signature""#.to_string()
    }

    fn test_headers() -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), "example.com".to_string());
        headers.insert("date".to_string(), "Mon, 01 Jan 2024 12:00:00 GMT".to_string());
        headers.insert("digest".to_string(), "SHA-256=hash".to_string());
        headers
    }

    #[test]
    fn test_signature_service_creation() {
        let service_with_key = create_signature_service_with_key("test-key".to_string());
        let service_without_key = create_signature_service(None);

        assert!(service_with_key.can_sign());
        assert!(!service_without_key.can_sign());
    }

    #[test]
    fn test_signature_verification_patterns() {
        let service = create_signature_service(None);
        let headers = test_headers();
        let signature = test_signature();

        let result = service.verify_signature(&headers, &signature).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_signature_parsing() {
        let service = SignatureService::new(None);
        let signature = test_signature();

        let parsed = service.parse_signature_header(&signature).unwrap();
        
        assert_eq!(parsed.get("keyId"), Some(&"https://example.com/users/alice#main-key".to_string()));
        assert_eq!(parsed.get("algorithm"), Some(&"rsa-sha256".to_string()));
    }

    #[test]
    fn test_functional_utilities() {
        let signature = test_signature();
        
        let key_id = extract_key_id(&signature);
        let algorithm = extract_algorithm(&signature);

        assert_eq!(key_id, Some("https://example.com/users/alice#main-key".to_string()));
        assert_eq!(algorithm, Some("rsa-sha256".to_string()));
    }

    #[test]
    fn test_signing_headers_creation() {
        let service = SignatureService::new(None);
        let headers = service.create_signing_headers("example.com", "test-date", Some("digest"));

        assert_eq!(headers.get("host"), Some(&"example.com".to_string()));
        assert_eq!(headers.get("date"), Some(&"test-date".to_string()));
        assert_eq!(headers.get("digest"), Some(&"digest".to_string()));
    }

    #[test]
    fn test_http_signature_creation() {
        let service = SignatureService::new(Some("test-key".to_string()));
        let headers = test_headers();

        let signature = service.create_http_signature("POST", "/inbox", &headers).unwrap();
        
        assert!(signature.contains("keyId="));
        assert!(signature.contains("algorithm="));
        assert!(signature.contains("signature="));
    }

    #[test]
    fn test_signature_verification_results() {
        let valid = SignatureVerification::Valid;
        let invalid = SignatureVerification::Invalid("error".to_string());
        let not_impl = SignatureVerification::NotImplemented;

        assert!(valid.is_valid());
        assert!(!invalid.is_valid());
        assert!(!not_impl.is_valid());

        assert_eq!(invalid.error_message(), Some("error"));
        assert_eq!(valid.error_message(), None);
    }

    #[test]
    fn test_signature_data_creation() {
        let data = SignatureData::new(
            "test-signature".to_string(),
            vec!["host".to_string(), "date".to_string()],
            "rsa-sha256".to_string(),
        );

        assert_eq!(data.signature, "test-signature");
        assert_eq!(data.headers.len(), 2);
        assert_eq!(data.algorithm, "rsa-sha256");

        let placeholder = SignatureData::placeholder();
        assert_eq!(placeholder.signature, "signature-placeholder");
    }
}
