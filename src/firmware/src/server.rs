use crate::container::EmbeddedContainer;
use crate::models::{Activity, Actor, EmbeddedConfig};
use anyhow::Result;
use esp_idf_svc::http::server::{EspHttpServer, Request, Response};
use heapless::{String as HeaplessString, Vec as HeaplessVec};
use log::{error, info, warn};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

/// ActivityPub server wrapper for ESP32
pub struct ActivityPubServer {
    pub container: Arc<Mutex<EmbeddedContainer>>,
}

impl ActivityPubServer {
    pub fn new(container: Arc<Mutex<EmbeddedContainer>>) -> Self {
        Self { container }
    }

    /// Register all ActivityPub HTTP handlers
    pub fn register_handlers(&self, server: &mut EspHttpServer) -> Result<()> {
        let container = self.container.clone();
        
        // WebFinger endpoint
        server.fn_handler("/.well-known/webfinger", esp_idf_svc::http::Method::Get, move |request| {
            Self::handle_webfinger(request, container.clone())
        })?;

        let container = self.container.clone();
        
        // Actor profile endpoint
        server.fn_handler("/users/*", esp_idf_svc::http::Method::Get, move |request| {
            Self::handle_actor_profile(request, container.clone())
        })?;

        let container = self.container.clone();
        
        // Inbox endpoint
        server.fn_handler("/users/*/inbox", esp_idf_svc::http::Method::Post, move |request| {
            Self::handle_inbox(request, container.clone())
        })?;

        let container = self.container.clone();
        
        // Outbox endpoint (GET)
        server.fn_handler("/users/*/outbox", esp_idf_svc::http::Method::Get, move |request| {
            Self::handle_outbox_get(request, container.clone())
        })?;

        let container = self.container.clone();
        
        // Outbox endpoint (POST)
        server.fn_handler("/users/*/outbox", esp_idf_svc::http::Method::Post, move |request| {
            Self::handle_outbox_post(request, container.clone())
        })?;

        let container = self.container.clone();
        
        // Health check endpoint
        server.fn_handler("/health", esp_idf_svc::http::Method::Get, move |request| {
            Self::handle_health_check(request, container.clone())
        })?;

        let container = self.container.clone();
        
        // Node info endpoint
        server.fn_handler("/nodeinfo/2.0", esp_idf_svc::http::Method::Get, move |request| {
            Self::handle_nodeinfo(request, container.clone())
        })?;

        info!("ActivityPub HTTP handlers registered successfully");
        Ok(())
    }

    /// Handle WebFinger requests
    fn handle_webfinger(request: Request<&mut EspHttpServer>, container: Arc<Mutex<EmbeddedContainer>>) -> Result<()> {
        info!("Handling WebFinger request");
        
        // Parse query parameters
        let uri = request.uri();
        let query = uri.split('?').nth(1).unwrap_or("");
        
        // Look for resource parameter
        let resource = query.split('&')
            .find(|param| param.starts_with("resource="))
            .and_then(|param| param.split('=').nth(1));

        let container = container.lock().map_err(|e| anyhow::anyhow!("Failed to lock container: {}", e))?;
        let config = container.config();
        let actor = container.actor();

        // Check if the requested resource matches our actor
        let expected_resource = format!("acct:{}@{}", 
            actor.preferred_username.as_str(), 
            config.server_url.as_str().trim_start_matches("http://").trim_start_matches("https://")
        );

        if let Some(res) = resource {
            if res != expected_resource {
                return Self::send_error_response(request, 404, "Resource not found");
            }
        } else {
            return Self::send_error_response(request, 400, "Missing resource parameter");
        }

        // Create WebFinger response
        let webfinger_response = json!({
            "subject": format!("acct:{}@{}", 
                actor.preferred_username.as_str(),
                config.server_url.as_str().trim_start_matches("http://").trim_start_matches("https://")
            ),
            "links": [
                {
                    "rel": "self",
                    "type": "application/activity+json",
                    "href": actor.id.as_str()
                }
            ]
        });

        Self::send_json_response(request, &webfinger_response)
    }

    /// Handle actor profile requests
    fn handle_actor_profile(request: Request<&mut EspHttpServer>, container: Arc<Mutex<EmbeddedContainer>>) -> Result<()> {
        info!("Handling actor profile request");
        
        let container = container.lock().map_err(|e| anyhow::anyhow!("Failed to lock container: {}", e))?;
        let actor = container.actor();

        // Convert actor to JSON
        let actor_json = serde_json::to_value(actor)
            .map_err(|e| anyhow::anyhow!("Failed to serialize actor: {}", e))?;

        Self::send_json_response(request, &actor_json)
    }

    /// Handle inbox requests (receive activities)
    fn handle_inbox(mut request: Request<&mut EspHttpServer>, container: Arc<Mutex<EmbeddedContainer>>) -> Result<()> {
        info!("Handling inbox request");
        
        // Read request body
        let mut body = Vec::new();
        request.read_to_end(&mut body).map_err(|e| anyhow::anyhow!("Failed to read request body: {}", e))?;

        // Parse JSON activity
        let activity: Activity = serde_json::from_slice(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse activity: {}", e))?;

        info!("Received activity: {} from {}", activity.activity_type.as_str(), activity.actor.as_str());

        // Process the activity
        let mut container = container.lock().map_err(|e| anyhow::anyhow!("Failed to lock container: {}", e))?;
        
        match activity.activity_type.as_str() {
            "Follow" => {
                info!("Processing Follow activity");
                
                // Add follower to list
                let follower_inbox = format!("{}/inbox", activity.actor.as_str());
                if let Err(e) = container.add_follower(&follower_inbox) {
                    warn!("Failed to add follower: {}", e);
                }

                // Send Accept activity (in a real implementation, this would be async)
                let runtime = tokio::runtime::Runtime::new();
                if let Ok(rt) = runtime {
                    if let Err(e) = rt.block_on(container.accept_follow_request(activity)) {
                        error!("Failed to accept follow request: {}", e);
                    }
                }
            }
            "Undo" => {
                info!("Processing Undo activity");
                // Handle unfollow (remove from followers list)
                let follower_inbox = format!("{}/inbox", activity.actor.as_str());
                container.remove_follower(&follower_inbox);
            }
            "Create" => {
                info!("Processing Create activity");
                // Handle incoming posts/notes (could be replies, mentions, etc.)
                // For now, just log it
            }
            _ => {
                info!("Received unknown activity type: {}", activity.activity_type.as_str());
            }
        }

        // Send 200 OK response
        Self::send_success_response(request)
    }

    /// Handle outbox GET requests (list activities)
    fn handle_outbox_get(request: Request<&mut EspHttpServer>, container: Arc<Mutex<EmbeddedContainer>>) -> Result<()> {
        info!("Handling outbox GET request");
        
        let container = container.lock().map_err(|e| anyhow::anyhow!("Failed to lock container: {}", e))?;
        let actor = container.actor();

        // Create empty outbox collection (in a real implementation, this would contain actual activities)
        let outbox_response = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}/outbox", actor.id.as_str()),
            "type": "OrderedCollection",
            "totalItems": 0,
            "orderedItems": []
        });

        Self::send_json_response(request, &outbox_response)
    }

    /// Handle outbox POST requests (create activities)
    fn handle_outbox_post(mut request: Request<&mut EspHttpServer>, container: Arc<Mutex<EmbeddedContainer>>) -> Result<()> {
        info!("Handling outbox POST request");
        
        // Read request body
        let mut body = Vec::new();
        request.read_to_end(&mut body).map_err(|e| anyhow::anyhow!("Failed to read request body: {}", e))?;

        // Parse JSON activity
        let activity_data: Value = serde_json::from_slice(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse activity: {}", e))?;

        info!("Received outbox activity: {:?}", activity_data);

        // In a real implementation, this would create and send the activity
        // For now, just acknowledge receipt
        Self::send_success_response(request)
    }

    /// Handle health check requests
    fn handle_health_check(request: Request<&mut EspHttpServer>, container: Arc<Mutex<EmbeddedContainer>>) -> Result<()> {
        info!("Handling health check request");
        
        let container = container.lock().map_err(|e| anyhow::anyhow!("Failed to lock container: {}", e))?;
        
        let health_response = json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "server": container.config().server_name.as_str(),
            "actor": container.actor().preferred_username.as_str(),
            "followers": container.followers().len(),
            "public_inboxes": container.public_inboxes().len(),
            "free_heap": esp_idf_sys::esp_get_free_heap_size(),
            "uptime_ms": esp_idf_sys::esp_timer_get_time() / 1000
        });

        Self::send_json_response(request, &health_response)
    }

    /// Handle NodeInfo requests
    fn handle_nodeinfo(request: Request<&mut EspHttpServer>, container: Arc<Mutex<EmbeddedContainer>>) -> Result<()> {
        info!("Handling NodeInfo request");
        
        let container = container.lock().map_err(|e| anyhow::anyhow!("Failed to lock container: {}", e))?;
        
        let nodeinfo_response = json!({
            "version": "2.0",
            "software": {
                "name": "esp32-activitypub",
                "version": env!("CARGO_PKG_VERSION")
            },
            "protocols": ["activitypub"],
            "services": {
                "inbound": [],
                "outbound": []
            },
            "openRegistrations": false,
            "usage": {
                "users": {
                    "total": 1
                },
                "localPosts": 0
            },
            "metadata": {
                "nodeName": container.config().server_name.as_str(),
                "nodeDescription": "ESP32-based ActivityPub node",
                "maintainer": {
                    "name": "ESP32 Node",
                    "email": "admin@esp32.local"
                }
            }
        });

        Self::send_json_response(request, &nodeinfo_response)
    }

    /// Send JSON response
    fn send_json_response(mut request: Request<&mut EspHttpServer>, data: &Value) -> Result<()> {
        let json_str = serde_json::to_string(data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize JSON: {}", e))?;

        request.into_ok_response()?
            .write_all(json_str.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write response: {}", e))?;

        Ok(())
    }

    /// Send error response
    fn send_error_response(mut request: Request<&mut EspHttpServer>, status: u16, message: &str) -> Result<()> {
        let error_response = json!({
            "error": message,
            "status": status
        });

        let json_str = serde_json::to_string(&error_response)
            .map_err(|e| anyhow::anyhow!("Failed to serialize error JSON: {}", e))?;

        request.into_response(status, None, &[("Content-Type", "application/json")])?
            .write_all(json_str.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write error response: {}", e))?;

        Ok(())
    }

    /// Send success response
    fn send_success_response(mut request: Request<&mut EspHttpServer>) -> Result<()> {
        let success_response = json!({
            "status": "ok"
        });

        let json_str = serde_json::to_string(&success_response)
            .map_err(|e| anyhow::anyhow!("Failed to serialize success JSON: {}", e))?;

        request.into_ok_response()?
            .write_all(json_str.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write success response: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EmbeddedConfig;
    use crate::container::EmbeddedContainerBuilder;

    fn create_test_container() -> EmbeddedContainer {
        let config = EmbeddedConfig::new(
            "Test Node",
            "https://esp32.local",
            "testuser",
            "TestWiFi",
            "password123",
        ).unwrap();

        EmbeddedContainerBuilder::new()
            .with_config(config)
            .with_mocks()
            .build()
            .unwrap()
    }

    #[test]
    fn test_activitypub_server_creation() {
        let container = Arc::new(Mutex::new(create_test_container()));
        let server = ActivityPubServer::new(container);
        
        // Verify the server was created successfully
        assert!(server.container.lock().is_ok());
    }

    #[test]
    fn test_container_access() {
        let container = Arc::new(Mutex::new(create_test_container()));
        let server = ActivityPubServer::new(container);
        
        // Test that we can access the container
        let container_ref = server.container.lock().unwrap();
        assert_eq!(container_ref.config().server_name.as_str(), "Test Node");
        assert_eq!(container_ref.actor().preferred_username.as_str(), "testuser");
    }
}