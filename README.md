# Fediverse ActivityPub Implementation

This is a basic implementation of the ActivityPub protocol for the Fediverse, allowing you to send and receive messages between different nodes.

## Features

- ✅ ActivityPub core models (Actor, Activity, Note, etc.)
- ✅ WebFinger discovery (RFC 7033)
- ✅ Inbox/Outbox endpoints
- ✅ Basic message exchange (Create activities)
- ✅ HTTP server with proper content types
- ✅ Structured logging

## Quick Start

### 1. Build and Run

```bash
cargo build
cargo run
```

The server will start on `http://localhost:8080` by default.

### 2. Test the Implementation

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

#### Send a Message
```bash
curl -X POST http://localhost:8080/users/alice/inbox \
     -H "Content-Type: application/activity+json" \
     -d '{
       "@context": ["https://www.w3.org/ns/activitystreams"],
       "id": "https://example.com/activities/123",
       "type": "Create",
       "actor": "https://localhost:8080/users/alice",
       "object": {
         "@context": ["https://www.w3.org/ns/activitystreams"],
         "id": "https://example.com/notes/456",
         "type": "Note",
         "attributedTo": "https://localhost:8080/users/alice",
         "content": "Hello, Fediverse!",
         "to": ["https://www.w3.org/ns/activitystreams#Public"],
         "cc": ["https://localhost:8080/users/alice/followers"]
       },
       "to": ["https://www.w3.org/ns/activitystreams#Public"],
       "cc": ["https://localhost:8080/users/alice/followers"]
     }'
```

### 3. Run the Test Script

```bash
# In a separate terminal
cargo run --bin test_message_exchange
```

## Configuration

Set environment variables to customize the server:

```bash
export SERVER_NAME="My Fediverse Node"
export SERVER_URL="http://localhost:8080"
export PORT="8080"
export ACTOR_NAME="alice"
```

## Architecture

### Core Components

1. **Models** (`src/models/`)
   - `Actor`: Represents users/servers
   - `Activity`: Base activity types (Create, Follow, Accept, etc.)
   - `Note`: Basic message content
   - `Collection`: Ordered/unordered collections

2. **Handlers** (`src/handlers/`)
   - `webfinger.rs`: Service discovery endpoint
   - `actor.rs`: Actor profile endpoint
   - `inbox.rs`: Receives incoming activities
   - `outbox.rs`: Manages outgoing activities

3. **Services** (`src/services/`)
   - `delivery.rs`: Handles message delivery to other servers
   - `signature.rs`: HTTP signature verification (simplified)

### ActivityPub Endpoints

- `/.well-known/webfinger` - Service discovery
- `/users/{username}` - Actor profile
- `/users/{username}/inbox` - Receive activities
- `/users/{username}/outbox` - Send activities

## Message Flow

1. **Create a Note**: Send a `Create` activity with a `Note` object
2. **Receive in Inbox**: Other servers POST activities to your inbox
3. **Process Activities**: Handle different activity types (Create, Follow, Accept, etc.)
4. **Deliver to Followers**: Send activities to follower inboxes

## Supported Activity Types

- `Create` - Create a new Note
- `Follow` - Follow another actor
- `Accept` - Accept a Follow request
- `Undo` - Undo previous activities

## Next Steps

To make this a production-ready Fediverse server, you would need to add:

1. **Database Integration**: Store actors, activities, and relationships
2. **HTTP Signatures**: Full RFC 9421 implementation
3. **Content Moderation**: Filter inappropriate content
4. **Rate Limiting**: Prevent spam and abuse
5. **Media Handling**: Support for images, videos, etc.
6. **Federation**: Connect with other Fediverse servers
7. **Web Interface**: User-friendly frontend

## Testing with Other Fediverse Servers

This implementation should be compatible with:
- Mastodon
- Pleroma
- Misskey
- Other ActivityPub-compliant servers

## License

MIT License - feel free to use this as a starting point for your own Fediverse implementation! 