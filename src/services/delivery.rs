use crate::config::Config;
use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use tracing::{error, info, warn};

#[allow(dead_code)]
pub struct DeliveryService {
    client: Client,
    config: Config,
}

#[allow(dead_code)]
impl DeliveryService {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn deliver_activity(&self, inbox_url: &str, activity: Value) -> Result<()> {
        info!("Delivering activity to inbox: {}", inbox_url);

        let response = self
            .client
            .post(inbox_url)
            .header("Content-Type", "application/activity+json")
            .header(
                "User-Agent",
                format!("Fediverse-Node/{}", env!("CARGO_PKG_VERSION")),
            )
            .json(&activity)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Successfully delivered activity to {}", inbox_url);
        } else {
            warn!(
                "Failed to deliver activity to {}: {}",
                inbox_url,
                response.status()
            );
            if let Ok(error_text) = response.text().await {
                error!("Error response: {}", error_text);
            }
        }

        Ok(())
    }

    pub async fn deliver_to_followers(
        &self,
        activity: Value,
        followers: Vec<String>,
    ) -> Result<()> {
        info!("Delivering activity to {} followers", followers.len());

        for follower_inbox in followers {
            if let Err(e) = self
                .deliver_activity(&follower_inbox, activity.clone())
                .await
            {
                warn!("Failed to deliver to {}: {}", follower_inbox, e);
            }
        }

        Ok(())
    }

    pub async fn deliver_to_public(
        &self,
        activity: Value,
        public_inboxes: Vec<String>,
    ) -> Result<()> {
        info!(
            "Delivering activity to {} public inboxes",
            public_inboxes.len()
        );

        for inbox in public_inboxes {
            if let Err(e) = self.deliver_activity(&inbox, activity.clone()).await {
                warn!("Failed to deliver to public inbox {}: {}", inbox, e);
            }
        }

        Ok(())
    }
}
