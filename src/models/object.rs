use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub note_type: String,
    pub attributed_to: String,
    pub content: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub published: DateTime<Utc>,
    pub in_reply_to: Option<String>,
    pub tag: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "type")]
    pub tag_type: String,
    pub name: String,
    pub href: Option<String>,
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
    #[allow(dead_code)]
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
            note_type: "Note".to_string(),
            attributed_to,
            content,
            to,
            cc,
            published: Utc::now(),
            in_reply_to: None,
            tag: vec![],
        }
    }
}

impl Collection {
    #[allow(dead_code)]
    pub fn new(id: String, total_items: u32) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/activitystreams".to_string()],
            id: id.clone(),
            collection_type: "Collection".to_string(),
            total_items,
            first: format!("{id}?page=true"),
            last: format!("{id}?page=true"),
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
            first: format!("{id}?page=true"),
            last: format!("{id}?page=true"),
            ordered_items,
        }
    }
}
