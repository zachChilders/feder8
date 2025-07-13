use crate::traits::{HttpClient, HttpRequest, HttpResponse};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// Reqwest-based HTTP client for native environments
pub struct ReqwestClient {
    client: Client,
}

impl ReqwestClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { client })
    }

    pub fn with_timeout(timeout: Duration) -> Result<Self> {
        let client = Client::builder().timeout(timeout).build()?;
        Ok(Self { client })
    }

    /// Convenience method for GET requests
    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        let response = self.client.get(url).send().await?;
        let status_code = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.bytes().await?.to_vec();

        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }

    /// Convenience method for POST requests with JSON body
    pub async fn post_json(&self, url: &str, json: &Value) -> Result<HttpResponse> {
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(json)
            .send()
            .await?;

        let status_code = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.bytes().await?.to_vec();

        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }

    /// Convenience method for POST requests with custom headers
    pub async fn post_with_headers(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        json: &Value,
    ) -> Result<HttpResponse> {
        let mut request = self.client.post(url).json(json);

        for (name, value) in headers {
            request = request.header(name, value);
        }

        let response = request.send().await?;
        let status_code = response.status().as_u16();
        let response_headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.bytes().await?.to_vec();

        Ok(HttpResponse {
            status_code,
            headers: response_headers,
            body,
        })
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let method = reqwest::Method::from_bytes(request.method.as_bytes())?;
        let mut req = self.client.request(method, &request.url);

        // Add headers
        for (name, value) in request.headers {
            req = req.header(name, value);
        }

        // Add body if present
        if let Some(body) = request.body {
            req = req.body(body);
        }

        let response = req.send().await?;
        let status_code = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.bytes().await?.to_vec();

        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new().expect("Failed to create ReqwestClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_reqwest_client_creation() {
        let client = ReqwestClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_reqwest_client_with_timeout() {
        let timeout = Duration::from_secs(10);
        let client = ReqwestClient::with_timeout(timeout);
        assert!(client.is_ok());
    }
} 