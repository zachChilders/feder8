use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    println!("=== Fediverse Two-Node Test ===");
    println!("This example demonstrates sending messages between two Fediverse nodes");
    println!();

    // Wait for servers to start
    println!("Waiting for servers to start...");
    sleep(Duration::from_secs(2)).await;

    // Test Node A (localhost:8080)
    println!("1. Testing Node A (localhost:8080)");

    // Check actor profile
    let response = client
        .get("http://localhost:8080/users/alice")
        .header("Accept", "application/activity+json")
        .send()
        .await?;

    println!("   Actor profile status: {}", response.status());

    // Test WebFinger
    let response = client
        .get("http://localhost:8080/.well-known/webfinger?resource=acct:alice@localhost:8080")
        .header("Accept", "application/jrd+json")
        .send()
        .await?;

    println!("   WebFinger status: {}", response.status());

    // Test Node B (localhost:8081) - if running
    println!("\n2. Testing Node B (localhost:8081)");

    let response = client
        .get("http://localhost:8081/users/bob")
        .header("Accept", "application/activity+json")
        .send()
        .await;

    match response {
        Ok(resp) => println!("   Actor profile status: {}", resp.status()),
        Err(_) => println!("   Node B not running (expected)"),
    }

    // Send a message from Alice to Bob
    println!("\n3. Sending message from Alice to Bob");

    let note = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/notes/789",
        "type": "Note",
        "attributedTo": "https://localhost:8080/users/alice",
        "content": "Hello Bob! This is a test message from Alice.",
        "to": ["https://localhost:8081/users/bob"],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"],
        "published": "2024-01-01T12:00:00Z"
    });

    let create_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/activities/101",
        "type": "Create",
        "actor": "https://localhost:8080/users/alice",
        "object": note,
        "to": ["https://localhost:8081/users/bob"],
        "cc": ["https://www.w3.org/ns/activitystreams#Public"],
        "published": "2024-01-01T12:00:00Z"
    });

    let response = client
        .post("http://localhost:8081/users/bob/inbox")
        .header("Content-Type", "application/activity+json")
        .json(&create_activity)
        .send()
        .await;

    match response {
        Ok(resp) => {
            println!("   Message delivery status: {}", resp.status());
            if resp.status().is_success() {
                println!("   ✅ Message sent successfully!");
            } else {
                println!("   ❌ Message delivery failed");
            }
        }
        Err(e) => {
            println!("   ❌ Failed to send message: {e}");
            println!("   (This is expected if Node B is not running)");
        }
    }

    println!("\n=== Test Complete ===");
    println!("To run a full two-node test:");
    println!("1. Start Node A: cargo run");
    println!(
        "2. Start Node B: SERVER_URL=http://localhost:8081 PORT=8081 ACTOR_NAME=bob cargo run"
    );
    println!("3. Run this test: cargo run --example two_node_test");

    Ok(())
}
