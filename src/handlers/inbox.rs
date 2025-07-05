use crate::config::Config;
use crate::database::DatabaseRef;
use actix_web::{post, web, HttpResponse, Result};
use serde_json::Value;
use tracing::{info, warn};

#[post("/users/{username}/inbox")]
pub async fn inbox(
    path: web::Path<String>,
    payload: web::Json<Value>,
    config: web::Data<Config>,
    db: web::Data<DatabaseRef>,
) -> Result<HttpResponse> {
    let username = path.into_inner();
    let activity = payload.into_inner();

    info!(
        "Received activity in inbox for user {}: {:?}",
        username, activity
    );

    // First, get the target actor to make sure they exist
    let target_actor = match db.get_actor_by_username(&username).await {
        Ok(Some(actor)) => actor,
        Ok(None) => {
            warn!("Target actor not found for inbox: {}", username);
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Actor not found"
            })));
        }
        Err(e) => {
            warn!(
                "Database error while fetching target actor {}: {}",
                username, e
            );
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })));
        }
    };

    // Extract activity type
    if let Some(activity_type) = activity.get("type").and_then(|v| v.as_str()) {
        match activity_type {
            "Create" => {
                info!("Processing Create activity");
                // Handle Create activity (new post/note)
                if let Some(object) = activity.get("object") {
                    if let Some(object_type) = object.get("type").and_then(|v| v.as_str()) {
                        if object_type == "Note" {
                            info!("Received Note: {:?}", object);

                            // Extract note data
                            let note_id = object
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let attributed_to = object
                                .get("attributedTo")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let content = object
                                .get("content")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let to_recipients = object
                                .get("to")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_else(Vec::new);
                            let cc_recipients = object
                                .get("cc")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_else(Vec::new);

                            // Create the note in database if it doesn't exist
                            if let Ok(None) = db.get_note_by_id(&note_id).await {
                                let db_note = crate::database::DbNote {
                                    id: note_id.clone(),
                                    attributed_to,
                                    content,
                                    to_recipients: to_recipients.clone(),
                                    cc_recipients: cc_recipients.clone(),
                                    published: object
                                        .get("published")
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                                        .map(|dt| dt.with_timezone(&chrono::Utc))
                                        .unwrap_or_else(chrono::Utc::now),
                                    in_reply_to: object
                                        .get("inReplyTo")
                                        .and_then(|v| v.as_str().map(|s| s.to_string())),
                                    tags: vec![], // TODO: Extract tags from object
                                    created_at: chrono::Utc::now(),
                                };

                                if let Err(e) = db.create_note(&db_note).await {
                                    warn!("Database error while creating note from inbox: {}", e);
                                }
                            }

                            // Store the activity
                            let activity_id = activity
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let actor_id = activity
                                .get("actor")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let activity_to = activity
                                .get("to")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();
                            let activity_cc = activity
                                .get("cc")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();

                            let db_activity = crate::database::DbActivity {
                                id: activity_id,
                                actor_id,
                                activity_type: "Create".to_string(),
                                object: object.clone(),
                                to_recipients: activity_to,
                                cc_recipients: activity_cc,
                                published: activity
                                    .get("published")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                    .unwrap_or_else(chrono::Utc::now),
                                created_at: chrono::Utc::now(),
                            };

                            if let Err(e) = db.create_activity(&db_activity).await {
                                warn!("Database error while creating activity from inbox: {}", e);
                            }
                        }
                    }
                }
            }
            "Follow" => {
                info!("Processing Follow activity");
                // Handle Follow activity
                let _activity_id = activity
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let follower_id = activity
                    .get("actor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let following_id = activity
                    .get("object")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Check if this is targeting our actor
                if following_id == target_actor.id {
                    let follow_id =
                        format!("{}/follows/{}", config.server_url, uuid::Uuid::new_v4());
                    let db_follow = crate::database::DbFollowRelation {
                        id: follow_id,
                        follower_id,
                        following_id,
                        status: "pending".to_string(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    };

                    if let Err(e) = db.create_follow(&db_follow).await {
                        warn!("Database error while creating follow relationship: {}", e);
                    } else {
                        info!("Created follow relationship: {:?}", db_follow);
                        // TODO: Auto-accept or require manual approval
                    }
                }
            }
            "Accept" => {
                info!("Processing Accept activity");
                // Handle Accept activity (response to Follow)
                if let Some(object) = activity.get("object") {
                    if let Some(follow_id) = object.get("id").and_then(|v| v.as_str()) {
                        if let Err(e) = db.update_follow_status(follow_id, "accepted").await {
                            warn!("Database error while updating follow status: {}", e);
                        } else {
                            info!("Updated follow status to accepted for: {}", follow_id);
                        }
                    }
                }
            }
            "Undo" => {
                info!("Processing Undo activity");
                // Handle Undo activity
                if let Some(object) = activity.get("object") {
                    if let Some(object_type) = object.get("type").and_then(|v| v.as_str()) {
                        if object_type == "Follow" {
                            // Undo follow - delete the follow relationship
                            let follower_id =
                                activity.get("actor").and_then(|v| v.as_str()).unwrap_or("");
                            let following_id =
                                object.get("object").and_then(|v| v.as_str()).unwrap_or("");

                            if following_id == target_actor.id {
                                // Find and delete the follow relationship
                                // This is a simplified approach - in practice you'd query for the specific follow
                                info!(
                                    "Processing unfollow from {} to {}",
                                    follower_id, following_id
                                );
                            }
                        }
                    }
                }
            }
            _ => {
                warn!("Unknown activity type: {}", activity_type);
            }
        }
    }

    // Always return 202 Accepted for inbox POST requests
    Ok(HttpResponse::Accepted().finish())
}
