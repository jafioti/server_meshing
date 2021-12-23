use std::{collections::HashMap, sync::{mpsc::{Sender, Receiver}, Mutex}, net::UdpSocket};
use crate::game::InterpolatePosition;
use bevy::prelude::*;
use game_structs::{Player, operations::PositionUpdate};
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub fn sync_positions(
    mut other_player_query: Query<(Entity, &Player, &mut InterpolatePosition)>, 
    main_player_query: Query<(&Player, &Transform), Without<InterpolatePosition>>,
    current_player_struct: Res<Player>, 
    mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    socket: Res<UdpSocket>,
    receiver: Res<Mutex<Receiver<PositionUpdate>>>
) {
    let current_player_transform = main_player_query.iter().next().unwrap().1;

    // Unload all position updates from channel buffer
    let mut position_updates = HashMap::new();
    let receiver = receiver.lock().unwrap();
    while let Ok(position_update) = receiver.try_recv() {
        if position_update.player_id == current_player_struct.id {continue;} // Skip updating this player

        if let Some(pu) = position_updates.get_mut(&position_update.player_id) {
            *pu = position_update;
        } else {
            position_updates.insert(position_update.player_id, position_update);
        }
    }

    // Get players
    let mut players: HashMap<Uuid, bool> = reqwest::blocking::get("http://127.0.0.1:8000/get_players")
        .unwrap().json::<HashMap<Uuid, Player>>().unwrap() // Parse original hashmap
        .into_iter().map(|(k, _)| (k, k == current_player_struct.id)).collect(); // Replace values with false

    let mut entities_to_kill = vec![];
    for (entity, player, mut interpolate_position) in other_player_query.iter_mut() {
        if let Some(pu) = position_updates.get(&player.id) {
            interpolate_position.target = pu.position;
        }

        if let Some(p) = players.get_mut(&player.id) {
            *p = true;
        } else {
            // Player quit, remove
            entities_to_kill.push(entity);
        }
    }
    for entity in entities_to_kill {
        commands.entity(entity).despawn();
    }

    // Spawn players we haven't seen
    for (i, _) in players.iter().filter(|(_, s)| !**s) {
        let position = if let Some(pu) = position_updates.get(i) {pu.position} else {Vec3::ZERO};
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(1.0, 0.2, 0.2).into()),
            transform: Transform::from_translation(position),
            ..Default::default()
        }).insert(Player{id:*i})
        .insert(InterpolatePosition{target: position});
    }

    // Send position to server
    let position_update = PositionUpdate {
        player_id: current_player_struct.id,
        position: current_player_transform.translation
    };
    socket.send_to(&bincode::serialize(&position_update).unwrap(), "127.0.0.1:41794")
        .expect("Failed to send position update");
}

// Capture any position changes sent from server and put in queue
pub fn capture_changes(sender: Sender<PositionUpdate>, socket: UdpSocket) {
    loop {
        // Wait for a position update from server
        let mut buf = [0; 2048];
        let (amt, _) = socket.recv_from(&mut buf)
            .expect("Failed to receive");
        let update: PositionUpdate = bincode::deserialize(&buf[..amt]).unwrap();

        // Put update in channel queue
        sender.send(update)
            .expect("Failed to put update in queue");
    }
}

pub fn send_exit_to_server(player_id: Uuid) {
    reqwest::blocking::Client::new().post("http://127.0.0.1:8000/unregister_player").header("Content-Type", "application/json")
        .body(serde_json::to_string(&player_id).unwrap())
        .send().unwrap();
}