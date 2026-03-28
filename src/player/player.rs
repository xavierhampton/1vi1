use raylib::prelude::*;

use crate::game::cards::{CardId, PlayerStats};
use crate::physics::collision::AABB;

pub const HIT_FLASH_DURATION: f32 = 0.15;

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
    pub hit_flash_timer: f32,
    pub name: String,
    pub hp: f32,
    pub max_hp: f32,
    pub alive: bool,
    pub cards: Vec<(CardId, f32)>, // (card, cooldown) — abilities use cooldown, powerups store 0.0
    pub stats: PlayerStats,        // computed from held powerup cards
    pub poison_timer: f32,
    pub ghost_timer: f32,
    pub overclock_timer: f32,
    pub overclock_crash_timer: f32,
    pub adrenaline_timer: f32,
    pub bloodthirsty_timer: f32,
    pub slow_timer: f32,           // ice shots: -50% speed for 2s
    pub shake_timer: f32,          // CaseOh: screen shake duration
    pub soul_siphon_bonus_hp: f32, // permanent max HP from kills
    pub doppel_history: Vec<(f32, f32, f32, f32, bool)>, // (x, y, aim_x, aim_y, shot) for DoppelGanger
    pub doppel_ghost: (f32, f32, f32, f32), // (x, y, aim_x, aim_y) — ghost rendering pos
    pub upsized_stacks: i32,
    pub rewind_history: Vec<(f32, f32, f32)>, // (x, y, hp) snapshots for Rewind
    pub rewind_sample_timer: f32,
    pub wall_dir: i8, // -1 = wall left, 0 = none, 1 = wall right
    pub invuln_timer: f32, // spawn invulnerability (Smash Bros style)
    pub accessories: Vec<(u8, u8, u8, u8)>, // up to 3: (id, r, g, b)
    pub lava_sizzle_cd: f32, // throttle for lava sizzle SFX/particles
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
            hit_flash_timer: 0.0,
            name: name.to_string(),
            hp: 100.0,
            max_hp: 100.0,
            alive: true,
            cards: Vec::new(),
            stats: PlayerStats::default(),
            poison_timer: 0.0,
            ghost_timer: 0.0,
            overclock_timer: 0.0,
            overclock_crash_timer: 0.0,
            adrenaline_timer: 0.0,
            bloodthirsty_timer: 0.0,
            slow_timer: 0.0,
            shake_timer: 0.0,
            soul_siphon_bonus_hp: 0.0,
            doppel_history: Vec::new(),
            doppel_ghost: (0.0, 0.0, 0.0, 0.0),
            upsized_stacks: 0,
            rewind_history: Vec::new(),
            rewind_sample_timer: 0.0,
            wall_dir: 0,
            invuln_timer: 0.0,
            accessories: Vec::new(),
            lava_sizzle_cd: 0.0,
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
