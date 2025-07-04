use crate::config::Config;
use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct DeliveryService {
    client: Client,
    config: Config,
}

// Delivery result with functional patterns
#[derive(Debug)]
pub struct DeliveryResult {
    pub inbox_url: String,
    pub success: bool,
    pub error: Option<String>,
}

impl DeliveryResult {
    fn success(inbox_url: String) -> Self {
        Self {
            inbox_url,
            success: true,
            error: None,
        }
    }

    fn failure(inbox_url: String, error: String) -> Self {
        Self {
            inbox_url,
            success: false,
            error: Some(error),
        }
    }
}

impl DeliveryService {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    // Core delivery method with functional error handling
    pub async fn deliver_activity(&self, inbox_url: &str, activity: &Value) -> Result<DeliveryResult> {
        info!("Delivering activity to inbox: {}", inbox_url);

        let response = self
            .client
            .post(inbox_url)
            .header("Content-Type", "application/activity+json")
            .header(
                "User-Agent",
                format!("Fediverse-Node/{}", env!("CARGO_PKG_VERSION")),
            )
            .json(activity)
            .send()
            .await
            .map_err(|e| {
                warn!("Failed to send request to {}: {}", inbox_url, e);
                e
            })?;

        match response.status().is_success() {
            true => {
                info!("Successfully delivered activity to {}", inbox_url);
                Ok(DeliveryResult::success(inbox_url.to_string()))
            }
            false => {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                let error_msg = format!("HTTP {}: {}", status, error_text);
                
                warn!("Failed to deliver activity to {}: {}", inbox_url, error_msg);
                Ok(DeliveryResult::failure(inbox_url.to_string(), error_msg))
            }
        }
    }

    // Functional delivery to multiple recipients
    pub async fn deliver_to_inboxes(
        &self,
        activity: &Value,
        inboxes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Vec<DeliveryResult> {
        let futures = inboxes
            .into_iter()
            .map(|inbox| self.deliver_activity(inbox.as_ref(), activity));

        // Execute all deliveries in parallel and collect results
        futures::future::join_all(futures)
            .await
            .into_iter()
            .map(|result| result.unwrap_or_else(|e| {
                DeliveryResult::failure("unknown".to_string(), e.to_string())
            }))
            .collect()
    }

    // Simplified delivery methods using the core function
    pub async fn deliver_to_followers(&self, activity: &Value, followers: Vec<String>) -> Result<Vec<DeliveryResult>> {
        info!("Delivering activity to {} followers", followers.len());
        Ok(self.deliver_to_inboxes(activity, followers).await)
    }

    pub async fn deliver_to_public(&self, activity: &Value, public_inboxes: Vec<String>) -> Result<Vec<DeliveryResult>> {
        info!("Delivering activity to {} public inboxes", public_inboxes.len());
        Ok(self.deliver_to_inboxes(activity, public_inboxes).await)
    }

    // Functional utility for broadcast delivery
    pub async fn broadcast_activity(
        &self,
        activity: &Value,
        followers: Vec<String>,
        public_inboxes: Vec<String>,
    ) -> (Vec<DeliveryResult>, Vec<DeliveryResult>) {
        let (follower_results, public_results) = futures::future::join(
            self.deliver_to_followers(activity, followers),
            self.deliver_to_public(activity, public_inboxes),
        ).await;

        (
            follower_results.unwrap_or_default(),
            public_results.unwrap_or_default(),
        )
    }

    // Analyze delivery results functionally
    pub fn analyze_results(results: &[DeliveryResult]) -> DeliveryAnalysis {
        let total = results.len();
        let successful = results.iter().filter(|r| r.success).count();
        let failed = total - successful;
        let errors: Vec<String> = results
            .iter()
            .filter_map(|r| r.error.as_ref())
            .cloned()
            .collect();

        DeliveryAnalysis {
            total,
            successful,
            failed,
            success_rate: if total > 0 { successful as f64 / total as f64 } else { 0.0 },
            errors,
        }
    }
}

#[derive(Debug)]
pub struct DeliveryAnalysis {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub success_rate: f64,
    pub errors: Vec<String>,
}

impl DeliveryAnalysis {
    pub fn is_success(&self) -> bool {
        self.success_rate > 0.5
    }

    pub fn log_summary(&self) {
        info!(
            "Delivery summary: {}/{} successful ({:.1}%)",
            self.successful,
            self.total,
            self.success_rate * 100.0
        );

        if !self.errors.is_empty() {
            warn!("Delivery errors: {:?}", self.errors);
        }
    }
}

// Functional constructors for common delivery patterns
pub fn create_delivery_service(config: Config) -> DeliveryService {
    DeliveryService::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_config() -> Config {
        Config {
            server_name: "Test Server".to_string(),
            server_url: "https://test.example.com".to_string(),
            port: 8080,
            actor_name: "testuser".to_string(),
            private_key_path: None,
            public_key_path: None,
        }
    }

    fn test_activity() -> Value {
        json!({
            "@context": ["https://www.w3.org/ns/activitystreams"],
            "id": "https://test.example.com/activities/123",
            "type": "Create",
            "actor": "https://test.example.com/users/alice",
            "object": {
                "type": "Note",
                "content": "Hello, world!",
                "attributedTo": "https://test.example.com/users/alice"
            },
            "to": ["https://www.w3.org/ns/activitystreams#Public"]
        })
    }

    #[test]
    fn test_delivery_service_creation() {
        let service = create_delivery_service(test_config());
        assert_eq!(service.config.server_name, "Test Server");
    }

    #[test]
    fn test_delivery_result_constructors() {
        let success = DeliveryResult::success("https://example.com/inbox".to_string());
        let failure = DeliveryResult::failure("https://example.com/inbox".to_string(), "Error".to_string());

        assert!(success.success);
        assert!(success.error.is_none());
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }

    #[test]
    fn test_delivery_analysis() {
        let results = vec![
            DeliveryResult::success("inbox1".to_string()),
            DeliveryResult::success("inbox2".to_string()),
            DeliveryResult::failure("inbox3".to_string(), "Error".to_string()),
        ];

        let analysis = DeliveryService::analyze_results(&results);
        
        assert_eq!(analysis.total, 3);
        assert_eq!(analysis.successful, 2);
        assert_eq!(analysis.failed, 1);
        assert!((analysis.success_rate - 0.666).abs() < 0.01);
        assert!(analysis.is_success());
        assert_eq!(analysis.errors.len(), 1);
    }

    #[test]
    fn test_delivery_analysis_empty() {
        let analysis = DeliveryService::analyze_results(&[]);
        
        assert_eq!(analysis.total, 0);
        assert_eq!(analysis.success_rate, 0.0);
        assert!(!analysis.is_success());
    }

    #[tokio::test]
    async fn test_functional_patterns() {
        let service = create_delivery_service(test_config());
        let activity = test_activity();
        
        // Test empty delivery
        let results = service.deliver_to_inboxes(&activity, Vec::<String>::new()).await;
        assert!(results.is_empty());
        
        let analysis = DeliveryService::analyze_results(&results);
        assert_eq!(analysis.total, 0);
    }
}
