use std::{collections::HashMap, sync::{mpsc::{Sender, Receiver}, Mutex, atomic::Ordering}, net::UdpSocket};
use crate::game::InterpolatePosition;
use bevy::prelude::*;
use game_structs::{Player, operations::{PositionUpdate, PlayerRegister}};
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub fn sync_positions(
    mut other_player_query: Query<(Entity, &Player, &mut Transform, &mut InterpolatePosition)>, 
    main_player_query: Query<(&Player, &Transform), Without<InterpolatePosition>>,
    current_player_struct: Res<Player>, 
    mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    socket: Res<UdpSocket>,
    receiver: Res<Mutex<Receiver<PositionUpdate>>>,
    server: Res<crate::Server>,
    receive_port: Res<crate::ReceivePort>
) {
    let current_player_transform = main_player_query.iter().next().unwrap().1;
    let server_num: usize = if current_player_transform.translation.x < 0. {1} else {0}; // Positive X goes on server 0, negative goes on server 1
    let last_server = server.0.load(Ordering::Relaxed);
    let switched_server = last_server != server_num;

    // Switch server if nessacary
    if switched_server {
        // Send leave request to last server
        reqwest::blocking::Client::new().post(format!("{}/unregister_player", crate::SERVER_ADDRESSES[last_server])).header("Content-Type", "application/json")
            .body(serde_json::to_string(&current_player_struct.id).unwrap())
            .send().unwrap();
        // Send join request to new server
        reqwest::blocking::Client::new().post(format!("{}/register_player", crate::SERVER_ADDRESSES[server_num])).header("Content-Type", "application/json")
            .body(serde_json::to_string(
                &PlayerRegister {
                    player: current_player_struct.clone(),
                    address: format!("127.0.0.1:{}", receive_port.0)
                }
            ).unwrap())
            .send().unwrap();
        // Switch server resource
        server.0.store(server_num, Ordering::Release);
    }

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
    let mut players: HashMap<Uuid, bool> = reqwest::blocking::get(format!("{}/get_players", crate::SERVER_ADDRESSES[server_num]))
        .unwrap().json::<HashMap<Uuid, Player>>().unwrap() // Parse original hashmap
        .into_iter().map(|(k, _)| (k, k == current_player_struct.id)).collect(); // Replace values with false

    let mut entities_to_kill = vec![];
    for (entity, player, mut transform, mut interpolate_position) in other_player_query.iter_mut() {
        if let Some(pu) = position_updates.get(&player.id) {
            if transform.translation == Vec3::ZERO {
                // First time setting position, don't interpolate
                transform.translation = pu.position;
            }
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
    socket.send_to(&bincode::serialize(&position_update).unwrap(), crate::SERVER_RECEIVING_PORTS[server_num])
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

pub fn send_exit_to_server(player_id: Uuid, server_num: usize) {
    reqwest::blocking::Client::new().post(format!("{}/unregister_player", crate::SERVER_ADDRESSES[server_num])).header("Content-Type", "application/json")
        .body(serde_json::to_string(&player_id).unwrap())
        .send().unwrap();
}