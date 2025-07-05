use async_trait::async_trait;
use anyhow::Result;
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

/// HTTP request representation
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
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
        self.body = Some(serde_json::to_vec(json)?);
        self.headers.insert("content-type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}

/// HTTP response representation
#[derive(Debug)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn text(&self) -> Result<String> {
        Ok(String::from_utf8(self.body.clone())?)
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_slice(&self.body)?)
    }
}

/// Abstract HTTP client trait
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
        let request = HttpRequest::new("POST", url)
            .with_json_body(json)?;
        self.send(request).await
    }

    /// Convenience method for POST requests with custom headers
    async fn post_with_headers(&self, url: &str, headers: HashMap<String, String>, json: &Value) -> Result<HttpResponse> {
        let mut request = HttpRequest::new("POST", url)
            .with_json_body(json)?;
        
        for (name, value) in headers {
            request.headers.insert(name, value);
        }
        
        self.send(request).await
    }
}

/// reqwest implementation of HttpClient
pub mod reqwest {
    use super::*;
    use reqwest::Client;
    use std::time::Duration;

    pub struct ReqwestClient {
        client: Client,
    }

    impl ReqwestClient {
        pub fn new() -> Self {
            Self {
                client: Client::builder()
                    .timeout(Duration::from_secs(30))
                    .build()
                    .expect("Failed to create reqwest client"),
            }
        }

        pub fn with_timeout(timeout: Duration) -> Self {
            Self {
                client: Client::builder()
                    .timeout(timeout)
                    .build()
                    .expect("Failed to create reqwest client"),
            }
        }
    }

    impl Default for ReqwestClient {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl HttpClient for ReqwestClient {
        async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
            let method = reqwest::Method::from_bytes(request.method.as_bytes())?;
            let mut req_builder = self.client.request(method, &request.url);

            // Add headers
            for (name, value) in &request.headers {
                req_builder = req_builder.header(name, value);
            }

            // Add body if present
            if let Some(body) = request.body {
                req_builder = req_builder.body(body);
            }

            let response = req_builder.send().await?;
            let status = StatusCode(response.status().as_u16());
            
            let mut headers = HashMap::new();
            for (name, value) in response.headers() {
                headers.insert(
                    name.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                );
            }

            let body = response.bytes().await?.to_vec();

            Ok(HttpResponse {
                status,
                headers,
                body,
            })
        }
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
        assert_eq!(request.headers.get("Authorization").unwrap(), "Bearer token");
    }

    #[test]
    fn test_http_request_with_json() {
        let json_data = json!({"key": "value"});
        let request = HttpRequest::new("POST", "https://example.com")
            .with_json_body(&json_data)
            .unwrap();

        assert_eq!(request.method, "POST");
        assert!(request.body.is_some());
        assert_eq!(request.headers.get("content-type").unwrap(), "application/json");
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
}