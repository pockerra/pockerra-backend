mod helpers;
mod player;
mod room;
mod story;
mod timer;
mod vote;

use socketioxide::extract::{SocketRef, State};
use socketioxide::SocketIo;
use std::sync::Arc;
use tracing::info;

use crate::models::events::PlayerUpdatedResponse;
use crate::state::app_state::AppState;

use self::helpers::emit_to_all_in_room;

pub async fn on_connect(socket: SocketRef, state: State<Arc<AppState>>, io: SocketIo) {
    let sid = socket.id.to_string();
    info!("Socket connected: {}", sid);

    room::register(&socket, &state);
    vote::register(&socket, &state);
    story::register(&socket, &state);
    timer::register(&socket, &state, &io);
    player::register(&socket, &state);

    socket.on_disconnect({
        let state = state.clone();
        move |socket: SocketRef| {
            let state = state.clone();
            async move {
                let sid = socket.id.to_string();
                info!("Socket disconnected: {}", sid);

                if let Some((_, (player_id, room_id))) = state.connections.remove(&sid) {
                    // Mark player as disconnected instead of removing them
                    let mut all_disconnected = false;
                    let mut player_clone = None;

                    if let Some(mut rs) = state.rooms.get_mut(&room_id) {
                        if let Some(p) = rs.players.iter_mut().find(|p| p.id == player_id) {
                            p.connected = false;
                            player_clone = Some(p.clone());
                        }
                        all_disconnected = rs.players.iter().all(|p| !p.connected);
                    }

                    if all_disconnected {
                        state.rooms.remove(&room_id);
                        info!("Room removed (all disconnected): {}", room_id);
                    } else if let Some(player) = player_clone {
                        // Broadcast updated player so clients hide them
                        emit_to_all_in_room(&socket, &room_id, "player:updated", &PlayerUpdatedResponse { player });
                        info!("Player {} disconnected from room {}", player_id, room_id);
                    }
                }
            }
        }
    });
}
