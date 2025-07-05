use crate::config::Config;
use crate::http::HttpClient;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

#[allow(dead_code)]
pub struct DeliveryService {
    client: Arc<dyn HttpClient>,
    config: Config,
}

#[allow(dead_code)]
impl DeliveryService {
    pub fn new(config: Config, client: Arc<dyn HttpClient>) -> Self {
        Self {
            client,
            config,
        }
    }

    pub async fn deliver_activity(&self, inbox_url: &str, activity: Value) -> Result<()> {
        info!("Delivering activity to inbox: {}", inbox_url);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/activity+json".to_string());
        headers.insert(
            "User-Agent".to_string(),
            format!("Fediverse-Node/{}", env!("CARGO_PKG_VERSION")),
        );

        let response = self
            .client
            .post_with_headers(inbox_url, headers, &activity)
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

    pub async fn deliver_to_followers(
        &self,
        activity: Value,
        followers: Vec<String>,
    ) -> Result<()> {
        info!("Delivering activity to {} followers", followers.len());

        for follower_inbox in followers {
            if let Err(e) = self
                .deliver_activity(&follower_inbox, activity.clone())
                .await
            {
                warn!("Failed to deliver to {}: {}", follower_inbox, e);
            }
        }

        Ok(())
    }

    pub async fn deliver_to_public(
        &self,
        activity: Value,
        public_inboxes: Vec<String>,
    ) -> Result<()> {
        info!(
            "Delivering activity to {} public inboxes",
            public_inboxes.len()
        );

        for inbox in public_inboxes {
            if let Err(e) = self.deliver_activity(&inbox, activity.clone()).await {
                warn!("Failed to deliver to public inbox {}: {}", inbox, e);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{HttpClient, HttpRequest, HttpResponse, StatusCode};
    use serde_json::json;
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

    #[async_trait::async_trait]
    impl HttpClient for MockHttpClient {
        async fn send(&self, _request: HttpRequest) -> Result<HttpResponse> {
            if self.should_succeed {
                Ok(HttpResponse {
                    status: StatusCode(200),
                    headers: std::collections::HashMap::new(),
                    body: b"OK".to_vec(),
                })
            } else {
                Ok(HttpResponse {
                    status: StatusCode(500),
                    headers: std::collections::HashMap::new(),
                    body: b"Internal Server Error".to_vec(),
                })
            }
        }
    }

    fn create_test_config() -> Config {
        Config {
            server_name: "Test Server".to_string(),
            server_url: "https://test.example.com".to_string(),
            port: 8080,
            actor_name: "testuser".to_string(),
            private_key_path: None,
            public_key_path: None,
        }
    }

    fn create_test_activity() -> Value {
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
            "to": ["https://www.w3.org/ns/activitystreams#Public"],
            "cc": ["https://test.example.com/users/alice/followers"]
        })
    }

    #[test]
    fn test_delivery_service_new() {
        let config = create_test_config();
        let client = Arc::new(MockHttpClient::new(true));
        let service = DeliveryService::new(config.clone(), client);

        assert_eq!(service.config.server_name, config.server_name);
        assert_eq!(service.config.server_url, config.server_url);
        assert_eq!(service.config.port, config.port);
        assert_eq!(service.config.actor_name, config.actor_name);
    }

    #[test]
    fn test_delivery_service_with_different_configs() {
        let config1 = Config {
            server_name: "Server 1".to_string(),
            server_url: "https://server1.com".to_string(),
            port: 8080,
            actor_name: "alice".to_string(),
            private_key_path: None,
            public_key_path: None,
        };

        let config2 = Config {
            server_name: "Server 2".to_string(),
            server_url: "https://server2.com".to_string(),
            port: 9090,
            actor_name: "bob".to_string(),
            private_key_path: Some("/path/to/key".to_string()),
            public_key_path: Some("/path/to/pub".to_string()),
        };

        let client1 = Arc::new(MockHttpClient::new(true));
        let client2 = Arc::new(MockHttpClient::new(true));
        let service1 = DeliveryService::new(config1.clone(), client1);
        let service2 = DeliveryService::new(config2.clone(), client2);

        assert_eq!(service1.config.server_name, "Server 1");
        assert_eq!(service1.config.actor_name, "alice");
        assert_eq!(service2.config.server_name, "Server 2");
        assert_eq!(service2.config.actor_name, "bob");
        assert_eq!(service2.config.port, 9090);
    }

    // Note: The following tests would require actual HTTP mocking to test properly.
    // In a real-world scenario, you'd use a library like mockito or similar to mock HTTP responses.
    // For now, we're testing the service creation and structure.

    #[tokio::test]
    async fn test_deliver_to_followers_empty_list() {
        let config = create_test_config();
        let client = Arc::new(MockHttpClient::new(true));
        let service = DeliveryService::new(config, client);
        let activity = create_test_activity();
        let followers = vec![];

        // This should complete without error even with empty followers list
        let result = service.deliver_to_followers(activity, followers).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deliver_to_public_empty_list() {
        let config = create_test_config();
        let client = Arc::new(MockHttpClient::new(true));
        let service = DeliveryService::new(config, client);
        let activity = create_test_activity();
        let public_inboxes = vec![];

        // This should complete without error even with empty inboxes list
        let result = service.deliver_to_public(activity, public_inboxes).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_activity_structure() {
        let activity = create_test_activity();

        assert_eq!(activity["type"], "Create");
        assert_eq!(activity["actor"], "https://test.example.com/users/alice");
        assert_eq!(activity["object"]["type"], "Note");
        assert_eq!(activity["object"]["content"], "Hello, world!");
        assert!(activity["to"]
            .as_array()
            .unwrap()
            .contains(&json!("https://www.w3.org/ns/activitystreams#Public")));
    }

    #[test]
    fn test_complex_activity_creation() {
        let complex_activity = json!({
            "@context": [
                "https://www.w3.org/ns/activitystreams",
                "https://w3id.org/security/v1"
            ],
            "id": "https://mastodon.social/activities/123456",
            "type": "Follow",
            "actor": "https://mastodon.social/users/alice",
            "object": "https://pleroma.instance/users/bob",
            "to": ["https://pleroma.instance/users/bob"],
            "cc": [],
            "published": "2024-01-01T12:00:00Z",
            "signature": {
                "type": "RsaSignature2017",
                "creator": "https://mastodon.social/users/alice#main-key",
                "created": "2024-01-01T12:00:00Z",
                "signatureValue": "signature..."
            }
        });

        assert_eq!(complex_activity["type"], "Follow");
        assert_eq!(
            complex_activity["actor"],
            "https://mastodon.social/users/alice"
        );
        assert_eq!(
            complex_activity["object"],
            "https://pleroma.instance/users/bob"
        );
        assert!(complex_activity["signature"]["type"] == "RsaSignature2017");
    }

    #[test]
    fn test_multiple_followers_structure() {
        let followers = [
            "https://mastodon.social/users/alice/inbox".to_string(),
            "https://pleroma.instance/users/bob/inbox".to_string(),
            "https://misskey.io/users/charlie/inbox".to_string(),
        ];

        assert_eq!(followers.len(), 3);
        assert!(followers.contains(&"https://mastodon.social/users/alice/inbox".to_string()));
        assert!(followers.contains(&"https://pleroma.instance/users/bob/inbox".to_string()));
        assert!(followers.contains(&"https://misskey.io/users/charlie/inbox".to_string()));
    }

    #[test]
    fn test_public_inboxes_structure() {
        let public_inboxes = [
            "https://relay.fediverse.org/inbox".to_string(),
            "https://relay.activitypub.org/inbox".to_string(),
        ];

        assert_eq!(public_inboxes.len(), 2);
        assert!(public_inboxes.contains(&"https://relay.fediverse.org/inbox".to_string()));
        assert!(public_inboxes.contains(&"https://relay.activitypub.org/inbox".to_string()));
    }

    #[test]
    fn test_activity_cloning() {
        let activity = create_test_activity();
        let cloned_activity = activity.clone();

        assert_eq!(activity, cloned_activity);
        assert_eq!(activity["id"], cloned_activity["id"]);
        assert_eq!(activity["type"], cloned_activity["type"]);
        assert_eq!(activity["actor"], cloned_activity["actor"]);
    }

    #[test]
    fn test_delivery_service_config_persistence() {
        let original_config = create_test_config();
        let client = Arc::new(MockHttpClient::new(true));
        let service = DeliveryService::new(original_config.clone(), client);

        // Verify that the service maintains a copy of the config
        assert_eq!(service.config.server_name, original_config.server_name);
        assert_eq!(service.config.server_url, original_config.server_url);
        assert_eq!(service.config.port, original_config.port);
        assert_eq!(service.config.actor_name, original_config.actor_name);
        assert_eq!(
            service.config.private_key_path,
            original_config.private_key_path
        );
        assert_eq!(
            service.config.public_key_path,
            original_config.public_key_path
        );
    }

    // Test different activity types
    #[test]
    fn test_different_activity_types() {
        let follow_activity = json!({
            "type": "Follow",
            "actor": "https://example.com/users/alice",
            "object": "https://example.com/users/bob"
        });

        let accept_activity = json!({
            "type": "Accept",
            "actor": "https://example.com/users/bob",
            "object": {
                "type": "Follow",
                "actor": "https://example.com/users/alice",
                "object": "https://example.com/users/bob"
            }
        });

        let undo_activity = json!({
            "type": "Undo",
            "actor": "https://example.com/users/alice",
            "object": {
                "type": "Follow",
                "actor": "https://example.com/users/alice",
                "object": "https://example.com/users/bob"
            }
        });

        assert_eq!(follow_activity["type"], "Follow");
        assert_eq!(accept_activity["type"], "Accept");
        assert_eq!(undo_activity["type"], "Undo");
    }
}
