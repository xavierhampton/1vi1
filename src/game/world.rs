use raylib::prelude::*;

use crate::combat::bullet::{Bullet, SHOOT_COOLDOWN};
use crate::combat::combat::update_bullets;
use crate::combat::particles::{spawn_death_explosion, spawn_player_hit, spawn_terrain_hit, update_particles, Particle, Rng};
use crate::game::net::{BulletSnapshot, GameEvent, PlayerSnapshot, WorldSnapshot};
use crate::game::state::GameState;
use crate::level::level::Level;
use crate::lobby::state::LobbyState;
use crate::player::input::{self, PlayerInput};
use crate::player::movement;
use crate::player::player::Player;

pub const MAX_BULLETS: i32 = 3;
pub const RELOAD_TIME: f32 = 1.5;

pub const COUNTDOWN_DURATION: f32 = 3.0;
const ROUND_END_DURATION: f32 = 3.5;
const SLOW_MO_FACTOR: f32 = 0.25;

const PLAYER_COLORS: [Color; 4] = [
    Color::new(80, 180, 255, 255),
    Color::new(255, 100, 80, 255),
    Color::new(100, 230, 120, 255),
    Color::new(255, 200, 60, 255),
];

const PLAYER_NAMES: [&str; 4] = ["Xavier", "Keehin", "P3", "P4"];

pub struct World {
    pub players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    pub particles: Vec<Particle>,
    pub level: Level,
    pub state: GameState,
    pub scores: Vec<i32>,
    pub rng: Rng,
}

impl World {
    pub fn new() -> Self {
        Self::with_player_count(2)
    }

    pub fn with_player_count(count: usize) -> Self {
        let count = count.clamp(2, 4);
        let level = Level::test_level();
        let players = (0..count)
            .map(|i| {
                Player::new(
                    level.spawn_points[i],
                    Vector3::new(0.6, 1.6, 0.6),
                    PLAYER_COLORS[i],
                    PLAYER_NAMES[i],
                )
            })
            .collect();
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);

        Self {
            players,
            bullets: Vec::new(),
            particles: Vec::new(),
            level,
            state: GameState::RoundStart { timer: COUNTDOWN_DURATION },
            scores: vec![0; count],
            rng: Rng::new(seed),
        }
    }

    pub fn from_lobby(lobby: &LobbyState) -> Self {
        let level = Level::test_level();
        let count = lobby.slots.len().clamp(2, 4);
        let players = lobby.slots.iter().enumerate().take(count).map(|(i, slot)| {
            Player::new(
                level.spawn_points[i],
                Vector3::new(0.6, 1.6, 0.6),
                slot.color.to_color(),
                &slot.name,
            )
        }).collect();
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);

        Self {
            players,
            bullets: Vec::new(),
            particles: Vec::new(),
            level,
            state: GameState::RoundStart { timer: COUNTDOWN_DURATION },
            scores: vec![0; count],
            rng: Rng::new(seed),
        }
    }

    fn reset_round(&mut self) {
        for (i, player) in self.players.iter_mut().enumerate() {
            player.position = self.level.spawn_points[i];
            player.velocity = Vector3::new(0.0, 0.0, 0.0);
            player.hp = player.max_hp;
            player.alive = true;
            player.hit_flash_timer = 0.0;
            player.bullets_remaining = MAX_BULLETS;
            player.reload_timer = 0.0;
            player.shoot_cooldown = 0.0;
            player.aim_dir = Vector2::new(if i % 2 == 0 { 1.0 } else { -1.0 }, 0.0);
        }
        self.bullets.clear();
        self.particles.clear();
        self.state = GameState::RoundStart { timer: COUNTDOWN_DURATION };
    }

    fn kill_player_server(&mut self, idx: usize, events: &mut Vec<GameEvent>) {
        self.players[idx].alive = false;
        let pos = self.players[idx].render_center();
        let c = self.players[idx].color;
        events.push(GameEvent::PlayerDied {
            x: pos.x, y: pos.y, z: pos.z,
            r: c.r, g: c.g, b: c.b,
        });
    }

    fn kill_player_local(&mut self, idx: usize) {
        self.players[idx].alive = false;
        let pos = self.players[idx].render_center();
        let color = self.players[idx].color;
        spawn_death_explosion(&mut self.particles, &mut self.rng, pos, color);
    }

    fn alive_count(&self) -> usize {
        self.players.iter().filter(|p| p.alive).count()
    }

    fn last_alive(&self) -> Option<usize> {
        let alive: Vec<usize> = self.players.iter().enumerate()
            .filter(|(_, p)| p.alive)
            .map(|(i, _)| i)
            .collect();
        if alive.len() == 1 { Some(alive[0]) } else { None }
    }

    fn winner_index(&self) -> u8 {
        if let GameState::RoundEnd { .. } = &self.state {
            self.last_alive().unwrap_or(0) as u8
        } else {
            0
        }
    }

    // ── Server-authoritative update (processes ALL players) ──────────────────

    pub fn server_update(&mut self, inputs: &[PlayerInput], dt: f32) -> Vec<GameEvent> {
        let mut events = Vec::new();

        match &self.state {
            GameState::RoundStart { timer } => {
                let new_timer = *timer - dt;
                if new_timer <= 0.0 {
                    self.state = GameState::Playing;
                } else {
                    self.state = GameState::RoundStart { timer: new_timer };
                }
            }
            GameState::Playing => {
                self.server_update_playing(inputs, dt, &mut events);
            }
            GameState::RoundEnd { timer, .. } => {
                let timer = *timer;
                let slow_dt = dt * SLOW_MO_FACTOR;

                // Winner can still move during slow-mo
                if let Some(winner_idx) = self.last_alive() {
                    if let Some(inp) = inputs.get(winner_idx) {
                        movement::update(&mut self.players[winner_idx], inp, &self.level.platforms, slow_dt);
                        self.players[winner_idx].aim_dir = inp.aim_dir;
                    }
                }

                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    self.reset_round();
                } else {
                    let name = if let GameState::RoundEnd { winner_name, .. } = &self.state {
                        winner_name.clone()
                    } else { String::new() };
                    let color = if let GameState::RoundEnd { winner_color, .. } = &self.state {
                        *winner_color
                    } else { (255, 255, 255) };
                    self.state = GameState::RoundEnd { winner_name: name, winner_color: color, timer: new_timer };
                }
            }
        }

        events
    }

    fn server_update_playing(&mut self, inputs: &[PlayerInput], dt: f32, events: &mut Vec<GameEvent>) {
        // Process input for ALL alive players
        for i in 0..self.players.len() {
            if !self.players[i].alive {
                continue;
            }
            let inp = inputs.get(i).cloned().unwrap_or_else(PlayerInput::empty);

            movement::update(&mut self.players[i], &inp, &self.level.platforms, dt);
            self.players[i].aim_dir = inp.aim_dir;

            // Reload
            if self.players[i].reload_timer > 0.0 {
                self.players[i].reload_timer = (self.players[i].reload_timer - dt).max(0.0);
                if self.players[i].reload_timer <= 0.0 {
                    self.players[i].bullets_remaining = MAX_BULLETS;
                }
            }

            // Shoot
            self.players[i].shoot_cooldown = (self.players[i].shoot_cooldown - dt).max(0.0);
            if inp.shoot_pressed
                && self.players[i].shoot_cooldown <= 0.0
                && self.players[i].bullets_remaining > 0
                && self.players[i].reload_timer <= 0.0
            {
                let aim = self.players[i].aim_dir;
                let color = self.players[i].color;
                let spawn = Vector3::new(
                    self.players[i].position.x + aim.x * 0.5,
                    self.players[i].position.y + 1.1 + aim.y * 0.5,
                    self.players[i].position.z,
                );
                self.bullets.push(Bullet::new(spawn, aim, i, color));
                self.players[i].bullets_remaining -= 1;
                self.players[i].shoot_cooldown = SHOOT_COOLDOWN;
                if self.players[i].bullets_remaining == 0 {
                    self.players[i].reload_timer = RELOAD_TIME;
                }
            }
        }

        // Tick hit flash timers
        for player in self.players.iter_mut() {
            player.hit_flash_timer = (player.hit_flash_timer - dt).max(0.0);
        }

        // Update bullets
        let bullet_events = update_bullets(&mut self.bullets, &mut self.players, &self.level.platforms, dt);
        events.extend(bullet_events);

        // Check for deaths
        for i in 0..self.players.len() {
            if !self.players[i].alive {
                continue;
            }
            if self.players[i].hp <= 0.0 || self.players[i].position.y < -10.0 {
                self.kill_player_server(i, events);
            }
        }

        // Check for round end
        if self.alive_count() <= 1 {
            if let Some(winner_idx) = self.last_alive() {
                let name = self.players[winner_idx].name.clone();
                let c = self.players[winner_idx].color;
                self.scores[winner_idx] += 1;
                self.state = GameState::RoundEnd {
                    winner_name: name,
                    winner_color: (c.r, c.g, c.b),
                    timer: ROUND_END_DURATION,
                };
            } else {
                self.reset_round();
            }
        }
    }

    // ── Snapshot generation (server) ─────────────────────────────────────────

    pub fn to_snapshot(&self, events: Vec<GameEvent>) -> WorldSnapshot {
        let (state_tag, state_timer, time_scale) = match &self.state {
            GameState::RoundStart { timer } => (0, *timer, 0.0),
            GameState::Playing => (1, 0.0, 1.0),
            GameState::RoundEnd { timer, .. } => (2, *timer, SLOW_MO_FACTOR),
        };

        let players: Vec<PlayerSnapshot> = self.players.iter().map(|p| PlayerSnapshot {
            pos_x: p.position.x,
            pos_y: p.position.y,
            vel_x: p.velocity.x,
            vel_y: p.velocity.y,
            aim_x: p.aim_dir.x,
            aim_y: p.aim_dir.y,
            hp: p.hp,
            hit_flash: p.hit_flash_timer,
            reload_timer: p.reload_timer,
            shoot_cooldown: p.shoot_cooldown,
            bullets_remaining: p.bullets_remaining as i8,
            alive: p.alive,
        }).collect();

        let bullets: Vec<BulletSnapshot> = self.bullets.iter().map(|b| BulletSnapshot {
            pos_x: b.position.x,
            pos_y: b.position.y,
            pos_z: b.position.z,
            prev_x: b.prev_position.x,
            prev_y: b.prev_position.y,
            prev_z: b.prev_position.z,
            vel_x: b.velocity.x,
            vel_y: b.velocity.y,
            owner: b.owner as u8,
            lifetime: b.lifetime,
        }).collect();

        WorldSnapshot {
            state_tag,
            state_timer,
            time_scale,
            winner_index: self.winner_index(),
            player_count: self.players.len() as u8,
            players,
            scores: self.scores.clone(),
            bullets,
            events,
        }
    }

    // ── Snapshot application (client) ────────────────────────────────────────

    pub fn apply_snapshot(&mut self, snap: &WorldSnapshot) {
        // Update game state
        let names: Vec<String> = self.players.iter().map(|p| p.name.clone()).collect();
        let colors: Vec<Color> = self.players.iter().map(|p| p.color).collect();
        self.state = snap.game_state(&names, &colors);

        // Update players
        for (i, ps) in snap.players.iter().enumerate() {
            if i >= self.players.len() { break; }
            let p = &mut self.players[i];
            p.position.x = ps.pos_x;
            p.position.y = ps.pos_y;
            p.velocity.x = ps.vel_x;
            p.velocity.y = ps.vel_y;
            p.aim_dir.x = ps.aim_x;
            p.aim_dir.y = ps.aim_y;
            p.hp = ps.hp;
            p.hit_flash_timer = ps.hit_flash;
            p.reload_timer = ps.reload_timer;
            p.shoot_cooldown = ps.shoot_cooldown;
            p.bullets_remaining = ps.bullets_remaining as i32;
            p.alive = ps.alive;
        }

        // Update scores
        for (i, &s) in snap.scores.iter().enumerate() {
            if i < self.scores.len() {
                self.scores[i] = s;
            }
        }

        // Rebuild bullets from snapshot
        self.bullets.clear();
        for bs in &snap.bullets {
            let owner = bs.owner as usize;
            let color = if owner < self.players.len() {
                self.players[owner].color
            } else {
                Color::WHITE
            };
            self.bullets.push(Bullet {
                position: Vector3::new(bs.pos_x, bs.pos_y, bs.pos_z),
                prev_position: Vector3::new(bs.prev_x, bs.prev_y, bs.prev_z),
                velocity: Vector2::new(bs.vel_x, bs.vel_y),
                owner,
                lifetime: bs.lifetime,
                color,
            });
        }

        // Spawn particles from events
        for ev in &snap.events {
            match ev {
                GameEvent::PlayerHit { x, y, z, r, g, b } => {
                    spawn_player_hit(&mut self.particles, &mut self.rng,
                        Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
                }
                GameEvent::PlayerDied { x, y, z, r, g, b } => {
                    spawn_death_explosion(&mut self.particles, &mut self.rng,
                        Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
                }
                GameEvent::TerrainHit { x, y, z, r, g, b } => {
                    spawn_terrain_hit(&mut self.particles, &mut self.rng,
                        Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
                }
                GameEvent::BulletFired { .. } => {} // bullets already in snapshot
            }
        }
    }

    // ── Local single-player update (kept for menu/testing) ───────────────────

    pub fn update(&mut self, rl: &RaylibHandle, camera: &Camera3D, dt: f32) {
        match &self.state {
            GameState::RoundStart { timer } => {
                let timer = *timer;
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    self.state = GameState::Playing;
                } else {
                    self.state = GameState::RoundStart { timer: new_timer };
                }
            }
            GameState::Playing => {
                self.update_playing_local(rl, camera, dt);
            }
            GameState::RoundEnd { timer, .. } => {
                let timer = *timer;
                let slow_dt = dt * SLOW_MO_FACTOR;

                if let Some(winner_idx) = self.last_alive() {
                    if winner_idx == 0 {
                        let center = Vector2::new(
                            self.players[0].position.x,
                            self.players[0].position.y + self.players[0].size.y / 2.0,
                        );
                        let p1_input = input::read_input(rl, camera, center);
                        movement::update(&mut self.players[0], &p1_input, &self.level.platforms, slow_dt);
                        self.players[0].aim_dir = p1_input.aim_dir;
                    }
                }

                update_particles(&mut self.particles, slow_dt);

                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    self.reset_round();
                } else {
                    let name = if let GameState::RoundEnd { winner_name, .. } = &self.state {
                        winner_name.clone()
                    } else { String::new() };
                    let color = if let GameState::RoundEnd { winner_color, .. } = &self.state {
                        *winner_color
                    } else { (255, 255, 255) };
                    self.state = GameState::RoundEnd { winner_name: name, winner_color: color, timer: new_timer };
                }
            }
        }
    }

    fn update_playing_local(&mut self, rl: &RaylibHandle, camera: &Camera3D, dt: f32) {
        if self.players[0].alive {
            let center = Vector2::new(
                self.players[0].position.x,
                self.players[0].position.y + self.players[0].size.y / 2.0,
            );
            let p1_input = input::read_input(rl, camera, center);
            movement::update(&mut self.players[0], &p1_input, &self.level.platforms, dt);
            self.players[0].aim_dir = p1_input.aim_dir;

            if self.players[0].reload_timer > 0.0 {
                self.players[0].reload_timer = (self.players[0].reload_timer - dt).max(0.0);
                if self.players[0].reload_timer <= 0.0 {
                    self.players[0].bullets_remaining = MAX_BULLETS;
                }
            }

            self.players[0].shoot_cooldown = (self.players[0].shoot_cooldown - dt).max(0.0);
            if p1_input.shoot_pressed
                && self.players[0].shoot_cooldown <= 0.0
                && self.players[0].bullets_remaining > 0
                && self.players[0].reload_timer <= 0.0
            {
                let aim = self.players[0].aim_dir;
                let color = self.players[0].color;
                let spawn = Vector3::new(
                    self.players[0].position.x + aim.x * 0.5,
                    self.players[0].position.y + 1.1 + aim.y * 0.5,
                    self.players[0].position.z,
                );
                self.bullets.push(Bullet::new(spawn, aim, 0, color));
                self.players[0].bullets_remaining -= 1;
                self.players[0].shoot_cooldown = SHOOT_COOLDOWN;
                if self.players[0].bullets_remaining == 0 {
                    self.players[0].reload_timer = RELOAD_TIME;
                }
            }
        }

        for player in self.players.iter_mut() {
            player.hit_flash_timer = (player.hit_flash_timer - dt).max(0.0);
        }

        let bullet_events = update_bullets(&mut self.bullets, &mut self.players, &self.level.platforms, dt);
        for ev in &bullet_events {
            match ev {
                GameEvent::TerrainHit { x, y, z, r, g, b } => {
                    spawn_terrain_hit(&mut self.particles, &mut self.rng,
                        Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
                }
                GameEvent::PlayerHit { x, y, z, r, g, b } => {
                    spawn_player_hit(&mut self.particles, &mut self.rng,
                        Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
                }
                _ => {}
            }
        }
        update_particles(&mut self.particles, dt);

        for i in 0..self.players.len() {
            if !self.players[i].alive { continue; }
            if self.players[i].hp <= 0.0 || self.players[i].position.y < -10.0 {
                self.kill_player_local(i);
            }
        }

        if self.alive_count() <= 1 {
            if let Some(winner_idx) = self.last_alive() {
                let name = self.players[winner_idx].name.clone();
                let c = self.players[winner_idx].color;
                self.scores[winner_idx] += 1;
                self.state = GameState::RoundEnd {
                    winner_name: name,
                    winner_color: (c.r, c.g, c.b),
                    timer: ROUND_END_DURATION,
                };
            } else {
                self.reset_round();
            }
        }
    }
}
