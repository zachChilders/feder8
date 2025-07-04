#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Trait for all actor-related objects
pub trait ActorObject {
    fn context() -> Vec<String> {
        vec![
            "https://www.w3.org/ns/activitystreams".to_string(),
            "https://w3id.org/security/v1".to_string(),
        ]
    }

    fn timestamp() -> DateTime<Utc> {
        Utc::now()
    }
}

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

// Actor builder using functional patterns
pub struct ActorBuilder {
    name: String,
    username: String,
    server_url: String,
    public_key_pem: String,
    summary: Option<String>,
    icon: Option<Icon>,
    actor_type: String,
}

impl ActorBuilder {
    pub fn new(
        name: impl Into<String>,
        username: impl Into<String>,
        server_url: impl Into<String>,
        public_key_pem: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            username: username.into(),
            server_url: server_url.into(),
            public_key_pem: public_key_pem.into(),
            summary: None,
            icon: None,
            actor_type: "Person".to_string(),
        }
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_type(mut self, actor_type: impl Into<String>) -> Self {
        self.actor_type = actor_type.into();
        self
    }

    pub fn build(self) -> Actor {
        let actor_id = format!("{}/users/{}", self.server_url, self.username);

        Actor {
            context: Actor::context(),
            id: actor_id.clone(),
            actor_type: self.actor_type,
            name: self.name,
            preferred_username: self.username,
            summary: self.summary,
            url: actor_id.clone(),
            inbox: format!("{actor_id}/inbox"),
            outbox: format!("{actor_id}/outbox"),
            followers: format!("{actor_id}/followers"),
            following: format!("{actor_id}/following"),
            public_key: PublicKey {
                id: format!("{actor_id}#main-key"),
                key_type: "Key".to_string(),
                owner: actor_id,
                public_key_pem: self.public_key_pem,
            },
            published: Actor::timestamp(),
            icon: self.icon,
        }
    }
}

impl ActorObject for Actor {}

impl Actor {
    pub fn new(
        _id: String,
        name: String,
        username: String,
        server_url: &str,
        public_key_pem: String,
    ) -> Self {
        ActorBuilder::new(name, username, server_url.to_string(), public_key_pem).build()
    }

    // Functional helper methods
    pub fn with_summary(self, summary: impl Into<String>) -> Self {
        Self {
            summary: Some(summary.into()),
            ..self
        }
    }

    pub fn with_icon(self, icon: Icon) -> Self {
        Self {
            icon: Some(icon),
            ..self
        }
    }

    // URL generators using functional patterns
    pub fn generate_urls(&self) -> ActorUrls {
        ActorUrls {
            actor: self.id.clone(),
            inbox: format!("{}/inbox", self.id),
            outbox: format!("{}/outbox", self.id),
            followers: format!("{}/followers", self.id),
            following: format!("{}/following", self.id),
            public_key: format!("{}#main-key", self.id),
        }
    }
}

// Functional URL structure
#[derive(Debug, Clone, PartialEq)]
pub struct ActorUrls {
    pub actor: String,
    pub inbox: String,
    pub outbox: String,
    pub followers: String,
    pub following: String,
    pub public_key: String,
}

// Icon constructors using functional patterns
impl Icon {
    pub fn image(url: impl Into<String>, media_type: impl Into<String>) -> Self {
        Self {
            icon_type: "Image".to_string(),
            url: url.into(),
            media_type: media_type.into(),
        }
    }

    pub fn png(url: impl Into<String>) -> Self {
        Self::image(url, "image/png")
    }

    pub fn jpeg(url: impl Into<String>) -> Self {
        Self::image(url, "image/jpeg")
    }

    pub fn webp(url: impl Into<String>) -> Self {
        Self::image(url, "image/webp")
    }
}

// PublicKey constructors
impl PublicKey {
    pub fn new(owner: impl Into<String>, public_key_pem: impl Into<String>) -> Self {
        let owner_str = owner.into();
        Self {
            id: format!("{owner_str}#main-key"),
            key_type: "Key".to_string(),
            owner: owner_str,
            public_key_pem: public_key_pem.into(),
        }
    }

    pub fn with_custom_id(
        id: impl Into<String>,
        owner: impl Into<String>,
        public_key_pem: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            key_type: "Key".to_string(),
            owner: owner.into(),
            public_key_pem: public_key_pem.into(),
        }
    }
}

// Utility functions for common actor operations
pub fn create_person_actor(
    name: impl Into<String>,
    username: impl Into<String>,
    server_url: impl Into<String>,
    public_key_pem: impl Into<String>,
) -> Actor {
    ActorBuilder::new(name, username, server_url, public_key_pem).build()
}

pub fn create_service_actor(
    name: impl Into<String>,
    username: impl Into<String>,
    server_url: impl Into<String>,
    public_key_pem: impl Into<String>,
) -> Actor {
    ActorBuilder::new(name, username, server_url, public_key_pem)
        .with_type("Service")
        .build()
}

pub fn create_bot_actor(
    name: impl Into<String>,
    username: impl Into<String>,
    server_url: impl Into<String>,
    public_key_pem: impl Into<String>,
) -> Actor {
    ActorBuilder::new(name, username, server_url, public_key_pem)
        .with_type("Bot")
        .build()
}

// Pattern matching for actor types
pub fn match_actor_type(actor: &Actor) -> ActorTypeResult {
    match actor.actor_type.as_str() {
        "Person" => ActorTypeResult::Person,
        "Service" => ActorTypeResult::Service,
        "Bot" => ActorTypeResult::Bot,
        "Application" => ActorTypeResult::Application,
        "Group" => ActorTypeResult::Group,
        "Organization" => ActorTypeResult::Organization,
        _ => ActorTypeResult::Unknown(actor.actor_type.clone()),
    }
}

#[derive(Debug, PartialEq)]
pub enum ActorTypeResult {
    Person,
    Service,
    Bot,
    Application,
    Group,
    Organization,
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_actor_data() -> (String, String, String, String) {
        (
            "Test User".to_string(),
            "testuser".to_string(),
            "https://example.com".to_string(),
            "-----BEGIN PUBLIC KEY-----\ntest\n-----END PUBLIC KEY-----".to_string(),
        )
    }

    #[test]
    fn test_actor_builder_pattern() {
        let (name, username, server_url, key) = test_actor_data();

        let actor = ActorBuilder::new(
            name.clone(),
            username.clone(),
            server_url.clone(),
            key.clone(),
        )
        .with_summary("Test summary")
        .with_icon(Icon::png("https://example.com/avatar.png"))
        .build();

        assert_eq!(actor.name, name);
        assert_eq!(actor.preferred_username, username);
        assert_eq!(actor.summary, Some("Test summary".to_string()));
        assert!(actor.icon.is_some());
    }

    #[test]
    fn test_functional_constructors() {
        let (name, username, server_url, key) = test_actor_data();

        let person = create_person_actor(
            name.clone(),
            username.clone(),
            server_url.clone(),
            key.clone(),
        );
        let service = create_service_actor(
            name.clone(),
            username.clone(),
            server_url.clone(),
            key.clone(),
        );
        let bot = create_bot_actor(name, username, server_url, key);

        assert_eq!(person.actor_type, "Person");
        assert_eq!(service.actor_type, "Service");
        assert_eq!(bot.actor_type, "Bot");
    }

    #[test]
    fn test_icon_constructors() {
        let png_icon = Icon::png("https://example.com/avatar.png");
        let jpeg_icon = Icon::jpeg("https://example.com/avatar.jpg");
        let custom_icon = Icon::image("https://example.com/avatar.webp", "image/webp");

        assert_eq!(png_icon.media_type, "image/png");
        assert_eq!(jpeg_icon.media_type, "image/jpeg");
        assert_eq!(custom_icon.media_type, "image/webp");
    }

    #[test]
    fn test_url_generation() {
        let (name, username, server_url, key) = test_actor_data();
        let actor = create_person_actor(name, username, server_url, key);
        let urls = actor.generate_urls();

        assert_eq!(urls.actor, "https://example.com/users/testuser");
        assert_eq!(urls.inbox, "https://example.com/users/testuser/inbox");
        assert_eq!(urls.outbox, "https://example.com/users/testuser/outbox");
        assert_eq!(
            urls.public_key,
            "https://example.com/users/testuser#main-key"
        );
    }

    #[test]
    fn test_actor_type_matching() {
        let (name, username, server_url, key) = test_actor_data();

        let person = create_person_actor(
            name.clone(),
            username.clone(),
            server_url.clone(),
            key.clone(),
        );
        let service = create_service_actor(name, username, server_url, key);

        assert_eq!(match_actor_type(&person), ActorTypeResult::Person);
        assert_eq!(match_actor_type(&service), ActorTypeResult::Service);
    }

    #[test]
    fn test_public_key_constructors() {
        let key1 = PublicKey::new("https://example.com/users/alice", "test-key");
        let key2 =
            PublicKey::with_custom_id("custom-id", "https://example.com/users/bob", "test-key");

        assert_eq!(key1.id, "https://example.com/users/alice#main-key");
        assert_eq!(key1.owner, "https://example.com/users/alice");
        assert_eq!(key2.id, "custom-id");
    }

    #[test]
    fn test_actor_functional_methods() {
        let (name, username, server_url, key) = test_actor_data();

        let actor = create_person_actor(name, username, server_url, key)
            .with_summary("Updated summary")
            .with_icon(Icon::jpeg("https://example.com/new-avatar.jpg"));

        assert_eq!(actor.summary, Some("Updated summary".to_string()));
        assert!(actor.icon.is_some());
        assert_eq!(actor.icon.unwrap().media_type, "image/jpeg");
    }

    #[test]
    fn test_serialization_compatibility() {
        let (name, username, server_url, key) = test_actor_data();
        let actor = create_person_actor(name, username, server_url, key);

        let json = serde_json::to_string(&actor).unwrap();
        let deserialized: Actor = serde_json::from_str(&json).unwrap();

        assert_eq!(actor.id, deserialized.id);
        assert_eq!(actor.name, deserialized.name);
        assert_eq!(actor.preferred_username, deserialized.preferred_username);
    }
}
