mod endpoints;
mod streaming;

use std::sync::mpsc::{Sender, Receiver, self};
use std::thread;
use std::{sync::{Arc, RwLock}, collections::HashMap};
use uuid::Uuid;
use rocket::routes;
use game_structs::{Player};
use endpoints::*;
use streaming::*;

#[derive(Default, Debug, Clone)]
pub struct SessionStruct {
    pub players: HashMap<Uuid, Player>,
    pub addresses: HashMap<Uuid, String>
}

pub type Session = Arc<RwLock<SessionStruct>>;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let (sender, receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
    let session = Arc::new(RwLock::new(SessionStruct::default()));
    let session1 = session.clone();

    // Launch sender and receiver threads
    let sender_handle = thread::spawn(move || {
        send_positions(session1, receiver);
    });
    let receive_handle = thread::spawn(move || {
        receive_positions(sender);
    });

    rocket::build()
        .mount("/", routes![register_player, unregister_player, get_players])
        .manage(session)
        .launch().await?;

    // For some reason doesn't work
    sender_handle.join().expect("Failed to join sending thread");
    receive_handle.join().expect("Failed to join receiving thread");
    Ok(())
}