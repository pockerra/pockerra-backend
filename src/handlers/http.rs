use axum::extract::{Path, State as AxumState};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::game::GamePhase;
use crate::models::room::{Room, RoomSettings};
use crate::state::app_state::{AppState, RoomState};
use crate::utils::decks::get_deck;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomRequest {
    pub deck_type: crate::models::game::DeckType,
    pub settings: Option<RoomSettings>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomResponse {
    pub room_id: String,
}

pub async fn create_room(
    AxumState(state): AxumState<Arc<AppState>>,
    Json(body): Json<CreateRoomRequest>,
) -> impl IntoResponse {
    let room_id = Uuid::new_v4().to_string()[..8].to_string();
    let settings = body.settings.unwrap_or_default();
    let deck = get_deck(&body.deck_type);

    let room = Room {
        id: room_id.clone(),
        name: None,
        deck_type: body.deck_type,
        deck,
        facilitator_id: String::new(), // Set when first player joins via socket
        phase: GamePhase::Waiting,
        settings,
    };

    state.rooms.insert(
        room_id.clone(),
        RoomState {
            room,
            players: vec![],
            votes: vec![],
            stories: vec![],
            current_story_id: None,
            timer_handle: None,
        },
    );

    (StatusCode::CREATED, Json(CreateRoomResponse { room_id }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRoomResponse {
    pub room: Room,
}

pub async fn get_room(
    AxumState(state): AxumState<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> impl IntoResponse {
    match state.rooms.get(&room_id) {
        Some(rs) => Ok(Json(GetRoomResponse {
            room: rs.room.clone(),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn health() -> &'static str {
    "ok"
}
