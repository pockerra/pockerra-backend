# Pockerra Backend

Real-time Planning Poker backend built with Rust. Powers collaborative estimation sessions via REST API and Socket.io WebSocket events.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| HTTP Framework | Axum 0.8 |
| WebSocket | Socketioxide 0.15 (Socket.io protocol) |
| Async Runtime | Tokio |
| State Management | DashMap (concurrent hashmap) |
| Serialization | Serde (camelCase JSON) |
| Logging | tracing + tracing-subscriber |

## Getting Started

### Prerequisites

- Rust (latest stable)

### Run

```sh
# Clone and enter the directory
cd pockerra-backend

# Optional: configure via .env
echo 'PORT=3000' > .env
echo 'CORS_ORIGIN=http://localhost:5173' >> .env

# Run
cargo run
```

The server starts at `http://localhost:3000` by default.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | Server listen port |
| `CORS_ORIGIN` | `http://localhost:5173` | Allowed CORS origin (Svelte dev server) |
| `RUST_LOG` | `info` | Log level filter |

## Project Structure

```
src/
├── main.rs              # Server setup, routes, middleware
├── config.rs            # Environment config loader
├── errors.rs            # Error types
├── models/
│   ├── game.rs          # GamePhase, DeckType, CardValue, Vote, Story
│   ├── player.rs        # Player, Role
│   ├── room.rs          # Room, RoomSettings
│   └── events.rs        # All Socket.io event payloads
├── state/
│   └── app_state.rs     # AppState (DashMap<RoomId, RoomState>)
├── handlers/
│   ├── http.rs          # REST API handlers
│   └── socket.rs        # Socket.io event handlers
├── services/
│   └── mod.rs           # Service layer (WIP)
└── utils/
    └── decks.rs         # Predefined card decks
```

## REST API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `POST` | `/api/rooms` | Create a room |
| `GET` | `/api/rooms/:roomId` | Get room info |

### Create Room

```sh
curl -X POST http://localhost:3000/api/rooms \
  -H 'Content-Type: application/json' \
  -d '{"deckType": "fibonacci"}'
```

```json
{ "roomId": "a1b2c3d4" }
```

## WebSocket Events (Socket.io)

### Client -> Server

| Event | Payload | Description |
|-------|---------|-------------|
| `room:create` | `{ deckType, settings? }` | Create new room |
| `room:join` | `{ roomId, displayName, role }` | Join a room |
| `room:leave` | `{ roomId }` | Leave room |
| `vote:submit` | `{ roomId, value }` | Submit vote |
| `vote:reveal` | `{ roomId }` | Reveal all votes (facilitator only) |
| `vote:reset` | `{ roomId }` | Reset round (facilitator only) |
| `story:add` | `{ roomId, title, description? }` | Add story |
| `story:select` | `{ roomId, storyId }` | Set active story |
| `story:update` | `{ roomId, storyId, estimate }` | Save estimate |
| `story:delete` | `{ roomId, storyId }` | Remove story |
| `story:import` | `{ roomId, stories[] }` | Bulk import stories |
| `timer:start` | `{ roomId, duration }` | Start countdown |
| `timer:stop` | `{ roomId }` | Stop timer |
| `player:role` | `{ roomId, playerId, role }` | Change player role (facilitator only) |
| `player:kick` | `{ roomId, playerId }` | Kick player (facilitator only) |

### Server -> Client

| Event | Payload | Description |
|-------|---------|-------------|
| `room:state` | `{ room, players, stories, currentStory? }` | Full state sync |
| `room:joined` | `{ player }` | Player joined |
| `room:left` | `{ playerId }` | Player left |
| `vote:submitted` | `{ playerId }` | Someone voted (value hidden) |
| `vote:revealed` | `{ votes[] }` | All votes revealed |
| `vote:reset` | `{}` | Votes cleared |
| `story:added` | `{ story }` | Story added |
| `story:selected` | `{ storyId }` | Active story changed |
| `story:updated` | `{ story }` | Story updated |
| `story:deleted` | `{ storyId }` | Story removed |
| `timer:tick` | `{ remaining }` | Timer countdown |
| `timer:expired` | `{}` | Timer finished |
| `player:updated` | `{ player }` | Player info changed |
| `error` | `{ message, code }` | Error |

## Card Decks

| Deck | Values |
|------|--------|
| Fibonacci | 0, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, ?, ☕ |
| T-Shirt | XS, S, M, L, XL, XXL, ?, ☕ |
| Powers of 2 | 0, 1, 2, 4, 8, 16, 32, 64, ?, ☕ |
| Custom | User-defined |

## Key Design Decisions

- **Vote secrecy**: Vote values are never broadcast until the facilitator triggers reveal
- **Auto-reveal**: When all non-spectator players have voted, votes can auto-reveal (if enabled in settings)
- **Facilitator authorization**: Role/kick/reveal/reset operations are gated to the room facilitator
- **Concurrent state**: DashMap allows lock-free concurrent access to room state
- **Disconnect cleanup**: Players are automatically removed on socket disconnect; empty rooms are cleaned up
- **Timer isolation**: Each timer runs as a separate Tokio task with an abortable handle
