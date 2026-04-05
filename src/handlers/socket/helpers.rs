use socketioxide::extract::SocketRef;

use crate::models::events::{ErrorResponse, RoomStateResponse};
use crate::state::app_state::{AppState, RoomState};

pub fn emit_error(socket: &SocketRef, message: &str, code: &str) {
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

pub fn is_facilitator(state: &AppState, room_id: &str, player_id: &str) -> bool {
    state
        .rooms
        .get(room_id)
        .is_some_and(|rs| rs.room.facilitator_id == player_id)
}

pub fn get_player_id(state: &AppState, socket_id: &str) -> Option<String> {
    state
        .connections
        .get(socket_id)
        .map(|entry| entry.0.clone())
}

pub fn build_room_state_response(rs: &RoomState) -> RoomStateResponse {
    let current_story = rs.current_story();
    RoomStateResponse {
        room: rs.room.clone(),
        players: rs.players.clone(),
        stories: rs.stories.clone(),
        current_story,
        votes: rs.votes.clone(),
    }
}

pub fn broadcast_to_room(socket: &SocketRef, room_id: &str, event: &str, data: &impl serde::Serialize) {
    socket.to(room_id.to_string()).emit(event, data).ok();
}

pub fn emit_to_all_in_room(socket: &SocketRef, room_id: &str, event: &str, data: &impl serde::Serialize) {
    socket.within(room_id.to_string()).emit(event, data).ok();
}
