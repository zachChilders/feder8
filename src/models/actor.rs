use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Icon {
    #[serde(rename = "type")]
    pub icon_type: String,
    pub url: String,
    pub media_type: String,
}

impl Actor {
    pub fn new(
        id: String,
        name: String,
        username: String,
        server_url: &str,
        public_key_pem: String,
    ) -> Self {
        let actor_id = format!("{}/users/{}", server_url, username);
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
            inbox: format!("{}/inbox", actor_id),
            outbox: format!("{}/outbox", actor_id),
            followers: format!("{}/followers", actor_id),
            following: format!("{}/following", actor_id),
            public_key: PublicKey {
                id: format!("{}#main-key", actor_id),
                key_type: "Key".to_string(),
                owner: actor_id,
                public_key_pem,
            },
            published: Utc::now(),
            icon: None,
        }
    }
} 