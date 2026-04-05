use socketioxide::extract::{Data, SocketRef};
use std::sync::Arc;
use tracing::info;

use crate::models::events::*;
use crate::models::game::GamePhase;
use crate::models::player::Role;
use crate::state::app_state::AppState;

use super::helpers::{emit_error, emit_to_all_in_room, get_player_id, is_facilitator};

pub fn register(socket: &SocketRef, state: &Arc<AppState>) {
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
}

pub fn do_reveal(socket: &SocketRef, state: &AppState, room_id: &str) {
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
