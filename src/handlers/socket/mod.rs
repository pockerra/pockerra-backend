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

use crate::state::app_state::AppState;

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

                if let Some((_, (_player_id, room_id))) = state.connections.remove(&sid) {
                    room::handle_leave(&socket, &state, &room_id);
                }
            }
        }
    });
}
