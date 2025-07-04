use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
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