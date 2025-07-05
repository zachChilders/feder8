use crate::delivery::{DeliveryService, EmbeddedDeliveryService, MockDeliveryService, DeliveryServiceExt};
use crate::http::{EspHttpClient, HttpClient};
use crate::models::{Actor, EmbeddedConfig};
use anyhow::Result;
use heapless::{String as HeaplessString, Vec as HeaplessVec};
use std::sync::Arc;

/// Dependency injection container for embedded ActivityPub node
#[derive(Clone)]
pub struct EmbeddedContainer {
    config: EmbeddedConfig,
    actor: Actor,
    http_client: Arc<dyn HttpClient>,
    delivery_service: Arc<dyn DeliveryService>,
    followers: HeaplessVec<HeaplessString<128>, 32>,
    public_inboxes: HeaplessVec<HeaplessString<128>, 8>,
}

impl EmbeddedContainer {
    /// Create a new container with default implementations
    pub fn new(config: EmbeddedConfig) -> Result<Self> {
        // Create actor from config
        let public_key = config.public_key_pem.as_ref()
            .map(|key| key.as_str())
            .unwrap_or("-----BEGIN PUBLIC KEY-----\ntemp-key\n-----END PUBLIC KEY-----");

        let actor = Actor::new(
            config.actor_name.as_str(),
            config.actor_name.as_str(),
            config.server_url.as_str(),
            public_key,
        ).map_err(|e| anyhow::anyhow!("Failed to create actor: {}", e))?;

        // Create HTTP client
        let http_client: Arc<dyn HttpClient> = Arc::new(EspHttpClient::new()?);

        // Create delivery service
        let delivery_service = Arc::new(EmbeddedDeliveryService::new(config.clone(), http_client.clone()));

        Ok(Self {
            config,
            actor,
            http_client,
            delivery_service: delivery_service as Arc<dyn DeliveryService>,
            followers: HeaplessVec::new(),
            public_inboxes: HeaplessVec::new(),
        })
    }

    /// Create a new container with custom HTTP client
    pub fn with_http_client(
        config: EmbeddedConfig,
        http_client: Arc<dyn HttpClient>,
    ) -> Result<Self> {
        let public_key = config.public_key_pem.as_ref()
            .map(|key| key.as_str())
            .unwrap_or("-----BEGIN PUBLIC KEY-----\ntemp-key\n-----END PUBLIC KEY-----");

        let actor = Actor::new(
            config.actor_name.as_str(),
            config.actor_name.as_str(),
            config.server_url.as_str(),
            public_key,
        ).map_err(|e| anyhow::anyhow!("Failed to create actor: {}", e))?;

        let delivery_service = Arc::new(EmbeddedDeliveryService::new(config.clone(), http_client.clone()));

        Ok(Self {
            config,
            actor,
            http_client,
            delivery_service: delivery_service as Arc<dyn DeliveryService>,
            followers: HeaplessVec::new(),
            public_inboxes: HeaplessVec::new(),
        })
    }

    /// Create a new container with custom delivery service
    pub fn with_delivery_service(
        config: EmbeddedConfig,
        delivery_service: Arc<dyn DeliveryService>,
    ) -> Result<Self> {
        let public_key = config.public_key_pem.as_ref()
            .map(|key| key.as_str())
            .unwrap_or("-----BEGIN PUBLIC KEY-----\ntemp-key\n-----END PUBLIC KEY-----");

        let actor = Actor::new(
            config.actor_name.as_str(),
            config.actor_name.as_str(),
            config.server_url.as_str(),
            public_key,
        ).map_err(|e| anyhow::anyhow!("Failed to create actor: {}", e))?;

        let http_client: Arc<dyn HttpClient> = Arc::new(EspHttpClient::new()?);

        Ok(Self {
            config,
            actor,
            http_client,
            delivery_service,
            followers: HeaplessVec::new(),
            public_inboxes: HeaplessVec::new(),
        })
    }

    /// Create a container with mock services for testing
    pub fn with_mocks(config: EmbeddedConfig) -> Result<Self> {
        let public_key = config.public_key_pem.as_ref()
            .map(|key| key.as_str())
            .unwrap_or("-----BEGIN PUBLIC KEY-----\ntest-key\n-----END PUBLIC KEY-----");

        let actor = Actor::new(
            config.actor_name.as_str(),
            config.actor_name.as_str(),
            config.server_url.as_str(),
            public_key,
        ).map_err(|e| anyhow::anyhow!("Failed to create actor: {}", e))?;

        let http_client: Arc<dyn HttpClient> = Arc::new(EspHttpClient::new()?);
        let delivery_service: Arc<dyn DeliveryService> = Arc::new(MockDeliveryService::new());

        Ok(Self {
            config,
            actor,
            http_client,
            delivery_service,
            followers: HeaplessVec::new(),
            public_inboxes: HeaplessVec::new(),
        })
    }

    /// Get the configuration
    pub fn config(&self) -> &EmbeddedConfig {
        &self.config
    }

    /// Get the actor
    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    /// Get the HTTP client
    pub fn http_client(&self) -> &Arc<dyn HttpClient> {
        &self.http_client
    }

    /// Get the delivery service
    pub fn delivery_service(&self) -> &Arc<dyn DeliveryService> {
        &self.delivery_service
    }

    /// Get followers list
    pub fn followers(&self) -> &HeaplessVec<HeaplessString<128>, 32> {
        &self.followers
    }

    /// Get public inboxes list
    pub fn public_inboxes(&self) -> &HeaplessVec<HeaplessString<128>, 8> {
        &self.public_inboxes
    }

    /// Add a follower inbox
    pub fn add_follower(&mut self, inbox_url: &str) -> Result<()> {
        if inbox_url.len() > 128 {
            return Err(anyhow::anyhow!("Inbox URL too long for embedded constraints"));
        }

        self.followers.push(HeaplessString::from(inbox_url))
            .map_err(|_| anyhow::anyhow!("Followers list is full"))?;

        Ok(())
    }

    /// Remove a follower inbox
    pub fn remove_follower(&mut self, inbox_url: &str) {
        self.followers.retain(|follower| follower.as_str() != inbox_url);
    }

    /// Add a public inbox
    pub fn add_public_inbox(&mut self, inbox_url: &str) -> Result<()> {
        if inbox_url.len() > 128 {
            return Err(anyhow::anyhow!("Inbox URL too long for embedded constraints"));
        }

        self.public_inboxes.push(HeaplessString::from(inbox_url))
            .map_err(|_| anyhow::anyhow!("Public inboxes list is full"))?;

        Ok(())
    }

    /// Remove a public inbox
    pub fn remove_public_inbox(&mut self, inbox_url: &str) {
        self.public_inboxes.retain(|inbox| inbox.as_str() != inbox_url);
    }

    /// Send a note to followers
    pub async fn send_note(&self, content: &str) -> Result<()> {
        let actor_id = format!("{}/users/{}", self.config.server_url.as_str(), self.config.actor_name.as_str());
        deliver_note_helper(
            &self.delivery_service,
            content,
            &self.followers,
            &actor_id,
            self.config.server_url.as_str(),
        ).await
    }

    /// Send a follow request (simplified version)
    pub async fn send_follow_request(&self, target_actor_id: &str) -> Result<()> {
        // Create a simplified follow activity
        let actor_id = format!("{}/users/{}", self.config.server_url.as_str(), self.config.actor_name.as_str());
        let activity = crate::models::Activity::new_follow(
            &actor_id,
            target_actor_id,
            self.config.server_url.as_str(),
        )?;
        
        // Extract inbox URL from target actor (simplified)
        let inbox_url = format!("{}/inbox", target_actor_id);
        
        // Deliver follow request
        self.delivery_service.deliver_activity(&inbox_url, &activity).await
    }

    /// Accept a follow request (simplified version)
    pub async fn accept_follow_request(&self, original_follow: crate::models::Activity) -> Result<()> {
        // Create actor ID from config
        let actor_id = format!("{}/users/{}", self.config.server_url.as_str(), self.config.actor_name.as_str());

        // Create accept activity
        let accept_activity = crate::models::Activity::new_accept_follow(
            &actor_id,
            original_follow.clone(),
            self.config.server_url.as_str(),
        )?;

        // Extract inbox URL from original follow actor
        let inbox_url = format!("{}/inbox", original_follow.actor.as_str());

        // Deliver accept response
        self.delivery_service.deliver_activity(&inbox_url, &accept_activity).await
    }
}

/// Builder pattern for creating containers with different configurations
pub struct EmbeddedContainerBuilder {
    config: Option<EmbeddedConfig>,
    http_client: Option<Arc<dyn HttpClient>>,
    delivery_service: Option<Arc<dyn DeliveryService>>,
    use_mocks: bool,
}

impl EmbeddedContainerBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            http_client: None,
            delivery_service: None,
            use_mocks: false,
        }
    }

    pub fn with_config(mut self, config: EmbeddedConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_http_client(mut self, http_client: Arc<dyn HttpClient>) -> Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn with_delivery_service(mut self, delivery_service: Arc<dyn DeliveryService>) -> Self {
        self.delivery_service = Some(delivery_service);
        self
    }

    pub fn with_mocks(mut self) -> Self {
        self.use_mocks = true;
        self
    }

    pub fn build(self) -> Result<EmbeddedContainer> {
        let config = self.config.ok_or_else(|| anyhow::anyhow!("Config is required"))?;

        if self.use_mocks {
            return EmbeddedContainer::with_mocks(config);
        }

        match (self.http_client, self.delivery_service) {
            (Some(http_client), Some(delivery_service)) => {
                // Custom HTTP client and delivery service
                let mut container = EmbeddedContainer::with_http_client(config, http_client)?;
                container.delivery_service = delivery_service;
                Ok(container)
            }
            (Some(http_client), None) => {
                // Custom HTTP client, default delivery service
                EmbeddedContainer::with_http_client(config, http_client)
            }
            (None, Some(delivery_service)) => {
                // Default HTTP client, custom delivery service
                EmbeddedContainer::with_delivery_service(config, delivery_service)
            }
            (None, None) => {
                // Default implementations
                EmbeddedContainer::new(config)
            }
        }
    }
}

impl Default for EmbeddedContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a note activity and deliver it
async fn deliver_note_helper(
    delivery_service: &Arc<dyn DeliveryService>,
    note_content: &str,
    followers: &HeaplessVec<HeaplessString<128>, 32>,
    actor_id: &str,
    server_url: &str,
) -> Result<()> {
    // Create a simple note activity
    let activity = crate::models::Activity::new_create_note(
        actor_id,
        note_content,
        server_url,
    )?;

    delivery_service.deliver_to_followers(&activity, followers).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EmbeddedConfig;

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
    fn test_container_creation() {
        let config = create_test_config();
        let container = EmbeddedContainer::new(config.clone()).unwrap();

        assert_eq!(container.config().server_name.as_str(), "Test Node");
        assert_eq!(container.config().server_url.as_str(), "https://esp32.local");
        assert_eq!(container.config().actor_name.as_str(), "testuser");
        assert_eq!(container.actor().name.as_str(), "testuser");
        assert_eq!(container.followers().len(), 0);
        assert_eq!(container.public_inboxes().len(), 0);
    }

    #[test]
    fn test_container_with_mocks() {
        let config = create_test_config();
        let container = EmbeddedContainer::with_mocks(config.clone()).unwrap();

        assert_eq!(container.config().server_name.as_str(), "Test Node");
        assert_eq!(container.config().server_url.as_str(), "https://esp32.local");
        assert_eq!(container.config().actor_name.as_str(), "testuser");
    }

    #[test]
    fn test_container_builder() {
        let config = create_test_config();
        let container = EmbeddedContainerBuilder::new()
            .with_config(config.clone())
            .with_mocks()
            .build()
            .unwrap();

        assert_eq!(container.config().server_name.as_str(), "Test Node");
        assert_eq!(container.config().server_url.as_str(), "https://esp32.local");
        assert_eq!(container.config().actor_name.as_str(), "testuser");
    }

    #[test]
    fn test_container_builder_missing_config() {
        let result = EmbeddedContainerBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_add_remove_follower() {
        let config = create_test_config();
        let mut container = EmbeddedContainer::new(config).unwrap();

        assert_eq!(container.followers().len(), 0);

        // Add follower
        container.add_follower("https://mastodon.social/users/alice/inbox").unwrap();
        assert_eq!(container.followers().len(), 1);

        // Remove follower
        container.remove_follower("https://mastodon.social/users/alice/inbox");
        assert_eq!(container.followers().len(), 0);
    }

    #[test]
    fn test_add_remove_public_inbox() {
        let config = create_test_config();
        let mut container = EmbeddedContainer::new(config).unwrap();

        assert_eq!(container.public_inboxes().len(), 0);

        // Add public inbox
        container.add_public_inbox("https://relay.fediverse.org/inbox").unwrap();
        assert_eq!(container.public_inboxes().len(), 1);

        // Remove public inbox
        container.remove_public_inbox("https://relay.fediverse.org/inbox");
        assert_eq!(container.public_inboxes().len(), 0);
    }

    #[test]
    fn test_follower_limit() {
        let config = create_test_config();
        let mut container = EmbeddedContainer::new(config).unwrap();

        // Try to add more than 32 followers
        for i in 0..33 {
            let inbox = format!("https://example.com/users/user{}/inbox", i);
            let result = container.add_follower(&inbox);
            
            if i < 32 {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_public_inbox_limit() {
        let config = create_test_config();
        let mut container = EmbeddedContainer::new(config).unwrap();

        // Try to add more than 8 public inboxes
        for i in 0..9 {
            let inbox = format!("https://relay{}.example.com/inbox", i);
            let result = container.add_public_inbox(&inbox);
            
            if i < 8 {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_container_clone() {
        let config = create_test_config();
        let container = EmbeddedContainer::new(config).unwrap();
        let cloned = container.clone();

        assert_eq!(container.config().server_name.as_str(), cloned.config().server_name.as_str());
        assert_eq!(container.config().server_url.as_str(), cloned.config().server_url.as_str());
        assert_eq!(container.config().actor_name.as_str(), cloned.config().actor_name.as_str());
    }
}