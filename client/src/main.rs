mod game;
mod multiplayer;

use bevy::{prelude::*, core::FixedTimestep};
use game_structs::{
    Player
};
use uuid::Uuid;

fn main() {
    // Create player
    let mut player = Player {
        id: Uuid::default()
    };
    player.id = reqwest::blocking::Client::new().post("http://127.0.0.1:8000/register_player").header("Content-Type", "application/json")
        .body(serde_json::to_string(&player).unwrap())
        .send().unwrap()
        .json().unwrap();

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(player)
        .add_plugins(DefaultPlugins)
        .add_startup_system(game::setup.system())
        .add_system(game::move_block.system())
        .add_system(game::interpolate_positions.system())
        .add_system(game::exit_system.system())
        .add_stage("multiplayer_sync", SystemStage::parallel()
            .with_run_criteria(FixedTimestep::steps_per_second(20.0))
            .with_system(multiplayer::sync_positions.system())
        )
        .run();
}