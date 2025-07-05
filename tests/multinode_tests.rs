use actix_web::{middleware::Logger, web, App, HttpServer};
use feder8::{
    config::Config,
    database::{create_configured_mock_database, DatabaseRef},
    handlers,
};
use rand::Rng;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

static NODE_COUNT: usize = 7;

mod test_harness {
    use super::*;

    static INIT: Once = Once::new();
    static NODES: Mutex<Option<Arc<Mutex<Vec<JoinHandle<()>>>>>> = Mutex::new(None);

    pub struct TestContext {
        pub client: Client,
        pub node_urls: Vec<String>,
        pub actor_names: Vec<String>,
        pub base_port: u16,
    }

    impl TestContext {
        pub fn new(node_count: usize, base_port: u16) -> Self {
            let node_urls = (0..node_count)
                .map(|i| format!("http://localhost:{}", base_port + i as u16))
                .collect();
            let actor_names = (0..node_count).map(|i| format!("actor{}", i + 1)).collect();
            Self {
                client: Client::new(),
                node_urls,
                actor_names,
                base_port,
            }
        }

        pub async fn wait_for_nodes(&self) {
            println!("Waiting for nodes to start...");
            sleep(Duration::from_secs(2)).await;
            let mut attempts = 0;
            while attempts < 20 {
                let mut all_ready = true;
                for url in &self.node_urls {
                    if !self.is_node_ready(url).await {
                        all_ready = false;
                        break;
                    }
                }
                if all_ready {
                    println!("{} nodes are ready!", self.node_urls.len());
                    return;
                }
                sleep(Duration::from_millis(250)).await;
                attempts += 1;
            }
            panic!("Nodes failed to start within expected time");
        }

        async fn is_node_ready(&self, url: &str) -> bool {
            (self
                .client
                .get(url)
                .timeout(Duration::from_secs(1))
                .send()
                .await)
                .is_ok()
        }
    }

    async fn start_node(port: u16, actor_name: &str) -> JoinHandle<()> {
        let config = Config {
            server_name: format!("Test Node {actor_name}"),
            server_url: format!("http://localhost:{port}"),
            port,
            actor_name: actor_name.to_string(),
            private_key_path: None,
            public_key_path: None,
        };

        let config_clone = config.clone();
        let server_handle = tokio::spawn(async move {
            // Initialize database (using mock for tests)
            let db: DatabaseRef = Arc::new(create_configured_mock_database());

            let _ = HttpServer::new(move || {
                App::new()
                    .wrap(Logger::default())
                    .app_data(web::Data::new(config_clone.clone()))
                    .app_data(web::Data::new(db.clone()))
                    .service(handlers::webfinger::webfinger)
                    .service(handlers::actor::get_actor)
                    .service(handlers::inbox::inbox)
                    .service(handlers::outbox::get_outbox)
                    .service(handlers::outbox::post_outbox)
            })
            .bind(("127.0.0.1", port))
            .unwrap_or_else(|e| {
                eprintln!("Failed to bind to port {port}: {e}");
                std::process::exit(1);
            })
            .run()
            .await;
        });

        server_handle
    }

    pub async fn setup_nodes(node_count: usize, base_port: u16) {
        INIT.call_once(|| {
            println!("Setting up test nodes...");
        });
        let mut handles = Vec::with_capacity(node_count);
        for i in 0..node_count {
            let port = base_port + i as u16;
            let actor_name = format!("actor{}", i + 1);
            handles.push(start_node(port, &actor_name).await);
        }
        let nodes = Arc::new(Mutex::new(handles));
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

use test_harness::{setup_nodes, teardown_nodes, TestContext};

#[tokio::test]
async fn test_node_setup_and_teardown() {
    let node_count = NODE_COUNT;
    let base_port = rand::thread_rng().gen_range(20000..60000);
    setup_nodes(node_count, base_port).await;

    let context = TestContext::new(node_count, base_port);
    context.wait_for_nodes().await;

    // Verify both nodes are running
    for url in &context.node_urls {
        assert!(context
            .client
            .get(url)
            .timeout(Duration::from_secs(1))
            .send()
            .await
            .is_ok());
    }

    teardown_nodes();
}

#[tokio::test]
async fn test_alice_actor_profile() {
    let node_count = NODE_COUNT;
    let base_port = rand::thread_rng().gen_range(20000..60000);
    setup_nodes(node_count, base_port).await;

    let context = TestContext::new(node_count, base_port);
    context.wait_for_nodes().await;

    let response = context
        .client
        .get(format!(
            "{}/users/{}",
            context.node_urls[0], context.actor_names[0]
        ))
        .header("Accept", "application/activity+json")
        .send()
        .await
        .expect("Failed to get actor profile");

    assert!(response.status().is_success());

    let actor_data: serde_json::Value = response.json().await.expect("Failed to parse actor JSON");

    // Check both camelCase and snake_case for backward compatibility
    let username_field = actor_data
        .get("preferredUsername")
        .or_else(|| actor_data.get("preferred_username"));
    assert_eq!(
        username_field,
        Some(&serde_json::Value::String(context.actor_names[0].clone()))
    );
    assert_eq!(actor_data["type"], "Person");
    assert!(actor_data["inbox"]
        .as_str()
        .unwrap()
        .contains(&format!("/users/{}", context.actor_names[0])));

    teardown_nodes();
}

#[tokio::test]
async fn test_bob_actor_profile() {
    let node_count = NODE_COUNT;
    let base_port = rand::thread_rng().gen_range(20000..60000);
    setup_nodes(node_count, base_port).await;
    let context = TestContext::new(node_count, base_port);
    context.wait_for_nodes().await;
    let response = context
        .client
        .get(format!(
            "{}/users/{}",
            context.node_urls[1], context.actor_names[1]
        ))
        .header("Accept", "application/activity+json")
        .send()
        .await
        .expect("Failed to get actor profile");
    assert!(response.status().is_success());
    let actor_data: serde_json::Value = response.json().await.expect("Failed to parse actor JSON");
    let username_field = actor_data
        .get("preferredUsername")
        .or_else(|| actor_data.get("preferred_username"));
    assert_eq!(
        username_field,
        Some(&serde_json::Value::String(context.actor_names[1].clone()))
    );
    assert_eq!(actor_data["type"], "Person");
    assert!(actor_data["inbox"]
        .as_str()
        .unwrap()
        .contains(&format!("/users/{}", context.actor_names[1])));
    teardown_nodes();
}

#[tokio::test]
async fn test_webfinger_discovery() {
    let node_count = NODE_COUNT;
    let base_port = rand::thread_rng().gen_range(20000..60000);
    setup_nodes(node_count, base_port).await;
    let context = TestContext::new(node_count, base_port);
    context.wait_for_nodes().await;
    let response = context
        .client
        .get(format!(
            "{}/.well-known/webfinger?resource=acct:{}@localhost:{}",
            context.node_urls[0], context.actor_names[0], context.base_port
        ))
        .header("Accept", "application/jrd+json")
        .send()
        .await
        .expect("Failed to get WebFinger response");
    assert!(response.status().is_success());
    let webfinger_data: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse WebFinger JSON");
    assert_eq!(
        webfinger_data["subject"],
        format!(
            "acct:{}@localhost:{}",
            context.actor_names[0], context.base_port
        )
    );
    let links = webfinger_data["links"]
        .as_array()
        .expect("Links should be an array");
    assert!(!links.is_empty());
    let activitypub_link = links
        .iter()
        .find(|link| link["rel"] == "self" && link["type"] == "application/activity+json");
    assert!(activitypub_link.is_some());
    teardown_nodes();
}

#[tokio::test]
async fn test_message_delivery_between_nodes() {
    let node_count = NODE_COUNT;
    let base_port = rand::thread_rng().gen_range(20000..60000);
    setup_nodes(node_count, base_port).await;
    let context = TestContext::new(node_count, base_port);
    context.wait_for_nodes().await;
    let note = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/notes/789",
        "type": "Note",
        "attributedTo": format!("{}/users/{}", context.node_urls[0], context.actor_names[0]),
        "content": "Hello! This is a test message.",
        "to": [format!("{}/users/{}", context.node_urls[1], context.actor_names[1])],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"],
        "published": "2024-01-01T12:00:00Z"
    });
    let create_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/activities/101",
        "type": "Create",
        "actor": format!("{}/users/{}", context.node_urls[0], context.actor_names[0]),
        "object": note,
        "to": [format!("{}/users/{}", context.node_urls[1], context.actor_names[1])],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"],
        "published": "2024-01-01T12:00:00Z"
    });
    let response = context
        .client
        .post(format!(
            "{}/users/{}/inbox",
            context.node_urls[1], context.actor_names[1]
        ))
        .header("Content-Type", "application/activity+json")
        .json(&create_activity)
        .send()
        .await
        .expect("Failed to send message to inbox");
    assert!(response.status().is_success() || response.status().as_u16() == 202);
    teardown_nodes();
}

#[tokio::test]
async fn test_cross_node_actor_discovery() {
    let node_count = NODE_COUNT;
    let base_port = rand::thread_rng().gen_range(20000..60000);
    setup_nodes(node_count, base_port).await;
    let context = TestContext::new(node_count, base_port);
    context.wait_for_nodes().await;
    let response = context
        .client
        .get(format!(
            "{}/users/{}",
            context.node_urls[1], context.actor_names[1]
        ))
        .header("Accept", "application/activity+json")
        .send()
        .await
        .expect("Failed to get actor profile");
    assert!(response.status().is_success());
    let actor_data: serde_json::Value = response.json().await.expect("Failed to parse actor JSON");
    let username_field = actor_data
        .get("preferredUsername")
        .or_else(|| actor_data.get("preferred_username"));
    assert_eq!(
        username_field,
        Some(&serde_json::Value::String(context.actor_names[1].clone()))
    );
    assert_eq!(
        actor_data["id"],
        format!("{}/users/{}", context.node_urls[1], context.actor_names[1])
    );
    teardown_nodes();
}

#[tokio::test]
async fn test_inbox_endpoint_accepts_activities() {
    let node_count = NODE_COUNT;
    let base_port = rand::thread_rng().gen_range(20000..60000);
    setup_nodes(node_count, base_port).await;
    let context = TestContext::new(node_count, base_port);
    context.wait_for_nodes().await;
    let test_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/activities/test-123",
        "type": "Create",
        "actor": format!("{}/users/{}", context.node_urls[0], context.actor_names[0]),
        "object": {
            "@context": ["https://www.w3.org/ns/activitystreams"],
            "id": "https://example.com/notes/test-456",
            "type": "Note",
            "content": "Test message for inbox endpoint",
            "attributedTo": format!("{}/users/{}", context.node_urls[0], context.actor_names[0])
        },
        "to": [format!("{}/users/{}", context.node_urls[1], context.actor_names[1])],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"]
    });
    let response = context
        .client
        .post(format!(
            "{}/users/{}/inbox",
            context.node_urls[1], context.actor_names[1]
        ))
        .header("Content-Type", "application/activity+json")
        .json(&test_activity)
        .send()
        .await
        .expect("Failed to post activity to inbox");
    assert_eq!(response.status().as_u16(), 202);
    teardown_nodes();
}

#[tokio::test]
async fn test_outbox_endpoint_returns_collection() {
    let node_count = NODE_COUNT;
    let base_port = rand::thread_rng().gen_range(20000..60000);
    setup_nodes(node_count, base_port).await;
    let context = TestContext::new(node_count, base_port);
    context.wait_for_nodes().await;
    let response = context
        .client
        .get(format!(
            "{}/users/{}/outbox",
            context.node_urls[0], context.actor_names[0]
        ))
        .header("Accept", "application/activity+json")
        .send()
        .await
        .expect("Failed to get outbox");
    assert!(response.status().is_success());
    let outbox_data: serde_json::Value =
        response.json().await.expect("Failed to parse outbox JSON");
    assert_eq!(outbox_data["type"], "OrderedCollection");
    assert_eq!(
        outbox_data["id"],
        format!(
            "{}/users/{}/outbox",
            context.node_urls[0], context.actor_names[0]
        )
    );
    teardown_nodes();
}
