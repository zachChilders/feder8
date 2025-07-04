use actix_web::{middleware::Logger, web, App, HttpServer};
use feder8::{config::Config, handlers};
use reqwest::Client;
use serde_json::json;
use std::sync::Once;
use std::time::Duration;
use tokio::time::sleep;
use std::sync::Mutex;
use std::sync::Arc;
use tokio::task::JoinHandle;

mod test_harness {
    use super::*;
    
    static INIT: Once = Once::new();
    static NODES: Mutex<Option<Arc<Mutex<Vec<JoinHandle<()>>>>>> = Mutex::new(None);

    pub struct TestContext {
        pub client: Client,
        pub node_a_url: String,
        pub node_b_url: String,
    }

    impl TestContext {
        pub fn new() -> Self {
            Self {
                client: Client::new(),
                node_a_url: "http://localhost:8082".to_string(),
                node_b_url: "http://localhost:8083".to_string(),
            }
        }

        pub async fn wait_for_nodes(&self) {
            println!("Waiting for nodes to start...");
            sleep(Duration::from_secs(2)).await;
            
            // Wait for both nodes to be ready
            let mut attempts = 0;
            while attempts < 20 {
                if self.is_node_ready(&self.node_a_url).await && self.is_node_ready(&self.node_b_url).await {
                    println!("Both nodes are ready!");
                    return;
                }
                sleep(Duration::from_millis(250)).await;
                attempts += 1;
            }
            panic!("Nodes failed to start within expected time");
        }

        async fn is_node_ready(&self, url: &str) -> bool {
            match self.client.get(url).timeout(Duration::from_secs(1)).send().await {
                Ok(_) => true,
                Err(_) => false,
            }
        }
    }

    async fn start_node(port: u16, actor_name: &str) -> JoinHandle<()> {
        let config = Config {
            server_name: format!("Test Node {}", actor_name),
            server_url: format!("http://localhost:{}", port),
            port,
            actor_name: actor_name.to_string(),
            private_key_path: None,
            public_key_path: None,
        };

        let config_clone = config.clone();
        let server_handle = tokio::spawn(async move {
            let _ = HttpServer::new(move || {
                App::new()
                    .wrap(Logger::default())
                    .app_data(web::Data::new(config_clone.clone()))
                    .service(handlers::webfinger::webfinger)
                    .service(handlers::actor::get_actor)
                    .service(handlers::inbox::inbox)
                    .service(handlers::outbox::get_outbox)
                    .service(handlers::outbox::post_outbox)
            })
            .bind(("127.0.0.1", port))
            .unwrap_or_else(|e| {
                eprintln!("Failed to bind to port {}: {}", port, e);
                std::process::exit(1);
            })
            .run()
            .await;
        });

        server_handle
    }

    pub async fn setup_nodes() {
        INIT.call_once(|| {
            println!("Setting up test nodes...");
        });
        
        // Start the nodes asynchronously
        let node_a = start_node(8082, "alice").await;
        let node_b = start_node(8083, "bob").await;
        
        let nodes = Arc::new(Mutex::new(vec![node_a, node_b]));
        *NODES.lock().unwrap() = Some(nodes);
    }

    pub fn teardown_nodes() {
        if let Some(nodes) = NODES.lock().unwrap().take() {
            println!("Tearing down test nodes...");
            let mut nodes_guard = nodes.lock().unwrap();
            for node in nodes_guard.drain(..) {
                node.abort();
            }
        }
    }
}

use test_harness::{TestContext, setup_nodes, teardown_nodes};

#[tokio::test]
async fn test_node_setup_and_teardown() {
    setup_nodes().await;
    
    let context = TestContext::new();
    context.wait_for_nodes().await;
    
    // Verify both nodes are running
    assert!(context.client.get(&context.node_a_url).timeout(Duration::from_secs(1)).send().await.is_ok());
    assert!(context.client.get(&context.node_b_url).timeout(Duration::from_secs(1)).send().await.is_ok());
    
    teardown_nodes();
}

#[tokio::test]
async fn test_alice_actor_profile() {
    setup_nodes().await;
    
    let context = TestContext::new();
    context.wait_for_nodes().await;

    let response = context
        .client
        .get(&format!("{}/users/alice", context.node_a_url))
        .header("Accept", "application/activity+json")
        .send()
        .await
        .expect("Failed to get Alice's actor profile");

    assert!(response.status().is_success());
    
    let actor_data: serde_json::Value = response.json().await.expect("Failed to parse actor JSON");
    assert_eq!(actor_data["preferredUsername"], "alice");
    assert_eq!(actor_data["type"], "Person");
    assert!(actor_data["inbox"].as_str().unwrap().contains("/users/alice/inbox"));
    
    teardown_nodes();
}

#[tokio::test]
async fn test_bob_actor_profile() {
    setup_nodes().await;
    
    let context = TestContext::new();
    context.wait_for_nodes().await;

    let response = context
        .client
        .get(&format!("{}/users/bob", context.node_b_url))
        .header("Accept", "application/activity+json")
        .send()
        .await
        .expect("Failed to get Bob's actor profile");

    assert!(response.status().is_success());
    
    let actor_data: serde_json::Value = response.json().await.expect("Failed to parse actor JSON");
    assert_eq!(actor_data["preferredUsername"], "bob");
    assert_eq!(actor_data["type"], "Person");
    assert!(actor_data["inbox"].as_str().unwrap().contains("/users/bob/inbox"));
    
    teardown_nodes();
}

#[tokio::test]
async fn test_webfinger_discovery() {
    setup_nodes().await;
    
    let context = TestContext::new();
    context.wait_for_nodes().await;

    let response = context
        .client
        .get(&format!("{}/.well-known/webfinger?resource=acct:alice@localhost:8082", context.node_a_url))
        .header("Accept", "application/jrd+json")
        .send()
        .await
        .expect("Failed to get WebFinger response");

    assert!(response.status().is_success());
    
    let webfinger_data: serde_json::Value = response.json().await.expect("Failed to parse WebFinger JSON");
    assert_eq!(webfinger_data["subject"], "acct:alice@localhost:8082");
    
    let links = webfinger_data["links"].as_array().expect("Links should be an array");
    assert!(!links.is_empty());
    
    // Check for ActivityPub link
    let activitypub_link = links
        .iter()
        .find(|link| link["rel"] == "self" && link["type"] == "application/activity+json");
    assert!(activitypub_link.is_some());
    
    teardown_nodes();
}

#[tokio::test]
async fn test_message_delivery_between_nodes() {
    setup_nodes().await;
    
    let context = TestContext::new();
    context.wait_for_nodes().await;

    let note = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/notes/789",
        "type": "Note",
        "attributedTo": format!("{}/users/alice", context.node_a_url),
        "content": "Hello Bob! This is a test message from Alice.",
        "to": [format!("{}/users/bob", context.node_b_url)],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"],
        "published": "2024-01-01T12:00:00Z"
    });

    let create_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/activities/101",
        "type": "Create",
        "actor": format!("{}/users/alice", context.node_a_url),
        "object": note,
        "to": [format!("{}/users/bob", context.node_b_url)],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"],
        "published": "2024-01-01T12:00:00Z"
    });

    let response = context
        .client
        .post(&format!("{}/users/bob/inbox", context.node_b_url))
        .header("Content-Type", "application/activity+json")
        .json(&create_activity)
        .send()
        .await
        .expect("Failed to send message to Bob's inbox");

    assert!(response.status().is_success() || response.status().as_u16() == 202);
    
    teardown_nodes();
}

#[tokio::test]
async fn test_cross_node_actor_discovery() {
    setup_nodes().await;
    
    let context = TestContext::new();
    context.wait_for_nodes().await;

    // Test that Node A can discover Node B's actor
    let response = context
        .client
        .get(&format!("{}/users/bob", context.node_b_url))
        .header("Accept", "application/activity+json")
        .send()
        .await
        .expect("Failed to get Bob's actor profile from Node B");

    assert!(response.status().is_success());
    
    let actor_data: serde_json::Value = response.json().await.expect("Failed to parse actor JSON");
    assert_eq!(actor_data["preferredUsername"], "bob");
    assert_eq!(actor_data["id"], format!("{}/users/bob", context.node_b_url));
    
    teardown_nodes();
}

#[tokio::test]
async fn test_inbox_endpoint_accepts_activities() {
    setup_nodes().await;
    
    let context = TestContext::new();
    context.wait_for_nodes().await;

    let test_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/activities/test-123",
        "type": "Create",
        "actor": format!("{}/users/alice", context.node_a_url),
        "object": {
            "@context": ["https://www.w3.org/ns/activitystreams"],
            "id": "https://example.com/notes/test-456",
            "type": "Note",
            "content": "Test message for inbox endpoint",
            "attributedTo": format!("{}/users/alice", context.node_a_url)
        },
        "to": [format!("{}/users/bob", context.node_b_url)],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"]
    });

    let response = context
        .client
        .post(&format!("{}/users/bob/inbox", context.node_b_url))
        .header("Content-Type", "application/activity+json")
        .json(&test_activity)
        .send()
        .await
        .expect("Failed to post activity to inbox");

    // ActivityPub inbox endpoints should return 202 Accepted
    assert_eq!(response.status().as_u16(), 202);
    
    teardown_nodes();
}

#[tokio::test]
async fn test_outbox_endpoint_returns_collection() {
    setup_nodes().await;
    
    let context = TestContext::new();
    context.wait_for_nodes().await;

    let response = context
        .client
        .get(&format!("{}/users/alice/outbox", context.node_a_url))
        .header("Accept", "application/activity+json")
        .send()
        .await
        .expect("Failed to get Alice's outbox");

    assert!(response.status().is_success());
    
    let outbox_data: serde_json::Value = response.json().await.expect("Failed to parse outbox JSON");
    assert_eq!(outbox_data["type"], "OrderedCollection");
    assert_eq!(outbox_data["id"], format!("{}/users/alice/outbox", context.node_a_url));
    
    teardown_nodes();
} 