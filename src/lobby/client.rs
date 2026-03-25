use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

use crate::lobby::protocol::{self, ClientMsg, ReadBuffer, ServerIncoming, ServerMsg};
use crate::lobby::state::LobbyState;

pub struct LobbyClient {
    pub state: LobbyState,
    pub my_index: u8,
    pub rejected: bool,
    pub game_starting: bool,
    pub host_disbanded: Option<String>,
    pub(crate) write_stream: Option<TcpStream>,
    pub(crate) incoming_rx: Option<Receiver<ServerIncoming>>,
    pub(crate) shutdown: Arc<AtomicBool>,
    _reader_handle: thread::JoinHandle<()>,
}

impl LobbyClient {
    pub fn connect(addr: &str, name: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?,
            Duration::from_secs(3),
        )?;
        let _ = stream.set_nodelay(true);

        let write_stream = stream.try_clone()?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();

        let shutdown_clone = shutdown.clone();
        let reader_handle = thread::spawn(move || {
            read_server(stream, tx, shutdown_clone);
        });

        let mut client = Self {
            state: LobbyState { slots: Vec::new(), settings: crate::lobby::state::GameSettings::default() },
            my_index: 0,
            rejected: false,
            game_starting: false,
            host_disbanded: None,
            write_stream: Some(write_stream),
            incoming_rx: Some(rx),
            shutdown,
            _reader_handle: reader_handle,
        };

        // Send join message
        client.send(ClientMsg::Join { name: name.to_string() });

        Ok(client)
    }

    pub fn update(&mut self) {
        let incoming_rx = match &self.incoming_rx {
            Some(rx) => rx,
            None => return,
        };
        while let Ok(incoming) = incoming_rx.try_recv() {
            match incoming {
                ServerIncoming::Lobby(msg) => match msg {
                    ServerMsg::LobbySnapshot { my_index, state } => {
                        self.my_index = my_index;
                        self.state = state;
                    }
                    ServerMsg::Rejected { .. } => {
                        self.rejected = true;
                    }
                    ServerMsg::GameStart => {
                        self.game_starting = true;
                    }
                    ServerMsg::Disbanded { host_name } => {
                        self.host_disbanded = Some(host_name);
                    }
                    ServerMsg::PlayerLeft { .. } => {}
                },
                ServerIncoming::Snapshot(_) => {} // ignore during lobby
                ServerIncoming::Disconnected => {
                    if self.host_disbanded.is_none() {
                        self.host_disbanded = Some(String::new());
                    }
                }
            }
        }
    }

    pub fn send(&mut self, msg: ClientMsg) {
        let data = protocol::encode_client(&msg);
        if let Some(stream) = &mut self.write_stream {
            let _ = stream.write_all(&data);
        }
    }

    pub fn change_color(&mut self, color: u8) {
        self.send(ClientMsg::ChangeColor { color });
    }

    pub fn toggle_ready(&mut self) {
        self.send(ClientMsg::ToggleReady);
    }

    /// Hand off TCP infrastructure to GameClient. Reader thread keeps running.
    pub fn into_game_parts(&mut self) -> GameClientParts {
        GameClientParts {
            write_stream: self.write_stream.take().unwrap(),
            incoming_rx: self.incoming_rx.take().unwrap(),
            shutdown: self.shutdown.clone(),
            my_index: self.my_index,
        }
    }
}

pub struct GameClientParts {
    pub write_stream: TcpStream,
    pub incoming_rx: Receiver<ServerIncoming>,
    pub shutdown: Arc<AtomicBool>,
    pub my_index: u8,
}

impl Drop for LobbyClient {
    fn drop(&mut self) {
        // Only shutdown if we haven't handed off to GameClient
        if self.incoming_rx.is_some() {
            self.shutdown.store(true, Ordering::Relaxed);
        }
        if let Some(stream) = &mut self.write_stream {
            let _ = stream.write_all(&protocol::encode_client(&ClientMsg::Leave));
        }
    }
}

fn read_server(
    mut stream: TcpStream,
    tx: mpsc::Sender<ServerIncoming>,
    shutdown: Arc<AtomicBool>,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
    let mut read_buf = ReadBuffer::new();
    let mut tmp = [0u8; 2048];

    while !shutdown.load(Ordering::Relaxed) {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => {
                read_buf.append(&tmp[..n]);
                while let Some(msg) = read_buf.try_decode_server_incoming() {
                    if tx.send(msg).is_err() {
                        return;
                    }
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                continue;
            }
            Err(_) => break,
        }
    }
    if !shutdown.load(Ordering::Relaxed) {
        let _ = tx.send(ServerIncoming::Disconnected);
    }
}
