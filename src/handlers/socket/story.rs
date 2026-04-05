use socketioxide::extract::{Data, SocketRef};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::events::*;
use crate::models::game::Story;
use crate::state::app_state::AppState;

use super::helpers::emit_to_all_in_room;

pub fn register(socket: &SocketRef, state: &Arc<AppState>) {
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
}
