use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

use crate::lobby::protocol::{self, ClientMsg, ReadBuffer, ServerMsg};
use crate::lobby::state::LobbyState;

pub struct LobbyClient {
    pub state: LobbyState,
    pub my_index: u8,
    pub rejected: bool,
    pub game_starting: bool,
    write_stream: TcpStream,
    incoming_rx: Receiver<ServerMsg>,
    shutdown: Arc<AtomicBool>,
    _reader_handle: thread::JoinHandle<()>,
}

impl LobbyClient {
    pub fn connect(addr: &str, name: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?,
            Duration::from_secs(3),
        )?;

        let write_stream = stream.try_clone()?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();

        let shutdown_clone = shutdown.clone();
        let reader_handle = thread::spawn(move || {
            read_server(stream, tx, shutdown_clone);
        });

        let mut client = Self {
            state: LobbyState { slots: Vec::new() },
            my_index: 0,
            rejected: false,
            game_starting: false,
            write_stream,
            incoming_rx: rx,
            shutdown,
            _reader_handle: reader_handle,
        };

        // Send join message
        client.send(ClientMsg::Join { name: name.to_string() });

        Ok(client)
    }

    pub fn update(&mut self) {
        while let Ok(msg) = self.incoming_rx.try_recv() {
            match msg {
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
            }
        }
    }

    pub fn send(&mut self, msg: ClientMsg) {
        let data = protocol::encode_client(&msg);
        let _ = self.write_stream.write_all(&data);
    }

    pub fn change_color(&mut self, color: u8) {
        self.send(ClientMsg::ChangeColor { color });
    }

    pub fn toggle_ready(&mut self) {
        self.send(ClientMsg::ToggleReady);
    }
}

impl Drop for LobbyClient {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        let _ = self.write_stream.write_all(&protocol::encode_client(&ClientMsg::Leave));
    }
}

fn read_server(
    mut stream: TcpStream,
    tx: mpsc::Sender<ServerMsg>,
    shutdown: Arc<AtomicBool>,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
    let mut read_buf = ReadBuffer::new();
    let mut tmp = [0u8; 512];

    while !shutdown.load(Ordering::Relaxed) {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => {
                read_buf.append(&tmp[..n]);
                while let Some(msg) = read_buf.try_decode_server() {
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
}
