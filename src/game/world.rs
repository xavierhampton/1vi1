use raylib::prelude::*;

use crate::combat::bullet::{Bullet, SHOOT_COOLDOWN};
use crate::combat::combat::update_bullets;
use crate::combat::particles::{spawn_death_explosion, update_particles, Particle, Rng};
use crate::game::state::GameState;
use crate::level::level::Level;
use crate::lobby::state::LobbyState;
use crate::player::input;
use crate::player::movement;
use crate::player::player::Player;

pub const MAX_BULLETS: i32 = 3;
pub const RELOAD_TIME: f32 = 1.5;

const COUNTDOWN_DURATION: f32 = 3.0;
const ROUND_END_DURATION: f32 = 3.5;
const SLOW_MO_FACTOR: f32 = 0.25;

const PLAYER_COLORS: [Color; 4] = [
    Color::new(80, 180, 255, 255),  // Blue
    Color::new(255, 100, 80, 255),  // Red
    Color::new(100, 230, 120, 255), // Green
    Color::new(255, 200, 60, 255),  // Yellow
];

const PLAYER_NAMES: [&str; 4] = ["Xavier", "Keehin", "P3", "P4"];

pub struct World {
    pub players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    pub particles: Vec<Particle>,
    pub level: Level,
    pub state: GameState,
    pub scores: Vec<i32>,
    rng: Rng,
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

    fn kill_player(&mut self, idx: usize) {
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
                self.update_playing(rl, camera, dt);
            }
            GameState::RoundEnd { timer, .. } => {
                let timer = *timer;
                let slow_dt = dt * SLOW_MO_FACTOR;

                // Winner can still move during slow-mo
                if let Some(winner_idx) = self.last_alive() {
                    if winner_idx == 0 {
                        let p1_input = input::read_input(rl, camera);
                        movement::update(&mut self.players[0], &p1_input, &self.level.platforms, slow_dt);

                        let center_y = self.players[0].position.y + self.players[0].size.y / 2.0;
                        let dx = p1_input.aim_target.x - self.players[0].position.x;
                        let dy = p1_input.aim_target.y - center_y;
                        let len = (dx * dx + dy * dy).sqrt();
                        if len > 0.001 {
                            self.players[0].aim_dir = Vector2::new(dx / len, dy / len);
                        }
                    }
                }

                update_particles(&mut self.particles, slow_dt);

                let new_timer = timer - dt; // Timer runs at real speed
                if new_timer <= 0.0 {
                    self.reset_round();
                } else {
                    // Reconstruct state with updated timer
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

    fn update_playing(&mut self, rl: &RaylibHandle, camera: &Camera3D, dt: f32) {
        // Only process input for player 0 if alive
        if self.players[0].alive {
            let p1_input = input::read_input(rl, camera);
            movement::update(&mut self.players[0], &p1_input, &self.level.platforms, dt);

            // Aim direction from mouse
            let center_y = self.players[0].position.y + self.players[0].size.y / 2.0;
            let dx = p1_input.aim_target.x - self.players[0].position.x;
            let dy = p1_input.aim_target.y - center_y;
            let len = (dx * dx + dy * dy).sqrt();
            if len > 0.001 {
                self.players[0].aim_dir = Vector2::new(dx / len, dy / len);
            }

            // Reload
            if self.players[0].reload_timer > 0.0 {
                self.players[0].reload_timer = (self.players[0].reload_timer - dt).max(0.0);
                if self.players[0].reload_timer <= 0.0 {
                    self.players[0].bullets_remaining = MAX_BULLETS;
                }
            }

            // Shoot
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

        // Tick hit flash timers
        for player in self.players.iter_mut() {
            player.hit_flash_timer = (player.hit_flash_timer - dt).max(0.0);
        }

        // Update bullets + particles
        update_bullets(&mut self.bullets, &mut self.players, &self.level.platforms, &mut self.particles, &mut self.rng, dt);
        update_particles(&mut self.particles, dt);

        // Check for deaths (HP <= 0 or fallen off map)
        for i in 0..self.players.len() {
            if !self.players[i].alive {
                continue;
            }
            if self.players[i].hp <= 0.0 || self.players[i].position.y < -10.0 {
                self.kill_player(i);
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
                // Everyone died — draw, just restart
                self.reset_round();
            }
        }
    }
}
