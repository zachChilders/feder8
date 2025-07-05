pub mod client;

// Re-export the main traits for easy access
pub use client::{HttpClient, HttpRequest, HttpResponse, StatusCode};

// Re-export implementations
pub use client::reqwest::ReqwestClient;
