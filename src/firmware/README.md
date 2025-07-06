# ESP32 ActivityPub Node Firmware

This firmware implements a complete ActivityPub node that runs on ESP32 microcontrollers, enabling your ESP32 to participate in the fediverse as an autonomous social media node.

## Features

- 🌐 **Full ActivityPub Support**: Implements core ActivityPub protocol features including WebFinger, Actor profiles, Inbox/Outbox endpoints
- 📡 **WiFi Connectivity**: Automatic WiFi connection with reconnection handling
- 🔄 **Activity Delivery**: Send notes, follow/unfollow users, and handle incoming activities
- 🏗️ **Dependency Injection**: Modular architecture with swappable HTTP clients and delivery services
- 💾 **Memory Optimized**: Uses `heapless` collections for embedded constraints
- 🔍 **Health Monitoring**: Built-in health check and system monitoring endpoints
- 📊 **NodeInfo Support**: Exposes node information for fediverse discovery

## Hardware Requirements

- ESP32 or ESP32-S3 development board
- At least 4MB flash memory
- WiFi antenna (usually built-in)
- Minimum 520KB RAM (for ActivityPub operations)

## Quick Start

### 1. Configure WiFi Credentials

Edit `src/main.rs` and update the WiFi configuration:

```rust
const DEFAULT_WIFI_SSID: &str = "YourWiFiSSID";
const DEFAULT_WIFI_PASSWORD: &str = "YourWiFiPassword";
```

### 2. Build and Flash

```bash
# Set up ESP-IDF environment
. $HOME/esp/esp-idf/export.sh

# Build and flash
cargo build --release
espflash flash target/xtensa-esp32-espidf/release/feder8-firmware
```

### 3. Monitor Output

```bash
cargo espmonitor --port /dev/ttyUSB0
```

## ActivityPub Endpoints

Once running, your ESP32 will expose the following endpoints:

- **WebFinger**: `/.well-known/webfinger?resource=acct:esp32node@{IP_ADDRESS}`
- **Actor Profile**: `/users/esp32node`
- **Inbox**: `/users/esp32node/inbox` (POST)
- **Outbox**: `/users/esp32node/outbox` (GET/POST)
- **Health Check**: `/health`
- **NodeInfo**: `/nodeinfo/2.0`

## Architecture

### Dependency Injection Container

The firmware uses a dependency injection pattern for modularity:

```rust
use feder8_firmware::{EmbeddedContainerBuilder, EmbeddedConfig};

// Create configuration
let config = EmbeddedConfig::new(
    "My ESP32 Node",
    "http://192.168.1.100",
    "esp32user",
    "MyWiFi",
    "password123",
)?;

// Build container with dependencies
let container = EmbeddedContainerBuilder::new()
    .with_config(config)
    .build()?;
```

### Swappable HTTP Client

The HTTP client can be swapped for testing or different transport layers:

```rust
use feder8_firmware::{EspHttpClient, EmbeddedContainerBuilder};

// Use custom HTTP client
let custom_client = Arc::new(EspHttpClient::with_timeout(60)?);
let container = EmbeddedContainerBuilder::new()
    .with_config(config)
    .with_http_client(custom_client)
    .build()?;
```

### Mock Services for Testing

```rust
let container = EmbeddedContainerBuilder::new()
    .with_config(config)
    .with_mocks()  // Use mock services
    .build()?;
```

## Memory Constraints

The firmware is designed for embedded systems with limited memory:

- **Actors**: Limited to 32 followers, 8 public inboxes
- **Strings**: HeaplessString with fixed capacities
- **HTTP**: 4KB request body limit, 8KB response limit
- **Activities**: Simplified structure optimized for memory

## Configuration

### EmbeddedConfig

```rust
pub struct EmbeddedConfig {
    pub server_name: HeaplessString<64>,      // Node display name
    pub server_url: HeaplessString<128>,      // Node URL (http://IP_ADDRESS)
    pub actor_name: HeaplessString<64>,       // Actor username
    pub wifi_ssid: HeaplessString<64>,        // WiFi network name
    pub wifi_password: HeaplessString<64>,    // WiFi password
    pub private_key_pem: Option<HeaplessString<1024>>, // RSA private key
    pub public_key_pem: Option<HeaplessString<512>>,   // RSA public key
}
```

### WiFi Configuration

```rust
let mut wifi_manager = WiFiManager::new(modem, sysloop, nvs)?;
wifi_manager.connect("MyWiFi", "password123")?;
wifi_manager.wait_for_connection(30)?;
```

## ActivityPub Protocol Implementation

### Supported Activities

- **Create**: Post notes/messages
- **Follow**: Send follow requests
- **Accept**: Accept follow requests
- **Undo**: Handle unfollows
- **Announce**: Boost/share posts (basic support)

### WebFinger Discovery

Your ESP32 node can be discovered via WebFinger:

```
GET /.well-known/webfinger?resource=acct:esp32node@192.168.1.100
```

Response:
```json
{
  "subject": "acct:esp32node@192.168.1.100",
  "links": [
    {
      "rel": "self",
      "type": "application/activity+json",
      "href": "http://192.168.1.100/users/esp32node"
    }
  ]
}
```

### Actor Profile

```
GET /users/esp32node
```

Returns the full ActivityPub Actor object with public key for signature verification.

### Sending Activities

```rust
// Send a note
container.send_note("Hello from ESP32!").await?;

// Send follow request
container.send_follow_request("https://mastodon.social/users/alice").await?;

// Accept follow request
container.accept_follow_request(follow_activity).await?;
```

## Development

### Running Tests

```bash
# Run embedded tests
cargo test

# Run with mock services
cargo test --features mock-services
```

### Debugging

Enable verbose logging:

```rust
esp_idf_svc::log::EspLogger::initialize_default();
log::set_max_level(log::LevelFilter::Debug);
```

### Memory Monitoring

The firmware includes built-in memory monitoring:

```bash
# Check free heap via HTTP
curl http://192.168.1.100/health

# Monitor via serial output
cargo espmonitor --port /dev/ttyUSB0
```

## Limitations

- **No Signature Verification**: ActivityPub HTTP signatures not implemented
- **Limited Storage**: No persistent storage for activities or followers
- **Basic Federation**: Simplified ActivityPub implementation
- **No Media Support**: Text-only activities
- **Fixed Limits**: Hardcoded limits for followers and activities

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Troubleshooting

### Common Issues

1. **WiFi Connection Failed**
   - Check SSID and password
   - Ensure 2.4GHz network (ESP32 doesn't support 5GHz)
   - Check signal strength

2. **Memory Issues**
   - Reduce number of followers
   - Simplify activity content
   - Monitor heap usage via `/health` endpoint

3. **HTTP Request Failures**
   - Check network connectivity
   - Verify target server accepts ActivityPub requests
   - Monitor serial output for detailed error messages

### Serial Output

The firmware provides detailed logging:

```
I (12345) esp32-activitypub: Starting ESP32 ActivityPub Node...
I (12456) esp32-activitypub: Connecting to WiFi: MyWiFi
I (15678) esp32-activitypub: WiFi connected! IP: 192.168.1.100
I (15789) esp32-activitypub: ActivityPub server started on port 80
I (15890) esp32-activitypub: Node status - Followers: 0, Free heap: 245760 bytes
```

## Examples

### Basic Usage

```rust
use feder8_firmware::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create configuration
    let config = EmbeddedConfig::new(
        "My ESP32 Node",
        "http://192.168.1.100",
        "esp32bot",
        "MyWiFi",
        "password123",
    )?;

    // Initialize container
    let container = EmbeddedContainerBuilder::new()
        .with_config(config)
        .build()?;

    // Send periodic messages
    loop {
        container.send_note("Hello from ESP32! 🤖").await?;
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
```

### Custom HTTP Client

```rust
struct CustomHttpClient;

#[async_trait]
impl HttpClient for CustomHttpClient {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        // Custom HTTP implementation
        todo!()
    }
}

let container = EmbeddedContainerBuilder::new()
    .with_config(config)
    .with_http_client(Arc::new(CustomHttpClient))
    .build()?;
```

This implementation provides a solid foundation for running ActivityPub nodes on ESP32 devices, enabling IoT devices to participate in decentralized social networks.