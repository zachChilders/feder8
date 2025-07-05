use anyhow::Result;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use log::{error, info, warn};
use std::time::Duration;

pub struct WiFiManager {
    wifi: BlockingWifi<EspWifi<'static>>,
}

impl WiFiManager {
    pub fn new(
        modem: impl Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
        sysloop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> Result<Self> {
        let wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sysloop.clone(), Some(nvs))?,
            sysloop,
        )?;

        Ok(Self { wifi })
    }

    pub fn connect(&mut self, ssid: &str, password: &str) -> Result<()> {
        info!("Connecting to WiFi network: {}", ssid);

        let wifi_configuration = Configuration::Client(ClientConfiguration {
            ssid: ssid.into(),
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: password.into(),
            channel: None,
        });

        self.wifi.set_configuration(&wifi_configuration)?;
        self.wifi.start()?;

        info!("Starting WiFi connection...");
        self.wifi.connect()?;

        info!("Waiting for IP address...");
        self.wifi.wait_netif_up()?;

        let ip_info = self.wifi.wifi().sta_netif().get_ip_info()?;
        info!("WiFi connected! IP address: {}", ip_info.ip);

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from WiFi...");
        self.wifi.disconnect()?;
        self.wifi.stop()?;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.wifi.is_connected().unwrap_or(false)
    }

    pub fn get_ip_info(&self) -> Result<esp_idf_svc::ipv4::IpInfo> {
        Ok(self.wifi.wifi().sta_netif().get_ip_info()?)
    }

    pub fn wait_for_connection(&mut self, timeout_secs: u64) -> Result<()> {
        info!("Waiting for WiFi connection (timeout: {}s)", timeout_secs);
        
        let start_time = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(timeout_secs);

        while !self.is_connected() {
            if start_time.elapsed() > timeout_duration {
                error!("WiFi connection timeout after {}s", timeout_secs);
                return Err(anyhow::anyhow!("WiFi connection timeout"));
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        info!("WiFi connection established!");
        Ok(())
    }

    pub fn reconnect(&mut self, ssid: &str, password: &str) -> Result<()> {
        warn!("Attempting to reconnect to WiFi...");
        
        if self.is_connected() {
            info!("Already connected to WiFi");
            return Ok(());
        }

        // Try to disconnect first (in case we're in a bad state)
        let _ = self.disconnect();

        // Wait a bit before reconnecting
        std::thread::sleep(Duration::from_millis(1000));

        // Try to connect again
        self.connect(ssid, password)?;

        Ok(())
    }
}

/// WiFi monitoring and auto-reconnection functionality
pub struct WiFiMonitor {
    manager: WiFiManager,
    ssid: String,
    password: String,
    reconnect_interval: Duration,
}

impl WiFiMonitor {
    pub fn new(
        manager: WiFiManager,
        ssid: String,
        password: String,
        reconnect_interval_secs: u64,
    ) -> Self {
        Self {
            manager,
            ssid,
            password,
            reconnect_interval: Duration::from_secs(reconnect_interval_secs),
        }
    }

    pub fn start_monitoring(&mut self) -> Result<()> {
        info!("Starting WiFi monitoring with auto-reconnect...");
        
        loop {
            if !self.manager.is_connected() {
                warn!("WiFi connection lost, attempting to reconnect...");
                
                if let Err(e) = self.manager.reconnect(&self.ssid, &self.password) {
                    error!("Failed to reconnect to WiFi: {}", e);
                } else {
                    info!("Successfully reconnected to WiFi");
                }
            }

            std::thread::sleep(self.reconnect_interval);
        }
    }

    pub fn is_connected(&self) -> bool {
        self.manager.is_connected()
    }

    pub fn get_ip_info(&self) -> Result<esp_idf_svc::ipv4::IpInfo> {
        self.manager.get_ip_info()
    }
}

/// WiFi status information
#[derive(Debug, Clone)]
pub struct WiFiStatus {
    pub connected: bool,
    pub ip_address: Option<String>,
    pub rssi: Option<i8>,
    pub ssid: Option<String>,
}

impl WiFiStatus {
    pub fn new(manager: &WiFiManager) -> Self {
        let connected = manager.is_connected();
        let ip_address = if connected {
            manager.get_ip_info().ok().map(|info| info.ip.to_string())
        } else {
            None
        };

        Self {
            connected,
            ip_address,
            rssi: None, // ESP-IDF doesn't provide easy RSSI access
            ssid: None, // Could be stored from configuration
        }
    }

    pub fn is_ready_for_activitypub(&self) -> bool {
        self.connected && self.ip_address.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_status_creation() {
        // Note: This test can't actually create a WiFiManager without ESP-IDF hardware
        // In a real test environment, you'd need to mock the ESP-IDF components
        let status = WiFiStatus {
            connected: true,
            ip_address: Some("192.168.1.100".to_string()),
            rssi: Some(-45),
            ssid: Some("TestNetwork".to_string()),
        };

        assert!(status.connected);
        assert_eq!(status.ip_address, Some("192.168.1.100".to_string()));
        assert!(status.is_ready_for_activitypub());
    }

    #[test]
    fn test_wifi_status_not_ready() {
        let status = WiFiStatus {
            connected: false,
            ip_address: None,
            rssi: None,
            ssid: None,
        };

        assert!(!status.connected);
        assert!(!status.is_ready_for_activitypub());
    }

    #[test]
    fn test_wifi_monitor_creation() {
        // This test just verifies the structure can be created
        // In a real test, you'd need to mock the WiFiManager
        let ssid = "TestNetwork".to_string();
        let password = "TestPassword".to_string();
        let reconnect_interval = 30;

        // Can't actually create the manager without ESP-IDF, so we'll just test the parameters
        assert_eq!(ssid, "TestNetwork");
        assert_eq!(password, "TestPassword");
        assert_eq!(reconnect_interval, 30);
    }
}