use socketioxide::extract::{Data, SocketRef};
use std::sync::Arc;

use crate::models::events::*;
use crate::state::app_state::AppState;

use super::helpers::{emit_error, emit_to_all_in_room, get_player_id, is_facilitator};

pub fn register(socket: &SocketRef, state: &Arc<AppState>) {
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
}
