use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// HTTP client trait for making HTTP requests
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn send(
        &self,
        request: HttpRequest,
    ) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>>;

    /// Convenience method for GET requests
    async fn get(
        &self,
        url: &str,
    ) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request = HttpRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
        };
        self.send(request).await
    }

    /// Convenience method for POST requests with JSON body
    async fn post_json(
        &self,
        url: &str,
        json: &serde_json::Value,
    ) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let body = serde_json::to_vec(json)?;
        let request = HttpRequest {
            method: "POST".to_string(),
            url: url.to_string(),
            headers,
            body: Some(body),
        };
        self.send(request).await
    }

    /// Convenience method for POST requests with custom headers
    async fn post_with_headers(
        &self,
        url: &str,
        headers: std::collections::HashMap<String, String>,
        json: &serde_json::Value,
    ) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut request_headers = headers;
        request_headers.insert("Content-Type".to_string(), "application/json".to_string());

        let body = serde_json::to_vec(json)?;
        let request = HttpRequest {
            method: "POST".to_string(),
            url: url.to_string(),
            headers: request_headers,
            body: Some(body),
        };
        self.send(request).await
    }
}

/// HTTP request structure
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// HTTP response structure
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Get the status code
    pub fn status(&self) -> u16 {
        self.status_code
    }

    /// Check if the response indicates success
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// Get the response body as text
    pub fn text(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(String::from_utf8(self.body.clone())?)
    }

    /// Get the response body as JSON
    pub fn json<T: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::from_slice(&self.body)?)
    }
}

/// Delivery service trait for sending ActivityPub activities
#[async_trait]
pub trait DeliveryService: Send + Sync {
    async fn deliver_activity(
        &self,
        activity: Value,
        inbox_url: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn deliver_to_followers(
        &self,
        activity: Value,
        followers: &[String],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Database trait for storing and retrieving ActivityPub data
#[async_trait]
pub trait Database: Send + Sync {
    async fn create_actor(
        &self,
        actor: &DbActor,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_actor_by_id(
        &self,
        id: &str,
    ) -> Result<Option<DbActor>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_actor_by_username(
        &self,
        username: &str,
    ) -> Result<Option<DbActor>, Box<dyn std::error::Error + Send + Sync>>;
    async fn update_actor(
        &self,
        actor: &DbActor,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn delete_actor(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn create_activity(
        &self,
        activity: &DbActivity,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_activity_by_id(
        &self,
        id: &str,
    ) -> Result<Option<DbActivity>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_activities_by_actor(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbActivity>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_inbox_activities(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbActivity>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Database actor structure
#[derive(Debug, Clone)]
pub struct DbActor {
    pub id: String,
    pub username: String,
    pub name: String,
    pub summary: Option<String>,
    pub public_key_pem: String,
    pub private_key_pem: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database activity structure
#[derive(Debug, Clone)]
pub struct DbActivity {
    pub id: String,
    pub actor_id: String,
    pub activity_type: String,
    pub object: Value,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub published: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
