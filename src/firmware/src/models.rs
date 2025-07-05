use chrono::{DateTime, Utc};
use heapless::{String as HeaplessString, Vec as HeaplessVec};
use serde::{Deserialize, Serialize};

/// ActivityPub Actor adapted for embedded systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    #[serde(rename = "@context")]
    pub context: HeaplessVec<HeaplessString<64>, 4>, // Limited context entries
    pub id: HeaplessString<128>,
    #[serde(rename = "type")]
    pub actor_type: HeaplessString<32>,
    pub name: HeaplessString<64>,
    #[serde(rename = "preferredUsername")]
    pub preferred_username: HeaplessString<64>,
    pub summary: Option<HeaplessString<256>>,
    pub url: HeaplessString<128>,
    pub inbox: HeaplessString<128>,
    pub outbox: HeaplessString<128>,
    pub followers: HeaplessString<128>,
    pub following: HeaplessString<128>,
    #[serde(rename = "publicKey")]
    pub public_key: PublicKey,
    pub published: DateTime<Utc>,
    pub icon: Option<Icon>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub id: HeaplessString<128>,
    #[serde(rename = "type")]
    pub key_type: HeaplessString<32>,
    pub owner: HeaplessString<128>,
    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: HeaplessString<512>, // RSA public key in PEM format
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Icon {
    #[serde(rename = "type")]
    pub icon_type: HeaplessString<32>,
    pub url: HeaplessString<128>,
    #[serde(rename = "mediaType")]
    pub media_type: HeaplessString<32>,
}

impl Actor {
    pub fn new(
        name: &str,
        username: &str,
        server_url: &str,
        public_key_pem: &str,
    ) -> Result<Self, &'static str> {
        if name.len() > 64 || username.len() > 64 || server_url.len() > 64 {
            return Err("String too long for embedded constraints");
        }
        
        if public_key_pem.len() > 512 {
            return Err("Public key too long for embedded constraints");
        }

        let actor_id = format!("{}/users/{}", server_url, username);
        if actor_id.len() > 128 {
            return Err("Actor ID too long for embedded constraints");
        }

        let mut context = HeaplessVec::new();
        context.push(HeaplessString::from("https://www.w3.org/ns/activitystreams")).map_err(|_| "Context too long")?;
        context.push(HeaplessString::from("https://w3id.org/security/v1")).map_err(|_| "Context too long")?;

        Ok(Self {
            context,
            id: HeaplessString::from(actor_id.as_str()),
            actor_type: HeaplessString::from("Person"),
            name: HeaplessString::from(name),
            preferred_username: HeaplessString::from(username),
            summary: None,
            url: HeaplessString::from(actor_id.as_str()),
            inbox: HeaplessString::from(format!("{}/inbox", actor_id).as_str()),
            outbox: HeaplessString::from(format!("{}/outbox", actor_id).as_str()),
            followers: HeaplessString::from(format!("{}/followers", actor_id).as_str()),
            following: HeaplessString::from(format!("{}/following", actor_id).as_str()),
            public_key: PublicKey {
                id: HeaplessString::from(format!("{}#main-key", actor_id).as_str()),
                key_type: HeaplessString::from("Key"),
                owner: HeaplessString::from(actor_id.as_str()),
                public_key_pem: HeaplessString::from(public_key_pem),
            },
            published: Utc::now(),
            icon: None,
        })
    }

    pub fn with_summary(mut self, summary: &str) -> Result<Self, &'static str> {
        if summary.len() > 256 {
            return Err("Summary too long for embedded constraints");
        }
        self.summary = Some(HeaplessString::from(summary));
        Ok(self)
    }

    pub fn with_icon(mut self, icon_url: &str, media_type: &str) -> Result<Self, &'static str> {
        if icon_url.len() > 128 || media_type.len() > 32 {
            return Err("Icon parameters too long for embedded constraints");
        }
        
        self.icon = Some(Icon {
            icon_type: HeaplessString::from("Image"),
            url: HeaplessString::from(icon_url),
            media_type: HeaplessString::from(media_type),
        });
        Ok(self)
    }
}

/// ActivityPub Activity adapted for embedded systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    #[serde(rename = "@context")]
    pub context: HeaplessVec<HeaplessString<64>, 4>,
    pub id: HeaplessString<128>,
    #[serde(rename = "type")]
    pub activity_type: HeaplessString<32>,
    pub actor: HeaplessString<128>,
    pub object: ActivityObject,
    pub published: DateTime<Utc>,
    pub to: Option<HeaplessVec<HeaplessString<128>, 8>>,
    pub cc: Option<HeaplessVec<HeaplessString<128>, 8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActivityObject {
    Note(Note),
    Follow(HeaplessString<128>), // Just the actor ID for Follow activities
    Accept(Box<Activity>),       // Nested activity for Accept/Reject
    Announce(HeaplessString<128>), // Object ID for Announce (boost)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    #[serde(rename = "type")]
    pub note_type: HeaplessString<32>,
    pub id: HeaplessString<128>,
    pub content: HeaplessString<512>, // Limited content size for embedded
    #[serde(rename = "attributedTo")]
    pub attributed_to: HeaplessString<128>,
    pub published: DateTime<Utc>,
    pub to: Option<HeaplessVec<HeaplessString<128>, 8>>,
    pub cc: Option<HeaplessVec<HeaplessString<128>, 8>>,
}

impl Activity {
    pub fn new_create_note(
        actor_id: &str,
        note_content: &str,
        server_url: &str,
    ) -> Result<Self, &'static str> {
        if actor_id.len() > 128 || note_content.len() > 512 || server_url.len() > 64 {
            return Err("Parameters too long for embedded constraints");
        }

        let activity_id = format!("{}/activities/{}", server_url, uuid::Uuid::new_v4());
        let note_id = format!("{}/notes/{}", server_url, uuid::Uuid::new_v4());

        let mut context = HeaplessVec::new();
        context.push(HeaplessString::from("https://www.w3.org/ns/activitystreams")).map_err(|_| "Context too long")?;

        let note = Note {
            note_type: HeaplessString::from("Note"),
            id: HeaplessString::from(note_id.as_str()),
            content: HeaplessString::from(note_content),
            attributed_to: HeaplessString::from(actor_id),
            published: Utc::now(),
            to: None,
            cc: None,
        };

        Ok(Self {
            context,
            id: HeaplessString::from(activity_id.as_str()),
            activity_type: HeaplessString::from("Create"),
            actor: HeaplessString::from(actor_id),
            object: ActivityObject::Note(note),
            published: Utc::now(),
            to: None,
            cc: None,
        })
    }

    pub fn new_follow(
        actor_id: &str,
        target_actor_id: &str,
        server_url: &str,
    ) -> Result<Self, &'static str> {
        if actor_id.len() > 128 || target_actor_id.len() > 128 || server_url.len() > 64 {
            return Err("Parameters too long for embedded constraints");
        }

        let activity_id = format!("{}/activities/{}", server_url, uuid::Uuid::new_v4());

        let mut context = HeaplessVec::new();
        context.push(HeaplessString::from("https://www.w3.org/ns/activitystreams")).map_err(|_| "Context too long")?;

        let mut to = HeaplessVec::new();
        to.push(HeaplessString::from(target_actor_id)).map_err(|_| "To field too long")?;

        Ok(Self {
            context,
            id: HeaplessString::from(activity_id.as_str()),
            activity_type: HeaplessString::from("Follow"),
            actor: HeaplessString::from(actor_id),
            object: ActivityObject::Follow(HeaplessString::from(target_actor_id)),
            published: Utc::now(),
            to: Some(to),
            cc: None,
        })
    }

    pub fn new_accept_follow(
        actor_id: &str,
        original_follow: Activity,
        server_url: &str,
    ) -> Result<Self, &'static str> {
        if actor_id.len() > 128 || server_url.len() > 64 {
            return Err("Parameters too long for embedded constraints");
        }

        let activity_id = format!("{}/activities/{}", server_url, uuid::Uuid::new_v4());

        let mut context = HeaplessVec::new();
        context.push(HeaplessString::from("https://www.w3.org/ns/activitystreams")).map_err(|_| "Context too long")?;

        let mut to = HeaplessVec::new();
        to.push(original_follow.actor.clone()).map_err(|_| "To field too long")?;

        Ok(Self {
            context,
            id: HeaplessString::from(activity_id.as_str()),
            activity_type: HeaplessString::from("Accept"),
            actor: HeaplessString::from(actor_id),
            object: ActivityObject::Accept(Box::new(original_follow)),
            published: Utc::now(),
            to: Some(to),
            cc: None,
        })
    }
}

/// Configuration for the embedded ActivityPub node
#[derive(Debug, Clone)]
pub struct EmbeddedConfig {
    pub server_name: HeaplessString<64>,
    pub server_url: HeaplessString<128>,
    pub actor_name: HeaplessString<64>,
    pub wifi_ssid: HeaplessString<64>,
    pub wifi_password: HeaplessString<64>,
    pub private_key_pem: Option<HeaplessString<1024>>,
    pub public_key_pem: Option<HeaplessString<512>>,
}

impl EmbeddedConfig {
    pub fn new(
        server_name: &str,
        server_url: &str,
        actor_name: &str,
        wifi_ssid: &str,
        wifi_password: &str,
    ) -> Result<Self, &'static str> {
        if server_name.len() > 64 || server_url.len() > 128 || actor_name.len() > 64 {
            return Err("Server configuration parameters too long");
        }
        
        if wifi_ssid.len() > 64 || wifi_password.len() > 64 {
            return Err("WiFi configuration parameters too long");
        }

        Ok(Self {
            server_name: HeaplessString::from(server_name),
            server_url: HeaplessString::from(server_url),
            actor_name: HeaplessString::from(actor_name),
            wifi_ssid: HeaplessString::from(wifi_ssid),
            wifi_password: HeaplessString::from(wifi_password),
            private_key_pem: None,
            public_key_pem: None,
        })
    }

    pub fn with_keys(mut self, private_key: &str, public_key: &str) -> Result<Self, &'static str> {
        if private_key.len() > 1024 || public_key.len() > 512 {
            return Err("Keys too long for embedded constraints");
        }

        self.private_key_pem = Some(HeaplessString::from(private_key));
        self.public_key_pem = Some(HeaplessString::from(public_key));
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_creation() {
        let actor = Actor::new(
            "Test User",
            "testuser",
            "https://esp32.local",
            "-----BEGIN PUBLIC KEY-----\ntest\n-----END PUBLIC KEY-----",
        ).unwrap();

        assert_eq!(actor.name.as_str(), "Test User");
        assert_eq!(actor.preferred_username.as_str(), "testuser");
        assert_eq!(actor.actor_type.as_str(), "Person");
        assert_eq!(actor.id.as_str(), "https://esp32.local/users/testuser");
    }

    #[test]
    fn test_activity_creation() {
        let activity = Activity::new_create_note(
            "https://esp32.local/users/testuser",
            "Hello from ESP32!",
            "https://esp32.local",
        ).unwrap();

        assert_eq!(activity.activity_type.as_str(), "Create");
        assert_eq!(activity.actor.as_str(), "https://esp32.local/users/testuser");
        
        if let ActivityObject::Note(note) = &activity.object {
            assert_eq!(note.content.as_str(), "Hello from ESP32!");
        } else {
            panic!("Expected Note object");
        }
    }

    #[test]
    fn test_config_creation() {
        let config = EmbeddedConfig::new(
            "ESP32 Node",
            "https://esp32.local",
            "esp32user",
            "MyWiFi",
            "password123",
        ).unwrap();

        assert_eq!(config.server_name.as_str(), "ESP32 Node");
        assert_eq!(config.server_url.as_str(), "https://esp32.local");
        assert_eq!(config.actor_name.as_str(), "esp32user");
        assert_eq!(config.wifi_ssid.as_str(), "MyWiFi");
    }

    #[test]
    fn test_string_length_validation() {
        let long_name = "a".repeat(100);
        let result = Actor::new(
            &long_name,
            "testuser",
            "https://esp32.local",
            "test-key",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_follow_activity() {
        let activity = Activity::new_follow(
            "https://esp32.local/users/esp32user",
            "https://mastodon.social/users/alice",
            "https://esp32.local",
        ).unwrap();

        assert_eq!(activity.activity_type.as_str(), "Follow");
        
        if let ActivityObject::Follow(target) = &activity.object {
            assert_eq!(target.as_str(), "https://mastodon.social/users/alice");
        } else {
            panic!("Expected Follow object");
        }
    }
}