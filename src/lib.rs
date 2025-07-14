// Core ActivityPub models and traits
pub mod config;
pub mod errors;
pub mod models;
pub mod traits;

// Conditional modules based on target
#[cfg(feature = "native")]
pub mod native {
    pub mod container;
    pub mod database;
    pub mod handlers;
    pub mod http;
    pub mod services;
}

#[cfg(feature = "esp32")]
pub mod esp32 {
    pub mod container;
    pub mod delivery;
    pub mod http;
    pub mod server;
    pub mod utils;
    pub mod wifi;
}

// Re-export commonly used types
pub use config::Config;
pub use errors::Feder8Error;
pub use models::{Activity, Actor, Icon, Note, PublicKey};
pub use traits::{Database, DeliveryService, HttpClient};

// Conditional re-exports
#[cfg(feature = "native")]
pub use native::{container::Container, database::DatabaseRef};

#[cfg(feature = "esp32")]
pub use config::EmbeddedConfig;
#[cfg(feature = "esp32")]
pub use esp32::{container::EmbeddedContainer, server::ActivityPubServer};
