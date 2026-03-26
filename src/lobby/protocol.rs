use crate::game::net::{self, WorldSnapshot};
use crate::lobby::state::{GameSettings, LobbyColor, LobbyState, PlayerSlot};
use crate::player::input::PlayerInput;

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ClientMsg {
    Join { name: String, accessories: Vec<(u8, u8, u8, u8)> },
    ChangeColor { color: u8 },
    ToggleReady,
    Leave,
}

#[derive(Debug)]
pub enum ServerMsg {
    LobbySnapshot { my_index: u8, state: LobbyState },
    Rejected { reason: u8 },
    GameStart,
    PlayerLeft { name: String },
    Disbanded { host_name: String },
    Rematch,
}

// Wrapper enums: reader threads decode both lobby and game messages
#[derive(Debug)]
pub enum ClientIncoming {
    Lobby(ClientMsg),
    GameInput(PlayerInput),
    CardChoice(u8), // card slot index 0-2
}

#[derive(Debug)]
pub enum ServerIncoming {
    Lobby(ServerMsg),
    Snapshot(WorldSnapshot),
    Disconnected,
}

// Rejection reasons
pub const REJECT_FULL: u8 = 1;

// ── Wire format: [u16 len][u8 type][payload...] ─────────────────────────────

pub fn encode_client(msg: &ClientMsg) -> Vec<u8> {
    let mut payload = Vec::new();
    match msg {
        ClientMsg::Join { name, accessories } => {
            payload.push(0x01);
            let bytes = name.as_bytes();
            payload.push(bytes.len() as u8);
            payload.extend_from_slice(bytes);
            payload.push(accessories.len().min(3) as u8);
            for &(id, r, g, b) in accessories.iter().take(3) {
                payload.push(id);
                payload.push(r);
                payload.push(g);
                payload.push(b);
            }
        }
        ClientMsg::ChangeColor { color } => {
            payload.push(0x02);
            payload.push(*color);
        }
        ClientMsg::ToggleReady => {
            payload.push(0x03);
        }
        ClientMsg::Leave => {
            payload.push(0x04);
        }
    }
    let len = payload.len() as u16;
    let mut out = Vec::with_capacity(2 + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend(payload);
    out
}

pub fn encode_server(msg: &ServerMsg) -> Vec<u8> {
    let mut payload = Vec::new();
    match msg {
        ServerMsg::LobbySnapshot { my_index, state } => {
            payload.push(0x81);
            payload.push(*my_index);
            payload.push(state.slots.len() as u8);
            for slot in &state.slots {
                let name_bytes = slot.name.as_bytes();
                payload.push(name_bytes.len() as u8);
                payload.extend_from_slice(name_bytes);
                payload.push(slot.color as u8);
                payload.push(slot.ready as u8);
                payload.push(slot.is_host as u8);
                payload.push(slot.accessories.len().min(3) as u8);
                for &(id, r, g, b) in slot.accessories.iter().take(3) {
                    payload.push(id);
                    payload.push(r);
                    payload.push(g);
                    payload.push(b);
                }
            }
            // Game settings
            payload.push(state.settings.wins_to_match as u8);
            payload.extend_from_slice(&state.settings.spawn_invuln.to_be_bytes());
            payload.extend_from_slice(&state.settings.starting_hp.to_be_bytes());
            payload.extend_from_slice(&state.settings.gravity_scale.to_be_bytes());
            payload.extend_from_slice(&state.settings.turbo_speed.to_be_bytes());
            payload.push(state.settings.sudden_death as u8);
            payload.push(state.settings.everyone_picks as u8);
        }
        ServerMsg::Rejected { reason } => {
            payload.push(0x82);
            payload.push(*reason);
        }
        ServerMsg::GameStart => {
            payload.push(0x83);
        }
        ServerMsg::PlayerLeft { name } => {
            payload.push(0x84);
            let bytes = name.as_bytes();
            payload.push(bytes.len() as u8);
            payload.extend_from_slice(bytes);
        }
        ServerMsg::Disbanded { host_name } => {
            payload.push(0x85);
            let bytes = host_name.as_bytes();
            payload.push(bytes.len() as u8);
            payload.extend_from_slice(bytes);
        }
        ServerMsg::Rematch => {
            payload.push(0x86);
        }
    }
    let len = payload.len() as u16;
    let mut out = Vec::with_capacity(2 + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend(payload);
    out
}

// ── Decode (handles both lobby + game messages) ──────────────────────────────

fn decode_frame(buf: &[u8]) -> Option<(&[u8], usize)> {
    if buf.len() < 3 {
        return None;
    }
    let len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
    if len == 0 || buf.len() < 2 + len {
        return None;
    }
    Some((&buf[2..2 + len], 2 + len))
}

pub fn decode_client_incoming(buf: &[u8]) -> Option<(ClientIncoming, usize)> {
    let (data, consumed) = decode_frame(buf)?;
    let msg = match data[0] {
        // Lobby messages
        0x01 => {
            let name_len = data[1] as usize;
            if data.len() < 2 + name_len { return None; }
            let name = String::from_utf8_lossy(&data[2..2 + name_len]).to_string();
            let mut pos = 2 + name_len;
            let mut accessories = Vec::new();
            if pos < data.len() {
                let count = data[pos] as usize;
                pos += 1;
                for _ in 0..count.min(3) {
                    if pos + 4 > data.len() { break; }
                    accessories.push((data[pos], data[pos+1], data[pos+2], data[pos+3]));
                    pos += 4;
                }
            }
            ClientIncoming::Lobby(ClientMsg::Join { name, accessories })
        }
        0x02 => ClientIncoming::Lobby(ClientMsg::ChangeColor { color: data[1] }),
        0x03 => ClientIncoming::Lobby(ClientMsg::ToggleReady),
        0x04 => ClientIncoming::Lobby(ClientMsg::Leave),
        // Game messages
        0x10 => {
            let input = net::decode_game_input(&data[1..])?;
            ClientIncoming::GameInput(input)
        }
        0x11 => {
            if data.len() < 2 { return None; }
            ClientIncoming::CardChoice(data[1])
        }
        _ => return None,
    };
    Some((msg, consumed))
}

pub fn decode_server_incoming(buf: &[u8]) -> Option<(ServerIncoming, usize)> {
    let (data, consumed) = decode_frame(buf)?;
    let msg = match data[0] {
        // Lobby messages
        0x81 => {
            let my_index = data[1];
            let slot_count = data[2] as usize;
            let mut pos = 3;
            let mut slots = Vec::new();
            for _ in 0..slot_count {
                if pos >= data.len() { return None; }
                let name_len = data[pos] as usize;
                pos += 1;
                if pos + name_len > data.len() { return None; }
                let name = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
                pos += name_len;
                if pos + 3 > data.len() { return None; }
                let color = LobbyColor::from_u8(data[pos]).unwrap_or(LobbyColor::Blue);
                let ready = data[pos + 1] != 0;
                let is_host = data[pos + 2] != 0;
                pos += 3;
                let mut accessories = Vec::new();
                if pos < data.len() {
                    let acc_count = data[pos] as usize;
                    pos += 1;
                    for _ in 0..acc_count.min(3) {
                        if pos + 4 > data.len() { break; }
                        accessories.push((data[pos], data[pos+1], data[pos+2], data[pos+3]));
                        pos += 4;
                    }
                }
                slots.push(PlayerSlot { name, color, ready, is_host, accessories });
            }
            // Decode game settings (with fallback defaults for backwards compat)
            let settings = decode_settings_from_snapshot(data, &mut pos);
            ServerIncoming::Lobby(ServerMsg::LobbySnapshot {
                my_index,
                state: LobbyState { slots, settings },
            })
        }
        0x82 => ServerIncoming::Lobby(ServerMsg::Rejected { reason: data[1] }),
        0x83 => ServerIncoming::Lobby(ServerMsg::GameStart),
        0x84 => {
            let name_len = data[1] as usize;
            if data.len() < 2 + name_len { return None; }
            let name = String::from_utf8_lossy(&data[2..2 + name_len]).to_string();
            ServerIncoming::Lobby(ServerMsg::PlayerLeft { name })
        }
        0x85 => {
            let name_len = data[1] as usize;
            if data.len() < 2 + name_len { return None; }
            let name = String::from_utf8_lossy(&data[2..2 + name_len]).to_string();
            ServerIncoming::Lobby(ServerMsg::Disbanded { host_name: name })
        }
        0x86 => ServerIncoming::Lobby(ServerMsg::Rematch),
        // Game message
        0x90 => {
            let snap = net::decode_snapshot(&data[1..])?;
            ServerIncoming::Snapshot(snap)
        }
        _ => return None,
    };
    Some((msg, consumed))
}

fn read_f32_lobby(data: &[u8], pos: &mut usize) -> f32 {
    if *pos + 4 > data.len() { return 0.0; }
    let v = f32::from_be_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3]]);
    *pos += 4;
    v
}

fn read_u8_lobby(data: &[u8], pos: &mut usize) -> u8 {
    if *pos >= data.len() { return 0; }
    let v = data[*pos];
    *pos += 1;
    v
}

fn decode_settings_from_snapshot(data: &[u8], pos: &mut usize) -> GameSettings {
    if *pos >= data.len() {
        return GameSettings::default();
    }
    let wins_to_match = read_u8_lobby(data, pos) as i32;
    let spawn_invuln = read_f32_lobby(data, pos);
    let starting_hp = read_f32_lobby(data, pos);
    let gravity_scale = read_f32_lobby(data, pos);
    let turbo_speed = read_f32_lobby(data, pos);
    let sudden_death = read_u8_lobby(data, pos) != 0;
    let everyone_picks = read_u8_lobby(data, pos) != 0;
    // Validate: if we got zero values from short data, use defaults
    let defaults = GameSettings::default();
    GameSettings {
        wins_to_match: if wins_to_match == 0 { defaults.wins_to_match } else { wins_to_match },
        spawn_invuln,
        starting_hp: if starting_hp == 0.0 { defaults.starting_hp } else { starting_hp },
        gravity_scale: if gravity_scale == 0.0 { defaults.gravity_scale } else { gravity_scale },
        turbo_speed: if turbo_speed == 0.0 { defaults.turbo_speed } else { turbo_speed },
        sudden_death,
        everyone_picks,
    }
}

// ── Read buffer for streaming TCP ────────────────────────────────────────────

pub struct ReadBuffer {
    buf: Vec<u8>,
}

impl ReadBuffer {
    pub fn new() -> Self {
        Self { buf: Vec::with_capacity(1024) }
    }

    pub fn append(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    pub fn try_decode_client_incoming(&mut self) -> Option<ClientIncoming> {
        if let Some((msg, consumed)) = decode_client_incoming(&self.buf) {
            self.buf.drain(..consumed);
            Some(msg)
        } else {
            None
        }
    }

    pub fn try_decode_server_incoming(&mut self) -> Option<ServerIncoming> {
        if let Some((msg, consumed)) = decode_server_incoming(&self.buf) {
            self.buf.drain(..consumed);
            Some(msg)
        } else {
            None
        }
    }
}

pub fn encode_card_choice(card_index: u8) -> Vec<u8> {
    let payload_len: u16 = 2;
    let mut out = Vec::with_capacity(4);
    out.extend_from_slice(&payload_len.to_be_bytes());
    out.push(0x11);
    out.push(card_index);
    out
}
