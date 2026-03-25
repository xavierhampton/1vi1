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
    Explosion { x: f32, y: f32, z: f32, r: u8, g: u8, b: u8, radius: f32 },
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
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cards: Vec<(u8, f32)>,
    pub poison_timer: f32,
    pub ghost_timer: f32,
    pub overclock_timer: f32,
    pub overclock_crash_timer: f32,
    pub adrenaline_timer: f32,
    pub bloodthirsty_timer: f32,
    pub slow_timer: f32,
    pub shake_timer: f32,
    pub soul_siphon_bonus_hp: f32,
    pub doppel_ghost_x: f32,
    pub doppel_ghost_y: f32,
    pub doppel_ghost_aim_x: f32,
    pub doppel_ghost_aim_y: f32,
    pub upsized_stacks: i32,
    pub invuln_timer: f32,
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
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct StickyBombSnapshot {
    pub x: f32,
    pub y: f32,
    pub owner: u8,
    pub fuse: f32,
    pub stuck_to: u8,
}

#[derive(Debug, Clone)]
pub struct HealingZoneSnapshot {
    pub x: f32,
    pub y: f32,
    pub owner: u8,
    pub lifetime: f32,
}

#[derive(Debug, Clone)]
pub struct WorldSnapshot {
    pub state_tag: u8,
    pub state_timer: f32,
    pub time_scale: f32,
    pub level_id: u8,
    pub winner_index: u8,
    pub player_count: u8,
    pub players: Vec<PlayerSnapshot>,
    pub scores: Vec<i32>,
    pub bullets: Vec<BulletSnapshot>,
    pub events: Vec<GameEvent>,
    // Card pick fields
    pub card_current_picker: u8,
    pub card_offered: [u8; 3],
    pub card_remaining_pickers: u8,
    pub card_phase_timer: f32,
    pub card_chosen: u8,
    pub card_exit_timer: f32,
    pub card_hover: u8,
    // MatchOver fields
    pub match_winner: u8,
    pub match_timer: f32,
    // Entity snapshots
    pub sticky_bombs: Vec<StickyBombSnapshot>,
    pub healing_zones: Vec<HealingZoneSnapshot>,
}

// ── Encode/decode GameInput ──────────────────────────────────────────────────

pub fn encode_game_input(input: &PlayerInput) -> Vec<u8> {
    let mut payload = Vec::with_capacity(24);
    payload.push(0x10);
    payload.extend_from_slice(&input.move_dir.to_le_bytes());
    let flags: u8 = (input.jump_pressed as u8)
        | ((input.jump_held as u8) << 1)
        | ((input.shoot_pressed as u8) << 2)
        | ((input.ability_pressed as u8) << 3)
        | ((input.shoot_held as u8) << 4);
    payload.push(flags);
    payload.extend_from_slice(&input.aim_dir.x.to_le_bytes());
    payload.extend_from_slice(&input.aim_dir.y.to_le_bytes());
    payload.extend_from_slice(&input.cursor_x.to_le_bytes());
    payload.extend_from_slice(&input.cursor_y.to_le_bytes());
    payload.push(input.hover_card);

    let len = payload.len() as u16;
    let mut out = Vec::with_capacity(2 + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend(payload);
    out
}

pub fn decode_game_input(data: &[u8]) -> Option<PlayerInput> {
    if data.len() < 13 {
        return None;
    }
    let move_dir = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let flags = data[4];
    let aim_x = f32::from_le_bytes([data[5], data[6], data[7], data[8]]);
    let aim_y = f32::from_le_bytes([data[9], data[10], data[11], data[12]]);
    let (cursor_x, cursor_y) = if data.len() >= 21 {
        let cx = f32::from_le_bytes([data[13], data[14], data[15], data[16]]);
        let cy = f32::from_le_bytes([data[17], data[18], data[19], data[20]]);
        (cx, cy)
    } else {
        (0.5, 0.5)
    };
    let hover_card = if data.len() >= 22 { data[21] } else { 0xFF };
    Some(PlayerInput {
        move_dir,
        jump_pressed: flags & 1 != 0,
        jump_held: flags & 2 != 0,
        shoot_pressed: flags & 4 != 0,
        shoot_held: flags & 16 != 0,
        ability_pressed: flags & 8 != 0,
        aim_dir: Vector2::new(aim_x, aim_y),
        cursor_x,
        cursor_y,
        hover_card,
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
    payload.push(0x90);

    payload.push(snap.state_tag);
    push_f32(&mut payload, snap.state_timer);
    push_f32(&mut payload, snap.time_scale);
    payload.push(snap.level_id);
    payload.push(snap.winner_index);

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
        push_f32(&mut payload, p.cursor_x);
        push_f32(&mut payload, p.cursor_y);
        payload.push(p.cards.len() as u8);
        for (card_id, cooldown) in &p.cards {
            payload.push(*card_id);
            push_f32(&mut payload, *cooldown);
        }
        push_f32(&mut payload, p.poison_timer);
        push_f32(&mut payload, p.ghost_timer);
        push_f32(&mut payload, p.overclock_timer);
        push_f32(&mut payload, p.overclock_crash_timer);
        push_f32(&mut payload, p.adrenaline_timer);
        push_f32(&mut payload, p.bloodthirsty_timer);
        push_f32(&mut payload, p.slow_timer);
        push_f32(&mut payload, p.shake_timer);
        push_f32(&mut payload, p.soul_siphon_bonus_hp);
        push_f32(&mut payload, p.doppel_ghost_x);
        push_f32(&mut payload, p.doppel_ghost_y);
        push_f32(&mut payload, p.doppel_ghost_aim_x);
        push_f32(&mut payload, p.doppel_ghost_aim_y);
        push_i32(&mut payload, p.upsized_stacks);
        push_f32(&mut payload, p.invuln_timer);
    }

    for s in &snap.scores {
        push_i32(&mut payload, *s);
    }

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
        push_f32(&mut payload, b.radius);
    }

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
            GameEvent::Explosion { x, y, z, r, g, b, radius } => {
                payload.push(4);
                push_f32(&mut payload, *x);
                push_f32(&mut payload, *y);
                push_f32(&mut payload, *z);
                payload.push(*r); payload.push(*g); payload.push(*b);
                push_f32(&mut payload, *radius);
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

    // Card pick fields
    payload.push(snap.card_current_picker);
    payload.push(snap.card_offered[0]);
    payload.push(snap.card_offered[1]);
    payload.push(snap.card_offered[2]);
    payload.push(snap.card_remaining_pickers);
    push_f32(&mut payload, snap.card_phase_timer);
    payload.push(snap.card_chosen);
    push_f32(&mut payload, snap.card_exit_timer);
    payload.push(snap.card_hover);

    // MatchOver fields
    payload.push(snap.match_winner);
    push_f32(&mut payload, snap.match_timer);

    // Entity snapshots
    payload.push(snap.sticky_bombs.len() as u8);
    for s in &snap.sticky_bombs {
        push_f32(&mut payload, s.x);
        push_f32(&mut payload, s.y);
        payload.push(s.owner);
        push_f32(&mut payload, s.fuse);
        payload.push(s.stuck_to);
    }
    payload.push(snap.healing_zones.len() as u8);
    for h in &snap.healing_zones {
        push_f32(&mut payload, h.x);
        push_f32(&mut payload, h.y);
        payload.push(h.owner);
        push_f32(&mut payload, h.lifetime);
    }

    let len = payload.len() as u16;
    let mut out = Vec::with_capacity(2 + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend(payload);
    out
}

pub fn decode_snapshot(data: &[u8]) -> Option<WorldSnapshot> {
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
        if pos + 50 > data.len() { return None; }
        let ps_pos_x = read_f32(data, &mut pos);
        let ps_pos_y = read_f32(data, &mut pos);
        let ps_vel_x = read_f32(data, &mut pos);
        let ps_vel_y = read_f32(data, &mut pos);
        let ps_aim_x = read_f32(data, &mut pos);
        let ps_aim_y = read_f32(data, &mut pos);
        let ps_hp = read_f32(data, &mut pos);
        let ps_hit_flash = read_f32(data, &mut pos);
        let ps_reload_timer = read_f32(data, &mut pos);
        let ps_shoot_cooldown = read_f32(data, &mut pos);
        let ps_bullets_remaining = read_u8(data, &mut pos) as i8;
        let ps_alive = read_u8(data, &mut pos) != 0;
        let ps_cursor_x = read_f32(data, &mut pos);
        let ps_cursor_y = read_f32(data, &mut pos);
        let card_count = if pos < data.len() { read_u8(data, &mut pos) } else { 0 };
        let mut cards = Vec::with_capacity(card_count as usize);
        for _ in 0..card_count {
            if pos + 5 > data.len() { break; }
            let card_id = read_u8(data, &mut pos);
            let cooldown = read_f32(data, &mut pos);
            cards.push((card_id, cooldown));
        }
        let ps_poison = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_ghost = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_overclock = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_overclock_crash = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_adrenaline = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_bloodthirsty = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_slow = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_shake = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_soul_siphon = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_doppel_gx = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_doppel_gy = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_doppel_ax = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_doppel_ay = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        let ps_upsized = if pos + 4 <= data.len() { read_i32(data, &mut pos) } else { 0 };
        let ps_invuln = if pos + 4 <= data.len() { read_f32(data, &mut pos) } else { 0.0 };
        players.push(PlayerSnapshot {
            pos_x: ps_pos_x, pos_y: ps_pos_y,
            vel_x: ps_vel_x, vel_y: ps_vel_y,
            aim_x: ps_aim_x, aim_y: ps_aim_y,
            hp: ps_hp, hit_flash: ps_hit_flash,
            reload_timer: ps_reload_timer, shoot_cooldown: ps_shoot_cooldown,
            bullets_remaining: ps_bullets_remaining, alive: ps_alive,
            cursor_x: ps_cursor_x, cursor_y: ps_cursor_y,
            cards,
            poison_timer: ps_poison,
            ghost_timer: ps_ghost,
            overclock_timer: ps_overclock,
            overclock_crash_timer: ps_overclock_crash,
            adrenaline_timer: ps_adrenaline,
            bloodthirsty_timer: ps_bloodthirsty,
            slow_timer: ps_slow,
            shake_timer: ps_shake,
            soul_siphon_bonus_hp: ps_soul_siphon,
            doppel_ghost_x: ps_doppel_gx,
            doppel_ghost_y: ps_doppel_gy,
            doppel_ghost_aim_x: ps_doppel_ax,
            doppel_ghost_aim_y: ps_doppel_ay,
            upsized_stacks: ps_upsized,
            invuln_timer: ps_invuln,
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
        if pos + 41 > data.len() { return None; }
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
            radius: read_f32(data, &mut pos),
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
            4 => {
                if pos + 19 > data.len() { return None; }
                let x = read_f32(data, &mut pos);
                let y = read_f32(data, &mut pos);
                let z = read_f32(data, &mut pos);
                let r = read_u8(data, &mut pos);
                let g = read_u8(data, &mut pos);
                let b = read_u8(data, &mut pos);
                let radius = read_f32(data, &mut pos);
                events.push(GameEvent::Explosion { x, y, z, r, g, b, radius });
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

    // Card pick fields
    let (card_current_picker, card_offered, card_remaining_pickers, card_phase_timer, card_chosen, card_exit_timer, card_hover) =
        if pos + 15 <= data.len() {
            let picker = read_u8(data, &mut pos);
            let c0 = read_u8(data, &mut pos);
            let c1 = read_u8(data, &mut pos);
            let c2 = read_u8(data, &mut pos);
            let remaining = read_u8(data, &mut pos);
            let phase = read_f32(data, &mut pos);
            let chosen = read_u8(data, &mut pos);
            let exit = read_f32(data, &mut pos);
            let hover = read_u8(data, &mut pos);
            (picker, [c0, c1, c2], remaining, phase, chosen, exit, hover)
        } else {
            (0, [0, 0, 0], 0, 0.0, 0xFF, 0.0, 0xFF)
        };

    // MatchOver fields
    let (match_winner, match_timer) = if pos + 5 <= data.len() {
        let w = read_u8(data, &mut pos);
        let t = read_f32(data, &mut pos);
        (w, t)
    } else {
        (0, 0.0)
    };

    // Entity snapshots
    let mut sticky_bombs = Vec::new();
    if pos < data.len() {
        let count = read_u8(data, &mut pos);
        for _ in 0..count {
            if pos + 14 > data.len() { break; }
            sticky_bombs.push(StickyBombSnapshot {
                x: read_f32(data, &mut pos),
                y: read_f32(data, &mut pos),
                owner: read_u8(data, &mut pos),
                fuse: read_f32(data, &mut pos),
                stuck_to: read_u8(data, &mut pos),
            });
        }
    }
    let mut healing_zones = Vec::new();
    if pos < data.len() {
        let count = read_u8(data, &mut pos);
        for _ in 0..count {
            if pos + 13 > data.len() { break; }
            healing_zones.push(HealingZoneSnapshot {
                x: read_f32(data, &mut pos),
                y: read_f32(data, &mut pos),
                owner: read_u8(data, &mut pos),
                lifetime: read_f32(data, &mut pos),
            });
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
        card_current_picker,
        card_offered,
        card_remaining_pickers,
        card_phase_timer,
        card_chosen,
        card_exit_timer,
        card_hover,
        match_winner,
        match_timer,
        sticky_bombs,
        healing_zones,
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
                    winner_index: self.winner_index,
                    winner_name: name,
                    winner_color: (c.r, c.g, c.b),
                    timer: self.state_timer,
                }
            }
            3 => GameState::CardPick {
                winner_index: self.winner_index,
                current_picker: self.card_current_picker,
                offered_cards: self.card_offered,
                pick_order: Vec::new(),
                phase_timer: self.card_phase_timer,
                chosen_card: if self.card_chosen == 0xFF { None } else { Some(self.card_chosen) },
                exit_timer: self.card_exit_timer,
            },
            4 => GameState::MatchOver {
                winner_index: self.match_winner,
                timer: self.match_timer,
            },
            _ => GameState::Playing,
        }
    }
}
