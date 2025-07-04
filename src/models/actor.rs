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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
