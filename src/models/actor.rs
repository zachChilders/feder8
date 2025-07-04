use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub actor_type: String,
    pub name: String,
    pub preferred_username: String,
    pub summary: Option<String>,
    pub url: String,
    pub inbox: String,
    pub outbox: String,
    pub followers: String,
    pub following: String,
    pub public_key: PublicKey,
    pub published: DateTime<Utc>,
    pub icon: Option<Icon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub id: String,
    #[serde(rename = "type")]
    pub key_type: String,
    pub owner: String,
    pub public_key_pem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Icon {
    #[serde(rename = "type")]
    pub icon_type: String,
    pub url: String,
    pub media_type: String,
}

impl Actor {
    pub fn new(
        _id: String,
        name: String,
        username: String,
        server_url: &str,
        public_key_pem: String,
    ) -> Self {
        let actor_id = format!("{server_url}/users/{username}");
        Self {
            context: vec![
                "https://www.w3.org/ns/activitystreams".to_string(),
                "https://w3id.org/security/v1".to_string(),
            ],
            id: actor_id.clone(),
            actor_type: "Person".to_string(),
            name,
            preferred_username: username,
            summary: None,
            url: actor_id.clone(),
            inbox: format!("{actor_id}/inbox"),
            outbox: format!("{actor_id}/outbox"),
            followers: format!("{actor_id}/followers"),
            following: format!("{actor_id}/following"),
            public_key: PublicKey {
                id: format!("{actor_id}#main-key"),
                key_type: "Key".to_string(),
                owner: actor_id,
                public_key_pem,
            },
            published: Utc::now(),
            icon: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_new() {
        let name = "Test User".to_string();
        let username = "testuser".to_string();
        let server_url = "https://example.com";
        let public_key_pem =
            "-----BEGIN PUBLIC KEY-----\ntest\n-----END PUBLIC KEY-----".to_string();

        let actor = Actor::new(
            "unused_id".to_string(),
            name.clone(),
            username.clone(),
            server_url,
            public_key_pem.clone(),
        );

        let expected_id = "https://example.com/users/testuser";

        assert_eq!(
            actor.context,
            vec![
                "https://www.w3.org/ns/activitystreams".to_string(),
                "https://w3id.org/security/v1".to_string()
            ]
        );
        assert_eq!(actor.id, expected_id);
        assert_eq!(actor.actor_type, "Person");
        assert_eq!(actor.name, name);
        assert_eq!(actor.preferred_username, username);
        assert_eq!(actor.summary, None);
        assert_eq!(actor.url, expected_id);
        assert_eq!(actor.inbox, format!("{expected_id}/inbox"));
        assert_eq!(actor.outbox, format!("{expected_id}/outbox"));
        assert_eq!(actor.followers, format!("{expected_id}/followers"));
        assert_eq!(actor.following, format!("{expected_id}/following"));
        assert_eq!(actor.icon, None);

        // Test public key
        assert_eq!(actor.public_key.id, format!("{expected_id}#main-key"));
        assert_eq!(actor.public_key.key_type, "Key");
        assert_eq!(actor.public_key.owner, expected_id);
        assert_eq!(actor.public_key.public_key_pem, public_key_pem);
    }

    #[test]
    fn test_actor_serialization() {
        let actor = Actor::new(
            "test_id".to_string(),
            "Test User".to_string(),
            "testuser".to_string(),
            "https://example.com",
            "test_key".to_string(),
        );

        let json = serde_json::to_string(&actor).unwrap();
        let deserialized: Actor = serde_json::from_str(&json).unwrap();

        assert_eq!(actor.id, deserialized.id);
        assert_eq!(actor.name, deserialized.name);
        assert_eq!(actor.preferred_username, deserialized.preferred_username);
        assert_eq!(actor.actor_type, deserialized.actor_type);
        assert_eq!(
            actor.public_key.public_key_pem,
            deserialized.public_key.public_key_pem
        );
    }

    #[test]
    fn test_actor_with_icon() {
        let mut actor = Actor::new(
            "test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            "https://example.com",
            "key".to_string(),
        );

        actor.icon = Some(Icon {
            icon_type: "Image".to_string(),
            url: "https://example.com/avatar.png".to_string(),
            media_type: "image/png".to_string(),
        });

        assert!(actor.icon.is_some());
        let icon = actor.icon.unwrap();
        assert_eq!(icon.icon_type, "Image");
        assert_eq!(icon.url, "https://example.com/avatar.png");
        assert_eq!(icon.media_type, "image/png");
    }

    #[test]
    fn test_actor_with_summary() {
        let mut actor = Actor::new(
            "test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            "https://example.com",
            "key".to_string(),
        );

        actor.summary = Some("This is a test actor".to_string());
        assert_eq!(actor.summary, Some("This is a test actor".to_string()));
    }

    #[test]
    fn test_public_key_creation() {
        let public_key = PublicKey {
            id: "https://example.com/users/test#main-key".to_string(),
            key_type: "Key".to_string(),
            owner: "https://example.com/users/test".to_string(),
            public_key_pem: "-----BEGIN PUBLIC KEY-----\ntest\n-----END PUBLIC KEY-----"
                .to_string(),
        };

        assert_eq!(public_key.id, "https://example.com/users/test#main-key");
        assert_eq!(public_key.key_type, "Key");
        assert_eq!(public_key.owner, "https://example.com/users/test");
        assert!(public_key.public_key_pem.contains("BEGIN PUBLIC KEY"));
    }

    #[test]
    fn test_icon_creation() {
        let icon = Icon {
            icon_type: "Image".to_string(),
            url: "https://example.com/image.jpg".to_string(),
            media_type: "image/jpeg".to_string(),
        };

        assert_eq!(icon.icon_type, "Image");
        assert_eq!(icon.url, "https://example.com/image.jpg");
        assert_eq!(icon.media_type, "image/jpeg");
    }

    #[test]
    fn test_actor_clone() {
        let actor = Actor::new(
            "test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            "https://example.com",
            "key".to_string(),
        );

        let cloned = actor.clone();
        assert_eq!(actor.id, cloned.id);
        assert_eq!(actor.name, cloned.name);
        assert_eq!(actor.preferred_username, cloned.preferred_username);
    }

    #[test]
    fn test_actor_urls_generation() {
        let actor = Actor::new(
            "test".to_string(),
            "Alice".to_string(),
            "alice".to_string(),
            "https://mastodon.social",
            "key".to_string(),
        );

        let base_url = "https://mastodon.social/users/alice";
        assert_eq!(actor.id, base_url);
        assert_eq!(actor.url, base_url);
        assert_eq!(actor.inbox, format!("{base_url}/inbox"));
        assert_eq!(actor.outbox, format!("{base_url}/outbox"));
        assert_eq!(actor.followers, format!("{base_url}/followers"));
        assert_eq!(actor.following, format!("{base_url}/following"));
        assert_eq!(actor.public_key.id, format!("{base_url}#main-key"));
        assert_eq!(actor.public_key.owner, base_url);
    }
}
