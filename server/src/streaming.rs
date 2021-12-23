use std::{sync::mpsc::{Sender, Receiver}, net::UdpSocket};

use crate::SessionStruct;

pub fn send_positions(session: std::sync::Arc<std::sync::RwLock<SessionStruct>>, receiver: Receiver<Vec<u8>>) {
    let socket = UdpSocket::bind("127.0.0.1:27900").expect("Failed to bind");

    // Get position update from queue
    while let Ok(position_update) = receiver.recv() {
        // Send position update to all recipients
        for address in session.read().unwrap().addresses.values() {
            socket.send_to(&position_update, address)
                .expect("Error on send");
        }
    }
}

pub fn receive_positions(sender: Sender<Vec<u8>>) {
    let socket = UdpSocket::bind("127.0.0.1:41794").expect("Failed to bind");
    loop {
        // Wait till we receive an update
        let mut buf = [0; 2048];
        let (amt, address) = socket.recv_from(&mut buf)
            .expect("Failed to receive");
        // Put update into channel
        sender.send(buf[..amt].to_vec())
            .expect("Failed to send");
    }
}