use crate::config::Config;
use crate::models::OrderedCollection;
use actix_web::{get, post, web, HttpResponse, Result};
use serde_json::Value;
use tracing::info;

#[get("/users/{username}/outbox")]
pub async fn get_outbox(
    path: web::Path<String>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let username = path.into_inner();

    // For now, return an empty outbox
    // In a real implementation, you'd load activities from a database
    let outbox = OrderedCollection::new(
        format!("{}/users/{}/outbox", config.server_url, username),
        0,
        vec![],
    );

    Ok(HttpResponse::Ok()
        .content_type("application/activity+json")
        .json(outbox))
}

#[post("/users/{username}/outbox")]
pub async fn post_outbox(
    path: web::Path<String>,
    payload: web::Json<Value>,
    _config: web::Data<Config>,
) -> Result<HttpResponse> {
    let username = path.into_inner();
    let activity = payload.into_inner();

    info!("Received outbox POST for user {}: {:?}", username, activity);

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
                            // Store the note in your database
                            // For now, just log it

                            // In a real implementation, you'd also deliver this to followers
                            // and other servers
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
