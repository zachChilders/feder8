use anyhow::Result;
use async_trait::async_trait;
use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use std::io::Read;
use heapless::Vec as HeaplessVec;
use serde_json::Value;
use std::collections::HashMap;

/// HTTP status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

impl StatusCode {
    pub fn is_success(&self) -> bool {
        self.0 >= 200 && self.0 < 300
    }
}

/// HTTP request representation adapted for embedded systems
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<HeaplessVec<u8, 4096>>, // Limited to 4KB for embedded constraints
}

impl HttpRequest {
    pub fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_json_body(mut self, json: &Value) -> Result<Self> {
        let json_bytes = serde_json::to_vec(json)?;
        if json_bytes.len() > 4096 {
            return Err(anyhow::anyhow!("JSON payload too large for embedded system"));
        }
        
        let mut body = HeaplessVec::new();
        body.extend_from_slice(&json_bytes).map_err(|_| {
            anyhow::anyhow!("Failed to store JSON body in embedded buffer")
        })?;
        
        self.body = Some(body);
        self.headers.insert("content-type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn with_body(mut self, body_data: &[u8]) -> Result<Self> {
        if body_data.len() > 4096 {
            return Err(anyhow::anyhow!("Body too large for embedded system"));
        }
        
        let mut body = HeaplessVec::new();
        body.extend_from_slice(body_data).map_err(|_| {
            anyhow::anyhow!("Failed to store body in embedded buffer")
        })?;
        
        self.body = Some(body);
        Ok(self)
    }
}

/// HTTP response representation adapted for embedded systems
#[derive(Debug)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: HeaplessVec<u8, 8192>, // Limited to 8KB for response buffer
}

impl HttpResponse {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn text(&self) -> Result<String> {
        Ok(String::from_utf8(self.body.to_vec())?)
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_slice(&self.body)?)
    }
}

/// Abstract HTTP client trait compatible with embedded systems
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Send an HTTP request
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse>;

    /// Convenience method for GET requests
    async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.send(HttpRequest::new("GET", url)).await
    }

    /// Convenience method for POST requests with JSON body
    async fn post_json(&self, url: &str, json: &Value) -> Result<HttpResponse> {
        let request = HttpRequest::new("POST", url).with_json_body(json)?;
        self.send(request).await
    }

    /// Convenience method for POST requests with custom headers
    async fn post_with_headers(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        json: &Value,
    ) -> Result<HttpResponse> {
        let mut request = HttpRequest::new("POST", url).with_json_body(json)?;

        for (name, value) in headers {
            request.headers.insert(name, value);
        }

        self.send(request).await
    }
}

/// ESP-IDF HTTP client implementation
pub struct EspHttpClient {
    config: Configuration,
}

impl EspHttpClient {
    pub fn new() -> Result<Self> {
        let config = Configuration {
            timeout: Some(core::time::Duration::from_secs(30)),
            buffer_size: Some(8192),
            ..Default::default()
        };

        Ok(Self { config })
    }

    pub fn with_timeout(timeout_secs: u64) -> Result<Self> {
        let config = Configuration {
            timeout: Some(core::time::Duration::from_secs(timeout_secs)),
            buffer_size: Some(8192),
            ..Default::default()
        };

        Ok(Self { config })
    }
}

impl Default for EspHttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create ESP HTTP client")
    }
}

#[async_trait]
impl HttpClient for EspHttpClient {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        let connection = EspHttpConnection::new(&self.config)?;
        let mut client = Client::wrap(connection);

        // Parse method
        let method = match request.method.as_str() {
            "GET" => embedded_svc::http::Method::Get,
            "POST" => embedded_svc::http::Method::Post,
            "PUT" => embedded_svc::http::Method::Put,
            "DELETE" => embedded_svc::http::Method::Delete,
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", request.method)),
        };

        // Build headers
        let mut headers = std::vec![];
        for (name, value) in &request.headers {
            headers.push((name.as_str(), value.as_str()));
        }

        // Prepare request
        let mut req = client.request(method, &request.url, &headers)?;

        // Send body if present
        if let Some(body) = &request.body {
            req.write_all(body.as_slice())?;
        }

        // Submit request
        let response = req.submit()?;
        let status = StatusCode(response.status());

        // Read response headers
        let mut resp_headers = HashMap::new();
        // Note: ESP-IDF doesn't provide easy access to response headers
        // This is a limitation of the embedded environment

        // Read response body
        let mut body = HeaplessVec::new();
        let mut buffer = [0u8; 512];
        let mut response = response;
        
        loop {
            match response.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(bytes_read) => {
                    if body.len() + bytes_read > body.capacity() {
                        return Err(anyhow::anyhow!("Response body too large for embedded buffer"));
                    }
                    body.extend_from_slice(&buffer[..bytes_read]).map_err(|_| {
                        anyhow::anyhow!("Failed to store response body")
                    })?;
                }
                Err(e) => return Err(anyhow::anyhow!("Failed to read response: {}", e)),
            }
        }

        Ok(HttpResponse {
            status,
            headers: resp_headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_http_request_builder() {
        let request = HttpRequest::new("GET", "https://example.com")
            .with_header("Authorization", "Bearer token");

        assert_eq!(request.method, "GET");
        assert_eq!(request.url, "https://example.com");
        assert_eq!(
            request.headers.get("Authorization").unwrap(),
            "Bearer token"
        );
    }

    #[test]
    fn test_http_request_with_json() {
        let json_data = json!({"key": "value"});
        let request = HttpRequest::new("POST", "https://example.com")
            .with_json_body(&json_data)
            .unwrap();

        assert_eq!(request.method, "POST");
        assert!(request.body.is_some());
        assert_eq!(
            request.headers.get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_status_code_success() {
        assert!(StatusCode(200).is_success());
        assert!(StatusCode(201).is_success());
        assert!(StatusCode(299).is_success());
        assert!(!StatusCode(300).is_success());
        assert!(!StatusCode(400).is_success());
        assert!(!StatusCode(500).is_success());
    }

    #[test]
    fn test_large_json_rejection() {
        let mut large_data = std::collections::HashMap::new();
        // Create a JSON that's too large for embedded constraints
        for i in 0..1000 {
            large_data.insert(format!("key_{}", i), format!("value_{}", i.to_string().repeat(100)));
        }
        let large_json = serde_json::to_value(&large_data).unwrap();
        
        let result = HttpRequest::new("POST", "https://example.com")
            .with_json_body(&large_json);

        assert!(result.is_err());
    }
}