use chrono::Utc;
use feder8::database::{
    create_configured_mock_database, DatabaseRef, DbActivity, DbActor, DbFollowRelation, DbNote,
    MockDatabase,
};
use mockall::predicate::*;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_mock_database_actor_operations() {
    let mut mock = MockDatabase::new();

    // Test get_actor_by_username
    mock.expect_get_actor_by_username()
        .with(eq("testuser"))
        .returning(|_| {
            Ok(Some(DbActor {
                id: "https://example.com/users/testuser".to_string(),
                username: "testuser".to_string(),
                name: "Test User".to_string(),
                summary: Some("A test user".to_string()),
                public_key_pem: "-----BEGIN PUBLIC KEY-----\ntest\n-----END PUBLIC KEY-----"
                    .to_string(),
                private_key_pem: Some(
                    "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----".to_string(),
                ),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    // Test get_actor_by_username for non-existent user
    mock.expect_get_actor_by_username()
        .with(eq("nonexistent"))
        .returning(|_| Ok(None));

    let db: DatabaseRef = Arc::new(mock);

    // Test existing user
    let actor = db.get_actor_by_username("testuser").await.unwrap();
    assert!(actor.is_some());
    let actor = actor.unwrap();
    assert_eq!(actor.username, "testuser");
    assert_eq!(actor.name, "Test User");

    // Test non-existent user
    let actor = db.get_actor_by_username("nonexistent").await.unwrap();
    assert!(actor.is_none());
}

#[tokio::test]
async fn test_mock_database_activity_operations() {
    let mut mock = MockDatabase::new();

    let test_actor_id = "https://example.com/users/testuser".to_string();
    let test_activity = DbActivity {
        id: format!("https://example.com/activities/{}", Uuid::new_v4()),
        actor_id: test_actor_id.clone(),
        activity_type: "Create".to_string(),
        object: json!({"type": "Note", "content": "Hello, world!"}),
        to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        cc_recipients: vec![],
        published: Utc::now(),
        created_at: Utc::now(),
    };

    // Test create_activity
    mock.expect_create_activity().returning(|_| Ok(()));

    // Test get_activities_by_actor
    mock.expect_get_activities_by_actor()
        .with(eq(test_actor_id.clone()), eq(20), eq(0))
        .returning(move |_, _, _| Ok(vec![test_activity.clone()]));

    // Test get_actor_outbox_count
    mock.expect_get_actor_outbox_count()
        .with(eq(test_actor_id.clone()))
        .returning(|_| Ok(1));

    let db: DatabaseRef = Arc::new(mock);

    // Test creating activity
    let new_activity = DbActivity {
        id: format!("https://example.com/activities/{}", Uuid::new_v4()),
        actor_id: test_actor_id.clone(),
        activity_type: "Create".to_string(),
        object: json!({"type": "Note", "content": "New note"}),
        to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        cc_recipients: vec![],
        published: Utc::now(),
        created_at: Utc::now(),
    };

    db.create_activity(&new_activity).await.unwrap();

    // Test getting activities
    let activities = db
        .get_activities_by_actor(&test_actor_id, 20, 0)
        .await
        .unwrap();
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].activity_type, "Create");

    // Test getting outbox count
    let count = db.get_actor_outbox_count(&test_actor_id).await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_mock_database_note_operations() {
    let mut mock = MockDatabase::new();

    let test_note = DbNote {
        id: format!("https://example.com/notes/{}", Uuid::new_v4()),
        attributed_to: "https://example.com/users/testuser".to_string(),
        content: "This is a test note".to_string(),
        to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        cc_recipients: vec![],
        published: Utc::now(),
        in_reply_to: None,
        tags: vec![],
        created_at: Utc::now(),
    };
    let test_note_clone1 = test_note.clone();
    let test_note_clone2 = test_note.clone();

    // Test create_note
    mock.expect_create_note().returning(|_| Ok(()));

    // Test get_note_by_id
    mock.expect_get_note_by_id()
        .with(eq(test_note.id.clone()))
        .returning(move |_| Ok(Some(test_note_clone1.clone())));

    // Test get_notes_by_actor
    mock.expect_get_notes_by_actor()
        .with(eq("https://example.com/users/testuser"), eq(20), eq(0))
        .returning(move |_, _, _| Ok(vec![test_note_clone2.clone()]));

    let db: DatabaseRef = Arc::new(mock);

    // Test creating note
    let new_note = DbNote {
        id: format!("https://example.com/notes/{}", Uuid::new_v4()),
        attributed_to: "https://example.com/users/testuser".to_string(),
        content: "Another test note".to_string(),
        to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        cc_recipients: vec![],
        published: Utc::now(),
        in_reply_to: None,
        tags: vec![],
        created_at: Utc::now(),
    };

    db.create_note(&new_note).await.unwrap();

    // Test getting note by ID
    let note = db.get_note_by_id(&test_note.id).await.unwrap();
    assert!(note.is_some());
    let note = note.unwrap();
    assert_eq!(note.content, "This is a test note");

    // Test getting notes by actor
    let notes = db
        .get_notes_by_actor("https://example.com/users/testuser", 20, 0)
        .await
        .unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "This is a test note");
}

#[tokio::test]
async fn test_mock_database_follow_operations() {
    let mut mock = MockDatabase::new();

    let test_follow = DbFollowRelation {
        id: format!("https://example.com/follows/{}", Uuid::new_v4()),
        follower_id: "https://example.com/users/alice".to_string(),
        following_id: "https://example.com/users/bob".to_string(),
        status: "accepted".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let test_follow_clone1 = test_follow.clone();
    let test_follow_clone2 = test_follow.clone();

    // Test create_follow
    mock.expect_create_follow().returning(|_| Ok(()));

    // Test get_followers
    mock.expect_get_followers()
        .with(eq("https://example.com/users/bob"), eq(20), eq(0))
        .returning(move |_, _, _| Ok(vec![test_follow_clone1.clone()]));

    // Test get_following
    mock.expect_get_following()
        .with(eq("https://example.com/users/alice"), eq(20), eq(0))
        .returning(move |_, _, _| Ok(vec![test_follow_clone2.clone()]));

    // Test get_actor_followers_count
    mock.expect_get_actor_followers_count()
        .with(eq("https://example.com/users/bob"))
        .returning(|_| Ok(1));

    // Test get_actor_following_count
    mock.expect_get_actor_following_count()
        .with(eq("https://example.com/users/alice"))
        .returning(|_| Ok(1));

    // Test update_follow_status
    mock.expect_update_follow_status()
        .with(eq(test_follow.id.clone()), eq("accepted"))
        .returning(|_, _| Ok(()));

    let db: DatabaseRef = Arc::new(mock);

    // Test creating follow
    let new_follow = DbFollowRelation {
        id: format!("https://example.com/follows/{}", Uuid::new_v4()),
        follower_id: "https://example.com/users/charlie".to_string(),
        following_id: "https://example.com/users/alice".to_string(),
        status: "pending".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    db.create_follow(&new_follow).await.unwrap();

    // Test getting followers
    let followers = db
        .get_followers("https://example.com/users/bob", 20, 0)
        .await
        .unwrap();
    assert_eq!(followers.len(), 1);
    assert_eq!(followers[0].follower_id, "https://example.com/users/alice");

    // Test getting following
    let following = db
        .get_following("https://example.com/users/alice", 20, 0)
        .await
        .unwrap();
    assert_eq!(following.len(), 1);
    assert_eq!(following[0].following_id, "https://example.com/users/bob");

    // Test getting follower count
    let count = db
        .get_actor_followers_count("https://example.com/users/bob")
        .await
        .unwrap();
    assert_eq!(count, 1);

    // Test getting following count
    let count = db
        .get_actor_following_count("https://example.com/users/alice")
        .await
        .unwrap();
    assert_eq!(count, 1);

    // Test updating follow status
    db.update_follow_status(&test_follow.id, "accepted")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_configured_mock_database() {
    let mock = create_configured_mock_database();
    let db: DatabaseRef = Arc::new(mock);

    // Test the preconfigured mock
    let actor = db.get_actor_by_username("testuser").await.unwrap();
    assert!(actor.is_some());
    let actor = actor.unwrap();
    assert!(actor.username.starts_with("testuser"));

    let count = db.get_actor_outbox_count(&actor.id).await.unwrap();
    assert_eq!(count, 5);

    let activities = db.get_activities_by_actor(&actor.id, 20, 0).await.unwrap();
    assert_eq!(activities.len(), 0); // Configured to return empty vec

    let inbox_count = db.get_actor_inbox_count(&actor.id).await.unwrap();
    assert_eq!(inbox_count, 3);
}

#[tokio::test]
async fn test_database_error_handling() {
    let mut mock = MockDatabase::new();

    // Test database errors
    mock.expect_get_actor_by_username()
        .returning(|_| Err(feder8::database::DatabaseError::NotFound));

    mock.expect_create_actor()
        .returning(|_| Err(feder8::database::DatabaseError::AlreadyExists));

    let db: DatabaseRef = Arc::new(mock);

    // Test error handling
    let result = db.get_actor_by_username("error_user").await;
    assert!(result.is_err());

    let test_actor = DbActor {
        id: "https://example.com/users/duplicate".to_string(),
        username: "duplicate".to_string(),
        name: "Duplicate User".to_string(),
        summary: None,
        public_key_pem: "test_key".to_string(),
        private_key_pem: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let result = db.create_actor(&test_actor).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_database_with_complex_data() {
    let mut mock = MockDatabase::new();

    let complex_note = DbNote {
        id: "https://example.com/notes/complex".to_string(),
        attributed_to: "https://example.com/users/author".to_string(),
        content: "Complex note with <strong>HTML</strong> and emoji 🚀".to_string(),
        to_recipients: vec![
            "https://www.w3.org/ns/activitystreams#Public".to_string(),
            "https://example.com/users/alice".to_string(),
        ],
        cc_recipients: vec!["https://example.com/users/author/followers".to_string()],
        published: Utc::now(),
        in_reply_to: Some("https://example.com/notes/original".to_string()),
        tags: vec!["#test".to_string(), "@alice".to_string()],
        created_at: Utc::now(),
    };

    mock.expect_get_note_by_id()
        .with(eq("https://example.com/notes/complex"))
        .returning(move |_| Ok(Some(complex_note.clone())));

    let db: DatabaseRef = Arc::new(mock);

    let note = db
        .get_note_by_id("https://example.com/notes/complex")
        .await
        .unwrap();
    assert!(note.is_some());
    let note = note.unwrap();

    assert!(note.content.contains("<strong>HTML</strong>"));
    assert!(note.content.contains("🚀"));
    assert_eq!(note.to_recipients.len(), 2);
    assert_eq!(note.cc_recipients.len(), 1);
    assert!(note.in_reply_to.is_some());
    assert_eq!(note.tags.len(), 2);
}

// Integration test with multiple database operations
#[tokio::test]
async fn test_database_integration_scenario() {
    let mut mock = MockDatabase::new();

    // Setup expectations for a complete scenario
    let actor_id = "https://example.com/users/alice".to_string();
    let follower_id = "https://example.com/users/bob".to_string();
    let note_id = "https://example.com/notes/1".to_string();
    let activity_id = "https://example.com/activities/1".to_string();

    // Create clones for closures
    let actor_id_clone1 = actor_id.clone();
    let actor_id_clone2 = actor_id.clone();
    let actor_id_clone3 = actor_id.clone();
    let actor_id_clone4 = actor_id.clone();
    let follower_id_clone1 = follower_id.clone();
    let follower_id_clone2 = follower_id.clone();
    let note_id_clone = note_id.clone();
    let activity_id_clone1 = activity_id.clone();
    let activity_id_clone2 = activity_id.clone();

    // Actor operations
    mock.expect_get_actor_by_username()
        .with(eq("alice"))
        .returning(move |_| {
            Ok(Some(DbActor {
                id: actor_id_clone1.clone(),
                username: "alice".to_string(),
                name: "Alice".to_string(),
                summary: Some("Alice's profile".to_string()),
                public_key_pem: "alice_key".to_string(),
                private_key_pem: Some("alice_private".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    // Note operations
    mock.expect_create_note().returning(|_| Ok(()));

    mock.expect_get_notes_by_actor()
        .with(eq(actor_id.clone()), eq(10), eq(0))
        .returning(move |_, _, _| {
            Ok(vec![DbNote {
                id: note_id_clone.clone(),
                attributed_to: actor_id_clone2.clone(),
                content: "Hello from Alice!".to_string(),
                to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
                cc_recipients: vec![],
                published: Utc::now(),
                in_reply_to: None,
                tags: vec![],
                created_at: Utc::now(),
            }])
        });

    // Activity operations
    mock.expect_create_activity().returning(|_| Ok(()));

    mock.expect_get_activities_by_actor()
        .with(eq(actor_id.clone()), eq(10), eq(0))
        .returning(move |_, _, _| {
            Ok(vec![DbActivity {
                id: activity_id_clone1.clone(),
                actor_id: actor_id_clone3.clone(),
                activity_type: "Create".to_string(),
                object: json!({"type": "Note", "content": "Hello from Alice!"}),
                to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
                cc_recipients: vec![],
                published: Utc::now(),
                created_at: Utc::now(),
            }])
        });

    // Follow operations
    mock.expect_create_follow().returning(|_| Ok(()));

    mock.expect_get_followers()
        .with(eq(actor_id.clone()), eq(10), eq(0))
        .returning(move |_, _, _| {
            Ok(vec![DbFollowRelation {
                id: "https://example.com/follows/1".to_string(),
                follower_id: follower_id_clone1.clone(),
                following_id: actor_id_clone4.clone(),
                status: "accepted".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }])
        });

    let db: DatabaseRef = Arc::new(mock);

    // Simulate a complete scenario
    // 1. Get actor
    let actor = db.get_actor_by_username("alice").await.unwrap().unwrap();
    assert_eq!(actor.name, "Alice");

    // 2. Create a note
    let note = DbNote {
        id: note_id.clone(),
        attributed_to: actor.id.clone(),
        content: "New note from Alice".to_string(),
        to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        cc_recipients: vec![],
        published: Utc::now(),
        in_reply_to: None,
        tags: vec![],
        created_at: Utc::now(),
    };
    db.create_note(&note).await.unwrap();

    // 3. Create activity
    let activity = DbActivity {
        id: activity_id_clone2.clone(),
        actor_id: actor.id.clone(),
        activity_type: "Create".to_string(),
        object: json!({"type": "Note", "content": "New note from Alice"}),
        to_recipients: vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        cc_recipients: vec![],
        published: Utc::now(),
        created_at: Utc::now(),
    };
    db.create_activity(&activity).await.unwrap();

    // 4. Create follow relationship
    let follow = DbFollowRelation {
        id: "https://example.com/follows/2".to_string(),
        follower_id: follower_id_clone2.clone(),
        following_id: actor.id.clone(),
        status: "pending".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_follow(&follow).await.unwrap();

    // 5. Get notes and activities
    let notes = db.get_notes_by_actor(&actor.id, 10, 0).await.unwrap();
    assert_eq!(notes.len(), 1);

    let activities = db.get_activities_by_actor(&actor.id, 10, 0).await.unwrap();
    assert_eq!(activities.len(), 1);

    let followers = db.get_followers(&actor.id, 10, 0).await.unwrap();
    assert_eq!(followers.len(), 1);
    assert_eq!(followers[0].follower_id, follower_id);
}
