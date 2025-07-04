use actix_web::{test, web, App, HttpResponse};
use chrono::Utc;
use feder8::config::Config;
use feder8::database::{DatabaseRef, DbActor, DbActivity, DbNote, MockDatabase};
use feder8::handlers;
use mockall::predicate::*;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

// Helper function to create a test app with mock database
fn create_test_app(db: DatabaseRef) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let config = Config::default();
    App::new()
        .app_data(web::Data::new(config))
        .app_data(web::Data::new(db))
        .service(handlers::actor::get_actor)
        .service(handlers::outbox::get_outbox)
        .service(handlers::outbox::post_outbox)
        .service(handlers::inbox::inbox)
}

#[tokio::test]
async fn test_get_actor_handler_success() {
    let mut mock = MockDatabase::new();

    mock.expect_get_actor_by_username()
        .with(eq("testuser"))
        .returning(|_| {
            Ok(Some(DbActor {
                id: "https://example.com/users/testuser".to_string(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                summary: Some("A test user".to_string()),
                public_key_pem: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----".to_string(),
                private_key_pem: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let req = test::TestRequest::get()
        .uri("/users/testuser")
        .insert_header(("Accept", "application/activity+json"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["type"], "Person");
    assert_eq!(body["preferredUsername"], "testuser");
    assert_eq!(body["name"], "Test User");
}

#[tokio::test]
async fn test_get_actor_handler_not_found() {
    let mut mock = MockDatabase::new();

    mock.expect_get_actor_by_username()
        .with(eq("nonexistent"))
        .returning(|_| Ok(None));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let req = test::TestRequest::get()
        .uri("/users/nonexistent")
        .insert_header(("Accept", "application/activity+json"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Actor not found");
}

#[tokio::test]
async fn test_get_actor_handler_database_error() {
    let mut mock = MockDatabase::new();

    mock.expect_get_actor_by_username()
        .with(eq("error_user"))
        .returning(|_| Err(feder8::database::DatabaseError::Query("Database error".to_string())));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let req = test::TestRequest::get()
        .uri("/users/error_user")
        .insert_header(("Accept", "application/activity+json"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Internal server error");
}

#[tokio::test]
async fn test_get_outbox_handler_success() {
    let mut mock = MockDatabase::new();

    let actor_id = "https://example.com/users/testuser".to_string();
    let activity_id = "https://example.com/activities/1".to_string();

    mock.expect_get_actor_by_username()
        .with(eq("testuser"))
        .returning(move |_| {
            Ok(Some(DbActor {
                id: actor_id.clone(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                summary: None,
                public_key_pem: "test_key".to_string(),
                private_key_pem: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    mock.expect_get_actor_outbox_count()
        .with(eq(actor_id.clone()))
        .returning(|_| Ok(2));

    mock.expect_get_activities_by_actor()
        .with(eq(actor_id.clone()), eq(20), eq(0))
        .returning(move |_, _, _| {
            Ok(vec![
                DbActivity {
                    id: activity_id.clone(),
                    actor_id: actor_id.clone(),
                    activity_type: "Create".to_string(),
                    object: json!({"type": "Note", "content": "Hello, world!"}),
                    to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
                    cc_recipients: vec![],
                    published: Utc::now(),
                    created_at: Utc::now(),
                },
            ])
        });

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let req = test::TestRequest::get()
        .uri("/users/testuser/outbox")
        .insert_header(("Accept", "application/activity+json"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["type"], "OrderedCollection");
    assert_eq!(body["totalItems"], 2);
    assert_eq!(body["orderedItems"].as_array().unwrap().len(), 1);
    assert_eq!(body["orderedItems"][0]["type"], "Create");
}

#[tokio::test]
async fn test_get_outbox_handler_actor_not_found() {
    let mut mock = MockDatabase::new();

    mock.expect_get_actor_by_username()
        .with(eq("nonexistent"))
        .returning(|_| Ok(None));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let req = test::TestRequest::get()
        .uri("/users/nonexistent/outbox")
        .insert_header(("Accept", "application/activity+json"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Actor not found");
}

#[tokio::test]
async fn test_post_outbox_handler_create_note() {
    let mut mock = MockDatabase::new();

    let actor_id = "https://example.com/users/testuser".to_string();

    mock.expect_get_actor_by_username()
        .with(eq("testuser"))
        .returning(move |_| {
            Ok(Some(DbActor {
                id: actor_id.clone(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                summary: None,
                public_key_pem: "test_key".to_string(),
                private_key_pem: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    mock.expect_create_note()
        .returning(|_| Ok(()));

    mock.expect_create_activity()
        .returning(|_| Ok(()));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let create_activity = json!({
        "type": "Create",
        "actor": "https://example.com/users/testuser",
        "object": {
            "type": "Note",
            "content": "Hello from the test!",
            "to": ["https://www.w3.org/ns/activitystreams#Public"]
        },
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": []
    });

    let req = test::TestRequest::post()
        .uri("/users/testuser/outbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&create_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["type"], "Create");
    assert_eq!(body["actor"], "https://example.com/users/testuser");
    assert!(body["id"].as_str().unwrap().starts_with("https://example.com/activities/"));
}

#[tokio::test]
async fn test_post_outbox_handler_actor_not_found() {
    let mut mock = MockDatabase::new();

    mock.expect_get_actor_by_username()
        .with(eq("nonexistent"))
        .returning(|_| Ok(None));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let create_activity = json!({
        "type": "Create",
        "object": {
            "type": "Note",
            "content": "Hello!"
        }
    });

    let req = test::TestRequest::post()
        .uri("/users/nonexistent/outbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&create_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Actor not found");
}

#[tokio::test]
async fn test_inbox_handler_create_note() {
    let mut mock = MockDatabase::new();

    let actor_id = "https://example.com/users/testuser".to_string();

    mock.expect_get_actor_by_username()
        .with(eq("testuser"))
        .returning(move |_| {
            Ok(Some(DbActor {
                id: actor_id.clone(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                summary: None,
                public_key_pem: "test_key".to_string(),
                private_key_pem: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    mock.expect_get_note_by_id()
        .returning(|_| Ok(None)); // Note doesn't exist yet

    mock.expect_create_note()
        .returning(|_| Ok(()));

    mock.expect_create_activity()
        .returning(|_| Ok(()));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let create_activity = json!({
        "id": "https://remote.example/activities/1",
        "type": "Create",
        "actor": "https://remote.example/users/alice",
        "object": {
            "id": "https://remote.example/notes/1",
            "type": "Note",
            "attributedTo": "https://remote.example/users/alice",
            "content": "Hello from remote server!",
            "to": ["https://example.com/users/testuser"],
            "published": "2023-01-01T00:00:00Z"
        },
        "to": ["https://example.com/users/testuser"],
        "published": "2023-01-01T00:00:00Z"
    });

    let req = test::TestRequest::post()
        .uri("/users/testuser/inbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&create_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 202); // Accepted
}

#[tokio::test]
async fn test_inbox_handler_follow_activity() {
    let mut mock = MockDatabase::new();

    let actor_id = "https://example.com/users/testuser".to_string();

    mock.expect_get_actor_by_username()
        .with(eq("testuser"))
        .returning(move |_| {
            Ok(Some(DbActor {
                id: actor_id.clone(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                summary: None,
                public_key_pem: "test_key".to_string(),
                private_key_pem: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    mock.expect_create_follow()
        .returning(|_| Ok(()));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let follow_activity = json!({
        "id": "https://remote.example/activities/follow/1",
        "type": "Follow",
        "actor": "https://remote.example/users/alice",
        "object": "https://example.com/users/testuser"
    });

    let req = test::TestRequest::post()
        .uri("/users/testuser/inbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&follow_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 202); // Accepted
}

#[tokio::test]
async fn test_inbox_handler_accept_activity() {
    let mut mock = MockDatabase::new();

    let actor_id = "https://example.com/users/testuser".to_string();

    mock.expect_get_actor_by_username()
        .with(eq("testuser"))
        .returning(move |_| {
            Ok(Some(DbActor {
                id: actor_id.clone(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                summary: None,
                public_key_pem: "test_key".to_string(),
                private_key_pem: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    mock.expect_update_follow_status()
        .with(eq("https://remote.example/activities/follow/1"), eq("accepted"))
        .returning(|_, _| Ok(()));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let accept_activity = json!({
        "id": "https://remote.example/activities/accept/1",
        "type": "Accept",
        "actor": "https://remote.example/users/alice",
        "object": {
            "id": "https://remote.example/activities/follow/1",
            "type": "Follow",
            "actor": "https://example.com/users/testuser",
            "object": "https://remote.example/users/alice"
        }
    });

    let req = test::TestRequest::post()
        .uri("/users/testuser/inbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&accept_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 202); // Accepted
}

#[tokio::test]
async fn test_inbox_handler_actor_not_found() {
    let mut mock = MockDatabase::new();

    mock.expect_get_actor_by_username()
        .with(eq("nonexistent"))
        .returning(|_| Ok(None));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    let create_activity = json!({
        "type": "Create",
        "object": {
            "type": "Note",
            "content": "Hello!"
        }
    });

    let req = test::TestRequest::post()
        .uri("/users/nonexistent/inbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&create_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Actor not found");
}

// Integration test that simulates a complete flow
#[tokio::test]
async fn test_complete_activity_flow() {
    let mut mock = MockDatabase::new();

    let actor_id = "https://example.com/users/alice".to_string();
    let follower_id = "https://example.com/users/bob".to_string();

    // Setup expectations for the complete flow
    mock.expect_get_actor_by_username()
        .with(eq("alice"))
        .returning(move |_| {
            Ok(Some(DbActor {
                id: actor_id.clone(),
                username: "alice".to_string(),
                name: "Alice".to_string(),
                summary: Some("Alice's profile".to_string()),
                public_key_pem: "alice_key".to_string(),
                private_key_pem: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    mock.expect_get_actor_by_username()
        .with(eq("bob"))
        .returning(move |_| {
            Ok(Some(DbActor {
                id: follower_id.clone(),
                username: "bob".to_string(),
                name: "Bob".to_string(),
                summary: Some("Bob's profile".to_string()),
                public_key_pem: "bob_key".to_string(),
                private_key_pem: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    mock.expect_create_note()
        .returning(|_| Ok(()));

    mock.expect_create_activity()
        .returning(|_| Ok(()));

    mock.expect_create_follow()
        .returning(|_| Ok(()));

    mock.expect_get_actor_outbox_count()
        .returning(|_| Ok(1));

    mock.expect_get_activities_by_actor()
        .returning(|_, _, _| {
            Ok(vec![DbActivity {
                id: "https://example.com/activities/1".to_string(),
                actor_id: "https://example.com/users/alice".to_string(),
                activity_type: "Create".to_string(),
                object: json!({"type": "Note", "content": "Hello, world!"}),
                to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
                cc_recipients: vec![],
                published: Utc::now(),
                created_at: Utc::now(),
            }])
        });

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    // 1. Alice creates a note
    let create_activity = json!({
        "type": "Create",
        "object": {
            "type": "Note",
            "content": "Hello, ActivityPub world!",
            "to": ["https://www.w3.org/ns/activitystreams#Public"]
        },
        "to": ["https://www.w3.org/ns/activitystreams#Public"]
    });

    let req = test::TestRequest::post()
        .uri("/users/alice/outbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&create_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // 2. Bob follows Alice (sent to Alice's inbox)
    let follow_activity = json!({
        "id": "https://example.com/activities/follow/1",
        "type": "Follow",
        "actor": "https://example.com/users/bob",
        "object": "https://example.com/users/alice"
    });

    let req = test::TestRequest::post()
        .uri("/users/alice/inbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&follow_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 202);

    // 3. Check Alice's outbox
    let req = test::TestRequest::get()
        .uri("/users/alice/outbox")
        .insert_header(("Accept", "application/activity+json"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["type"], "OrderedCollection");
    assert_eq!(body["totalItems"], 1);
    assert_eq!(body["orderedItems"][0]["type"], "Create");
}

#[tokio::test]
async fn test_error_handling_in_handlers() {
    let mut mock = MockDatabase::new();

    // Test database error in get_actor
    mock.expect_get_actor_by_username()
        .with(eq("error_actor"))
        .returning(|_| Err(feder8::database::DatabaseError::Query("Connection failed".to_string())));

    // Test database error in create_note
    mock.expect_get_actor_by_username()
        .with(eq("note_error"))
        .returning(|_| {
            Ok(Some(DbActor {
                id: "https://example.com/users/note_error".to_string(),
                username: "note_error".to_string(),
                name: "Error User".to_string(),
                summary: None,
                public_key_pem: "test_key".to_string(),
                private_key_pem: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    mock.expect_create_note()
        .returning(|_| Err(feder8::database::DatabaseError::Query("Insert failed".to_string())));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(create_test_app(db)).await;

    // Test actor error
    let req = test::TestRequest::get()
        .uri("/users/error_actor")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500);

    // Test note creation error
    let create_activity = json!({
        "type": "Create",
        "object": {
            "type": "Note",
            "content": "This will fail"
        }
    });

    let req = test::TestRequest::post()
        .uri("/users/note_error/outbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&create_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500);
}