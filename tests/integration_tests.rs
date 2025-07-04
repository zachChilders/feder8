use actix_web::{http::StatusCode, test, web, App};
use feder8::{
    config::Config,
    database::{create_configured_mock_database, DatabaseRef},
    handlers,
    models::Actor,
};
use serde_json::{json, Value};
use std::sync::Arc;

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

#[actix_web::test]
async fn test_webfinger_valid_request() {
    let config = create_test_config();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .service(handlers::webfinger::webfinger),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/.well-known/webfinger?resource=acct:testuser@test.example.com")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["subject"], "acct:testuser@test.example.com");
    assert!(body["links"].is_array());

    let links = body["links"].as_array().unwrap();
    assert!(!links.is_empty());

    // Check for ActivityPub link
    let activitypub_link = links
        .iter()
        .find(|link| link["rel"] == "self" && link["type"] == "application/activity+json");
    assert!(activitypub_link.is_some());
}

#[actix_web::test]
async fn test_webfinger_invalid_domain() {
    let config = create_test_config();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .service(handlers::webfinger::webfinger),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/.well-known/webfinger?resource=acct:testuser@different.domain.com")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_webfinger_malformed_resource() {
    let config = create_test_config();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .service(handlers::webfinger::webfinger),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/.well-known/webfinger?resource=invalid-resource")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_webfinger_missing_resource() {
    let config = create_test_config();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .service(handlers::webfinger::webfinger),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/.well-known/webfinger")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // This should fail due to missing required query parameter
    assert_ne!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_get_actor() {
    let config = create_test_config();
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(db))
            .service(handlers::actor::get_actor),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/users/alice")
        .insert_header(("Accept", "application/activity+json"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Actor = test::read_body_json(resp).await;
    assert_eq!(body.preferred_username, "alice");
    assert_eq!(body.actor_type, "Person");
    assert!(body.id.contains("/users/alice"));
    assert!(body.inbox.contains("/inbox"));
    assert!(body.outbox.contains("/outbox"));
}

#[actix_web::test]
async fn test_get_actor_different_username() {
    let config = create_test_config();
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::actor::get_actor),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/users/bob")
        .insert_header(("Accept", "application/activity+json"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Actor = test::read_body_json(resp).await;
    assert_eq!(body.preferred_username, "bob");
    assert_eq!(body.name, "Test User bob");
}

#[actix_web::test]
async fn test_inbox_create_activity() {
    let config = create_test_config();
    let mut mock = feder8::database::MockDatabase::new();

    // Set up expectations for inbox processing
    mock.expect_get_actor_by_username().returning(|username| {
        Ok(Some(feder8::database::DbActor {
            id: format!("https://test.example.com/users/{}", username),
            username: username.to_string(),
            name: format!("Test User {}", username),
            summary: Some("A test user".to_string()),
            public_key_pem: "test_key".to_string(),
            private_key_pem: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }))
    });

    mock.expect_get_note_by_id().returning(|_| Ok(None)); // Note doesn't exist yet

    mock.expect_create_note().returning(|_| Ok(()));

    mock.expect_create_activity().returning(|_| Ok(()));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::inbox::inbox),
    )
    .await;

    let create_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/activities/123",
        "type": "Create",
        "actor": "https://example.com/users/alice",
        "object": {
            "type": "Note",
            "content": "Hello, world!",
            "attributedTo": "https://example.com/users/alice"
        },
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": []
    });

    let req = test::TestRequest::post()
        .uri("/users/bob/inbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&create_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
}

#[actix_web::test]
async fn test_inbox_follow_activity() {
    let config = create_test_config();
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::inbox::inbox),
    )
    .await;

    let follow_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/activities/456",
        "type": "Follow",
        "actor": "https://example.com/users/alice",
        "object": "https://test.example.com/users/bob",
        "to": ["https://test.example.com/users/bob"],
        "cc": []
    });

    let req = test::TestRequest::post()
        .uri("/users/bob/inbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&follow_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
}

#[actix_web::test]
async fn test_inbox_unknown_activity() {
    let config = create_test_config();
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::inbox::inbox),
    )
    .await;

    let unknown_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "id": "https://example.com/activities/789",
        "type": "UnknownActivity",
        "actor": "https://example.com/users/alice",
        "object": "some-object",
        "to": [],
        "cc": []
    });

    let req = test::TestRequest::post()
        .uri("/users/bob/inbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&unknown_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED); // Should still accept unknown activities
}

#[actix_web::test]
async fn test_get_outbox() {
    let config = create_test_config();
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::outbox::get_outbox),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/users/alice/outbox")
        .insert_header(("Accept", "application/activity+json"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["type"], "OrderedCollection");
    assert_eq!(body["totalItems"], 5); // Mock returns 5 items
    assert!(body["orderedItems"].is_array());
    assert!(body["orderedItems"].as_array().unwrap().is_empty()); // But activities list is empty
}

#[actix_web::test]
async fn test_post_outbox_create_activity() {
    let config = create_test_config();
    let mut mock = feder8::database::MockDatabase::new();

    // Set up expectations for outbox processing
    mock.expect_get_actor_by_username().returning(|username| {
        Ok(Some(feder8::database::DbActor {
            id: format!("https://test.example.com/users/{}", username),
            username: username.to_string(),
            name: format!("Test User {}", username),
            summary: Some("A test user".to_string()),
            public_key_pem: "test_key".to_string(),
            private_key_pem: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }))
    });

    mock.expect_create_note().returning(|_| Ok(()));

    mock.expect_create_activity().returning(|_| Ok(()));

    let db: DatabaseRef = Arc::new(mock);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::outbox::post_outbox),
    )
    .await;

    let create_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "type": "Create",
        "actor": "https://test.example.com/users/alice",
        "object": {
            "type": "Note",
            "content": "This is a test note from Alice!",
            "attributedTo": "https://test.example.com/users/alice"
        },
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": ["https://test.example.com/users/alice/followers"]
    });

    let req = test::TestRequest::post()
        .uri("/users/alice/outbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&create_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[actix_web::test]
async fn test_post_outbox_unsupported_activity() {
    let config = create_test_config();
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::outbox::post_outbox),
    )
    .await;

    let follow_activity = json!({
        "@context": ["https://www.w3.org/ns/activitystreams"],
        "type": "Follow",
        "actor": "https://test.example.com/users/alice",
        "object": "https://example.com/users/bob",
        "to": ["https://example.com/users/bob"],
        "cc": []
    });

    let req = test::TestRequest::post()
        .uri("/users/alice/outbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_json(&follow_activity)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED); // Still accepts but logs as unsupported
}

#[actix_web::test]
async fn test_content_type_headers() {
    let config = create_test_config();
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::actor::get_actor)
            .service(handlers::outbox::get_outbox),
    )
    .await;

    // Test actor endpoint content type
    let req = test::TestRequest::get().uri("/users/alice").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type
        .to_str()
        .unwrap()
        .contains("application/activity+json"));

    // Test outbox endpoint content type
    let req = test::TestRequest::get()
        .uri("/users/alice/outbox")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type
        .to_str()
        .unwrap()
        .contains("application/activity+json"));
}

#[actix_web::test]
async fn test_webfinger_content_type() {
    let config = create_test_config();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .service(handlers::webfinger::webfinger),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/.well-known/webfinger?resource=acct:testuser@test.example.com")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type
        .to_str()
        .unwrap()
        .contains("application/jrd+json"));
}

#[actix_web::test]
async fn test_inbox_malformed_json() {
    let config = create_test_config();
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::inbox::inbox),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/users/bob/inbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_payload("{invalid json}")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should handle malformed JSON gracefully
    assert_ne!(resp.status(), StatusCode::ACCEPTED);
}

#[actix_web::test]
async fn test_outbox_malformed_json() {
    let config = create_test_config();
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(db))
            .service(handlers::outbox::post_outbox),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/users/alice/outbox")
        .insert_header(("Content-Type", "application/activity+json"))
        .set_payload("{invalid json}")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should handle malformed JSON gracefully
    assert_ne!(resp.status(), StatusCode::CREATED);
}
