use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Trait for ActivityPub objects
pub trait ActivityPubObject {
    fn context() -> Vec<String> {
        vec!["https://www.w3.org/ns/activitystreams".to_string()]
    }

    fn timestamp() -> DateTime<Utc> {
        Utc::now()
    }
}

// Generic base for ActivityPub objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectBase<T> {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    #[serde(flatten)]
    pub content: T,
    pub published: DateTime<Utc>,
}

// Content structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteContent {
    pub attributed_to: String,
    pub content: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub in_reply_to: Option<String>,
    pub tag: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionContent {
    #[serde(rename = "totalItems")]
    pub total_items: u32,
    pub first: String,
    pub last: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderedCollectionContent {
    #[serde(rename = "totalItems")]
    pub total_items: u32,
    pub first: String,
    pub last: String,
    #[serde(rename = "orderedItems")]
    pub ordered_items: Vec<serde_json::Value>,
}

// Type aliases
pub type Note = ObjectBase<NoteContent>;
pub type Collection = ObjectBase<CollectionContent>;
pub type OrderedCollection = ObjectBase<OrderedCollectionContent>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "type")]
    pub tag_type: String,
    pub name: String,
    pub href: Option<String>,
}

// Builder pattern for objects
pub struct ObjectBuilder<T> {
    id: String,
    object_type: String,
    content: T,
}

impl<T> ObjectBuilder<T> {
    pub fn new(id: impl Into<String>, object_type: impl Into<String>, content: T) -> Self {
        Self {
            id: id.into(),
            object_type: object_type.into(),
            content,
        }
    }

    pub fn build(self) -> ObjectBase<T> {
        ObjectBase {
            context: ObjectBase::<T>::context(),
            id: self.id,
            object_type: self.object_type,
            content: self.content,
            published: ObjectBase::<T>::timestamp(),
        }
    }
}

impl<T> ActivityPubObject for ObjectBase<T> {}

// Functional constructors
impl Note {
    pub fn new(
        id: impl Into<String>,
        attributed_to: impl Into<String>,
        content: impl Into<String>,
        to: Vec<String>,
        cc: Vec<String>,
    ) -> Self {
        let note_content = NoteContent {
            attributed_to: attributed_to.into(),
            content: content.into(),
            to,
            cc,
            in_reply_to: None,
            tag: vec![],
        };
        ObjectBuilder::new(id, "Note", note_content).build()
    }

    pub fn with_reply(mut self, reply_to: impl Into<String>) -> Self {
        self.content.in_reply_to = Some(reply_to.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<Tag>) -> Self {
        self.content.tag = tags;
        self
    }

    pub fn add_tag(mut self, tag: Tag) -> Self {
        self.content.tag.push(tag);
        self
    }
}

impl Collection {
    pub fn new(id: impl Into<String>, total_items: u32) -> Self {
        let id_str = id.into();
        let content = CollectionContent {
            total_items,
            first: format!("{}?page=true", id_str),
            last: format!("{}?page=true", id_str),
        };
        ObjectBuilder::new(id_str, "Collection", content).build()
    }
}

impl OrderedCollection {
    pub fn new(
        id: impl Into<String>,
        total_items: u32,
        ordered_items: Vec<serde_json::Value>,
    ) -> Self {
        let id_str = id.into();
        let content = OrderedCollectionContent {
            total_items,
            first: format!("{}?page=true", id_str),
            last: format!("{}?page=true", id_str),
            ordered_items,
        };
        ObjectBuilder::new(id_str, "OrderedCollection", content).build()
    }

    pub fn empty(id: impl Into<String>) -> Self {
        Self::new(id, 0, vec![])
    }

    pub fn add_item(mut self, item: serde_json::Value) -> Self {
        self.content.ordered_items.push(item);
        self.content.total_items += 1;
        self
    }
}

// Tag constructors using functional patterns
impl Tag {
    pub fn mention(name: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            tag_type: "Mention".to_string(),
            name: name.into(),
            href: Some(href.into()),
        }
    }

    pub fn hashtag(name: impl Into<String>, href: Option<String>) -> Self {
        Self {
            tag_type: "Hashtag".to_string(),
            name: name.into(),
            href,
        }
    }

    pub fn emoji(name: impl Into<String>) -> Self {
        Self {
            tag_type: "Emoji".to_string(),
            name: name.into(),
            href: None,
        }
    }
}

// Utility functions
pub fn create_public_note(
    id: impl Into<String>,
    author: impl Into<String>,
    content: impl Into<String>,
) -> Note {
    Note::new(
        id,
        author,
        content,
        vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        vec![],
    )
}

pub fn create_direct_note(
    id: impl Into<String>,
    author: impl Into<String>,
    content: impl Into<String>,
    recipient: impl Into<String>,
) -> Note {
    Note::new(id, author, content, vec![recipient.into()], vec![])
}

// Pattern matching for object types
pub fn match_object_type<T>(object: &ObjectBase<T>) -> ObjectTypeResult {
    match object.object_type.as_str() {
        "Note" => ObjectTypeResult::Note,
        "Article" => ObjectTypeResult::Article,
        "Collection" => ObjectTypeResult::Collection,
        "OrderedCollection" => ObjectTypeResult::OrderedCollection,
        "Image" => ObjectTypeResult::Image,
        "Video" => ObjectTypeResult::Video,
        "Audio" => ObjectTypeResult::Audio,
        _ => ObjectTypeResult::Unknown(object.object_type.clone()),
    }
}

#[derive(Debug, PartialEq)]
pub enum ObjectTypeResult {
    Note,
    Article,
    Collection,
    OrderedCollection,
    Image,
    Video,
    Audio,
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_note_data() -> (String, String, String) {
        (
            "https://example.com/notes/123".to_string(),
            "https://example.com/users/alice".to_string(),
            "Hello, world!".to_string(),
        )
    }

    #[test]
    fn test_note_builder_pattern() {
        let (id, author, content) = test_note_data();
        let to = vec!["https://www.w3.org/ns/activitystreams#Public".to_string()];

        let note = Note::new(id.clone(), author.clone(), content.clone(), to.clone(), vec![]);

        assert_eq!(note.id, id);
        assert_eq!(note.object_type, "Note");
        assert_eq!(note.content.attributed_to, author);
        assert_eq!(note.content.content, content);
        assert_eq!(note.content.to, to);
    }

    #[test]
    fn test_note_functional_methods() {
        let (id, author, content) = test_note_data();
        let tags = vec![
            Tag::mention("@alice", "https://example.com/users/alice"),
            Tag::hashtag("#test".to_string(), None),
        ];

        let note = Note::new(id, author, content, vec![], vec![])
            .with_reply("https://example.com/notes/456")
            .with_tags(tags.clone());

        assert_eq!(note.content.in_reply_to, Some("https://example.com/notes/456".to_string()));
        assert_eq!(note.content.tag.len(), 2);
        assert_eq!(note.content.tag[0].tag_type, "Mention");
        assert_eq!(note.content.tag[1].tag_type, "Hashtag");
    }

    #[test]
    fn test_collection_utilities() {
        let collection = Collection::new("https://example.com/collections/test", 42);
        let ordered = OrderedCollection::empty("https://example.com/outbox")
            .add_item(json!({"type": "Create"}))
            .add_item(json!({"type": "Follow"}));

        assert_eq!(collection.content.total_items, 42);
        assert_eq!(ordered.content.total_items, 2);
        assert_eq!(ordered.content.ordered_items.len(), 2);
    }

    #[test]
    fn test_tag_constructors() {
        let mention = Tag::mention("@alice", "https://example.com/users/alice");
        let hashtag = Tag::hashtag("#rust", Some("https://example.com/tags/rust".to_string()));
        let emoji = Tag::emoji(":heart:");

        assert_eq!(mention.tag_type, "Mention");
        assert!(mention.href.is_some());
        assert_eq!(hashtag.tag_type, "Hashtag");
        assert_eq!(emoji.tag_type, "Emoji");
        assert!(emoji.href.is_none());
    }

    #[test]
    fn test_utility_functions() {
        let public_note = create_public_note("id", "author", "content");
        let direct_note = create_direct_note("id", "author", "content", "recipient");

        assert!(public_note.content.to.contains(&"https://www.w3.org/ns/activitystreams#Public".to_string()));
        assert!(direct_note.content.to.contains(&"recipient".to_string()));
    }

    #[test]
    fn test_object_type_matching() {
        let note = Note::new("id", "author", "content", vec![], vec![]);
        let collection = Collection::new("id", 0);

        assert_eq!(match_object_type(&note), ObjectTypeResult::Note);
        assert_eq!(match_object_type(&collection), ObjectTypeResult::Collection);
    }

    #[test]
    fn test_serialization_compatibility() {
        let note = Note::new("test", "author", "content", vec![], vec![]);
        
        let json = serde_json::to_string(&note).unwrap();
        let deserialized: Note = serde_json::from_str(&json).unwrap();

        assert_eq!(note.id, deserialized.id);
        assert_eq!(note.content.content, deserialized.content.content);
    }
}
