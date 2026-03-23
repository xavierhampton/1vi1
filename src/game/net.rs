use raylib::prelude::*;

use crate::game::state::GameState;
use crate::player::input::PlayerInput;

// ── Game events (server → client, for spawning particles) ────────────────────

#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerHit { x: f32, y: f32, z: f32, r: u8, g: u8, b: u8 },
    PlayerDied { x: f32, y: f32, z: f32, r: u8, g: u8, b: u8 },
    TerrainHit { x: f32, y: f32, z: f32, r: u8, g: u8, b: u8 },
    BulletFired { x: f32, y: f32, z: f32, vx: f32, vy: f32, owner: u8, r: u8, g: u8, b: u8 },
}

// ── Snapshot types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PlayerSnapshot {
    pub pos_x: f32,
    pub pos_y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub aim_x: f32,
    pub aim_y: f32,
    pub hp: f32,
    pub hit_flash: f32,
    pub reload_timer: f32,
    pub shoot_cooldown: f32,
    pub bullets_remaining: i8,
    pub alive: bool,
}

#[derive(Debug, Clone)]
pub struct BulletSnapshot {
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub prev_x: f32,
    pub prev_y: f32,
    pub prev_z: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub owner: u8,
    pub lifetime: f32,
}

#[derive(Debug, Clone)]
pub struct WorldSnapshot {
    pub state_tag: u8, // 0=RoundStart, 1=Playing, 2=RoundEnd
    pub state_timer: f32,
    pub time_scale: f32, // 1.0 normal, 0.25 slow-mo, 0.0 frozen
    pub level_id: u8,
    pub winner_index: u8,
    pub player_count: u8,
    pub players: Vec<PlayerSnapshot>,
    pub scores: Vec<i32>,
    pub bullets: Vec<BulletSnapshot>,
    pub events: Vec<GameEvent>,
}

// ── Encode/decode GameInput ──────────────────────────────────────────────────

pub fn encode_game_input(input: &PlayerInput) -> Vec<u8> {
    let mut payload = Vec::with_capacity(15);
    payload.push(0x10);
    payload.extend_from_slice(&input.move_dir.to_le_bytes());
    let flags: u8 = (input.jump_pressed as u8)
        | ((input.jump_held as u8) << 1)
        | ((input.shoot_pressed as u8) << 2);
    payload.push(flags);
    payload.extend_from_slice(&input.aim_dir.x.to_le_bytes());
    payload.extend_from_slice(&input.aim_dir.y.to_le_bytes());

    let len = payload.len() as u16;
    let mut out = Vec::with_capacity(2 + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend(payload);
    out
}

pub fn decode_game_input(data: &[u8]) -> Option<PlayerInput> {
    // data starts after the type byte, so it should be 12 bytes
    if data.len() < 12 {
        return None;
    }
    let move_dir = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let flags = data[4];
    let aim_x = f32::from_le_bytes([data[5], data[6], data[7], data[8]]);
    let aim_y = f32::from_le_bytes([data[9], data[10], data[11], data[12]]);
    Some(PlayerInput {
        move_dir,
        jump_pressed: flags & 1 != 0,
        jump_held: flags & 2 != 0,
        shoot_pressed: flags & 4 != 0,
        aim_dir: Vector2::new(aim_x, aim_y),
    })
}

// ── Encode/decode WorldSnapshot ──────────────────────────────────────────────

fn push_f32(buf: &mut Vec<u8>, v: f32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn push_i32(buf: &mut Vec<u8>, v: i32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn read_f32(data: &[u8], pos: &mut usize) -> f32 {
    let v = f32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
    *pos += 4;
    v
}

fn read_i32(data: &[u8], pos: &mut usize) -> i32 {
    let v = i32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
    *pos += 4;
    v
}

fn read_u8(data: &[u8], pos: &mut usize) -> u8 {
    let v = data[*pos];
    *pos += 1;
    v
}

pub fn encode_snapshot(snap: &WorldSnapshot) -> Vec<u8> {
    let mut payload = Vec::with_capacity(512);
    payload.push(0x90); // type

    // Game state
    payload.push(snap.state_tag);
    push_f32(&mut payload, snap.state_timer);
    push_f32(&mut payload, snap.time_scale);
    payload.push(snap.level_id);
    payload.push(snap.winner_index);

    // Players
    payload.push(snap.player_count);
    for p in &snap.players {
        push_f32(&mut payload, p.pos_x);
        push_f32(&mut payload, p.pos_y);
        push_f32(&mut payload, p.vel_x);
        push_f32(&mut payload, p.vel_y);
        push_f32(&mut payload, p.aim_x);
        push_f32(&mut payload, p.aim_y);
        push_f32(&mut payload, p.hp);
        push_f32(&mut payload, p.hit_flash);
        push_f32(&mut payload, p.reload_timer);
        push_f32(&mut payload, p.shoot_cooldown);
        payload.push(p.bullets_remaining as u8);
        payload.push(p.alive as u8);
    }

    // Scores
    for s in &snap.scores {
        push_i32(&mut payload, *s);
    }

    // Bullets
    payload.push(snap.bullets.len() as u8);
    for b in &snap.bullets {
        push_f32(&mut payload, b.pos_x);
        push_f32(&mut payload, b.pos_y);
        push_f32(&mut payload, b.pos_z);
        push_f32(&mut payload, b.prev_x);
        push_f32(&mut payload, b.prev_y);
        push_f32(&mut payload, b.prev_z);
        push_f32(&mut payload, b.vel_x);
        push_f32(&mut payload, b.vel_y);
        payload.push(b.owner);
        push_f32(&mut payload, b.lifetime);
    }

    // Events
    payload.push(snap.events.len() as u8);
    for ev in &snap.events {
        match ev {
            GameEvent::PlayerHit { x, y, z, r, g, b } => {
                payload.push(0);
                push_f32(&mut payload, *x);
                push_f32(&mut payload, *y);
                push_f32(&mut payload, *z);
                payload.push(*r); payload.push(*g); payload.push(*b);
            }
            GameEvent::PlayerDied { x, y, z, r, g, b } => {
                payload.push(1);
                push_f32(&mut payload, *x);
                push_f32(&mut payload, *y);
                push_f32(&mut payload, *z);
                payload.push(*r); payload.push(*g); payload.push(*b);
            }
            GameEvent::TerrainHit { x, y, z, r, g, b } => {
                payload.push(2);
                push_f32(&mut payload, *x);
                push_f32(&mut payload, *y);
                push_f32(&mut payload, *z);
                payload.push(*r); payload.push(*g); payload.push(*b);
            }
            GameEvent::BulletFired { x, y, z, vx, vy, owner, r, g, b } => {
                payload.push(3);
                push_f32(&mut payload, *x);
                push_f32(&mut payload, *y);
                push_f32(&mut payload, *z);
                push_f32(&mut payload, *vx);
                push_f32(&mut payload, *vy);
                payload.push(*owner);
                payload.push(*r); payload.push(*g); payload.push(*b);
            }
        }
    }

    let len = payload.len() as u16;
    let mut out = Vec::with_capacity(2 + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend(payload);
    out
}

pub fn decode_snapshot(data: &[u8]) -> Option<WorldSnapshot> {
    // data starts after the type byte
    if data.len() < 4 {
        return None;
    }
    let mut pos = 0;

    let state_tag = read_u8(data, &mut pos);
    let state_timer = read_f32(data, &mut pos);
    let time_scale = read_f32(data, &mut pos);
    let level_id = read_u8(data, &mut pos);
    let winner_index = read_u8(data, &mut pos);

    let player_count = read_u8(data, &mut pos);
    let mut players = Vec::with_capacity(player_count as usize);
    for _ in 0..player_count {
        if pos + 42 > data.len() { return None; }
        players.push(PlayerSnapshot {
            pos_x: read_f32(data, &mut pos),
            pos_y: read_f32(data, &mut pos),
            vel_x: read_f32(data, &mut pos),
            vel_y: read_f32(data, &mut pos),
            aim_x: read_f32(data, &mut pos),
            aim_y: read_f32(data, &mut pos),
            hp: read_f32(data, &mut pos),
            hit_flash: read_f32(data, &mut pos),
            reload_timer: read_f32(data, &mut pos),
            shoot_cooldown: read_f32(data, &mut pos),
            bullets_remaining: read_u8(data, &mut pos) as i8,
            alive: read_u8(data, &mut pos) != 0,
        });
    }

    let mut scores = Vec::with_capacity(player_count as usize);
    for _ in 0..player_count {
        if pos + 4 > data.len() { return None; }
        scores.push(read_i32(data, &mut pos));
    }

    if pos >= data.len() { return None; }
    let bullet_count = read_u8(data, &mut pos);
    let mut bullets = Vec::with_capacity(bullet_count as usize);
    for _ in 0..bullet_count {
        if pos + 37 > data.len() { return None; }
        bullets.push(BulletSnapshot {
            pos_x: read_f32(data, &mut pos),
            pos_y: read_f32(data, &mut pos),
            pos_z: read_f32(data, &mut pos),
            prev_x: read_f32(data, &mut pos),
            prev_y: read_f32(data, &mut pos),
            prev_z: read_f32(data, &mut pos),
            vel_x: read_f32(data, &mut pos),
            vel_y: read_f32(data, &mut pos),
            owner: read_u8(data, &mut pos),
            lifetime: read_f32(data, &mut pos),
        });
    }

    if pos >= data.len() { return None; }
    let event_count = read_u8(data, &mut pos);
    let mut events = Vec::with_capacity(event_count as usize);
    for _ in 0..event_count {
        if pos >= data.len() { return None; }
        let etype = read_u8(data, &mut pos);
        match etype {
            0 | 1 | 2 => {
                if pos + 15 > data.len() { return None; }
                let x = read_f32(data, &mut pos);
                let y = read_f32(data, &mut pos);
                let z = read_f32(data, &mut pos);
                let r = read_u8(data, &mut pos);
                let g = read_u8(data, &mut pos);
                let b = read_u8(data, &mut pos);
                events.push(match etype {
                    0 => GameEvent::PlayerHit { x, y, z, r, g, b },
                    1 => GameEvent::PlayerDied { x, y, z, r, g, b },
                    _ => GameEvent::TerrainHit { x, y, z, r, g, b },
                });
            }
            3 => {
                if pos + 21 > data.len() { return None; }
                let x = read_f32(data, &mut pos);
                let y = read_f32(data, &mut pos);
                let z = read_f32(data, &mut pos);
                let vx = read_f32(data, &mut pos);
                let vy = read_f32(data, &mut pos);
                let owner = read_u8(data, &mut pos);
                let r = read_u8(data, &mut pos);
                let g = read_u8(data, &mut pos);
                let b = read_u8(data, &mut pos);
                events.push(GameEvent::BulletFired { x, y, z, vx, vy, owner, r, g, b });
            }
            _ => {}
        }
    }

    Some(WorldSnapshot {
        state_tag,
        state_timer,
        time_scale,
        level_id,
        winner_index,
        player_count,
        players,
        scores,
        bullets,
        events,
    })
}

// ── Helpers for GameState conversion ─────────────────────────────────────────

impl WorldSnapshot {
    pub fn game_state(&self, names: &[String], colors: &[Color]) -> GameState {
        match self.state_tag {
            0 => GameState::RoundStart { timer: self.state_timer },
            2 => {
                let wi = self.winner_index as usize;
                let name = names.get(wi).cloned().unwrap_or_default();
                let c = colors.get(wi).copied().unwrap_or(Color::WHITE);
                GameState::RoundEnd {
                    winner_name: name,
                    winner_color: (c.r, c.g, c.b),
                    timer: self.state_timer,
                }
            }
            _ => GameState::Playing,
        }
    }
}
