pub mod operations;

use serde::{Serialize, Deserialize};
use bevy::prelude::*;
pub use bevy::prelude::Vec3;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, Component)]
pub struct Player {
    pub id: Uuid
}