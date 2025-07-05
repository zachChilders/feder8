pub mod client;
pub mod server;

// Re-export the main traits for easy access
pub use client::{HttpClient, HttpRequest, HttpResponse as ClientResponse};
pub use server::{HttpServer, HttpHandler, HttpContext, HttpResponse as ServerResponse};

// Re-export implementations
pub use client::reqwest::ReqwestClient;
pub use server::actix::ActixServer;