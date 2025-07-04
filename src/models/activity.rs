use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Base trait for all ActivityPub objects
pub trait ActivityPubObject {
    fn context() -> Vec<String> {
        vec!["https://www.w3.org/ns/activitystreams".to_string()]
    }

    fn generate_id() -> String {
        format!("https://example.com/activities/{}", Uuid::new_v4())
    }

    fn timestamp() -> DateTime<Utc> {
        Utc::now()
    }
}

// Generic ActivityPub base structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPubBase<T> {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub actor: String,
    pub object: T,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub published: DateTime<Utc>,
}

// Type aliases for specific activity types
pub type Activity = ActivityPubBase<serde_json::Value>;
pub type Create = ActivityPubBase<serde_json::Value>;
pub type Follow = ActivityPubBase<String>;
pub type Accept = ActivityPubBase<serde_json::Value>;

// Builder for activities using functional patterns
pub struct ActivityBuilder<T> {
    activity_type: String,
    actor: String,
    object: T,
    to: Vec<String>,
    cc: Vec<String>,
}

impl<T> ActivityBuilder<T> {
    pub fn new(activity_type: impl Into<String>, actor: impl Into<String>, object: T) -> Self {
        Self {
            activity_type: activity_type.into(),
            actor: actor.into(),
            object,
            to: Vec::new(),
            cc: Vec::new(),
        }
    }

    pub fn to(mut self, recipients: Vec<String>) -> Self {
        self.to = recipients;
        self
    }

    pub fn cc(mut self, recipients: Vec<String>) -> Self {
        self.cc = recipients;
        self
    }

    pub fn add_to(mut self, recipient: impl Into<String>) -> Self {
        self.to.push(recipient.into());
        self
    }

    pub fn add_cc(mut self, recipient: impl Into<String>) -> Self {
        self.cc.push(recipient.into());
        self
    }

    pub fn build(self) -> ActivityPubBase<T> {
        ActivityPubBase {
            context: ActivityPubBase::<T>::context(),
            id: ActivityPubBase::<T>::generate_id(),
            object_type: self.activity_type,
            actor: self.actor,
            object: self.object,
            to: self.to,
            cc: self.cc,
            published: ActivityPubBase::<T>::timestamp(),
        }
    }
}

impl<T> ActivityPubObject for ActivityPubBase<T> {}

// Functional constructors using the builder pattern
impl Activity {
    pub fn new(
        activity_type: impl Into<String>,
        actor: impl Into<String>,
        object: serde_json::Value,
        to: Vec<String>,
        cc: Vec<String>,
    ) -> Self {
        ActivityBuilder::new(activity_type, actor, object)
            .to(to)
            .cc(cc)
            .build()
    }
}

impl Create {
    pub fn new(
        actor: impl Into<String>,
        object: serde_json::Value,
        to: Vec<String>,
        cc: Vec<String>,
    ) -> Self {
        ActivityBuilder::new("Create", actor, object)
            .to(to)
            .cc(cc)
            .build()
    }
}

impl Follow {
    pub fn new(
        actor: impl Into<String>,
        object: impl Into<String>,
        to: Vec<String>,
        cc: Vec<String>,
    ) -> Self {
        ActivityBuilder::new("Follow", actor, object.into())
            .to(to)
            .cc(cc)
            .build()
    }
}

impl Accept {
    pub fn new(
        actor: impl Into<String>,
        object: serde_json::Value,
        to: Vec<String>,
        cc: Vec<String>,
    ) -> Self {
        ActivityBuilder::new("Accept", actor, object)
            .to(to)
            .cc(cc)
            .build()
    }
}

// Functional utilities
pub fn create_public_activity<T>(
    activity_type: impl Into<String>,
    actor: impl Into<String>,
    object: T,
) -> ActivityPubBase<T> {
    ActivityBuilder::new(activity_type, actor, object)
        .add_to("https://www.w3.org/ns/activitystreams#Public")
        .build()
}

pub fn create_direct_activity<T>(
    activity_type: impl Into<String>,
    actor: impl Into<String>,
    object: T,
    recipient: impl Into<String>,
) -> ActivityPubBase<T> {
    ActivityBuilder::new(activity_type, actor, object)
        .add_to(recipient)
        .build()
}

// Pattern matching for activity types
pub fn match_activity_type(activity: &Activity) -> ActivityTypeResult {
    match activity.object_type.as_str() {
        "Create" => ActivityTypeResult::Create,
        "Follow" => ActivityTypeResult::Follow,
        "Accept" => ActivityTypeResult::Accept,
        "Reject" => ActivityTypeResult::Reject,
        "Undo" => ActivityTypeResult::Undo,
        "Delete" => ActivityTypeResult::Delete,
        "Like" => ActivityTypeResult::Like,
        "Announce" => ActivityTypeResult::Announce,
        _ => ActivityTypeResult::Unknown(activity.object_type.clone()),
    }
}

#[derive(Debug, PartialEq)]
pub enum ActivityTypeResult {
    Create,
    Follow,
    Accept,
    Reject,
    Undo,
    Delete,
    Like,
    Announce,
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_actor() -> String {
        "https://example.com/users/alice".to_string()
    }

    fn test_recipients() -> (Vec<String>, Vec<String>) {
        (
            vec!["https://example.com/users/bob".to_string()],
            vec!["https://www.w3.org/ns/activitystreams#Public".to_string()],
        )
    }

    #[test]
    fn test_activity_builder_pattern() {
        let object = json!({"type": "Note", "content": "Hello"});
        let (to, cc) = test_recipients();

        let activity = ActivityBuilder::new("Create", test_actor(), object.clone())
            .to(to.clone())
            .cc(cc.clone())
            .build();

        assert_eq!(activity.object_type, "Create");
        assert_eq!(activity.actor, test_actor());
        assert_eq!(activity.object, object);
        assert_eq!(activity.to, to);
        assert_eq!(activity.cc, cc);
        assert!(activity.id.starts_with("https://example.com/activities/"));
    }

    #[test]
    fn test_functional_constructors() {
        let (to, cc) = test_recipients();
        
        let create = Create::new(test_actor(), json!({"content": "test"}), to.clone(), cc.clone());
        let follow = Follow::new(test_actor(), "https://example.com/users/bob", to.clone(), cc.clone());
        let accept = Accept::new(test_actor(), json!({"type": "Follow"}), to.clone(), cc.clone());

        assert_eq!(create.object_type, "Create");
        assert_eq!(follow.object_type, "Follow");
        assert_eq!(accept.object_type, "Accept");
    }

    #[test]
    fn test_public_activity_helper() {
        let activity = create_public_activity("Create", test_actor(), json!({"content": "public"}));
        
        assert!(activity.to.contains(&"https://www.w3.org/ns/activitystreams#Public".to_string()));
        assert_eq!(activity.object_type, "Create");
    }

    #[test]
    fn test_direct_activity_helper() {
        let activity = create_direct_activity("Follow", test_actor(), "target".to_string(), "https://example.com/users/bob");
        
        assert!(activity.to.contains(&"https://example.com/users/bob".to_string()));
        assert_eq!(activity.object_type, "Follow");
    }

    #[test]
    fn test_activity_type_matching() {
        let activities = [
            ("Create", ActivityTypeResult::Create),
            ("Follow", ActivityTypeResult::Follow),
            ("Accept", ActivityTypeResult::Accept),
            ("CustomType", ActivityTypeResult::Unknown("CustomType".to_string())),
        ];

        for (activity_type, expected) in activities {
            let activity = Activity::new(activity_type, test_actor(), json!({}), vec![], vec![]);
            assert_eq!(match_activity_type(&activity), expected);
        }
    }

    #[test]
    fn test_builder_chaining() {
        let activity = ActivityBuilder::new("Create", test_actor(), json!({"content": "test"}))
            .add_to("https://example.com/users/bob")
            .add_to("https://example.com/users/charlie")
            .add_cc("https://www.w3.org/ns/activitystreams#Public")
            .build();

        assert_eq!(activity.to.len(), 2);
        assert_eq!(activity.cc.len(), 1);
        assert!(activity.to.contains(&"https://example.com/users/bob".to_string()));
        assert!(activity.to.contains(&"https://example.com/users/charlie".to_string()));
    }

    #[test]
    fn test_activity_serialization() {
        let activity = Create::new(test_actor(), json!({"content": "test"}), vec![], vec![]);
        
        let json = serde_json::to_string(&activity).unwrap();
        let deserialized: Create = serde_json::from_str(&json).unwrap();

        assert_eq!(activity.object_type, deserialized.object_type);
        assert_eq!(activity.actor, deserialized.actor);
        assert_eq!(activity.object, deserialized.object);
    }
}
