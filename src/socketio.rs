use std::io::{Read, Write};

use tungstenite::{client::IntoClientRequest, Message, WebSocket};

pub fn connect(req: impl IntoClientRequest) -> Result<impl SocketIOExt> {
    let mut socket = connect_engineio(req)?;
    connect_socketio(&mut socket)?;

    Ok(socket)
}

#[derive(Debug)]
pub enum Error {
    Websocket(tungstenite::Error),
    EngineIO(String),
    SocketIO(String),
    JSON(serde_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Self::Websocket(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::JSON(err)
    }
}

fn connect_engineio(req: impl IntoClientRequest) -> Result<WebSocket<impl Read + Write>> {
    let (mut socket, _) = tungstenite::connect(req)?;

    match socket.read_engineio_packet()? {
        EngineIOPacket('0', _) => Ok(socket),
        p => Err(Error::EngineIO(format!(
            "expected open packet, instead got {:?}",
            p
        ))),
    }
}

fn connect_socketio(socket: &mut impl SocketIOExt) -> Result<()> {
    // send connect request
    socket.write_socketio_packet("0")?;

    // expect connect response
    match socket.read_socketio_packet()? {
        SocketIOPacket('0', _) => Ok(()),
        p => Err(Error::SocketIO(format!(
            "expected connect package, instead got {:?}",
            p
        ))),
    }
}

#[derive(Debug)]
pub struct EngineIOPacket(char, String);

#[derive(Debug)]
pub struct SocketIOPacket(char, String);

pub trait SocketIOExt {
    fn read_raw_message(&mut self) -> Result<String>;

    fn write_raw_message(&mut self, msg: impl Into<String>) -> Result<()>;

    fn read_engineio_packet(&mut self) -> Result<EngineIOPacket> {
        let mut msg = self.read_raw_message()?;
        let t = msg
            .drain(0..1)
            .next()
            .ok_or(Error::EngineIO("invalid package format".into()))?;

        Ok(EngineIOPacket(t, msg))
    }

    fn write_engineio_packet(&mut self, msg: impl Into<String>) -> Result<()> {
        self.write_raw_message(msg)
    }

    fn read_socketio_packet(&mut self) -> Result<SocketIOPacket> {
        loop {
            let EngineIOPacket(t, mut msg) = self.read_engineio_packet()?;
            match t {
                // ping
                '2' => {
                    self.write_engineio_packet("3")?;
                }
                // message
                '4' => {
                    let t = msg
                        .drain(0..1)
                        .next()
                        .ok_or(Error::SocketIO("invalid package format".into()))?;
                    return Ok(SocketIOPacket(t, msg));
                }
                // noop
                '6' => {}
                // open, close, pong, upgrade
                _ => return Err(Error::EngineIO(format!("unexpected packet of type {}", t))),
            }
        }
    }

    fn write_socketio_packet(&mut self, msg: impl Into<String>) -> Result<()> {
        // transform into an engine.io message packet
        let mut msg = msg.into();
        msg.insert(0, '4');
        self.write_engineio_packet(msg)
    }

    fn read_event(&mut self) -> Result<(String, Option<serde_json::Value>)> {
        let payload = loop {
            let SocketIOPacket(t, msg) = self.read_socketio_packet()?;
            match t {
                // event
                '2' => break msg,
                // other
                _ => return Err(Error::SocketIO(format!("unexpected packet of type {}", t))),
            }
        };

        if let serde_json::Value::Array(elems) = serde_json::from_str(&payload)? {
            let mut elems = elems.into_iter();
            if let Some(serde_json::Value::String(event)) = elems.next() {
                return Ok((event, elems.next()));
            }
        }

        Err(Error::SocketIO(format!(
            "could not parse event payload {}",
            payload
        )))
    }

    fn expect_event(&mut self, event: &str) -> Result<Option<serde_json::Value>> {
        let (e, data) = self.read_event()?;
        if e == event {
            Ok(data)
        } else {
            Err(Error::SocketIO(format!("expected event type {}, got {}", event, e)))
        }
    }

    fn write_event(&mut self, event: impl Into<String>, data: impl Into<String>) -> Result<()> {
        let msg = format!("2{}", serde_json::to_string(&[event.into(), data.into()])?);
        self.write_socketio_packet(msg)
    }

    fn write_json_event(&mut self, event: impl Into<String>, data: &serde_json::Value) -> Result<()> {
        self.write_event(event, data.to_string())
    }
}

impl<S> SocketIOExt for WebSocket<S>
where
    S: Read + Write,
{
    fn read_raw_message(&mut self) -> Result<String> {
        match self.read_message()? {
            Message::Text(msg) => Ok(msg),
            msg => Err(Error::EngineIO(format!(
                "unexpected websocket message {}",
                msg
            ))),
        }
    }

    fn write_raw_message(&mut self, msg: impl Into<String>) -> Result<()> {
        Ok(self.write_message(Message::Text(msg.into()))?)
    }
}
