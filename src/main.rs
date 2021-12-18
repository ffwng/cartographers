use socketio::SocketIOExt;

mod board;
mod mask;
mod socketio;
mod scoring;

fn main() {
    let mut socket = socketio::connect("ws://localhost:3000/socket.io/?EIO=4&transport=websocket")
        .expect("cannot connect");

    socket
        .write_event("enterGame".into(), "Test".into())
        .expect("failed to send message");

    loop {
        let msg = socket.read_event().expect("failed to read message");
        println!("{:?}", msg);
    }
}
