use anyhow::Result;
use container::{EmbeddedContainer, EmbeddedContainerBuilder};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use log::*;
use models::EmbeddedConfig;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use wifi::{WiFiManager, WiFiStatus};

mod container;
mod delivery;
mod http;
mod models;
mod server;
mod wifi;

use server::ActivityPubServer;

const DEFAULT_WIFI_SSID: &str = "YourWiFiSSID";
const DEFAULT_WIFI_PASSWORD: &str = "YourWiFiPassword";
const DEFAULT_SERVER_NAME: &str = "ESP32 ActivityPub Node";
const DEFAULT_ACTOR_NAME: &str = "esp32node";

fn main() -> Result<()> {
    // Initialize ESP-IDF
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting ESP32 ActivityPub Node...");

    // Initialize peripherals
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Initialize WiFi manager
    let mut wifi_manager = WiFiManager::new(peripherals.modem, sysloop.clone(), nvs.clone())?;
    
    // Connect to WiFi
    info!("Connecting to WiFi: {}", DEFAULT_WIFI_SSID);
    wifi_manager.connect(DEFAULT_WIFI_SSID, DEFAULT_WIFI_PASSWORD)?;
    
    // Wait for connection
    wifi_manager.wait_for_connection(30)?;
    
    let wifi_status = WiFiStatus::new(&wifi_manager);
    if !wifi_status.is_ready_for_activitypub() {
        error!("WiFi not ready for ActivityPub. Status: {:?}", wifi_status);
        return Err(anyhow::anyhow!("WiFi connection failed"));
    }

    info!("WiFi connected! IP: {:?}", wifi_status.ip_address);

    // Create embedded configuration
    let server_url = format!("http://{}", wifi_status.ip_address.unwrap());
    let config = EmbeddedConfig::new(
        DEFAULT_SERVER_NAME,
        &server_url,
        DEFAULT_ACTOR_NAME,
        DEFAULT_WIFI_SSID,
        DEFAULT_WIFI_PASSWORD,
    )?;

    info!("ActivityPub server will be available at: {}", server_url);

    // Initialize dependency injection container
    let container = EmbeddedContainerBuilder::new()
        .with_config(config)
        .build()?;

    info!("Dependency injection container initialized");

    // Create HTTP server
    let server_config = Configuration {
        http_port: 80,
        https_port: 443,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&server_config)?;
    
    // Create ActivityPub server wrapper
    let activitypub_server = ActivityPubServer::new(Arc::new(Mutex::new(container)));
    
    // Register ActivityPub endpoints
    activitypub_server.register_handlers(&mut server)?;

    info!("ActivityPub server started on port 80");

    // Start periodic tasks
    let container_clone = activitypub_server.container.clone();
    thread::spawn(move || {
        periodic_tasks(container_clone);
    });

    // Keep the main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
        
        // Check WiFi connection periodically
        if !wifi_manager.is_connected() {
            warn!("WiFi connection lost, attempting to reconnect...");
            if let Err(e) = wifi_manager.reconnect(DEFAULT_WIFI_SSID, DEFAULT_WIFI_PASSWORD) {
                error!("Failed to reconnect to WiFi: {}", e);
            } else {
                info!("Successfully reconnected to WiFi");
            }
        }
    }
}

fn periodic_tasks(container: Arc<Mutex<EmbeddedContainer>>) {
    info!("Starting periodic tasks...");
    
    let mut counter = 0;
    loop {
        thread::sleep(Duration::from_secs(60)); // Run every minute
        counter += 1;
        
        // Send a periodic status update every 10 minutes
        if counter % 10 == 0 {
            info!("Periodic task: sending status update");
            
            if let Ok(container) = container.lock() {
                let status_message = format!(
                    "ESP32 ActivityPub Node is running! Uptime: {} minutes. Free heap: {} bytes",
                    counter,
                    esp_idf_sys::esp_get_free_heap_size()
                );
                
                let runtime = tokio::runtime::Runtime::new();
                if let Ok(rt) = runtime {
                    if let Err(e) = rt.block_on(container.send_note(&status_message)) {
                        error!("Failed to send status update: {}", e);
                    } else {
                        info!("Status update sent successfully");
                    }
                }
            }
        }
        
        // Log basic status every minute
        if let Ok(container) = container.lock() {
            info!(
                "Node status - Followers: {}, Public inboxes: {}, Free heap: {} bytes",
                container.followers().len(),
                container.public_inboxes().len(),
                esp_idf_sys::esp_get_free_heap_size()
            );
        }
    }
}
