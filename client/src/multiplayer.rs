use std::collections::HashMap;
use crate::game::InterpolatePosition;
use bevy::prelude::*;
use game_structs::{Player, operations::PositionUpdate};
use uuid::Uuid;

pub fn sync_positions(
    mut other_player_query: Query<(Entity, &Player, &mut InterpolatePosition)>, 
    main_player_query: Query<(&Player, &Transform), Without<InterpolatePosition>>,
    current_player_struct: Res<Player>, 
    mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let current_player_transform = main_player_query.iter().next().unwrap().1;

    let client = reqwest::blocking::Client::new();
    // Send current position to server
    if let Err(e) = client.post("http://127.0.0.1:8000/update_position").header("Content-Type", "application/json")
        .body(serde_json::to_string(
            &PositionUpdate {
                player_id: current_player_struct.id,
                position: current_player_transform.translation
            }
        ).unwrap())
        .send() {
        println!("Failed to update position: {}", e.to_string());
    }

    // Get positions from server
    let positions: HashMap<Uuid, Vec3> = client.get("http://127.0.0.1:8000/get_all_positions").header("Content-Type", "application/json")
        .send().unwrap()
        .json().unwrap();
    // Apply positions to game
    let mut seen_players = { // To track which players we've seen and which we haven't
        let mut tmp_map = HashMap::with_capacity(positions.len() - 1);
        for key in positions.keys() {
            if *key != current_player_struct.id {
                tmp_map.insert(key, false);   
            }
        }
        tmp_map
    };
    let mut entities_to_kill = vec![];
    for (entity, player, mut interpolate_position) in other_player_query.iter_mut() {
        if let Some(sp) = seen_players.get_mut(&player.id) {
            *sp = true;
            interpolate_position.target = positions[&player.id];
        } else {
            // Player quit, remove
            entities_to_kill.push(entity);
        }
    }
    for entity in entities_to_kill {
        commands.entity(entity).despawn();
    }

    // Spawn players we haven't seen
    for (i, _) in seen_players.iter().filter(|(_, s)| !**s) {
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(1.0, 0.2, 0.2).into()),
            transform: Transform::from_translation(positions[i]),
            ..Default::default()
        }).insert(Player{id:**i})
        .insert(InterpolatePosition{target: positions[i]});
    }
}

pub fn send_exit_to_server(player_id: Uuid) {
    reqwest::blocking::Client::new().post("http://127.0.0.1:8000/unregister_player").header("Content-Type", "application/json")
        .body(serde_json::to_string(&player_id).unwrap())
        .send().unwrap();
}