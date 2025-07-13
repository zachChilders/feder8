# Feder8 - Dual-Target ActivityPub Implementation

A Rust-based ActivityPub implementation that supports both native (desktop/server) and embedded (ESP32) targets. This project provides a complete foundation for building Fediverse-compatible applications.

## 🚀 Features

### Core ActivityPub Support
- ✅ ActivityPub core models (Actor, Activity, Note, Collection, etc.)
- ✅ WebFinger discovery (RFC 7033)
- ✅ Inbox/Outbox endpoints
- ✅ HTTP signature support
- ✅ Structured logging and tracing
- ✅ Comprehensive test suite

### Dual Target Architecture
- **Native Target**: Full-featured server implementation with SQLite database
- **ESP32 Target**: Embedded implementation for IoT devices and microcontrollers

### Database & Storage
- ✅ SQLite database with migrations
- ✅ Mock database for testing
- ✅ Actor, Activity, Note, and Follow relationship storage

## 🏗️ Project Structure

```
feder8/
├── src/                    # Core library (feder8-core)
│   ├── models/            # ActivityPub data models
│   ├── traits/            # Abstract interfaces
│   ├── config.rs          # Configuration management
│   ├── errors.rs          # Error handling
│   └── native/            # Native target implementation
│       ├── database/      # SQLite database layer
│       ├── handlers/      # HTTP request handlers
│       ├── services/      # Business logic services
│       ├── container.rs   # Dependency injection
│       └── http/          # HTTP client implementation
├── native/                # Native target binary
│   └── src/main.rs        # Server entry point
├── esp32/                 # ESP32 target implementation
│   ├── src/main.rs        # ESP32 entry point
│   ├── qemu_*.sh          # QEMU testing scripts
│   └── test_runner.rs     # ESP32 test framework
├── tests/                 # Integration tests
│   ├── database_tests.rs
│   ├── integration_tests.rs
│   ├── handler_integration_tests.rs
│   └── multinode_tests.rs
└── migrations/            # Database schema migrations
```

## 🛠️ Quick Start

### Prerequisites

- Rust 1.70+ and Cargo
- For ESP32 target: ESP-IDF v5.0+
- For native target: SQLite development libraries

### Building and Running

#### Native Target (Default)

```bash
# Build the core library
cargo build

# Run the native server
cargo run --package feder8-native

# Or run from the native directory
cd native && cargo run
```

The server will start on `http://localhost:8080` by default.

#### ESP32 Target

```bash
# Build for ESP32
cargo build --target xtensa-esp32-none-elf --features esp32

# Run in QEMU for testing
cd esp32 && ./qemu_esp32.sh
```

### Testing

```bash
# Run all tests (native feature enabled by default)
cargo test

# Run specific test suites
cargo test --test integration_tests
cargo test --test database_tests
cargo test --test handler_integration_tests
cargo test --test multinode_tests

# Run ESP32 tests
cd esp32 && ./run_tests.sh
```

## 🔧 Configuration

### Environment Variables

```bash
export SERVER_NAME="My Fediverse Node"
export SERVER_URL="http://localhost:8080"
export PORT="8080"
export ACTOR_NAME="alice"
```

### Database Setup

The native target uses SQLite with automatic migrations:

```bash
# Database will be created automatically at feder8.db
# Migrations are applied on first run
```

## 📡 API Endpoints

### ActivityPub Standard Endpoints

- `GET /.well-known/webfinger` - Service discovery
- `GET /users/{username}` - Actor profile
- `POST /users/{username}/inbox` - Receive activities
- `GET /users/{username}/outbox` - Send activities

### Example Usage

#### Check Actor Profile
```bash
curl -H "Accept: application/activity+json" \
     http://localhost:8080/users/alice
```

#### Test WebFinger Discovery
```bash
curl -H "Accept: application/jrd+json" \
     "http://localhost:8080/.well-known/webfinger?resource=acct:alice@localhost:8080"
```

#### Send a Create Activity
```bash
curl -X POST http://localhost:8080/users/alice/inbox \
     -H "Content-Type: application/activity+json" \
     -d '{
       "@context": ["https://www.w3.org/ns/activitystreams"],
       "id": "https://example.com/activities/123",
       "type": "Create",
       "actor": "https://localhost:8080/users/alice",
       "object": {
         "type": "Note",
         "content": "Hello, Fediverse!",
         "attributedTo": "https://localhost:8080/users/alice"
       },
       "to": ["https://www.w3.org/ns/activitystreams#Public"]
     }'
```

## 🏛️ Architecture

### Core Components

1. **Models** (`src/models/`)
   - `Actor`: Represents users/servers
   - `Activity`: Base activity types (Create, Follow, Accept, etc.)
   - `Note`: Basic message content
   - `Collection`: Ordered/unordered collections

2. **Traits** (`src/traits/`)
   - `HttpClient`: Abstract HTTP client interface
   - `Database`: Abstract database interface
   - `DeliveryService`: Activity delivery service

3. **Native Implementation** (`src/native/`)
   - `handlers/`: HTTP request handlers
   - `database/`: SQLite database implementation
   - `services/`: Business logic services
   - `container.rs`: Dependency injection container

4. **ESP32 Implementation** (`src/esp32/`)
   - `server.rs`: HTTP server wrapper
   - `wifi.rs`: WiFi connectivity management
   - `delivery.rs`: Embedded delivery service

### Supported Activity Types

- `Create` - Create a new Note
- `Follow` - Follow another actor
- `Accept` - Accept a Follow request
- `Undo` - Undo previous activities

## 🧪 Testing

The project includes comprehensive testing:

- **Unit Tests**: Core library functionality
- **Integration Tests**: HTTP endpoint testing
- **Database Tests**: Data persistence testing
- **Multi-node Tests**: Federation simulation
- **ESP32 Tests**: Embedded target validation

### Running Tests

```bash
# All tests
cargo test

# Specific test categories
cargo test --test integration_tests
cargo test --test database_tests

# ESP32 tests
cd esp32 && ./run_tests.sh
```

## 🔌 Federation

This implementation is compatible with:
- Mastodon
- Pleroma
- Misskey
- Other ActivityPub-compliant servers

The multi-node tests demonstrate federation capabilities between multiple instances.

## 🚧 Development Status

### ✅ Completed
- Core ActivityPub models and serialization
- WebFinger discovery
- Inbox/Outbox endpoints
- SQLite database integration
- Comprehensive test suite
- ESP32 embedded support
- HTTP signature framework

### 🔄 In Progress
- Enhanced federation features
- Content moderation
- Rate limiting
- Media handling

### 📋 Planned
- Web interface
- Advanced federation features
- Performance optimizations
- Additional embedded targets

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

This project builds on the ActivityPub specification and aims to provide a solid foundation for Fediverse applications. 