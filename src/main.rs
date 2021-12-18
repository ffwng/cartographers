use socketio::SocketIOExt;

use crate::protocol::Message;

mod board;
mod card;
mod game;
mod mask;
mod protocol;
mod scoring;
mod socketio;

fn main() {
    let mut socket = socketio::connect("ws://localhost:3000/socket.io/?EIO=4&transport=websocket")
        .expect("cannot connect");

    socket
        .write_event("enterGame", "Test")
        .expect("failed to send message");

    socket
        .write_event("startGame", "Test")
        .expect("failed to send message");

    loop {
        let (event, data) = socket.read_event().expect("failed to read message");
        if let Some(data) = data {
            println!("{} = {:?}", event, Message::parse(&event, &data));
        } else {
            println!("event without payload: {}", event);
        }
    }
}
