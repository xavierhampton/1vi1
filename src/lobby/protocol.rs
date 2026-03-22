use crate::lobby::state::{LobbyColor, LobbyState, PlayerSlot};

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ClientMsg {
    Join { name: String },
    ChangeColor { color: u8 },
    ToggleReady,
    Leave,
}

#[derive(Debug)]
pub enum ServerMsg {
    LobbySnapshot { my_index: u8, state: LobbyState },
    Rejected { reason: u8 },
    GameStart,
}

// Rejection reasons
pub const REJECT_FULL: u8 = 1;

// ── Wire format: [u16 len][u8 type][payload...] ─────────────────────────────

pub fn encode_client(msg: &ClientMsg) -> Vec<u8> {
    let mut payload = Vec::new();
    match msg {
        ClientMsg::Join { name } => {
            payload.push(0x01);
            let bytes = name.as_bytes();
            payload.push(bytes.len() as u8);
            payload.extend_from_slice(bytes);
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
            }
        }
        ServerMsg::Rejected { reason } => {
            payload.push(0x82);
            payload.push(*reason);
        }
        ServerMsg::GameStart => {
            payload.push(0x83);
        }
    }
    let len = payload.len() as u16;
    let mut out = Vec::with_capacity(2 + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend(payload);
    out
}

pub fn decode_client(buf: &[u8]) -> Option<(ClientMsg, usize)> {
    if buf.len() < 3 {
        return None;
    }
    let len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
    if buf.len() < 2 + len {
        return None;
    }
    let data = &buf[2..2 + len];
    let msg = match data[0] {
        0x01 => {
            let name_len = data[1] as usize;
            let name = String::from_utf8_lossy(&data[2..2 + name_len]).to_string();
            ClientMsg::Join { name }
        }
        0x02 => ClientMsg::ChangeColor { color: data[1] },
        0x03 => ClientMsg::ToggleReady,
        0x04 => ClientMsg::Leave,
        _ => return None,
    };
    Some((msg, 2 + len))
}

pub fn decode_server(buf: &[u8]) -> Option<(ServerMsg, usize)> {
    if buf.len() < 3 {
        return None;
    }
    let len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
    if buf.len() < 2 + len {
        return None;
    }
    let data = &buf[2..2 + len];
    let msg = match data[0] {
        0x81 => {
            let my_index = data[1];
            let slot_count = data[2] as usize;
            let mut pos = 3;
            let mut slots = Vec::new();
            for _ in 0..slot_count {
                let name_len = data[pos] as usize;
                pos += 1;
                let name = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
                pos += name_len;
                let color = LobbyColor::from_u8(data[pos]).unwrap_or(LobbyColor::Blue);
                let ready = data[pos + 1] != 0;
                let is_host = data[pos + 2] != 0;
                pos += 3;
                slots.push(PlayerSlot { name, color, ready, is_host });
            }
            ServerMsg::LobbySnapshot {
                my_index,
                state: LobbyState { slots },
            }
        }
        0x82 => ServerMsg::Rejected { reason: data[1] },
        0x83 => ServerMsg::GameStart,
        _ => return None,
    };
    Some((msg, 2 + len))
}

// ── Read buffer for streaming TCP ────────────────────────────────────────────

pub struct ReadBuffer {
    buf: Vec<u8>,
}

impl ReadBuffer {
    pub fn new() -> Self {
        Self { buf: Vec::with_capacity(512) }
    }

    pub fn append(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    pub fn try_decode_client(&mut self) -> Option<ClientMsg> {
        if let Some((msg, consumed)) = decode_client(&self.buf) {
            self.buf.drain(..consumed);
            Some(msg)
        } else {
            None
        }
    }

    pub fn try_decode_server(&mut self) -> Option<ServerMsg> {
        if let Some((msg, consumed)) = decode_server(&self.buf) {
            self.buf.drain(..consumed);
            Some(msg)
        } else {
            None
        }
    }
}
