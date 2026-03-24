use raylib::prelude::*;

use crate::game::cards::PlayerStats;
use crate::physics::collision::AABB;

pub const BULLET_SPEED: f32 = 30.0;
pub const BULLET_GRAVITY: f32 = 15.0;
pub const BULLET_RADIUS: f32 = 0.08;
pub const BULLET_LIFETIME: f32 = 3.0;
pub const BULLET_DAMAGE: f32 = 25.0;
pub const SHOOT_COOLDOWN: f32 = 0.3;

#[derive(Clone)]
pub struct Bullet {
    pub position: Vector3,
    pub prev_position: Vector3,
    pub velocity: Vector2,
    pub owner: usize,
    pub lifetime: f32,
    pub color: Color,
    // Modifier fields (set from owner's stats at creation)
    pub radius: f32,
    pub damage: f32,
    pub bounces_remaining: i32,
    pub homing: bool,
    pub piercing: bool,
    pub explosive: bool,
    pub poison: bool,
    pub gravity_mult: f32,
    pub phantom: bool,
    pub split_on_hit: bool,
    pub hot_potato: bool,
    pub sticky: bool,
}

impl Bullet {
    #[allow(dead_code)]
    pub fn new(position: Vector3, aim_dir: Vector2, owner: usize, color: Color) -> Self {
        Self {
            position,
            prev_position: position,
            velocity: Vector2::new(aim_dir.x * BULLET_SPEED, aim_dir.y * BULLET_SPEED),
            owner,
            lifetime: BULLET_LIFETIME,
            color,
            radius: BULLET_RADIUS,
            damage: BULLET_DAMAGE,
            bounces_remaining: 0,
            homing: false,
            piercing: false,
            explosive: false,
            poison: false,
            gravity_mult: 1.0,
            phantom: false,
            split_on_hit: false,
            hot_potato: false,
            sticky: false,
        }
    }

    /// Create a bullet with modifiers derived from owner's stats
    pub fn new_with_stats(
        position: Vector3,
        aim_dir: Vector2,
        owner: usize,
        color: Color,
        stats: &PlayerStats,
    ) -> Self {
        let speed = BULLET_SPEED * stats.bullet_speed_mult;
        let damage = BULLET_DAMAGE * stats.bullet_damage_mult;
        Self {
            position,
            prev_position: position,
            velocity: Vector2::new(aim_dir.x * speed, aim_dir.y * speed),
            owner,
            lifetime: BULLET_LIFETIME,
            color,
            radius: BULLET_RADIUS * stats.bullet_radius_mult,
            damage,
            bounces_remaining: stats.rubber_bounces,
            homing: stats.homing,
            piercing: stats.piercing,
            explosive: stats.explosive,
            poison: stats.poison,
            gravity_mult: stats.bullet_gravity_mult,
            phantom: stats.phantom,
            split_on_hit: stats.split_shot,
            hot_potato: stats.hot_potato,
            sticky: stats.sticky,
        }
    }

    pub fn aabb(&self) -> AABB {
        let r = self.radius;
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
