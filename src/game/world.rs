use raylib::prelude::*;

use crate::combat::bullet::{Bullet, SHOOT_COOLDOWN};
use crate::combat::combat::update_bullets;
use crate::combat::particles::{update_particles, Particle, Rng};
use crate::game::state::GameState;
use crate::level::level::Level;
use crate::player::input;
use crate::player::movement;
use crate::player::player::Player;

pub const MAX_BULLETS: i32 = 3;
pub const RELOAD_TIME: f32 = 1.5;

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
            state: GameState::Playing,
            rng: Rng::new(seed),
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, camera: &Camera3D, dt: f32) {
        match self.state {
            GameState::Playing => {
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

                // Tick hit flash timers
                for player in self.players.iter_mut() {
                    player.hit_flash_timer = (player.hit_flash_timer - dt).max(0.0);
                }

                // Update bullets + particles
                update_bullets(&mut self.bullets, &mut self.players, &self.level.platforms, &mut self.particles, &mut self.rng, dt);
                update_particles(&mut self.particles, dt);

                // Reset if fallen off map
                if self.players[0].position.y < -10.0 {
                    self.players[0].position = self.level.spawn_points[0];
                    self.players[0].velocity = Vector3::new(0.0, 0.0, 0.0);
                }
            }
        }
    }
}
