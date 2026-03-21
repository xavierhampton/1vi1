use raylib::prelude::*;

use crate::physics::collision::AABB;

pub struct Player {
    pub position: Vector3,
    pub velocity: Vector3,
    pub size: Vector3,
    pub color: Color,
    pub grounded: bool,
    pub coyote_timer: f32,
    pub jump_cut_applied: bool,
    pub air_jumps: i32,
    pub aim_dir: Vector2,
    pub shoot_cooldown: f32,
    pub bullets_remaining: i32,
    pub reload_timer: f32,
    pub name: String,
    pub hp: f32,
    pub max_hp: f32,
}

impl Player {
    pub fn new(position: Vector3, size: Vector3, color: Color, name: &str) -> Self {
        Self {
            position,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            size,
            color,
            grounded: false,
            coyote_timer: 0.0,
            jump_cut_applied: false,
            air_jumps: 0,
            aim_dir: Vector2::new(1.0, 0.0),
            shoot_cooldown: 0.0,
            bullets_remaining: 3,
            reload_timer: 0.0,
            name: name.to_string(),
            hp: 100.0,
            max_hp: 100.0,
        }
    }

    pub fn aabb(&self) -> AABB {
        AABB {
            min: Vector3::new(
                self.position.x - self.size.x / 2.0,
                self.position.y,
                self.position.z - self.size.z / 2.0,
            ),
            max: Vector3::new(
                self.position.x + self.size.x / 2.0,
                self.position.y + self.size.y,
                self.position.z + self.size.z / 2.0,
            ),
        }
    }

    pub fn render_center(&self) -> Vector3 {
        Vector3::new(
            self.position.x,
            self.position.y + self.size.y / 2.0,
            self.position.z,
        )
    }
}
