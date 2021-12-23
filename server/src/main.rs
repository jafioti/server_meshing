mod endpoints;
mod streaming;

use std::net::UdpSocket;
use std::sync::mpsc::{Sender, Receiver, self};
use std::thread;
use std::{sync::{Arc, RwLock}, collections::HashMap};
use uuid::Uuid;
use rocket::routes;
use game_structs::{Player};
use endpoints::*;
use streaming::*;
use clap::Parser;

#[derive(Default, Debug, Clone)]
pub struct SessionStruct {
    pub players: HashMap<Uuid, Player>,
    pub addresses: HashMap<Uuid, String>
}

pub type Session = Arc<RwLock<SessionStruct>>;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let args = Args::parse();

    // Create channel
    let (sender, receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

    // Create session
    let session = Arc::new(RwLock::new(SessionStruct::default()));
    let session1 = session.clone();

    // Create send/receive sockets
    let send_socket = UdpSocket::bind(format!("127.0.0.1:{}", args.send)).expect("Failed to bind send socket");
    let receive_socket = UdpSocket::bind(format!("127.0.0.1:{}", args.receive)).expect("Failed to bind receive socket");

    // Launch sender and receiver threads
    let sender_handle = thread::spawn(move || {
        send_positions(session1, receiver, send_socket);
    });
    let receive_handle = thread::spawn(move || {
        receive_positions(sender, receive_socket);
    });

    // Launch Rocket server
    let figment = rocket::Config::figment()
        .merge(("port", args.main));

    rocket::custom(figment)
        .mount("/", routes![register_player, unregister_player, get_players])
        .manage(session)
        .launch().await?;

    // For some reason doesn't work
    sender_handle.join().expect("Failed to join sending thread");
    receive_handle.join().expect("Failed to join receiving thread");
    Ok(())
}

#[derive(Parser, Debug)]
#[clap(name = "Server")]
struct Args {
    /// The port to send updates to the server from
    #[clap(short, long)]
    send: String,
    
    /// The port to receive updates from the server
    #[clap(short, long)]
    receive: String,

    /// The port number the Rocket server should run on
    #[clap(short, long)]
    main: i32
}