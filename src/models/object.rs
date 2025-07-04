use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub attributed_to: String,
    pub content: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub published: DateTime<Utc>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub collection_type: String,
    pub total_items: u32,
    pub first: String,
    pub last: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderedCollection {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub collection_type: String,
    pub total_items: u32,
    pub first: String,
    pub last: String,
    pub ordered_items: Vec<serde_json::Value>,
}

impl Note {
    pub fn new(
        id: String,
        attributed_to: String,
        content: String,
        to: Vec<String>,
        cc: Vec<String>,
    ) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/activitystreams".to_string()],
            id,
            object_type: "Note".to_string(),
            attributed_to,
            content,
            to,
            cc,
            published: Utc::now(),
            url: None,
        }
    }
}

impl Collection {
    pub fn new(id: String, total_items: u32) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/activitystreams".to_string()],
            id: id.clone(),
            collection_type: "Collection".to_string(),
            total_items,
            first: format!("{}?page=true", id),
            last: format!("{}?page=true", id),
        }
    }
}

impl OrderedCollection {
    pub fn new(id: String, total_items: u32, ordered_items: Vec<serde_json::Value>) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/activitystreams".to_string()],
            id: id.clone(),
            collection_type: "OrderedCollection".to_string(),
            total_items,
            first: format!("{}?page=true", id),
            last: format!("{}?page=true", id),
            ordered_items,
        }
    }
} 