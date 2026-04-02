use serde::{Deserialize, Serialize};

use super::game::{CardValue, DeckType, GamePhase};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomSettings {
    pub timer_enabled: bool,
    pub timer_duration: u32,
    pub auto_reveal: bool,
    pub allow_spectators: bool,
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            timer_enabled: false,
            timer_duration: 60,
            auto_reveal: false,
            allow_spectators: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub deck_type: DeckType,
    pub deck: Vec<CardValue>,
    pub facilitator_id: String,
    pub phase: GamePhase,
    pub settings: RoomSettings,
}
