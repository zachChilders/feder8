use crate::http::{HttpClient, StatusCode};
use crate::models::{Activity, EmbeddedConfig};
use anyhow::Result;
use heapless::{String as HeaplessString, Vec as HeaplessVec};
use log::{error, info, warn};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Embedded delivery service for ActivityPub messages
pub struct EmbeddedDeliveryService {
    client: Arc<dyn HttpClient>,
    config: EmbeddedConfig,
}

impl EmbeddedDeliveryService {
    /// Create a new embedded delivery service
    pub fn new(config: EmbeddedConfig, client: Arc<dyn HttpClient>) -> Self {
        Self { client, config }
    }

    /// Deliver an activity to a specific inbox
    pub async fn deliver_activity(&self, inbox_url: &str, activity: &Activity) -> Result<()> {
        info!("Delivering activity to inbox: {}", inbox_url);

        // Convert activity to JSON
        let activity_json = serde_json::to_value(activity)?;
        
        // Prepare headers
        let mut headers = HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            "application/activity+json".to_string(),
        );
        headers.insert(
            "User-Agent".to_string(),
            format!("ESP32-ActivityPub-Node/{}", env!("CARGO_PKG_VERSION")),
        );

        // Send request
        let response = self
            .client
            .post_with_headers(inbox_url, headers, &activity_json)
            .await?;

        if response.status().is_success() {
            info!("Successfully delivered activity to {}", inbox_url);
        } else {
            warn!(
                "Failed to deliver activity to {}: {}",
                inbox_url,
                response.status().0
            );
            if let Ok(error_text) = response.text() {
                error!("Error response: {}", error_text);
            }
        }

        Ok(())
    }

    /// Deliver activity to multiple followers (limited by embedded constraints)
    pub async fn deliver_to_followers(
        &self,
        activity: &Activity,
        followers: &HeaplessVec<HeaplessString<128>, 32>, // Limited to 32 followers
    ) -> Result<()> {
        info!("Delivering activity to {} followers", followers.len());

        for follower_inbox in followers {
            if let Err(e) = self.deliver_activity(follower_inbox.as_str(), activity).await {
                warn!("Failed to deliver to {}: {}", follower_inbox.as_str(), e);
                // Continue with other deliveries even if one fails
            }
        }

        Ok(())
    }

    /// Deliver activity to public (relay) servers
    pub async fn deliver_to_public(
        &self,
        activity: &Activity,
        public_inboxes: &HeaplessVec<HeaplessString<128>, 8>, // Limited to 8 public inboxes
    ) -> Result<()> {
        info!(
            "Delivering activity to {} public inboxes",
            public_inboxes.len()
        );

        for inbox in public_inboxes {
            if let Err(e) = self.deliver_activity(inbox.as_str(), activity).await {
                warn!("Failed to deliver to public inbox {}: {}", inbox.as_str(), e);
                // Continue with other deliveries even if one fails
            }
        }

        Ok(())
    }

    /// Deliver a simple note activity
    pub async fn deliver_note(
        &self,
        note_content: &str,
        followers: &HeaplessVec<HeaplessString<128>, 32>,
    ) -> Result<()> {
        // Create actor ID from config
        let actor_id = format!("{}/users/{}", self.config.server_url.as_str(), self.config.actor_name.as_str());

        // Create note activity
        let activity = Activity::new_create_note(
            &actor_id,
            note_content,
            self.config.server_url.as_str(),
        )?;

        // Deliver to followers
        self.deliver_to_followers(&activity, followers).await?;

        Ok(())
    }

    /// Send a follow request to another actor
    pub async fn send_follow_request(&self, target_actor_id: &str) -> Result<()> {
        info!("Sending follow request to {}", target_actor_id);

        // Create actor ID from config
        let actor_id = format!("{}/users/{}", self.config.server_url.as_str(), self.config.actor_name.as_str());

        // Create follow activity
        let activity = Activity::new_follow(
            &actor_id,
            target_actor_id,
            self.config.server_url.as_str(),
        )?;

        // Extract inbox URL from target actor (simplified - in real implementation would need WebFinger)
        let inbox_url = format!("{}/inbox", target_actor_id);

        // Deliver follow request
        self.deliver_activity(&inbox_url, &activity).await?;

        Ok(())
    }

    /// Accept a follow request
    pub async fn accept_follow_request(&self, original_follow: Activity) -> Result<()> {
        info!("Accepting follow request from {}", original_follow.actor.as_str());

        // Create actor ID from config
        let actor_id = format!("{}/users/{}", self.config.server_url.as_str(), self.config.actor_name.as_str());

        // Create accept activity
        let accept_activity = Activity::new_accept_follow(
            &actor_id,
            original_follow.clone(),
            self.config.server_url.as_str(),
        )?;

        // Extract inbox URL from original follow actor
        let inbox_url = format!("{}/inbox", original_follow.actor.as_str());

        // Deliver accept response
        self.deliver_activity(&inbox_url, &accept_activity).await?;

        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &EmbeddedConfig {
        &self.config
    }
}

/// Trait for swappable delivery implementations
pub trait DeliveryService: Send + Sync {
    /// Deliver an activity to a specific inbox
    async fn deliver_activity(&self, inbox_url: &str, activity: &Activity) -> Result<()>;

    /// Deliver activity to multiple followers
    async fn deliver_to_followers(
        &self,
        activity: &Activity,
        followers: &HeaplessVec<HeaplessString<128>, 32>,
    ) -> Result<()>;

    /// Deliver activity to public inboxes
    async fn deliver_to_public(
        &self,
        activity: &Activity,
        public_inboxes: &HeaplessVec<HeaplessString<128>, 8>,
    ) -> Result<()>;
}

/// Implement the trait for our embedded delivery service
impl DeliveryService for EmbeddedDeliveryService {
    async fn deliver_activity(&self, inbox_url: &str, activity: &Activity) -> Result<()> {
        self.deliver_activity(inbox_url, activity).await
    }

    async fn deliver_to_followers(
        &self,
        activity: &Activity,
        followers: &HeaplessVec<HeaplessString<128>, 32>,
    ) -> Result<()> {
        self.deliver_to_followers(activity, followers).await
    }

    async fn deliver_to_public(
        &self,
        activity: &Activity,
        public_inboxes: &HeaplessVec<HeaplessString<128>, 8>,
    ) -> Result<()> {
        self.deliver_to_public(activity, public_inboxes).await
    }
}

/// Extension trait to add downcast functionality to DeliveryService
pub trait DeliveryServiceExt {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl DeliveryServiceExt for EmbeddedDeliveryService {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DeliveryServiceExt for MockDeliveryService {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Mock delivery service for testing
pub struct MockDeliveryService {
    pub delivery_count: std::sync::atomic::AtomicUsize,
}

impl MockDeliveryService {
    pub fn new() -> Self {
        Self {
            delivery_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn get_delivery_count(&self) -> usize {
        self.delivery_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl DeliveryService for MockDeliveryService {
    async fn deliver_activity(&self, inbox_url: &str, _activity: &Activity) -> Result<()> {
        info!("Mock delivery to {}", inbox_url);
        self.delivery_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn deliver_to_followers(
        &self,
        _activity: &Activity,
        followers: &HeaplessVec<HeaplessString<128>, 32>,
    ) -> Result<()> {
        info!("Mock delivery to {} followers", followers.len());
        self.delivery_count.fetch_add(followers.len(), std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn deliver_to_public(
        &self,
        _activity: &Activity,
        public_inboxes: &HeaplessVec<HeaplessString<128>, 8>,
    ) -> Result<()> {
        info!("Mock delivery to {} public inboxes", public_inboxes.len());
        self.delivery_count.fetch_add(public_inboxes.len(), std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{HttpRequest, HttpResponse};
    use crate::models::{Activity, EmbeddedConfig};
    use async_trait::async_trait;
    use heapless::{String as HeaplessString, Vec as HeaplessVec};
    use std::sync::Arc;

    // Mock HTTP client for testing
    struct MockHttpClient {
        should_succeed: bool,
    }

    impl MockHttpClient {
        fn new(should_succeed: bool) -> Self {
            Self { should_succeed }
        }
    }

    #[async_trait]
    impl HttpClient for MockHttpClient {
        async fn send(&self, _request: HttpRequest) -> Result<HttpResponse> {
            if self.should_succeed {
                Ok(HttpResponse {
                    status: StatusCode(200),
                    headers: HashMap::new(),
                    body: HeaplessVec::new(),
                })
            } else {
                Ok(HttpResponse {
                    status: StatusCode(500),
                    headers: HashMap::new(),
                    body: HeaplessVec::new(),
                })
            }
        }
    }

    fn create_test_config() -> EmbeddedConfig {
        EmbeddedConfig::new(
            "Test Node",
            "https://esp32.local",
            "testuser",
            "TestWiFi",
            "password123",
        ).unwrap()
    }

    #[test]
    fn test_delivery_service_creation() {
        let config = create_test_config();
        let client = Arc::new(MockHttpClient::new(true));
        let service = EmbeddedDeliveryService::new(config.clone(), client);

        assert_eq!(service.config().server_name.as_str(), "Test Node");
        assert_eq!(service.config().server_url.as_str(), "https://esp32.local");
        assert_eq!(service.config().actor_name.as_str(), "testuser");
    }

    #[test]
    fn test_mock_delivery_service() {
        let mock_service = MockDeliveryService::new();
        assert_eq!(mock_service.get_delivery_count(), 0);
    }

    #[tokio::test]
    async fn test_deliver_to_followers_empty() {
        let config = create_test_config();
        let client = Arc::new(MockHttpClient::new(true));
        let service = EmbeddedDeliveryService::new(config, client);

        let activity = Activity::new_create_note(
            "https://esp32.local/users/testuser",
            "Hello, world!",
            "https://esp32.local",
        ).unwrap();

        let followers = HeaplessVec::new();
        let result = service.deliver_to_followers(&activity, &followers).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deliver_to_public_empty() {
        let config = create_test_config();
        let client = Arc::new(MockHttpClient::new(true));
        let service = EmbeddedDeliveryService::new(config, client);

        let activity = Activity::new_create_note(
            "https://esp32.local/users/testuser",
            "Hello, world!",
            "https://esp32.local",
        ).unwrap();

        let public_inboxes = HeaplessVec::new();
        let result = service.deliver_to_public(&activity, &public_inboxes).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deliver_note() {
        let config = create_test_config();
        let client = Arc::new(MockHttpClient::new(true));
        let service = EmbeddedDeliveryService::new(config, client);

        let followers = HeaplessVec::new();
        let result = service.deliver_note("Hello from ESP32!", &followers).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_delivery_service_trait() {
        let mock_service = MockDeliveryService::new();
        
        let activity = Activity::new_create_note(
            "https://esp32.local/users/testuser",
            "Hello, world!",
            "https://esp32.local",
        ).unwrap();

        let followers = HeaplessVec::new();
        let result = mock_service.deliver_to_followers(&activity, &followers).await;
        assert!(result.is_ok());
    }
}