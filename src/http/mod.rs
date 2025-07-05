pub mod client;
pub mod server;

// Re-export the main traits for easy access
#[allow(unused_imports)]
pub use client::{HttpClient, HttpRequest, HttpResponse as ClientResponse, StatusCode};
#[allow(unused_imports)]
pub use server::{HttpContext, HttpHandler, HttpResponse as ServerResponse, HttpServer};

// Re-export implementations
#[allow(unused_imports)]
pub use client::reqwest::ReqwestClient;
#[allow(unused_imports)]
pub use server::actix::ActixServer;
