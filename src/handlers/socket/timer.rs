use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use std::sync::Arc;
use tracing::info;

use crate::models::events::*;
use crate::state::app_state::AppState;

use super::helpers::{emit_error, get_player_id, is_facilitator};

pub fn register(socket: &SocketRef, state: &Arc<AppState>, io: &SocketIo) {
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
}
