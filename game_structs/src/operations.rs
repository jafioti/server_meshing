use serde::{Serialize, Deserialize};
use bevy::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct PositionUpdate {
    pub player_id: uuid::Uuid,
    pub position: Vec3
}