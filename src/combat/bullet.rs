use raylib::prelude::*;

use crate::physics::collision::AABB;

pub const BULLET_SPEED: f32 = 30.0;
pub const BULLET_GRAVITY: f32 = 15.0;
pub const BULLET_RADIUS: f32 = 0.08;
pub const BULLET_LIFETIME: f32 = 3.0;
pub const BULLET_DAMAGE: f32 = 25.0;
pub const SHOOT_COOLDOWN: f32 = 0.3;
pub struct Bullet {
    pub position: Vector3,
    pub prev_position: Vector3,
    pub velocity: Vector2,
    pub owner: usize,
    pub lifetime: f32,
    pub color: Color,
}

impl Bullet {
    pub fn new(position: Vector3, aim_dir: Vector2, owner: usize, color: Color) -> Self {
        Self {
            position,
            prev_position: position,
            velocity: Vector2::new(aim_dir.x * BULLET_SPEED, aim_dir.y * BULLET_SPEED),
            owner,
            lifetime: BULLET_LIFETIME,
            color,
        }
    }

    pub fn aabb(&self) -> AABB {
        let r = BULLET_RADIUS;
        AABB {
            min: Vector3::new(
                self.position.x - r,
                self.position.y - r,
                self.position.z - r,
            ),
            max: Vector3::new(
                self.position.x + r,
                self.position.y + r,
                self.position.z + r,
            ),
        }
    }
}
