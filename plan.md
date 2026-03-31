# Pockerra — Planning Poker Backend (Rust)

## Context
Backend server for the Pockerra Planning Poker app. Communicates with the Svelte 5 frontend via REST API + WebSocket (Socket.io protocol). Handles room management, real-time voting, story tracking, timer broadcasting, and player management.

Use **context7** to get latest docs for all crates and libraries before implementation.

---

## Tech Stack

| Category | Choice | Why |
|---|---|---|
| Language | Rust (latest stable) | Performance, safety, low resource usage |
| HTTP Framework | `axum` | Async, ergonomic, tower-based ecosystem |
| WebSocket | `socketioxide` | Socket.io server for Rust — compatible with frontend's `socket.io-client` |
| Async Runtime | `tokio` | Industry standard async runtime for Rust |
| Serialization | `serde` + `serde_json` | JSON serialization/deserialization |
| State | `DashMap` or `tokio::sync::RwLock<HashMap>` | Concurrent in-memory state (no DB for now) |
| IDs | `uuid` | Unique room/player/story IDs |
| Logging | `tracing` + `tracing-subscriber` | Structured async-aware logging |
| CORS | `tower-http` | CORS middleware for cross-origin requests |
| Config | `dotenvy` | Environment variable loading |
| Error Handling | `thiserror` + `anyhow` | Ergonomic error types |

---

## Project Structure

```
src/
├── main.rs                  # Entry point, server setup, routing
├── config.rs                # Environment config (port, cors origins, etc.)
├── models/
│   ├── mod.rs
│   ├── room.rs              # Room, RoomSettings structs
│   ├── player.rs            # Player struct, Role enum
│   ├── game.rs              # GamePhase, Vote, Story, DeckType
│   └── events.rs            # Socket.io event payload structs (client→server, server→client)
├── state/
│   ├── mod.rs
│   └── app_state.rs         # AppState: rooms map, player-to-room mapping
├── handlers/
│   ├── mod.rs
│   ├── http.rs              # REST API handlers (create room, get room)
│   └── socket.rs            # Socket.io event handlers (all WS events)
├── services/
│   ├── mod.rs
│   ├── room_service.rs      # Room lifecycle (create, join, leave, cleanup)
│   ├── vote_service.rs      # Vote submission, reveal, reset logic
│   ├── story_service.rs     # Story CRUD, selection, import
│   ├── timer_service.rs     # Timer management (spawn tick tasks)
│   └── player_service.rs    # Role changes, kick, permission checks
├── errors.rs                # Custom error types
└── utils/
    ├── mod.rs
    └── decks.rs             # Predefined deck definitions (must match frontend)
```

Each module should have a corresponding test file or `#[cfg(test)]` module for unit testing.

---

## Type Definitions (Rust)

These must serialize to JSON matching the frontend TypeScript types exactly.

```rust
use serde::{Deserialize, Serialize};

// --- Enums ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Facilitator,
    Participant,
    Spectator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GamePhase {
    Waiting,
    Voting,
    Revealed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DeckType {
    Fibonacci,
    Tshirt,
    Powers2,
    Custom,
}

// --- Value type (string or number) ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CardValue {
    Number(f64),
    Text(String),
}

// --- Core structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: String,
    pub name: String,
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    pub has_voted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
    pub player_id: String,
    pub value: CardValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Story {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomSettings {
    pub timer_enabled: bool,
    pub timer_duration: u32,  // seconds
    pub auto_reveal: bool,
    pub allow_spectators: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub deck_type: DeckType,
    pub deck: Vec<CardValue>,
    pub facilitator_id: String,
    pub phase: GamePhase,
    pub settings: RoomSettings,
}
```

---

## REST API Endpoints

| Method | Path | Request Body | Response | Description |
|---|---|---|---|---|
| `POST` | `/api/rooms` | `{ deckType, settings? }` | `{ roomId: string }` | Create a new room |
| `GET` | `/api/rooms/:roomId` | — | `{ room: Room }` | Get room info (exists check) |

### CORS
Allow origin: `http://localhost:5173` (dev) and production frontend URL.

---

## WebSocket Events (Socket.io Protocol)

The frontend uses `socket.io-client` with transports `['websocket', 'polling']`. The backend must implement Socket.io protocol via `socketioxide`.

### Client → Server

| Event | Payload | Handler Logic |
|---|---|---|
| `room:create` | `{ deckType: DeckType, settings: RoomSettings }` | Create room, add creator as facilitator, join socket room, emit `room:state` |
| `room:join` | `{ roomId: string, displayName: string, role: Role }` | Add player to room, join socket room, broadcast `room:joined`, emit `room:state` to joiner |
| `room:leave` | `{ roomId: string }` | Remove player, broadcast `room:left`, cleanup empty rooms |
| `vote:submit` | `{ roomId: string, value: CardValue }` | Store vote, broadcast `vote:submitted` (no value!), check auto-reveal |
| `vote:reveal` | `{ roomId: string }` | **Facilitator only**. Set phase=revealed, broadcast `vote:revealed` with all votes |
| `vote:reset` | `{ roomId: string }` | **Facilitator only**. Clear votes, set phase=voting, broadcast `vote:reset` |
| `story:add` | `{ roomId: string, title: string, description?: string }` | Create story, broadcast `story:added` |
| `story:select` | `{ roomId: string, storyId: string }` | Set current story, broadcast `story:selected` |
| `story:update` | `{ roomId: string, storyId: string, estimate: string }` | Update story estimate, broadcast `story:updated` |
| `story:delete` | `{ roomId: string, storyId: string }` | Remove story, broadcast `story:deleted` |
| `story:import` | `{ roomId: string, stories: [{title, description?}] }` | Bulk create stories, broadcast `story:added` for each |
| `timer:start` | `{ roomId: string, duration: u32 }` | **Facilitator only**. Spawn timer task, broadcast `timer:tick` every second |
| `timer:stop` | `{ roomId: string }` | **Facilitator only**. Cancel timer task |
| `player:role` | `{ roomId: string, playerId: string, role: Role }` | **Facilitator only**. Update role, broadcast `player:updated` |
| `player:kick` | `{ roomId: string, playerId: string }` | **Facilitator only**. Remove player, emit `room:left` to kicked player, broadcast `room:left` |

### Server → Client

| Event | Payload | When |
|---|---|---|
| `room:state` | `{ room: Room, players: Player[], stories: Story[], currentStory?: Story }` | On join, full state sync |
| `room:joined` | `{ player: Player }` | New player joins (broadcast to others) |
| `room:left` | `{ playerId: string }` | Player leaves or is kicked |
| `vote:submitted` | `{ playerId: string }` | Someone voted — **do NOT include vote value** |
| `vote:revealed` | `{ votes: Vote[] }` | Facilitator reveals — include all vote values |
| `vote:reset` | `{}` | New round started |
| `story:added` | `{ story: Story }` | Story added to queue |
| `story:selected` | `{ storyId: string }` | Active story changed |
| `story:updated` | `{ story: Story }` | Story estimate saved |
| `story:deleted` | `{ storyId: string }` | Story removed |
| `timer:tick` | `{ remaining: u32 }` | Every second during countdown |
| `timer:expired` | `{}` | Timer reached zero |
| `player:updated` | `{ player: Player }` | Player role changed |
| `error` | `{ message: string, code: string }` | Error notification |

---

## Deck Definitions (must match frontend)

```rust
pub fn get_deck(deck_type: &DeckType) -> Vec<CardValue> {
    match deck_type {
        DeckType::Fibonacci => vec![
            num(0.0), num(1.0), num(2.0), num(3.0), num(5.0), num(8.0),
            num(13.0), num(21.0), num(34.0), num(55.0), num(89.0),
            text("?"), text("☕"),
        ],
        DeckType::Tshirt => vec![
            text("XS"), text("S"), text("M"), text("L"), text("XL"), text("XXL"),
            text("?"), text("☕"),
        ],
        DeckType::Powers2 => vec![
            num(0.0), num(1.0), num(2.0), num(4.0), num(8.0), num(16.0),
            num(32.0), num(64.0), text("?"), text("☕"),
        ],
        DeckType::Custom => vec![],
    }
}
```

---

## App State Design

```rust
pub struct AppState {
    /// Room ID → RoomState
    pub rooms: DashMap<String, RoomState>,
    /// Socket ID → (Player ID, Room ID) — for disconnect cleanup
    pub connections: DashMap<String, (String, String)>,
}

pub struct RoomState {
    pub room: Room,
    pub players: Vec<Player>,
    pub votes: Vec<Vote>,
    pub stories: Vec<Story>,
    pub current_story_id: Option<String>,
    pub timer_handle: Option<tokio::task::JoinHandle<()>>,
}
```

---

## Implementation Phases

### Phase 1: Project Setup & Core Types
- Initialize Cargo project with workspace if needed
- Add dependencies to `Cargo.toml`
- Define all model structs/enums with serde derives
- Set up `AppState` with `DashMap`
- Configure `tracing` logging
- Basic `main.rs` with axum server + socketioxide setup
- CORS middleware via `tower-http`
- **Testable**: Server starts, responds to health check, Socket.io handshake works

### Phase 2: REST API + Room Creation via WebSocket
- `POST /api/rooms` — create room, return room ID
- `GET /api/rooms/:roomId` — check room exists
- `room:create` socket event — create room + add facilitator
- `room:join` — add player, broadcast, send state
- `room:leave` — remove player, broadcast, cleanup empty rooms
- Handle `disconnect` — auto-leave room
- **Testable**: Frontend can create room, join from another tab, see player list update

### Phase 3: Core Voting Flow
- `vote:submit` — store vote, broadcast `vote:submitted` (no value), check auto-reveal
- `vote:reveal` — facilitator only, broadcast all votes
- `vote:reset` — facilitator only, clear votes, reset phase
- Auto-reveal logic: if `settings.autoReveal == true` and all participants voted
- **Testable**: Full voting cycle works from frontend: pick card → reveal → new round

### Phase 4: Story Management
- `story:add` — create with UUID, broadcast
- `story:select` — set current, broadcast
- `story:update` — save estimate, broadcast
- `story:delete` — remove, broadcast
- `story:import` — bulk create, broadcast each
- **Testable**: Add stories, estimate them, navigate between stories

### Phase 5: Timer & Player Management
- `timer:start` — spawn tokio task that emits `timer:tick` every second
- `timer:stop` — abort timer task
- `timer:expired` — emit when timer reaches 0, optionally auto-reveal
- `player:role` — facilitator changes player role
- `player:kick` — facilitator removes player, disconnect their socket
- **Testable**: Timer counts down in frontend, facilitator can manage players

### Phase 6: Robustness & Edge Cases
- Reconnection handling (player reconnects with same ID)
- Room cleanup (remove stale rooms after inactivity)
- Rate limiting on events
- Input validation on all payloads
- Graceful shutdown
- **Testable**: Disconnect/reconnect maintains state, stale rooms get cleaned up

### Phase 7: Production Readiness
- Environment-based configuration (port, CORS origins, log level)
- Health check endpoint (`GET /health`)
- Metrics endpoint (optional)
- Docker support (`Dockerfile`)
- CI pipeline config
- Load testing with multiple concurrent rooms
- **Testable**: Deploy in container, handle 100+ concurrent connections

---

## Key Implementation Rules

1. **Vote secrecy**: Never broadcast vote values until `vote:reveal`. Only send `vote:submitted` with `playerId` (no value).

2. **Facilitator-only events**: `vote:reveal`, `vote:reset`, `timer:start`, `timer:stop`, `player:role`, `player:kick` must verify the sender is the room's facilitator. Emit `error` event if unauthorized.

3. **Socket.io rooms**: Use socketioxide's room mechanism — when a player joins, add their socket to a room named by the room ID. All broadcasts go to that room.

4. **Disconnect cleanup**: On socket disconnect, look up the player via `connections` map, remove from room, broadcast `room:left`. If room is empty, remove it.

5. **Auto-reveal**: After every `vote:submit`, check if `settings.autoReveal` is true and all non-spectator players have voted. If so, auto-trigger reveal.

6. **Timer isolation**: Each timer runs as a spawned tokio task. Store the `JoinHandle` in `RoomState` so it can be aborted on `timer:stop` or room cleanup.

7. **JSON field naming**: All structs use `#[serde(rename_all = "camelCase")]` to match the frontend's JavaScript naming convention.

---

## Testing Strategy

1. **Unit tests**: Each service module has `#[cfg(test)]` tests for business logic (vote counting, auto-reveal, facilitator checks, deck generation)
2. **Integration tests**: Use `axum::test` and socketioxide test utilities to verify full event flows
3. **Manual testing**: Connect the Svelte frontend at `http://localhost:5173` to the backend at `http://localhost:3000`, test full flow in multiple browser tabs
4. **Type checking**: `cargo check` — ensure no compile errors
5. **Linting**: `cargo clippy` — catch common issues

---

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `PORT` | `3000` | Server port |
| `CORS_ORIGIN` | `http://localhost:5173` | Allowed CORS origin(s) |
| `RUST_LOG` | `info` | Log level filter |
| `ROOM_TTL_SECS` | `3600` | Seconds before empty rooms are cleaned up |

---

## Cargo.toml Dependencies

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
socketioxide = { version = "0.15", features = ["state"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors"] }
dashmap = "6"
uuid = { version = "1", features = ["v4"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
thiserror = "2"
anyhow = "1"
```

> **Important**: Always use context7 to verify latest crate versions before adding to `Cargo.toml`.
