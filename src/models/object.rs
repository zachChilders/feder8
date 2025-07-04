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
    #[serde(rename = "totalItems")]
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
    #[serde(rename = "totalItems")]
    pub total_items: u32,
    pub first: String,
    pub last: String,
    #[serde(rename = "orderedItems")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_note_new() {
        let id = "https://example.com/notes/123".to_string();
        let attributed_to = "https://example.com/users/alice".to_string();
        let content = "Hello, world!".to_string();
        let to = vec!["https://www.w3.org/ns/activitystreams#Public".to_string()];
        let cc = vec!["https://example.com/users/alice/followers".to_string()];

        let note = Note::new(
            id.clone(),
            attributed_to.clone(),
            content.clone(),
            to.clone(),
            cc.clone(),
        );

        assert_eq!(note.context, vec!["https://www.w3.org/ns/activitystreams"]);
        assert_eq!(note.id, id);
        assert_eq!(note.note_type, "Note");
        assert_eq!(note.attributed_to, attributed_to);
        assert_eq!(note.content, content);
        assert_eq!(note.to, to);
        assert_eq!(note.cc, cc);
        assert_eq!(note.in_reply_to, None);
        assert!(note.tag.is_empty());
    }

    #[test]
    fn test_note_with_reply_and_tags() {
        let mut note = Note::new(
            "https://example.com/notes/456".to_string(),
            "https://example.com/users/bob".to_string(),
            "Reply with @alice and #test".to_string(),
            vec!["https://example.com/users/alice".to_string()],
            vec![],
        );

        note.in_reply_to = Some("https://example.com/notes/123".to_string());
        note.tag = vec![
            Tag {
                tag_type: "Mention".to_string(),
                name: "@alice".to_string(),
                href: Some("https://example.com/users/alice".to_string()),
            },
            Tag {
                tag_type: "Hashtag".to_string(),
                name: "#test".to_string(),
                href: Some("https://example.com/tags/test".to_string()),
            },
        ];

        assert_eq!(note.in_reply_to, Some("https://example.com/notes/123".to_string()));
        assert_eq!(note.tag.len(), 2);
        assert_eq!(note.tag[0].tag_type, "Mention");
        assert_eq!(note.tag[0].name, "@alice");
        assert_eq!(note.tag[1].tag_type, "Hashtag");
        assert_eq!(note.tag[1].name, "#test");
    }

    #[test]
    fn test_note_serialization() {
        let note = Note::new(
            "https://example.com/notes/test".to_string(),
            "https://example.com/users/test".to_string(),
            "Test content".to_string(),
            vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            vec![],
        );

        let json = serde_json::to_string(&note).unwrap();
        let deserialized: Note = serde_json::from_str(&json).unwrap();

        assert_eq!(note.id, deserialized.id);
        assert_eq!(note.attributed_to, deserialized.attributed_to);
        assert_eq!(note.content, deserialized.content);
        assert_eq!(note.note_type, deserialized.note_type);
    }

    #[test]
    fn test_tag_creation() {
        let mention_tag = Tag {
            tag_type: "Mention".to_string(),
            name: "@user".to_string(),
            href: Some("https://example.com/users/user".to_string()),
        };

        let hashtag = Tag {
            tag_type: "Hashtag".to_string(),
            name: "#topic".to_string(),
            href: Some("https://example.com/tags/topic".to_string()),
        };

        let emoji = Tag {
            tag_type: "Emoji".to_string(),
            name: ":heart:".to_string(),
            href: None,
        };

        assert_eq!(mention_tag.tag_type, "Mention");
        assert_eq!(mention_tag.name, "@user");
        assert!(mention_tag.href.is_some());

        assert_eq!(hashtag.tag_type, "Hashtag");
        assert_eq!(hashtag.name, "#topic");
        assert!(hashtag.href.is_some());

        assert_eq!(emoji.tag_type, "Emoji");
        assert_eq!(emoji.name, ":heart:");
        assert!(emoji.href.is_none());
    }

    #[test]
    fn test_collection_new() {
        let id = "https://example.com/collections/test".to_string();
        let total_items = 42;

        let collection = Collection::new(id.clone(), total_items);

        assert_eq!(collection.context, vec!["https://www.w3.org/ns/activitystreams"]);
        assert_eq!(collection.id, id);
        assert_eq!(collection.collection_type, "Collection");
        assert_eq!(collection.total_items, total_items);
        assert_eq!(collection.first, format!("{}?page=true", id));
        assert_eq!(collection.last, format!("{}?page=true", id));
    }

    #[test]
    fn test_collection_serialization() {
        let collection = Collection::new(
            "https://example.com/collections/test".to_string(),
            10,
        );

        let json = serde_json::to_string(&collection).unwrap();
        let deserialized: Collection = serde_json::from_str(&json).unwrap();

        assert_eq!(collection.id, deserialized.id);
        assert_eq!(collection.collection_type, deserialized.collection_type);
        assert_eq!(collection.total_items, deserialized.total_items);
        assert_eq!(collection.first, deserialized.first);
        assert_eq!(collection.last, deserialized.last);
    }

    #[test]
    fn test_ordered_collection_new() {
        let id = "https://example.com/outbox".to_string();
        let items = vec![
            json!({"type": "Create", "actor": "alice"}),
            json!({"type": "Follow", "actor": "bob"}),
        ];
        let total_items = items.len() as u32;

        let ordered_collection = OrderedCollection::new(id.clone(), total_items, items.clone());

        assert_eq!(ordered_collection.context, vec!["https://www.w3.org/ns/activitystreams"]);
        assert_eq!(ordered_collection.id, id);
        assert_eq!(ordered_collection.collection_type, "OrderedCollection");
        assert_eq!(ordered_collection.total_items, total_items);
        assert_eq!(ordered_collection.first, format!("{}?page=true", id));
        assert_eq!(ordered_collection.last, format!("{}?page=true", id));
        assert_eq!(ordered_collection.ordered_items, items);
    }

    #[test]
    fn test_ordered_collection_empty() {
        let id = "https://example.com/empty".to_string();
        let ordered_collection = OrderedCollection::new(id.clone(), 0, vec![]);

        assert_eq!(ordered_collection.total_items, 0);
        assert!(ordered_collection.ordered_items.is_empty());
    }

    #[test]
    fn test_ordered_collection_serialization() {
        let items = vec![
            json!({"type": "Note", "content": "Hello"}),
            json!({"type": "Note", "content": "World"}),
        ];
        let ordered_collection = OrderedCollection::new(
            "https://example.com/test".to_string(),
            2,
            items.clone(),
        );

        let json = serde_json::to_string(&ordered_collection).unwrap();
        let deserialized: OrderedCollection = serde_json::from_str(&json).unwrap();

        assert_eq!(ordered_collection.id, deserialized.id);
        assert_eq!(ordered_collection.collection_type, deserialized.collection_type);
        assert_eq!(ordered_collection.total_items, deserialized.total_items);
        assert_eq!(ordered_collection.ordered_items, deserialized.ordered_items);
    }

    #[test]
    fn test_note_clone() {
        let note = Note::new(
            "test".to_string(),
            "author".to_string(),
            "content".to_string(),
            vec![],
            vec![],
        );

        let cloned = note.clone();
        assert_eq!(note.id, cloned.id);
        assert_eq!(note.content, cloned.content);
        assert_eq!(note.attributed_to, cloned.attributed_to);
    }

    #[test]
    fn test_collection_clone() {
        let collection = Collection::new("test".to_string(), 5);
        let cloned = collection.clone();

        assert_eq!(collection.id, cloned.id);
        assert_eq!(collection.total_items, cloned.total_items);
    }

    #[test]
    fn test_ordered_collection_clone() {
        let items = vec![json!({"test": "value"})];
        let collection = OrderedCollection::new("test".to_string(), 1, items.clone());
        let cloned = collection.clone();

        assert_eq!(collection.id, cloned.id);
        assert_eq!(collection.ordered_items, cloned.ordered_items);
    }

    #[test]
    fn test_complex_note_content() {
        let note = Note::new(
            "https://example.com/notes/complex".to_string(),
            "https://example.com/users/author".to_string(),
            "<p>Complex <strong>HTML</strong> content with <a href=\"https://example.com\">links</a></p>".to_string(),
            vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
            vec!["https://example.com/users/author/followers".to_string()],
        );

        assert!(note.content.contains("<p>"));
        assert!(note.content.contains("<strong>"));
        assert!(note.content.contains("href="));
    }

    #[test]
    fn test_collection_urls() {
        let base_id = "https://example.com/users/alice/outbox";
        let collection = Collection::new(base_id.to_string(), 100);

        assert_eq!(collection.first, "https://example.com/users/alice/outbox?page=true");
        assert_eq!(collection.last, "https://example.com/users/alice/outbox?page=true");
    }
}
