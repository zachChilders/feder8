# ESP32 ActivityPub Node Firmware

This firmware implements a complete ActivityPub node that runs on ESP32 microcontrollers. It features a dependency injection architecture that allows swapping implementations for different embedded environments.

## Features

- **Full ActivityPub Protocol Support**: Implements core ActivityPub activities (Create, Follow, Accept, etc.)
- **Embedded-Optimized**: Uses heapless data structures and memory-constrained designs
- **Dependency Injection**: Pluggable HTTP clients and delivery services
- **WiFi Management**: Automatic connection and reconnection handling
- **LED Status Indicators**: Visual feedback for connection and activity status
- **Async/Await Support**: Modern async Rust for network operations

## Architecture

The firmware is built with a clean architecture using dependency injection:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Main Application                          │
│                     (ActivityPubNode)                           │
└─────────────────────────┬───────────────────────────────────────┘
                         │
┌─────────────────────────▼───────────────────────────────────────┐
│                  Dependency Container                            │
│                 (EmbeddedContainer)                              │
└─────────────┬─────────────────────┬─────────────────────────────┘
             │                     │
┌────────────▼─────────────┐ ┌──────▼──────────────────────────────┐
│    HTTP Client Trait     │ │      Delivery Service Trait        │
│   (EspHttpClient)        │ │   (EmbeddedDeliveryService)         │
└──────────────────────────┘ └─────────────────────────────────────┘
```

### Key Components

1. **Models** (`src/models.rs`):
   - `Actor`: ActivityPub actor representation
   - `Activity`: ActivityPub activity objects
   - `EmbeddedConfig`: Configuration for embedded systems

2. **HTTP Client** (`src/http.rs`):
   - `HttpClient` trait for pluggable HTTP implementations
   - `EspHttpClient` for ESP-IDF HTTP client
   - Memory-constrained request/response handling

3. **Delivery Service** (`src/delivery.rs`):
   - `DeliveryService` trait for pluggable delivery implementations
   - `EmbeddedDeliveryService` for ActivityPub message delivery
   - `MockDeliveryService` for testing

4. **Container** (`src/container.rs`):
   - `EmbeddedContainer` for dependency injection
   - `EmbeddedContainerBuilder` for flexible configuration
   - Follower and public inbox management

5. **WiFi Management** (`src/wifi.rs`):
   - `WiFiManager` for connection handling
   - `WiFiMonitor` for auto-reconnection
   - `WiFiStatus` for status monitoring

## Configuration

Update the configuration constants in `src/main.rs`:

```rust
const WIFI_SSID: &str = "YourWiFiNetwork";
const WIFI_PASSWORD: &str = "YourWiFiPassword";
const SERVER_NAME: &str = "ESP32 ActivityPub Node";
const SERVER_URL: &str = "https://esp32.local";
const ACTOR_NAME: &str = "esp32bot";
```

## Memory Constraints

The firmware is designed for embedded systems with limited memory:

- **Follower List**: Maximum 32 followers
- **Public Inboxes**: Maximum 8 public relay inboxes
- **HTTP Requests**: Limited to 4KB body size
- **HTTP Responses**: Limited to 8KB body size
- **String Fields**: Various limits (64-512 characters) based on usage

## Building and Flashing

### Prerequisites

1. Install Rust with ESP32 support:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup install nightly
   rustup component add rust-src --toolchain nightly
   ```

2. Install ESP-IDF:
   ```bash
   git clone -b v4.4.2 --recursive https://github.com/espressif/esp-idf.git
   cd esp-idf
   ./install.sh
   . export.sh
   ```

3. Install cargo-espflash:
   ```bash
   cargo install cargo-espflash
   ```

### Building

```bash
cd src/firmware
cargo build --release
```

### Flashing

```bash
cargo espflash flash --release --monitor
```

## Usage

Once flashed and connected to WiFi, the ESP32 will:

1. **Connect to WiFi**: Uses configured SSID/password
2. **Initialize ActivityPub Node**: Creates actor and sets up services
3. **LED Status Indicators**:
   - Solid ON: Connected and active
   - Blinking: No WiFi connection
4. **Send Periodic Messages**: Posts a message every 60 seconds
5. **Log Activity**: Outputs status to serial monitor

## API

### Adding Followers

```rust
// Add a follower's inbox URL
node.add_follower("https://mastodon.social/users/alice/inbox")?;
```

### Sending Follow Requests

```rust
// Send a follow request to another actor
node.send_follow_request("https://mastodon.social/users/bob").await?;
```

### Adding Public Relays

```rust
// Add a public relay for broader distribution
node.add_public_relay("https://relay.fediverse.org/inbox")?;
```

### Sending Notes

```rust
// Send a note to all followers
node.send_note("Hello from ESP32!").await?;
```

## Dependency Injection Examples

### Custom HTTP Client

```rust
use std::sync::Arc;

// Create a custom HTTP client
let custom_client: Arc<dyn HttpClient> = Arc::new(MyCustomHttpClient::new());

// Use it in the container
let container = EmbeddedContainerBuilder::new()
    .with_config(config)
    .with_http_client(custom_client)
    .build()?;
```

### Mock Services for Testing

```rust
// Create container with mock services
let container = EmbeddedContainerBuilder::new()
    .with_config(config)
    .with_mocks()
    .build()?;
```

### Custom Delivery Service

```rust
use std::sync::Arc;

// Create a custom delivery service
let custom_delivery: Arc<dyn DeliveryService> = Arc::new(MyCustomDeliveryService::new());

// Use it in the container
let container = EmbeddedContainerBuilder::new()
    .with_config(config)
    .with_delivery_service(custom_delivery)
    .build()?;
```

## Testing

Run tests with:

```bash
cargo test
```

Note: Some tests require mocking of ESP-IDF components and may not run on host systems.

## Limitations

- **No HTTP Server**: Currently only sends ActivityPub messages (no inbox for receiving)
- **No Cryptographic Signatures**: HTTP signatures not yet implemented
- **No WebFinger**: Actor discovery is simplified
- **No Database**: All data is stored in memory
- **No Media Attachments**: Only text notes supported

## Future Enhancements

1. **HTTP Server**: Add inbox endpoint for receiving ActivityPub messages
2. **Cryptographic Signatures**: Implement HTTP signatures for security
3. **WebFinger Support**: Add proper actor discovery
4. **Persistent Storage**: Use ESP32's flash for data persistence
5. **Media Support**: Add support for images and other media types
6. **Better Error Handling**: More robust error recovery and logging

## Contributing

When adding new features:

1. **Follow the dependency injection pattern**
2. **Respect memory constraints**
3. **Add comprehensive tests**
4. **Update documentation**
5. **Consider embedded limitations**

## License

This project is part of the feder8 ActivityPub implementation and follows the same license terms.