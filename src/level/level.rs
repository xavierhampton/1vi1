use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::level::platforms::Platform;

// ── TOML schema ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct LevelsFile {
    pub level: Vec<LevelDef>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LevelDef {
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub spawn_points: Vec<[f32; 2]>,
    pub platforms: Vec<PlatformDef>,
    #[allow(dead_code)]
    #[serde(default, skip_serializing)]
    pub sawblades: Vec<SawbladeDef>,
    #[serde(default)]
    pub bounce_pads: Vec<BouncePadDef>,
    #[serde(default)]
    pub lava_pools: Vec<LavaPoolDef>,
    #[serde(default)]
    pub lasers: Vec<LaserBeamDef>,
}

fn default_enabled() -> bool { true }

#[derive(Serialize, Deserialize, Clone)]
pub struct PlatformDef {
    #[serde(rename = "type")]
    pub kind: String, // "wall" or "platform"
    pub min: [f32; 2],
    pub max: [f32; 2],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SawbladeDef {
    pub pos: [f32; 2],
    #[serde(default = "default_saw_radius")]
    pub radius: f32,
    /// Speed in radians per second (default 4.0)
    #[serde(default = "default_saw_speed")]
    pub speed: f32,
}

fn default_saw_radius() -> f32 { 1.0 }
fn default_saw_speed() -> f32 { 4.0 }

#[derive(Serialize, Deserialize, Clone)]
pub struct BouncePadDef {
    pub min: [f32; 2],
    pub max: [f32; 2],
    #[serde(default = "default_pad_strength")]
    pub strength: f32,
}

fn default_pad_strength() -> f32 { 25.0 }

#[derive(Serialize, Deserialize, Clone)]
pub struct LavaPoolDef {
    pub min: [f32; 2],
    pub max: [f32; 2],
    #[serde(default = "default_lava_dps")]
    pub dps: f32,
}

fn default_lava_dps() -> f32 { 40.0 }

#[derive(Serialize, Deserialize, Clone)]
pub struct LaserBeamDef {
    pub start: [f32; 2],
    pub end: [f32; 2],
    #[serde(default = "default_laser_on")]
    pub on_time: f32,
    #[serde(default = "default_laser_off")]
    pub off_time: f32,
}

fn default_laser_on() -> f32 { 2.0 }
fn default_laser_off() -> f32 { 2.0 }

// ── Runtime level ───────────────────────────────────────────────────────────

pub struct BouncePad {
    pub aabb: crate::physics::collision::AABB,
    pub strength: f32,
}

pub struct LavaPool {
    pub aabb: crate::physics::collision::AABB,
    pub dps: f32,
}

pub struct LaserBeam {
    pub start: Vector3,
    pub end: Vector3,
    pub on_time: f32,
    pub off_time: f32,
}

pub struct Level {
    pub platforms: Vec<Platform>,
    pub spawn_points: Vec<Vector3>,
    pub bounce_pads: Vec<BouncePad>,
    pub lava_pools: Vec<LavaPool>,
    pub lasers: Vec<LaserBeam>,
    pub id: u8,
}

impl LevelDef {
    pub fn to_level(&self, id: u8) -> Level {
        let platforms = self
            .platforms
            .iter()
            .map(|p| {
                let min = Vector3::new(p.min[0], p.min[1], -2.0);
                let max = Vector3::new(p.max[0], p.max[1], 2.0);
                if p.kind == "wall" {
                    Platform::wall(min, max)
                } else {
                    Platform::platform(min, max)
                }
            })
            .collect();
        let spawn_points = self
            .spawn_points
            .iter()
            .map(|s| Vector3::new(s[0], s[1], 0.0))
            .collect();
        let bounce_pads = self
            .bounce_pads
            .iter()
            .map(|b| BouncePad {
                aabb: crate::physics::collision::AABB::new(
                    Vector3::new(b.min[0], b.min[1], -2.0),
                    Vector3::new(b.max[0], b.max[1], 2.0),
                ),
                strength: b.strength,
            })
            .collect();
        let lava_pools = self
            .lava_pools
            .iter()
            .map(|l| LavaPool {
                aabb: crate::physics::collision::AABB::new(
                    Vector3::new(l.min[0], l.min[1], -2.0),
                    Vector3::new(l.max[0], l.max[1], 2.0),
                ),
                dps: l.dps,
            })
            .collect();
        let lasers = self
            .lasers
            .iter()
            .map(|l| LaserBeam {
                start: Vector3::new(l.start[0], l.start[1], 0.0),
                end: Vector3::new(l.end[0], l.end[1], 0.0),
                on_time: l.on_time,
                off_time: l.off_time,
            })
            .collect();
        Level {
            platforms,
            spawn_points,
            bounce_pads,
            lava_pools,
            lasers,
            id,
        }
    }
}

// ── Loading ─────────────────────────────────────────────────────────────────

const LEVELS_FILE: &str = "levels.toml";

/// Embedded fallback so the game works even without levels.toml on disk.
const DEFAULT_LEVELS_TOML: &str = include_str!("../../levels.toml");

fn load_levels_file() -> LevelsFile {
    // Try reading from disk first (allows user edits), fall back to embedded.
    let text = if Path::new(LEVELS_FILE).exists() {
        std::fs::read_to_string(LEVELS_FILE).unwrap_or_else(|_| DEFAULT_LEVELS_TOML.to_string())
    } else {
        DEFAULT_LEVELS_TOML.to_string()
    };
    toml::from_str(&text).unwrap_or_else(|e| {
        eprintln!("Failed to parse levels.toml: {e}. Using embedded defaults.");
        toml::from_str(DEFAULT_LEVELS_TOML).expect("embedded levels.toml must be valid")
    })
}

// ── Level queue (no repeats until all enabled maps played) ─────────────────

pub struct LevelQueue {
    remaining: Vec<u8>, // indices into the levels file
}

impl LevelQueue {
    pub fn new() -> Self {
        Self { remaining: Vec::new() }
    }

    /// Pick the next level, reshuffling when the queue is exhausted.
    pub fn next(&mut self, rng_val: u64) -> Level {
        if self.remaining.is_empty() {
            self.refill(rng_val);
        }
        // If still empty after refill (no enabled levels), fall back
        if self.remaining.is_empty() {
            return level_by_id(0);
        }
        let id = self.remaining.remove(0);
        level_by_id(id)
    }

    fn refill(&mut self, rng_val: u64) {
        let file = load_levels_file();
        let mut ids: Vec<u8> = file.level.iter().enumerate()
            .filter(|(_, l)| l.enabled)
            .map(|(i, _)| i as u8)
            .collect();
        // Fisher-Yates shuffle using rng_val as seed
        let mut seed = rng_val.max(1);
        for i in (1..ids.len()).rev() {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let j = (seed as usize) % (i + 1);
            ids.swap(i, j);
        }
        self.remaining = ids;
    }
}

// ── Public API ─────────────────────────────────────────────────────────────

pub fn level_by_id(id: u8) -> Level {
    let file = load_levels_file();
    let idx = id as usize;
    if idx < file.level.len() {
        file.level[idx].to_level(id)
    } else if !file.level.is_empty() {
        file.level[0].to_level(0)
    } else {
        // Absolute fallback: empty level
        Level {
            platforms: vec![],
            spawn_points: vec![
                Vector3::new(-6.0, 0.0, 0.0),
                Vector3::new(6.0, 0.0, 0.0),
                Vector3::new(-10.0, 0.0, 0.0),
                Vector3::new(10.0, 0.0, 0.0),
            ],
            bounce_pads: vec![],
            lava_pools: vec![],
            lasers: vec![],
            id: 0,
        }
    }
}

pub fn level_name(id: u8) -> String {
    let file = load_levels_file();
    let idx = id as usize;
    if idx < file.level.len() {
        file.level[idx].name.clone()
    } else {
        "Unknown".to_string()
    }
}

/// Load all level definitions (for use by the editor and level selection).
pub fn load_all_levels() -> Vec<LevelDef> {
    load_levels_file().level
}

/// Save level definitions to disk.
pub fn save_all_levels(levels: &[LevelDef]) {
    let file = LevelsFile {
        level: levels.to_vec(),
    };
    let text = toml::to_string_pretty(&file).expect("serialize levels");
    std::fs::write(LEVELS_FILE, text).expect("write levels.toml");
}
