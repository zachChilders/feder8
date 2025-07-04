use actix_web::{post, web, HttpResponse, Result};
use serde_json::Value;
use crate::config::Config;
use tracing::{info, warn};

#[post("/users/{username}/inbox")]
pub async fn inbox(
    path: web::Path<String>,
    payload: web::Json<Value>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let username = path.into_inner();
    let activity = payload.into_inner();
    
    info!("Received activity in inbox for user {}: {:?}", username, activity);
    
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
                            // Store the note in your database
                            // For now, just log it
                        }
                    }
                }
            }
            "Follow" => {
                info!("Processing Follow activity");
                // Handle Follow activity
                // You would typically store the follow relationship
                // and potentially send an Accept response
            }
            "Accept" => {
                info!("Processing Accept activity");
                // Handle Accept activity (response to Follow)
            }
            "Undo" => {
                info!("Processing Undo activity");
                // Handle Undo activity
            }
            _ => {
                warn!("Unknown activity type: {}", activity_type);
            }
        }
    }
    
    // Always return 202 Accepted for inbox POST requests
    Ok(HttpResponse::Accepted().finish())
} 