use socketioxide::extract::{Data, SocketRef};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::models::events::*;
use crate::models::game::GamePhase;
use crate::models::player::{Player, Role};
use crate::models::room::Room;
use crate::state::app_state::{AppState, RoomState};
use crate::utils::decks::get_deck;

use super::helpers::{broadcast_to_room, build_room_state_response, emit_error};

pub fn register(socket: &SocketRef, state: &Arc<AppState>) {
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

                    broadcast_to_room(
                        &socket,
                        &data.room_id,
                        "room:joined",
                        &RoomJoinedResponse { player },
                    );

                    player_id
                };

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
}

pub fn handle_leave(socket: &SocketRef, state: &AppState, room_id: &str) {
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
