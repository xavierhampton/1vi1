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
    pub spawn_points: Vec<[f32; 2]>,
    pub platforms: Vec<PlatformDef>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlatformDef {
    #[serde(rename = "type")]
    pub kind: String, // "wall" or "platform"
    pub min: [f32; 2],
    pub max: [f32; 2],
}

// ── Runtime level ───────────────────────────────────────────────────────────

pub struct Level {
    pub platforms: Vec<Platform>,
    pub spawn_points: Vec<Vector3>,
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
        Level {
            platforms,
            spawn_points,
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

// ── Public API (unchanged signatures) ───────────────────────────────────────

pub fn random_level(rng_val: u64) -> Level {
    let file = load_levels_file();
    let count = file.level.len().max(1);
    let id = (rng_val % count as u64) as u8;
    level_by_id(id)
}

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
