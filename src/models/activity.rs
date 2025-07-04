use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    pub object: serde_json::Value,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Create {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    pub object: serde_json::Value,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Follow {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    pub object: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accept {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    pub object: serde_json::Value,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub published: DateTime<Utc>,
}

impl Activity {
    #[allow(dead_code)]
    pub fn new(
        activity_type: String,
        actor: String,
        object: serde_json::Value,
        to: Vec<String>,
        cc: Vec<String>,
    ) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/activitystreams".to_string()],
            id: format!("https://example.com/activities/{}", Uuid::new_v4()),
            activity_type,
            actor,
            object,
            to,
            cc,
            published: Utc::now(),
        }
    }
}

impl Create {
    #[allow(dead_code)]
    pub fn new(actor: String, object: serde_json::Value, to: Vec<String>, cc: Vec<String>) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/activitystreams".to_string()],
            id: format!("https://example.com/activities/{}", Uuid::new_v4()),
            activity_type: "Create".to_string(),
            actor,
            object,
            to,
            cc,
            published: Utc::now(),
        }
    }
}

impl Follow {
    #[allow(dead_code)]
    pub fn new(actor: String, object: String, to: Vec<String>, cc: Vec<String>) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/activitystreams".to_string()],
            id: format!("https://example.com/activities/{}", Uuid::new_v4()),
            activity_type: "Follow".to_string(),
            actor,
            object,
            to,
            cc,
            published: Utc::now(),
        }
    }
}

impl Accept {
    #[allow(dead_code)]
    pub fn new(actor: String, object: serde_json::Value, to: Vec<String>, cc: Vec<String>) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/activitystreams".to_string()],
            id: format!("https://example.com/activities/{}", Uuid::new_v4()),
            activity_type: "Accept".to_string(),
            actor,
            object,
            to,
            cc,
            published: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_activity_new() {
        let activity_type = "CustomActivity".to_string();
        let actor = "https://example.com/users/alice".to_string();
        let object = json!({"type": "Note", "content": "Hello"});
        let to = vec!["https://example.com/users/bob".to_string()];
        let cc = vec!["https://www.w3.org/ns/activitystreams#Public".to_string()];

        let activity = Activity::new(
            activity_type.clone(),
            actor.clone(),
            object.clone(),
            to.clone(),
            cc.clone(),
        );

        assert_eq!(
            activity.context,
            vec!["https://www.w3.org/ns/activitystreams"]
        );
        assert!(activity.id.starts_with("https://example.com/activities/"));
        assert_eq!(activity.activity_type, activity_type);
        assert_eq!(activity.actor, actor);
        assert_eq!(activity.object, object);
        assert_eq!(activity.to, to);
        assert_eq!(activity.cc, cc);
    }

    #[test]
    fn test_create_activity_new() {
        let actor = "https://example.com/users/alice".to_string();
        let object = json!({
            "type": "Note",
            "content": "This is a test note",
            "attributedTo": "https://example.com/users/alice"
        });
        let to = vec!["https://example.com/users/bob".to_string()];
        let cc = vec!["https://www.w3.org/ns/activitystreams#Public".to_string()];

        let create = Create::new(actor.clone(), object.clone(), to.clone(), cc.clone());

        assert_eq!(
            create.context,
            vec!["https://www.w3.org/ns/activitystreams"]
        );
        assert!(create.id.starts_with("https://example.com/activities/"));
        assert_eq!(create.activity_type, "Create");
        assert_eq!(create.actor, actor);
        assert_eq!(create.object, object);
        assert_eq!(create.to, to);
        assert_eq!(create.cc, cc);
    }

    #[test]
    fn test_follow_activity_new() {
        let actor = "https://example.com/users/alice".to_string();
        let object = "https://example.com/users/bob".to_string();
        let to = vec![object.clone()];
        let cc = vec![];

        let follow = Follow::new(actor.clone(), object.clone(), to.clone(), cc.clone());

        assert_eq!(
            follow.context,
            vec!["https://www.w3.org/ns/activitystreams"]
        );
        assert!(follow.id.starts_with("https://example.com/activities/"));
        assert_eq!(follow.activity_type, "Follow");
        assert_eq!(follow.actor, actor);
        assert_eq!(follow.object, object);
        assert_eq!(follow.to, to);
        assert_eq!(follow.cc, cc);
    }

    #[test]
    fn test_accept_activity_new() {
        let actor = "https://example.com/users/bob".to_string();
        let follow_object = json!({
            "type": "Follow",
            "actor": "https://example.com/users/alice",
            "object": "https://example.com/users/bob"
        });
        let to = vec!["https://example.com/users/alice".to_string()];
        let cc = vec![];

        let accept = Accept::new(actor.clone(), follow_object.clone(), to.clone(), cc.clone());

        assert_eq!(
            accept.context,
            vec!["https://www.w3.org/ns/activitystreams"]
        );
        assert!(accept.id.starts_with("https://example.com/activities/"));
        assert_eq!(accept.activity_type, "Accept");
        assert_eq!(accept.actor, actor);
        assert_eq!(accept.object, follow_object);
        assert_eq!(accept.to, to);
        assert_eq!(accept.cc, cc);
    }

    #[test]
    fn test_activity_serialization() {
        let activity = Activity::new(
            "TestType".to_string(),
            "https://example.com/users/test".to_string(),
            json!({"test": "value"}),
            vec!["https://example.com/users/target".to_string()],
            vec![],
        );

        let json = serde_json::to_string(&activity).unwrap();
        let deserialized: Activity = serde_json::from_str(&json).unwrap();

        assert_eq!(activity.activity_type, deserialized.activity_type);
        assert_eq!(activity.actor, deserialized.actor);
        assert_eq!(activity.object, deserialized.object);
        assert_eq!(activity.to, deserialized.to);
        assert_eq!(activity.cc, deserialized.cc);
    }

    #[test]
    fn test_create_serialization() {
        let create = Create::new(
            "https://example.com/users/alice".to_string(),
            json!({"type": "Note", "content": "Hello"}),
            vec!["https://example.com/users/bob".to_string()],
            vec![],
        );

        let json = serde_json::to_string(&create).unwrap();
        let deserialized: Create = serde_json::from_str(&json).unwrap();

        assert_eq!(create.activity_type, deserialized.activity_type);
        assert_eq!(create.actor, deserialized.actor);
        assert_eq!(create.object, deserialized.object);
    }

    #[test]
    fn test_follow_serialization() {
        let follow = Follow::new(
            "https://example.com/users/alice".to_string(),
            "https://example.com/users/bob".to_string(),
            vec!["https://example.com/users/bob".to_string()],
            vec![],
        );

        let json = serde_json::to_string(&follow).unwrap();
        let deserialized: Follow = serde_json::from_str(&json).unwrap();

        assert_eq!(follow.activity_type, deserialized.activity_type);
        assert_eq!(follow.actor, deserialized.actor);
        assert_eq!(follow.object, deserialized.object);
    }

    #[test]
    fn test_accept_serialization() {
        let accept = Accept::new(
            "https://example.com/users/bob".to_string(),
            json!({"type": "Follow", "actor": "alice"}),
            vec!["https://example.com/users/alice".to_string()],
            vec![],
        );

        let json = serde_json::to_string(&accept).unwrap();
        let deserialized: Accept = serde_json::from_str(&json).unwrap();

        assert_eq!(accept.activity_type, deserialized.activity_type);
        assert_eq!(accept.actor, deserialized.actor);
        assert_eq!(accept.object, deserialized.object);
    }

    #[test]
    fn test_activity_clone() {
        let activity = Activity::new(
            "Test".to_string(),
            "actor".to_string(),
            json!({}),
            vec![],
            vec![],
        );

        let cloned = activity.clone();
        assert_eq!(activity.id, cloned.id);
        assert_eq!(activity.activity_type, cloned.activity_type);
        assert_eq!(activity.actor, cloned.actor);
    }

    #[test]
    fn test_unique_ids_generated() {
        let activity1 = Activity::new(
            "Test".to_string(),
            "actor".to_string(),
            json!({}),
            vec![],
            vec![],
        );
        let activity2 = Activity::new(
            "Test".to_string(),
            "actor".to_string(),
            json!({}),
            vec![],
            vec![],
        );

        assert_ne!(activity1.id, activity2.id);
    }

    #[test]
    fn test_complex_object_handling() {
        let complex_object = json!({
            "type": "Note",
            "id": "https://example.com/notes/123",
            "content": "Complex note with <strong>HTML</strong>",
            "tag": [
                {"type": "Mention", "href": "https://example.com/users/alice", "name": "@alice"},
                {"type": "Hashtag", "href": "https://example.com/tags/test", "name": "#test"}
            ],
            "attachment": [
                {"type": "Image", "url": "https://example.com/image.jpg", "mediaType": "image/jpeg"}
            ]
        });

        let create = Create::new(
            "https://example.com/users/author".to_string(),
            complex_object.clone(),
            vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            vec!["https://example.com/users/author/followers".to_string()],
        );

        assert_eq!(create.object, complex_object);
        assert_eq!(
            create.to,
            vec!["https://www.w3.org/ns/activitystreams#Public"]
        );
        assert_eq!(
            create.cc,
            vec!["https://example.com/users/author/followers"]
        );
    }
}
