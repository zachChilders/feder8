use anyhow::Result;
use feder8_firmware::container::{EmbeddedContainer, EmbeddedContainerBuilder};
use feder8_firmware::models::EmbeddedConfig;
use feder8_firmware::server::ActivityPubServer;
use feder8_firmware::utils::{get_free_heap_size, get_timer_ms};
use feder8_firmware::wifi::{WiFiManager, WiFiStatus};
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(target_arch = "xtensa")]
use esp_idf_hal::peripherals::Peripherals;
#[cfg(target_arch = "xtensa")]
use esp_idf_svc::eventloop::EspSystemEventLoop;
#[cfg(target_arch = "xtensa")]
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
#[cfg(target_arch = "xtensa")]
use esp_idf_svc::nvs::EspDefaultNvsPartition;

const DEFAULT_WIFI_SSID: &str = "YourWiFiSSID";
const DEFAULT_WIFI_PASSWORD: &str = "YourWiFiPassword";
const DEFAULT_SERVER_NAME: &str = "ESP32 ActivityPub Node";
const DEFAULT_ACTOR_NAME: &str = "esp32node";

#[cfg(target_arch = "xtensa")]
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

#[cfg(not(target_arch = "xtensa"))]
fn main() -> Result<()> {
    println!("ESP32 ActivityPub Node (test environment)");

    // Create mock configuration for testing
    let config = EmbeddedConfig::new(
        DEFAULT_SERVER_NAME,
        "http://127.0.0.1:3000",
        DEFAULT_ACTOR_NAME,
        "MockWiFi",
        "password123",
    )?;

    // Initialize dependency injection container with mocks
    let container = EmbeddedContainerBuilder::new()
        .with_config(config)
        .with_mocks()
        .build()?;

    info!("Test container initialized");

    // Run a simple test
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        container
            .send_note("Test message from mock environment")
            .await?;
        Ok::<(), anyhow::Error>(())
    })?;

    info!("Test completed successfully");
    Ok(())
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
                let free_heap = get_free_heap_size();
                let status_message = format!(
                    "ESP32 ActivityPub Node is running! Uptime: {} minutes. Free heap: {} bytes",
                    counter, free_heap
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
            let free_heap = get_free_heap_size();
            info!(
                "Node status - Followers: {}, Public inboxes: {}, Free heap: {} bytes",
                container.followers().len(),
                container.public_inboxes().len(),
                free_heap
            );
        }
    }
}
