use serde::{Deserialize, Serialize};

use super::game::{CardValue, DeckType, Story, Vote};
use super::player::{Player, Role};
use super::room::{Room, RoomSettings};

// ── Client → Server ──

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomCreatePayload {
    pub deck_type: DeckType,
    pub settings: Option<RoomSettings>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomJoinPayload {
    pub room_id: String,
    pub display_name: String,
    pub role: Option<Role>,
    pub player_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomLeavePayload {
    pub room_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteSubmitPayload {
    pub room_id: String,
    pub value: CardValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteRevealPayload {
    pub room_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteResetPayload {
    pub room_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryAddPayload {
    pub room_id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySelectPayload {
    pub room_id: String,
    pub story_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryUpdatePayload {
    pub room_id: String,
    pub story_id: String,
    pub estimate: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryDeletePayload {
    pub room_id: String,
    pub story_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryImportItem {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryImportPayload {
    pub room_id: String,
    pub stories: Vec<StoryImportItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerStartPayload {
    pub room_id: String,
    pub duration: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerStopPayload {
    pub room_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRolePayload {
    pub room_id: String,
    pub player_id: String,
    pub role: Role,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerKickPayload {
    pub room_id: String,
    pub player_id: String,
}

// ── Server → Client ──

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomStateResponse {
    pub room: Room,
    pub players: Vec<Player>,
    pub stories: Vec<Story>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_story: Option<Story>,
    pub votes: Vec<Vote>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomJoinedResponse {
    pub player: Player,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomLeftResponse {
    pub player_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteSubmittedResponse {
    pub player_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteRevealedResponse {
    pub votes: Vec<Vote>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryAddedResponse {
    pub story: Story,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySelectedResponse {
    pub story_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryUpdatedResponse {
    pub story: Story,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryDeletedResponse {
    pub story_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerTickResponse {
    pub remaining: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUpdatedResponse {
    pub player: Player,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub message: String,
    pub code: String,
}
