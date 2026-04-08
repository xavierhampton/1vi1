use std::fs;
use std::path::PathBuf;

use super::customize::{Equipped, ACCESSORY_NONE, MAX_EQUIPPED};
use crate::lobby::state::LOBBY_COLOR_COUNT;

const SETTINGS_FILE: &str = "user_settings.cfg";

pub const FPS_OPTIONS: &[u32] = &[60, 144, 240, 300];

pub struct UserSettings {
    pub theme_index: usize,
    pub master_volume: f32,
    pub sound_volume: f32,
    pub music_volume: f32,
    pub target_fps: u32,
    pub player_name: String,
    pub preview_color: usize,
    pub accessories: Equipped,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme_index: 0,
            master_volume: 0.8,
            sound_volume: 0.17,
            music_volume: 0.1,
            target_fps: 144,
            player_name: "Player".into(),
            preview_color: 0,
            accessories: [(ACCESSORY_NONE, 255, 255, 255); MAX_EQUIPPED],
        }
    }
}

fn settings_path() -> PathBuf {
    PathBuf::from(SETTINGS_FILE)
}

impl UserSettings {
    pub fn load() -> Self {
        let path = settings_path();
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => return Self::default(),
        };

        let mut s = Self::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, val)) = line.split_once('=') else { continue };
            let key = key.trim();
            let val = val.trim();
            match key {
                "theme" => {
                    if let Ok(v) = val.parse::<usize>() {
                        s.theme_index = v;
                    }
                }
                // Legacy single "volume" key — apply to all three
                "volume" => {
                    if let Ok(v) = val.parse::<f32>() {
                        let v = v.clamp(0.0, 1.0);
                        s.master_volume = v;
                        s.sound_volume = v;
                        s.music_volume = v;
                    }
                }
                "master_volume" => {
                    if let Ok(v) = val.parse::<f32>() {
                        s.master_volume = v.clamp(0.0, 1.0);
                    }
                }
                "sound_volume" => {
                    if let Ok(v) = val.parse::<f32>() {
                        s.sound_volume = v.clamp(0.0, 1.0);
                    }
                }
                "music_volume" => {
                    if let Ok(v) = val.parse::<f32>() {
                        s.music_volume = v.clamp(0.0, 1.0);
                    }
                }
                "target_fps" => {
                    if let Ok(v) = val.parse::<u32>() {
                        if FPS_OPTIONS.contains(&v) {
                            s.target_fps = v;
                        }
                    }
                }
                "name" => {
                    s.player_name = val.to_string();
                }
                "color" => {
                    if let Ok(v) = val.parse::<usize>() {
                        s.preview_color = v.min(LOBBY_COLOR_COUNT - 1);
                    }
                }
                "acc0" | "acc1" | "acc2" => {
                    let slot = (key.as_bytes()[3] - b'0') as usize;
                    if slot < MAX_EQUIPPED {
                        if let Some(parsed) = parse_accessory(val) {
                            s.accessories[slot] = parsed;
                        }
                    }
                }
                _ => {}
            }
        }
        s
    }

    pub fn save(&self) {
        let mut out = String::with_capacity(256);
        out.push_str(&format!("theme={}\n", self.theme_index));
        out.push_str(&format!("master_volume={:.2}\n", self.master_volume));
        out.push_str(&format!("sound_volume={:.2}\n", self.sound_volume));
        out.push_str(&format!("music_volume={:.2}\n", self.music_volume));
        out.push_str(&format!("target_fps={}\n", self.target_fps));
        out.push_str(&format!("name={}\n", self.player_name));
        out.push_str(&format!("color={}\n", self.preview_color));
        for (i, acc) in self.accessories.iter().enumerate() {
            out.push_str(&format!("acc{}={},{},{},{}\n", i, acc.0, acc.1, acc.2, acc.3));
        }
        let _ = fs::write(settings_path(), out);
    }
}

fn parse_accessory(val: &str) -> Option<(u8, u8, u8, u8)> {
    let parts: Vec<&str> = val.split(',').collect();
    if parts.len() != 4 {
        return None;
    }
    Some((
        parts[0].trim().parse().ok()?,
        parts[1].trim().parse().ok()?,
        parts[2].trim().parse().ok()?,
        parts[3].trim().parse().ok()?,
    ))
}
