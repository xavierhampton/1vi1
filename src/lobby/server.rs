use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

use crate::lobby::protocol::{self, ClientIncoming, ClientMsg, ReadBuffer, REJECT_FULL};
use crate::lobby::state::{LobbyColor, LobbyState};

pub enum ServerEvent {
    ClientMessage(usize, ClientIncoming),
    ClientConnected(usize, TcpStream),
    ClientDisconnected(usize),
}

pub struct LobbyServer {
    pub state: LobbyState,
    pub my_addr: String,
    pub(crate) client_streams: Vec<Option<TcpStream>>,
    pub(crate) event_rx: Option<Receiver<ServerEvent>>,
    pub(crate) shutdown: Arc<AtomicBool>,
    _listener_handle: thread::JoinHandle<()>,
}

impl LobbyServer {
    pub fn start(host_name: &str, port: u16) -> std::io::Result<Self> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
        let local_addr = listener.local_addr()?.to_string();
        listener.set_nonblocking(true)?;

        let shutdown = Arc::new(AtomicBool::new(false));
        let (event_tx, event_rx) = mpsc::channel();
        let shutdown_clone = shutdown.clone();

        let listener_handle = thread::spawn(move || {
            let mut next_id: usize = 1; // 0 is host
            while !shutdown_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _addr)) => {
                        let client_id = next_id;
                        next_id += 1;
                        let _ = stream.set_nodelay(true);

                        let stream_clone = stream.try_clone().expect("clone stream");
                        let tx = event_tx.clone();
                        let _ = tx.send(ServerEvent::ClientConnected(client_id, stream));

                        let sd = shutdown_clone.clone();
                        thread::spawn(move || {
                            read_client(stream_clone, client_id, tx, sd);
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            state: LobbyState::new_host(host_name),
            my_addr: local_addr,
            client_streams: Vec::new(),
            event_rx: Some(event_rx),
            shutdown,
            _listener_handle: listener_handle,
        })
    }

    /// Process network events. Returns true if game should start.
    pub fn update(&mut self) -> bool {
        let mut changed = false;

        let event_rx = self.event_rx.as_ref().unwrap();
        let events: Vec<_> = event_rx.try_iter().collect();
        for event in events {
            match event {
                ServerEvent::ClientConnected(id, stream) => {
                    if self.state.slots.len() >= 4 {
                        // Lobby full, reject
                        let mut s = stream;
                        let _ = s.write_all(&protocol::encode_server(
                            &protocol::ServerMsg::Rejected { reason: REJECT_FULL },
                        ));
                    } else {
                        // Grow client_streams vec to fit
                        while self.client_streams.len() <= id {
                            self.client_streams.push(None);
                        }
                        self.client_streams[id] = Some(stream);
                        // Don't add slot yet — wait for Join message
                    }
                }
                ServerEvent::ClientMessage(id, incoming) => {
                    let msg = match incoming {
                        ClientIncoming::Lobby(m) => m,
                        ClientIncoming::GameInput(_) | ClientIncoming::CardChoice(_) => continue, // ignore during lobby
                    };
                    match msg {
                        ClientMsg::Join { name } => {
                            if self.state.slots.len() >= 4 {
                                // Reject
                                if let Some(Some(stream)) = self.client_streams.get_mut(id) {
                                    let _ = stream.write_all(&protocol::encode_server(
                                        &protocol::ServerMsg::Rejected { reason: REJECT_FULL },
                                    ));
                                }
                            } else {
                                let color = self.state.first_available_color()
                                    .unwrap_or(LobbyColor::Blue);
                                self.state.slots.push(crate::lobby::state::PlayerSlot {
                                    name,
                                    color,
                                    ready: false,
                                    is_host: false,
                                });
                                changed = true;
                            }
                        }
                        ClientMsg::ChangeColor { color } => {
                            if let Some(slot_idx) = self.client_slot_index(id) {
                                if let Some(c) = LobbyColor::from_u8(color) {
                                    if !self.state.color_taken(c, Some(slot_idx)) {
                                        self.state.slots[slot_idx].color = c;
                                        changed = true;
                                    }
                                }
                            }
                        }
                        ClientMsg::ToggleReady => {
                            if let Some(slot_idx) = self.client_slot_index(id) {
                                self.state.slots[slot_idx].ready = !self.state.slots[slot_idx].ready;
                                changed = true;
                            }
                        }
                        ClientMsg::Leave => {
                            self.remove_client(id);
                            changed = true;
                        }
                    }
                }
                ServerEvent::ClientDisconnected(id) => {
                    self.remove_client(id);
                    changed = true;
                }
            }
        }

        if changed {
            self.broadcast_snapshot();
        }

        // Check if all ready
        if self.state.all_ready() {
            self.broadcast_game_start();
            return true;
        }

        false
    }

    pub fn client_slot_index(&self, client_id: usize) -> Option<usize> {
        // Slot 0 = host, slots 1+ = connected clients in stream-order
        // Find this client_id's position among connected clients
        let mut slot = 1usize; // start after host
        for (cid, stream) in self.client_streams.iter().enumerate() {
            if stream.is_some() {
                if cid == client_id {
                    return if slot < self.state.slots.len() { Some(slot) } else { None };
                }
                slot += 1;
            }
        }
        None
    }

    fn remove_client(&mut self, client_id: usize) {
        if let Some(slot_idx) = self.client_slot_index(client_id) {
            if slot_idx < self.state.slots.len() {
                self.state.slots.remove(slot_idx);
            }
        }
        if client_id < self.client_streams.len() {
            self.client_streams[client_id] = None;
        }
    }

    fn broadcast_snapshot(&mut self) {
        let connected: Vec<usize> = self.client_streams.iter().enumerate()
            .filter(|(_, s)| s.is_some())
            .map(|(id, _)| id)
            .collect();

        for (order, &client_id) in connected.iter().enumerate() {
            let slot_index = (order + 1) as u8; // +1 for host at 0
            let msg = protocol::ServerMsg::LobbySnapshot {
                my_index: slot_index,
                state: self.state.clone(),
            };
            let data = protocol::encode_server(&msg);
            if let Some(Some(stream)) = self.client_streams.get_mut(client_id) {
                let _ = stream.write_all(&data);
            }
        }
    }

    fn broadcast_game_start(&mut self) {
        let data = protocol::encode_server(&protocol::ServerMsg::GameStart);
        for stream_opt in self.client_streams.iter_mut() {
            if let Some(stream) = stream_opt {
                let _ = stream.write_all(&data);
            }
        }
    }

    pub fn host_change_color(&mut self, color: LobbyColor) {
        if !self.state.color_taken(color, Some(0)) {
            self.state.slots[0].color = color;
            self.broadcast_snapshot();
        }
    }

    pub fn host_toggle_ready(&mut self) {
        self.state.slots[0].ready = !self.state.slots[0].ready;
        self.broadcast_snapshot();
    }

    /// Hand off TCP infrastructure to GameServer. Reader threads keep running.
    pub fn into_game_parts(&mut self) -> GameServerParts {
        let mut streams = std::mem::take(&mut self.client_streams);
        for s in streams.iter_mut().flatten() {
            let _ = s.set_nodelay(true);
        }
        GameServerParts {
            client_streams: streams,
            event_rx: self.event_rx.take().unwrap(),
            shutdown: self.shutdown.clone(),
        }
    }
}

pub struct GameServerParts {
    pub client_streams: Vec<Option<TcpStream>>,
    pub event_rx: Receiver<ServerEvent>,
    pub shutdown: Arc<AtomicBool>,
}

impl Drop for LobbyServer {
    fn drop(&mut self) {
        // Only shutdown if we haven't handed off to GameServer
        if self.event_rx.is_some() {
            self.shutdown.store(true, Ordering::Relaxed);
        }
    }
}

fn read_client(
    mut stream: TcpStream,
    client_id: usize,
    tx: Sender<ServerEvent>,
    shutdown: Arc<AtomicBool>,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
    let mut read_buf = ReadBuffer::new();
    let mut tmp = [0u8; 512];

    while !shutdown.load(Ordering::Relaxed) {
        match stream.read(&mut tmp) {
            Ok(0) => break, // Disconnected
            Ok(n) => {
                read_buf.append(&tmp[..n]);
                while let Some(msg) = read_buf.try_decode_client_incoming() {
                    if tx.send(ServerEvent::ClientMessage(client_id, msg)).is_err() {
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

    let _ = tx.send(ServerEvent::ClientDisconnected(client_id));
}
