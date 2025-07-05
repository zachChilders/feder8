use crate::config::Config;
use crate::database::DatabaseRef;
use crate::models::OrderedCollection;
use actix_web::{get, post, web, HttpResponse, Result};
use serde_json::Value;
use tracing::{info, warn};

#[get("/users/{username}/outbox")]
pub async fn get_outbox(
    path: web::Path<String>,
    config: web::Data<Config>,
    db: web::Data<DatabaseRef>,
) -> Result<HttpResponse> {
    let username = path.into_inner();

    // First, get the actor to make sure they exist
    let actor = match db.get_actor_by_username(&username).await {
        Ok(Some(actor)) => actor,
        Ok(None) => {
            warn!("Actor not found for outbox: {}", username);
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Actor not found"
            })));
        }
        Err(e) => {
            warn!("Database error while fetching actor {}: {}", username, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })));
        }
    };

    // Get the outbox count and activities
    let total_items = match db.get_actor_outbox_count(&actor.id).await {
        Ok(count) => count,
        Err(e) => {
            warn!(
                "Database error while counting outbox items for {}: {}",
                username, e
            );
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })));
        }
    };

    // Get recent activities (limit to 20 for now)
    let activities = match db.get_activities_by_actor(&actor.id, 20, 0).await {
        Ok(activities) => activities,
        Err(e) => {
            warn!(
                "Database error while fetching outbox activities for {}: {}",
                username, e
            );
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })));
        }
    };

    let activity_objects: Vec<Value> = activities
        .into_iter()
        .map(|activity| {
            serde_json::json!({
                "id": activity.id,
                "type": activity.activity_type,
                "actor": activity.actor_id,
                "object": activity.object,
                "to": activity.to_recipients,
                "cc": activity.cc_recipients,
                "published": activity.published
            })
        })
        .collect();

    let outbox = OrderedCollection::new(
        format!("{}/users/{}/outbox", config.server_url, username),
        total_items,
        activity_objects,
    );

    Ok(HttpResponse::Ok()
        .content_type("application/activity+json")
        .json(outbox))
}

#[post("/users/{username}/outbox")]
pub async fn post_outbox(
    path: web::Path<String>,
    payload: web::Json<Value>,
    config: web::Data<Config>,
    db: web::Data<DatabaseRef>,
) -> Result<HttpResponse> {
    let username = path.into_inner();
    let activity = payload.into_inner();

    info!("Received outbox POST for user {}: {:?}", username, activity);

    // First, get the actor to make sure they exist
    let actor = match db.get_actor_by_username(&username).await {
        Ok(Some(actor)) => actor,
        Ok(None) => {
            warn!("Actor not found for outbox POST: {}", username);
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Actor not found"
            })));
        }
        Err(e) => {
            warn!("Database error while fetching actor {}: {}", username, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })));
        }
    };

    // Extract activity type
    if let Some(activity_type) = activity.get("type").and_then(|v| v.as_str()) {
        match activity_type {
            "Create" => {
                info!("Processing Create activity in outbox");
                // Handle Create activity (new post/note)
                if let Some(object) = activity.get("object") {
                    if let Some(object_type) = object.get("type").and_then(|v| v.as_str()) {
                        if object_type == "Note" {
                            info!("Creating Note: {:?}", object);

                            // Generate unique IDs
                            let activity_id = format!(
                                "{}/activities/{}",
                                config.server_url,
                                uuid::Uuid::new_v4()
                            );
                            let note_id =
                                format!("{}/notes/{}", config.server_url, uuid::Uuid::new_v4());

                            // Extract note data
                            let content = object
                                .get("content")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            let to_recipients: Vec<String> = activity
                                .get("to")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();
                            let cc_recipients: Vec<String> = activity
                                .get("cc")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();

                            // Create the note in database
                            let db_note = crate::database::DbNote {
                                id: note_id.clone(),
                                attributed_to: actor.id.clone(),
                                content,
                                to_recipients: to_recipients.clone(),
                                cc_recipients: cc_recipients.clone(),
                                published: chrono::Utc::now(),
                                in_reply_to: object
                                    .get("inReplyTo")
                                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                                tags: vec![], // TODO: Extract tags from object
                                created_at: chrono::Utc::now(),
                            };

                            if let Err(e) = db.create_note(&db_note).await {
                                warn!("Database error while creating note: {}", e);
                                return Ok(HttpResponse::InternalServerError().json(
                                    serde_json::json!({
                                        "error": "Failed to create note"
                                    }),
                                ));
                            }

                            // Create the activity in database
                            let mut activity_object = object.clone();
                            activity_object["id"] = serde_json::Value::String(note_id);
                            activity_object["attributedTo"] =
                                serde_json::Value::String(actor.id.clone());

                            let db_activity = crate::database::DbActivity {
                                id: activity_id.clone(),
                                actor_id: actor.id.clone(),
                                activity_type: "Create".to_string(),
                                object: activity_object,
                                to_recipients,
                                cc_recipients,
                                published: chrono::Utc::now(),
                                created_at: chrono::Utc::now(),
                            };

                            if let Err(e) = db.create_activity(&db_activity).await {
                                warn!("Database error while creating activity: {}", e);
                                return Ok(HttpResponse::InternalServerError().json(
                                    serde_json::json!({
                                        "error": "Failed to create activity"
                                    }),
                                ));
                            }

                            info!("Successfully created note and activity");

                            // Return the created activity
                            return Ok(HttpResponse::Created().json(serde_json::json!({
                                "id": activity_id,
                                "type": "Create",
                                "actor": actor.id,
                                "object": db_activity.object,
                                "to": db_activity.to_recipients,
                                "cc": db_activity.cc_recipients,
                                "published": db_activity.published
                            })));
                        }
                    }
                }
            }
            _ => {
                info!("Unsupported activity type in outbox: {}", activity_type);
            }
        }
    }

    // Return 201 Created for successful outbox POST requests
    Ok(HttpResponse::Created().finish())
}
