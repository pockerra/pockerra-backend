use dashmap::DashMap;
use tokio::task::JoinHandle;

use crate::models::game::{Story, Vote};
use crate::models::player::Player;
use crate::models::room::Room;

pub struct RoomState {
    pub room: Room,
    pub players: Vec<Player>,
    pub votes: Vec<Vote>,
    pub stories: Vec<Story>,
    pub current_story_id: Option<String>,
    pub timer_handle: Option<JoinHandle<()>>,
}

impl RoomState {
    pub fn current_story(&self) -> Option<Story> {
        self.current_story_id
            .as_ref()
            .and_then(|id| self.stories.iter().find(|s| s.id == *id).cloned())
    }
}

impl Drop for RoomState {
    fn drop(&mut self) {
        if let Some(handle) = self.timer_handle.take() {
            handle.abort();
        }
    }
}

#[derive(Default)]
pub struct AppState {
    /// Room ID → RoomState
    pub rooms: DashMap<String, RoomState>,
    /// Socket ID → (Player ID, Room ID)
    pub connections: DashMap<String, (String, String)>,
}
