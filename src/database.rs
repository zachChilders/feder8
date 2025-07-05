#![allow(dead_code)]

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use mockall::automock;
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DbActor {
    pub id: String,
    pub username: String,
    pub name: String,
    pub summary: Option<String>,
    pub public_key_pem: String,
    pub private_key_pem: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DbActivity {
    pub id: String,
    pub actor_id: String,
    pub activity_type: String,
    pub object: Value,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub published: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DbNote {
    pub id: String,
    pub attributed_to: String,
    pub content: String,
    pub to_recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub published: DateTime<Utc>,
    pub in_reply_to: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DbFollowRelation {
    pub id: String,
    pub follower_id: String,
    pub following_id: String,
    pub status: String, // "pending", "accepted", "rejected"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[automock]
#[async_trait]
pub trait Database: Send + Sync {
    // Actor operations
    async fn create_actor(&self, actor: &DbActor) -> Result<(), DatabaseError>;
    async fn get_actor_by_id(&self, id: &str) -> Result<Option<DbActor>, DatabaseError>;
    async fn get_actor_by_username(&self, username: &str)
        -> Result<Option<DbActor>, DatabaseError>;
    async fn update_actor(&self, actor: &DbActor) -> Result<(), DatabaseError>;
    async fn delete_actor(&self, id: &str) -> Result<(), DatabaseError>;

    // Activity operations
    async fn create_activity(&self, activity: &DbActivity) -> Result<(), DatabaseError>;
    async fn get_activity_by_id(&self, id: &str) -> Result<Option<DbActivity>, DatabaseError>;
    async fn get_activities_by_actor(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbActivity>, DatabaseError>;
    async fn get_inbox_activities(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbActivity>, DatabaseError>;

    // Note operations
    async fn create_note(&self, note: &DbNote) -> Result<(), DatabaseError>;
    async fn get_note_by_id(&self, id: &str) -> Result<Option<DbNote>, DatabaseError>;
    async fn get_notes_by_actor(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbNote>, DatabaseError>;
    async fn delete_note(&self, id: &str) -> Result<(), DatabaseError>;

    // Follow operations
    async fn create_follow(&self, follow: &DbFollowRelation) -> Result<(), DatabaseError>;
    async fn get_follow_by_id(&self, id: &str) -> Result<Option<DbFollowRelation>, DatabaseError>;
    async fn get_followers(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbFollowRelation>, DatabaseError>;
    async fn get_following(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbFollowRelation>, DatabaseError>;
    async fn update_follow_status(
        &self,
        follow_id: &str,
        status: &str,
    ) -> Result<(), DatabaseError>;
    async fn delete_follow(&self, id: &str) -> Result<(), DatabaseError>;

    // Collection operations
    async fn get_actor_outbox_count(&self, actor_id: &str) -> Result<u32, DatabaseError>;
    async fn get_actor_inbox_count(&self, actor_id: &str) -> Result<u32, DatabaseError>;
    async fn get_actor_followers_count(&self, actor_id: &str) -> Result<u32, DatabaseError>;
    async fn get_actor_following_count(&self, actor_id: &str) -> Result<u32, DatabaseError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(String),
    #[error("Query error: {0}")]
    Query(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Not found")]
    NotFound,
    #[error("Already exists")]
    AlreadyExists,
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DatabaseError::NotFound,
            sqlx::Error::Database(db_err) => {
                if db_err.constraint().is_some() {
                    DatabaseError::AlreadyExists
                } else {
                    DatabaseError::Query(db_err.to_string())
                }
            }
            _ => DatabaseError::Query(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        DatabaseError::Serialization(err.to_string())
    }
}

pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), DatabaseError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::Connection(e.to_string()))?;
        Ok(())
    }

    fn naive_to_utc(naive: NaiveDateTime) -> DateTime<Utc> {
        Utc.from_utc_datetime(&naive)
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    async fn create_actor(&self, actor: &DbActor) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            INSERT INTO actors (id, username, name, summary, public_key_pem, private_key_pem, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            actor.id,
            actor.username,
            actor.name,
            actor.summary,
            actor.public_key_pem,
            actor.private_key_pem,
            actor.created_at,
            actor.updated_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_actor_by_id(&self, id: &str) -> Result<Option<DbActor>, DatabaseError> {
        let row = sqlx::query!(
            "SELECT id, username, name, summary, public_key_pem, private_key_pem, created_at, updated_at FROM actors WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DbActor {
            id: r.id.unwrap_or_default(),
            username: r.username,
            name: r.name,
            summary: r.summary,
            public_key_pem: r.public_key_pem,
            private_key_pem: r.private_key_pem,
            created_at: Self::naive_to_utc(r.created_at),
            updated_at: Self::naive_to_utc(r.updated_at),
        }))
    }

    async fn get_actor_by_username(
        &self,
        username: &str,
    ) -> Result<Option<DbActor>, DatabaseError> {
        let row = sqlx::query!(
            "SELECT id, username, name, summary, public_key_pem, private_key_pem, created_at, updated_at FROM actors WHERE username = ?",
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DbActor {
            id: r.id.unwrap_or_default(),
            username: r.username,
            name: r.name,
            summary: r.summary,
            public_key_pem: r.public_key_pem,
            private_key_pem: r.private_key_pem,
            created_at: Self::naive_to_utc(r.created_at),
            updated_at: Self::naive_to_utc(r.updated_at),
        }))
    }

    async fn update_actor(&self, actor: &DbActor) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            UPDATE actors 
            SET name = ?, summary = ?, public_key_pem = ?, private_key_pem = ?, updated_at = ?
            WHERE id = ?
            "#,
            actor.name,
            actor.summary,
            actor.public_key_pem,
            actor.private_key_pem,
            actor.updated_at,
            actor.id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_actor(&self, id: &str) -> Result<(), DatabaseError> {
        sqlx::query!("DELETE FROM actors WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn create_activity(&self, activity: &DbActivity) -> Result<(), DatabaseError> {
        let to_json = serde_json::to_string(&activity.to_recipients)?;
        let cc_json = serde_json::to_string(&activity.cc_recipients)?;
        let object_json = serde_json::to_string(&activity.object)?;

        sqlx::query!(
            r#"
            INSERT INTO activities (id, actor_id, activity_type, object, to_recipients, cc_recipients, published, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            activity.id,
            activity.actor_id,
            activity.activity_type,
            object_json,
            to_json,
            cc_json,
            activity.published,
            activity.created_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_activity_by_id(&self, id: &str) -> Result<Option<DbActivity>, DatabaseError> {
        let row = sqlx::query!(
            "SELECT id, actor_id, activity_type, object, to_recipients, cc_recipients, published, created_at FROM activities WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .map(|r| -> Result<DbActivity, DatabaseError> {
                Ok(DbActivity {
                    id: r.id.unwrap_or_default(),
                    actor_id: r.actor_id,
                    activity_type: r.activity_type,
                    object: serde_json::from_str(&r.object)?,
                    to_recipients: serde_json::from_str(&r.to_recipients)?,
                    cc_recipients: serde_json::from_str(&r.cc_recipients)?,
                    published: Self::naive_to_utc(r.published),
                    created_at: Self::naive_to_utc(r.created_at),
                })
            })
            .transpose()?)
    }

    async fn get_activities_by_actor(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbActivity>, DatabaseError> {
        let rows = sqlx::query!(
            "SELECT id, actor_id, activity_type, object, to_recipients, cc_recipients, published, created_at FROM activities WHERE actor_id = ? ORDER BY published DESC LIMIT ? OFFSET ?",
            actor_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| -> Result<DbActivity, DatabaseError> {
                Ok(DbActivity {
                    id: r.id.unwrap_or_default(),
                    actor_id: r.actor_id,
                    activity_type: r.activity_type,
                    object: serde_json::from_str(&r.object)?,
                    to_recipients: serde_json::from_str(&r.to_recipients)?,
                    cc_recipients: serde_json::from_str(&r.cc_recipients)?,
                    published: Self::naive_to_utc(r.published),
                    created_at: Self::naive_to_utc(r.created_at),
                })
            })
            .collect()
    }

    async fn get_inbox_activities(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbActivity>, DatabaseError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, actor_id, activity_type, object, to_recipients, cc_recipients, published, created_at
            FROM activities 
            WHERE to_recipients LIKE '%' || ? || '%' OR cc_recipients LIKE '%' || ? || '%'
            ORDER BY published DESC 
            LIMIT ? OFFSET ?
            "#,
            actor_id,
            actor_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| -> Result<DbActivity, DatabaseError> {
                Ok(DbActivity {
                    id: r.id.unwrap_or_default(),
                    actor_id: r.actor_id,
                    activity_type: r.activity_type,
                    object: serde_json::from_str(&r.object)?,
                    to_recipients: serde_json::from_str(&r.to_recipients)?,
                    cc_recipients: serde_json::from_str(&r.cc_recipients)?,
                    published: Self::naive_to_utc(r.published),
                    created_at: Self::naive_to_utc(r.created_at),
                })
            })
            .collect()
    }

    async fn create_note(&self, note: &DbNote) -> Result<(), DatabaseError> {
        let to_json = serde_json::to_string(&note.to_recipients)?;
        let cc_json = serde_json::to_string(&note.cc_recipients)?;
        let tags_json = serde_json::to_string(&note.tags)?;

        sqlx::query!(
            r#"
            INSERT INTO notes (id, attributed_to, content, to_recipients, cc_recipients, published, in_reply_to, tags, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            note.id,
            note.attributed_to,
            note.content,
            to_json,
            cc_json,
            note.published,
            note.in_reply_to,
            tags_json,
            note.created_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_note_by_id(&self, id: &str) -> Result<Option<DbNote>, DatabaseError> {
        let row = sqlx::query!(
            "SELECT id, attributed_to, content, to_recipients, cc_recipients, published, in_reply_to, tags, created_at FROM notes WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .map(|r| -> Result<DbNote, DatabaseError> {
                Ok(DbNote {
                    id: r.id.unwrap_or_default(),
                    attributed_to: r.attributed_to,
                    content: r.content,
                    to_recipients: serde_json::from_str(&r.to_recipients)?,
                    cc_recipients: serde_json::from_str(&r.cc_recipients)?,
                    published: Self::naive_to_utc(r.published),
                    in_reply_to: r.in_reply_to,
                    tags: serde_json::from_str(&r.tags)?,
                    created_at: Self::naive_to_utc(r.created_at),
                })
            })
            .transpose()?)
    }

    async fn get_notes_by_actor(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbNote>, DatabaseError> {
        let rows = sqlx::query!(
            "SELECT id, attributed_to, content, to_recipients, cc_recipients, published, in_reply_to, tags, created_at FROM notes WHERE attributed_to = ? ORDER BY published DESC LIMIT ? OFFSET ?",
            actor_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| -> Result<DbNote, DatabaseError> {
                Ok(DbNote {
                    id: r.id.unwrap_or_default(),
                    attributed_to: r.attributed_to,
                    content: r.content,
                    to_recipients: serde_json::from_str(&r.to_recipients)?,
                    cc_recipients: serde_json::from_str(&r.cc_recipients)?,
                    published: Self::naive_to_utc(r.published),
                    in_reply_to: r.in_reply_to,
                    tags: serde_json::from_str(&r.tags)?,
                    created_at: Self::naive_to_utc(r.created_at),
                })
            })
            .collect()
    }

    async fn delete_note(&self, id: &str) -> Result<(), DatabaseError> {
        sqlx::query!("DELETE FROM notes WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn create_follow(&self, follow: &DbFollowRelation) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            INSERT INTO follows (id, follower_id, following_id, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            follow.id,
            follow.follower_id,
            follow.following_id,
            follow.status,
            follow.created_at,
            follow.updated_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_follow_by_id(&self, id: &str) -> Result<Option<DbFollowRelation>, DatabaseError> {
        let row = sqlx::query!(
            "SELECT id, follower_id, following_id, status, created_at, updated_at FROM follows WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DbFollowRelation {
            id: r.id.unwrap_or_default(),
            follower_id: r.follower_id,
            following_id: r.following_id,
            status: r.status,
            created_at: Self::naive_to_utc(r.created_at),
            updated_at: Self::naive_to_utc(r.updated_at),
        }))
    }

    async fn get_followers(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbFollowRelation>, DatabaseError> {
        let rows = sqlx::query!(
            "SELECT id, follower_id, following_id, status, created_at, updated_at FROM follows WHERE following_id = ? AND status = 'accepted' ORDER BY created_at DESC LIMIT ? OFFSET ?",
            actor_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DbFollowRelation {
                id: r.id.unwrap_or_default(),
                follower_id: r.follower_id,
                following_id: r.following_id,
                status: r.status,
                created_at: Self::naive_to_utc(r.created_at),
                updated_at: Self::naive_to_utc(r.updated_at),
            })
            .collect())
    }

    async fn get_following(
        &self,
        actor_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<DbFollowRelation>, DatabaseError> {
        let rows = sqlx::query!(
            "SELECT id, follower_id, following_id, status, created_at, updated_at FROM follows WHERE follower_id = ? AND status = 'accepted' ORDER BY created_at DESC LIMIT ? OFFSET ?",
            actor_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DbFollowRelation {
                id: r.id.unwrap_or_default(),
                follower_id: r.follower_id,
                following_id: r.following_id,
                status: r.status,
                created_at: Self::naive_to_utc(r.created_at),
                updated_at: Self::naive_to_utc(r.updated_at),
            })
            .collect())
    }

    async fn update_follow_status(
        &self,
        follow_id: &str,
        status: &str,
    ) -> Result<(), DatabaseError> {
        let now = Utc::now();
        sqlx::query!(
            "UPDATE follows SET status = ?, updated_at = ? WHERE id = ?",
            status,
            now,
            follow_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_follow(&self, id: &str) -> Result<(), DatabaseError> {
        sqlx::query!("DELETE FROM follows WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_actor_outbox_count(&self, actor_id: &str) -> Result<u32, DatabaseError> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM activities WHERE actor_id = ?",
            actor_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count as u32)
    }

    async fn get_actor_inbox_count(&self, actor_id: &str) -> Result<u32, DatabaseError> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM activities WHERE to_recipients LIKE '%' || ? || '%' OR cc_recipients LIKE '%' || ? || '%'",
            actor_id,
            actor_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count as u32)
    }

    async fn get_actor_followers_count(&self, actor_id: &str) -> Result<u32, DatabaseError> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM follows WHERE following_id = ? AND status = 'accepted'",
            actor_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count as u32)
    }

    async fn get_actor_following_count(&self, actor_id: &str) -> Result<u32, DatabaseError> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM follows WHERE follower_id = ? AND status = 'accepted'",
            actor_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count as u32)
    }
}

// Helper function to create a pre-configured mock database with common expectations
pub fn create_configured_mock_database() -> MockDatabase {
    let mut mock = MockDatabase::new();

    // Configure default expectations for a test actor
    mock.expect_get_actor_by_username()
        .returning(|username| {
            Ok(Some(DbActor {
                id: format!("https://example.com/users/{username}"),
                username: username.to_string(),
                name: format!("Test User {username}"),
                summary: Some("A test user".to_string()),
                public_key_pem: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----".to_string(),
                private_key_pem: Some("-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC...\n-----END PRIVATE KEY-----".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    mock.expect_get_actor_outbox_count().returning(|_| Ok(5));

    mock.expect_get_activities_by_actor()
        .returning(|_, _, _| Ok(vec![]));

    mock.expect_get_actor_inbox_count().returning(|_| Ok(3));

    mock.expect_get_inbox_activities()
        .returning(|_, _, _| Ok(vec![]));

    // Add expectations for inbox handler operations
    mock.expect_get_note_by_id().returning(|_| Ok(None)); // Note doesn't exist, so create it

    mock.expect_create_note().returning(|_| Ok(())); // Successfully create note

    mock.expect_create_activity().returning(|_| Ok(())); // Successfully create activity

    mock.expect_create_follow().returning(|_| Ok(())); // Successfully create follow relationship

    mock.expect_update_follow_status().returning(|_, _| Ok(())); // Successfully update follow status

    mock
}

pub type DatabaseRef = Arc<dyn Database>;
