mod game;
mod multiplayer;

use clap::Parser;
use std::{sync::{mpsc::{Sender, Receiver, self}, Mutex}, net::UdpSocket, thread};
use bevy::{prelude::*, core::FixedTimestep};
use game_structs::{
    Player,
    operations::{
        PositionUpdate,
        PlayerRegister
    }
};
use uuid::Uuid;

fn main() {
    let args = Args::parse();
    // Create channel for position updates
    let (sender, receiver): (Sender<PositionUpdate>, Receiver<PositionUpdate>) = mpsc::channel();
    // Create sockets to send/recv updates with
    let send_socket = UdpSocket::bind(format!("127.0.0.1:{}", args.send)).expect("Failed to bind send socket");
    let receive_socket = UdpSocket::bind(format!("127.0.0.1:{}", args.receive)).expect("Failed to bind receive socket");
    
    // Create position update collector thread
    let collector_thread_handle = thread::spawn(move || {
        multiplayer::capture_changes(sender, receive_socket);
    });
    
    // Create player
    let mut player = Player {id: Uuid::default()};
    player.id = reqwest::blocking::Client::new().post("http://127.0.0.1:8000/register_player").header("Content-Type", "application/json")
        .body(serde_json::to_string(
            &PlayerRegister {
                player: player.clone(),
                address: format!("127.0.0.1:{}", args.receive)
            }
        ).unwrap())
        .send().unwrap()
        .json().unwrap();

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(player)
        .insert_resource(Mutex::new(receiver))
        .insert_resource(send_socket)
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

    collector_thread_handle.join().expect("Failed to join collector thread.");
}

#[derive(Parser, Debug)]
#[clap(name = "Sidekick AI System Version Orchestration")]
struct Args {
    /// The port to send updates to the server from
    #[clap(short, long)]
    send: String,
    
    /// The port to receive updates from the server
    #[clap(short, long)]
    receive: String,
}