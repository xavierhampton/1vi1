use raylib::prelude::*;

use crate::game::state::GameState;
use crate::level::level::Level;
use crate::player::input;
use crate::player::movement;
use crate::player::player::Player;

pub struct World {
    pub players: [Player; 2],
    pub level: Level,
    pub state: GameState,
}

impl World {
    pub fn new() -> Self {
        let level = Level::test_level();
        let players = [
            Player::new(
                level.spawn_points[0],
                Vector3::new(0.6, 1.6, 0.6),
                Color::new(80, 180, 255, 255),
            ),
            Player::new(
                level.spawn_points[1],
                Vector3::new(0.6, 1.6, 0.6),
                Color::new(255, 100, 80, 255),
            ),
        ];
        Self {
            players,
            level,
            state: GameState::Playing,
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

                // Reset if fallen off map
                if self.players[0].position.y < -10.0 {
                    self.players[0].position = self.level.spawn_points[0];
                    self.players[0].velocity = Vector3::new(0.0, 0.0, 0.0);
                }
            }
        }
    }
}
