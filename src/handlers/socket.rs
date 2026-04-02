use socketioxide::extract::{Data, SocketRef, State};
use socketioxide::SocketIo;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::models::events::*;
use crate::models::game::{GamePhase, Story};
use crate::models::player::{Player, Role};
use crate::models::room::Room;
use crate::state::app_state::{AppState, RoomState};
use crate::utils::decks::get_deck;

fn emit_error(socket: &SocketRef, message: &str, code: &str) {
    socket
        .emit(
            "error",
            &ErrorResponse {
                message: message.to_string(),
                code: code.to_string(),
            },
        )
        .ok();
}

fn is_facilitator(state: &AppState, room_id: &str, player_id: &str) -> bool {
    state
        .rooms
        .get(room_id)
        .is_some_and(|rs| rs.room.facilitator_id == player_id)
}

fn get_player_id(state: &AppState, socket_id: &str) -> Option<String> {
    state
        .connections
        .get(socket_id)
        .map(|entry| entry.0.clone())
}

fn build_room_state_response(rs: &RoomState) -> RoomStateResponse {
    let current_story = rs.current_story();
    RoomStateResponse {
        room: rs.room.clone(),
        players: rs.players.clone(),
        stories: rs.stories.clone(),
        current_story,
    }
}

fn broadcast_to_room(socket: &SocketRef, room_id: &str, event: &str, data: &impl serde::Serialize) {
    socket.to(room_id.to_string()).emit(event, data).ok();
}

fn emit_to_all_in_room(socket: &SocketRef, room_id: &str, event: &str, data: &impl serde::Serialize) {
    socket.within(room_id.to_string()).emit(event, data).ok();
}

pub async fn on_connect(socket: SocketRef, state: State<Arc<AppState>>, io: SocketIo) {
    let sid = socket.id.to_string();
    info!("Socket connected: {}", sid);

    // ── Room events ──

    socket.on("room:create", {
        let state = state.clone();
        move |socket: SocketRef, Data::<RoomCreatePayload>(data)| {
            let state = state.clone();
            async move {
                let room_id = Uuid::new_v4().to_string()[..8].to_string();
                let player_id = Uuid::new_v4().to_string();
                let settings = data.settings.unwrap_or_default();
                let deck = get_deck(&data.deck_type);
                let display_name = data.display_name.unwrap_or_else(|| "Host".to_string());

                let room = Room {
                    id: room_id.clone(),
                    name: None,
                    deck_type: data.deck_type,
                    deck,
                    facilitator_id: player_id.clone(),
                    phase: GamePhase::Voting,
                    settings,
                };

                let player = Player {
                    id: player_id.clone(),
                    name: display_name,
                    role: Role::Facilitator,
                    avatar: None,
                    has_voted: false,
                };

                let room_state = RoomState {
                    room,
                    players: vec![player],
                    votes: vec![],
                    stories: vec![],
                    current_story_id: None,
                    timer_handle: None,
                };

                state.rooms.insert(room_id.clone(), room_state);
                state
                    .connections
                    .insert(socket.id.to_string(), (player_id, room_id.clone()));

                socket.join(room_id.clone());

                if let Some(rs) = state.rooms.get(&room_id) {
                    socket.emit("room:state", &build_room_state_response(&rs)).ok();
                }

                info!("Room created: {}", room_id);
            }
        }
    });

    socket.on("room:join", {
        let state = state.clone();
        move |socket: SocketRef, Data::<RoomJoinPayload>(data)| {
            let state = state.clone();
            async move {
                // Check if this is a reconnect (player_id matches an existing player in the room)
                let is_reconnect = if let Some(ref pid) = data.player_id {
                    state
                        .rooms
                        .get(&data.room_id)
                        .is_some_and(|rs| rs.players.iter().any(|p| p.id == *pid))
                } else {
                    false
                };

                let player_id = if is_reconnect {
                    data.player_id.unwrap()
                } else {
                    let player_id = Uuid::new_v4().to_string();
                    let role = data.role.unwrap_or(Role::Participant);

                    let player = Player {
                        id: player_id.clone(),
                        name: data.display_name,
                        role,
                        avatar: None,
                        has_voted: false,
                    };

                    let room_exists = {
                        if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                            rs.players.push(player.clone());
                            true
                        } else {
                            false
                        }
                    };

                    if !room_exists {
                        emit_error(&socket, "Room not found", "ROOM_NOT_FOUND");
                        return;
                    }

                    // Broadcast new player to others in room (excludes sender)
                    broadcast_to_room(
                        &socket,
                        &data.room_id,
                        "room:joined",
                        &RoomJoinedResponse { player },
                    );

                    player_id
                };

                // Remove stale connection for this player (from previous socket)
                let stale_sid: Option<String> = state
                    .connections
                    .iter()
                    .find(|entry| entry.value().0 == player_id && entry.value().1 == data.room_id)
                    .map(|entry| entry.key().clone());
                if let Some(sid) = stale_sid {
                    state.connections.remove(&sid);
                }

                state
                    .connections
                    .insert(socket.id.to_string(), (player_id.clone(), data.room_id.clone()));

                socket.join(data.room_id.clone());

                // Send full state to the joining socket
                if let Some(rs) = state.rooms.get(&data.room_id) {
                    socket.emit("room:state", &build_room_state_response(&rs)).ok();
                }

                info!("Player {} {} room: {}", player_id, if is_reconnect { "reconnected to" } else { "joined" }, data.room_id);
            }
        }
    });

    socket.on("room:leave", {
        let state = state.clone();
        move |socket: SocketRef, Data::<RoomLeavePayload>(data)| {
            let state = state.clone();
            async move {
                handle_leave(&socket, &state, &data.room_id);
            }
        }
    });

    // ── Vote events ──

    socket.on("vote:submit", {
        let state = state.clone();
        move |socket: SocketRef, Data::<VoteSubmitPayload>(data)| {
            let state = state.clone();
            async move {
                let player_id = match get_player_id(&state, &socket.id.to_string()) {
                    Some(id) => id,
                    None => return,
                };

                let mut should_auto_reveal = false;

                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    rs.votes.retain(|v| v.player_id != player_id);
                    rs.votes.push(crate::models::game::Vote {
                        player_id: player_id.clone(),
                        value: data.value,
                    });

                    if let Some(p) = rs.players.iter_mut().find(|p| p.id == player_id) {
                        p.has_voted = true;
                    }

                    if rs.room.settings.auto_reveal {
                        let all_voted = rs
                            .players
                            .iter()
                            .filter(|p| p.role != Role::Spectator)
                            .all(|p| p.has_voted);
                        should_auto_reveal = all_voted;
                    }
                }

                let resp = VoteSubmittedResponse {
                    player_id: player_id.clone(),
                };
                emit_to_all_in_room(&socket, &data.room_id, "vote:submitted", &resp);

                if should_auto_reveal {
                    do_reveal(&socket, &state, &data.room_id);
                }
            }
        }
    });

    socket.on("vote:reveal", {
        let state = state.clone();
        move |socket: SocketRef, Data::<VoteRevealPayload>(data)| {
            let state = state.clone();
            async move {
                let player_id = match get_player_id(&state, &socket.id.to_string()) {
                    Some(id) => id,
                    None => return,
                };
                if !is_facilitator(&state, &data.room_id, &player_id) {
                    emit_error(&socket, "Only facilitator can reveal votes", "UNAUTHORIZED");
                    return;
                }
                do_reveal(&socket, &state, &data.room_id);
            }
        }
    });

    socket.on("vote:reset", {
        let state = state.clone();
        move |socket: SocketRef, Data::<VoteResetPayload>(data)| {
            let state = state.clone();
            async move {
                let player_id = match get_player_id(&state, &socket.id.to_string()) {
                    Some(id) => id,
                    None => return,
                };
                if !is_facilitator(&state, &data.room_id, &player_id) {
                    emit_error(&socket, "Only facilitator can reset votes", "UNAUTHORIZED");
                    return;
                }

                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    rs.votes.clear();
                    rs.room.phase = GamePhase::Voting;
                    for p in rs.players.iter_mut() {
                        p.has_voted = false;
                    }
                }

                let empty = serde_json::json!({});
                emit_to_all_in_room(&socket, &data.room_id, "vote:reset", &empty);

                info!("Votes reset in room: {}", data.room_id);
            }
        }
    });

    // ── Story events ──

    socket.on("story:add", {
        let state = state.clone();
        move |socket: SocketRef, Data::<StoryAddPayload>(data)| {
            let state = state.clone();
            async move {
                let story = Story {
                    id: Uuid::new_v4().to_string(),
                    title: data.title,
                    description: data.description,
                    estimate: None,
                };

                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    rs.stories.push(story.clone());
                }

                let resp = StoryAddedResponse { story };
                emit_to_all_in_room(&socket, &data.room_id, "story:added", &resp);
            }
        }
    });

    socket.on("story:select", {
        let state = state.clone();
        move |socket: SocketRef, Data::<StorySelectPayload>(data)| {
            let state = state.clone();
            async move {
                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    rs.current_story_id = Some(data.story_id.clone());
                }

                let resp = StorySelectedResponse {
                    story_id: data.story_id,
                };
                emit_to_all_in_room(&socket, &data.room_id, "story:selected", &resp);
            }
        }
    });

    socket.on("story:update", {
        let state = state.clone();
        move |socket: SocketRef, Data::<StoryUpdatePayload>(data)| {
            let state = state.clone();
            async move {
                let mut updated_story = None;

                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    if let Some(story) = rs.stories.iter_mut().find(|s| s.id == data.story_id) {
                        story.estimate = Some(data.estimate);
                        updated_story = Some(story.clone());
                    }
                }

                if let Some(story) = updated_story {
                    let resp = StoryUpdatedResponse { story };
                    emit_to_all_in_room(&socket, &data.room_id, "story:updated", &resp);
                }
            }
        }
    });

    socket.on("story:delete", {
        let state = state.clone();
        move |socket: SocketRef, Data::<StoryDeletePayload>(data)| {
            let state = state.clone();
            async move {
                let story_id = data.story_id.clone();
                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    rs.stories.retain(|s| s.id != story_id);
                    if rs.current_story_id.as_deref() == Some(story_id.as_str()) {
                        rs.current_story_id = None;
                    }
                }

                let resp = StoryDeletedResponse { story_id };
                emit_to_all_in_room(&socket, &data.room_id, "story:deleted", &resp);
            }
        }
    });

    socket.on("story:import", {
        let state = state.clone();
        move |socket: SocketRef, Data::<StoryImportPayload>(data)| {
            let state = state.clone();
            async move {
                for item in data.stories {
                    let story = Story {
                        id: Uuid::new_v4().to_string(),
                        title: item.title,
                        description: item.description,
                        estimate: None,
                    };

                    if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                        rs.stories.push(story.clone());
                    }

                    let resp = StoryAddedResponse { story };
                    emit_to_all_in_room(&socket, &data.room_id, "story:added", &resp);
                }
            }
        }
    });

    // ── Timer events ──

    socket.on("timer:start", {
        let state = state.clone();
        let io = io.clone();
        move |socket: SocketRef, Data::<TimerStartPayload>(data)| {
            let state = state.clone();
            let io = io.clone();
            async move {
                let player_id = match get_player_id(&state, &socket.id.to_string()) {
                    Some(id) => id,
                    None => return,
                };
                if !is_facilitator(&state, &data.room_id, &player_id) {
                    emit_error(&socket, "Only facilitator can start timer", "UNAUTHORIZED");
                    return;
                }

                // Cancel existing timer
                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    if let Some(handle) = rs.timer_handle.take() {
                        handle.abort();
                    }
                }

                let room_id = data.room_id.clone();
                let duration = data.duration;
                let state_clone = state.clone();

                let handle = tokio::spawn(async move {
                    for remaining in (0..=duration).rev() {
                        let _ = io
                            .within(room_id.clone())
                            .emit("timer:tick", &TimerTickResponse { remaining });

                        if remaining == 0 {
                            let _ = io
                                .within(room_id.clone())
                                .emit("timer:expired", &serde_json::json!({}));
                            break;
                        }

                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }

                    if let Some(mut rs) = state_clone.rooms.get_mut(&room_id) {
                        rs.timer_handle = None;
                    }
                });

                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    rs.timer_handle = Some(handle);
                }

                info!("Timer started in room: {}", data.room_id);
            }
        }
    });

    socket.on("timer:stop", {
        let state = state.clone();
        move |socket: SocketRef, Data::<TimerStopPayload>(data)| {
            let state = state.clone();
            async move {
                let player_id = match get_player_id(&state, &socket.id.to_string()) {
                    Some(id) => id,
                    None => return,
                };
                if !is_facilitator(&state, &data.room_id, &player_id) {
                    emit_error(&socket, "Only facilitator can stop timer", "UNAUTHORIZED");
                    return;
                }

                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    if let Some(handle) = rs.timer_handle.take() {
                        handle.abort();
                    }
                }

                info!("Timer stopped in room: {}", data.room_id);
            }
        }
    });

    // ── Player management events ──

    socket.on("player:role", {
        let state = state.clone();
        move |socket: SocketRef, Data::<PlayerRolePayload>(data)| {
            let state = state.clone();
            async move {
                let caller_id = match get_player_id(&state, &socket.id.to_string()) {
                    Some(id) => id,
                    None => return,
                };
                if !is_facilitator(&state, &data.room_id, &caller_id) {
                    emit_error(&socket, "Only facilitator can change roles", "UNAUTHORIZED");
                    return;
                }

                let mut updated_player = None;
                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    if let Some(p) = rs.players.iter_mut().find(|p| p.id == data.player_id) {
                        p.role = data.role;
                        updated_player = Some(p.clone());
                    }
                }

                if let Some(player) = updated_player {
                    let resp = PlayerUpdatedResponse { player };
                    emit_to_all_in_room(&socket, &data.room_id, "player:updated", &resp);
                }
            }
        }
    });

    socket.on("player:kick", {
        let state = state.clone();
        move |socket: SocketRef, Data::<PlayerKickPayload>(data)| {
            let state = state.clone();
            async move {
                let caller_id = match get_player_id(&state, &socket.id.to_string()) {
                    Some(id) => id,
                    None => return,
                };
                if !is_facilitator(&state, &data.room_id, &caller_id) {
                    emit_error(&socket, "Only facilitator can kick players", "UNAUTHORIZED");
                    return;
                }

                if let Some(mut rs) = state.rooms.get_mut(&data.room_id) {
                    rs.players.retain(|p| p.id != data.player_id);
                    rs.votes.retain(|v| v.player_id != data.player_id);
                }

                // Remove from connections
                let socket_to_remove: Option<String> = state
                    .connections
                    .iter()
                    .find(|entry| entry.value().0 == data.player_id && entry.value().1 == data.room_id)
                    .map(|entry| entry.key().clone());

                if let Some(sid) = socket_to_remove {
                    state.connections.remove(&sid);
                }

                let resp = RoomLeftResponse {
                    player_id: data.player_id,
                };
                emit_to_all_in_room(&socket, &data.room_id, "room:left", &resp);
            }
        }
    });

    // ── Disconnect ──

    socket.on_disconnect({
        let state = state.clone();
        move |socket: SocketRef| {
            let state = state.clone();
            async move {
                let sid = socket.id.to_string();
                info!("Socket disconnected: {}", sid);

                if let Some((_, (_player_id, room_id))) = state.connections.remove(&sid) {
                    handle_leave(&socket, &state, &room_id);
                }
            }
        }
    });
}

fn handle_leave(socket: &SocketRef, state: &AppState, room_id: &str) {
    let sid = socket.id.to_string();

    let player_id = state
        .connections
        .get(&sid)
        .map(|entry| entry.0.clone());

    let player_id = match player_id {
        Some(id) => id,
        None => return,
    };

    let mut room_empty = false;

    if let Some(mut rs) = state.rooms.get_mut(room_id) {
        rs.players.retain(|p| p.id != player_id);
        rs.votes.retain(|v| v.player_id != player_id);
        room_empty = rs.players.is_empty();
    }

    state.connections.remove(&sid);
    socket.leave(room_id.to_string());

    if room_empty {
        state.rooms.remove(room_id);
        info!("Room removed (empty): {}", room_id);
    } else {
        let resp = RoomLeftResponse {
            player_id: player_id.clone(),
        };
        broadcast_to_room(socket, room_id, "room:left", &resp);
        info!("Player {} left room {}", player_id, room_id);
    }
}

fn do_reveal(socket: &SocketRef, state: &AppState, room_id: &str) {
    let votes = if let Some(mut rs) = state.rooms.get_mut(room_id) {
        rs.room.phase = GamePhase::Revealed;
        rs.votes.clone()
    } else {
        return;
    };

    let response = VoteRevealedResponse { votes };
    emit_to_all_in_room(socket, room_id, "vote:revealed", &response);

    info!("Votes revealed in room: {}", room_id);
}
