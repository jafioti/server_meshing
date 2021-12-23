use serde::{Serialize, Deserialize};
use bevy::prelude::*;

use crate::Player;

#[derive(Serialize, Deserialize)]
pub struct PositionUpdate {
    pub player_id: uuid::Uuid,
    pub position: Vec3
}

#[derive(Serialize, Deserialize)]
pub struct PlayerRegister {
    pub player: Player,
    pub address: String
}