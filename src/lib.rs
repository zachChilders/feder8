// Core ActivityPub models and traits
pub mod models;
pub mod traits;
pub mod config;
pub mod errors;

// Conditional modules based on target
#[cfg(feature = "native")]
pub mod native {
    pub mod database;
    pub mod handlers;
    pub mod services;
    pub mod container;
    pub mod http;
}

#[cfg(feature = "esp32")]
pub mod esp32 {
    pub mod container;
    pub mod delivery;
    pub mod http;
    pub mod server;
    pub mod wifi;
    pub mod utils;
}

// Re-export commonly used types
pub use models::{Actor, Activity, Note, PublicKey, Icon};
pub use traits::{HttpClient, DeliveryService, Database};
pub use config::Config;
pub use errors::Feder8Error;

// Conditional re-exports
#[cfg(feature = "native")]
pub use native::{database::DatabaseRef, container::Container};

#[cfg(feature = "esp32")]
pub use config::EmbeddedConfig;
#[cfg(feature = "esp32")]
pub use esp32::{container::EmbeddedContainer, server::ActivityPubServer};
