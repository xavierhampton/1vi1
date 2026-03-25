use raylib::prelude::*;

#[derive(Clone, Debug)]
pub struct GameSettings {
    pub wins_to_match: i32,
    pub spawn_invuln: f32,
    pub starting_hp: f32,
    pub gravity_scale: f32,
    pub turbo_speed: f32,     // 1.0 = normal
    pub sudden_death: bool,
    pub everyone_picks: bool, // false = loser only, true = everyone
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            wins_to_match: 3,
            spawn_invuln: 2.5,
            starting_hp: 100.0,
            gravity_scale: 1.0,
            turbo_speed: 1.0,
            sudden_death: false,
            everyone_picks: false,
        }
    }
}

pub const LOBBY_COLORS: [(Color, &str); 4] = [
    (Color::new(80, 180, 255, 255), "Blue"),
    (Color::new(255, 100, 80, 255), "Red"),
    (Color::new(100, 230, 120, 255), "Green"),
    (Color::new(255, 200, 60, 255), "Yellow"),
];

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum LobbyColor {
    Blue = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
}

impl LobbyColor {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Blue),
            1 => Some(Self::Red),
            2 => Some(Self::Green),
            3 => Some(Self::Yellow),
            _ => None,
        }
    }

    pub fn to_color(self) -> Color {
        LOBBY_COLORS[self as usize].0
    }

    pub fn name(self) -> &'static str {
        LOBBY_COLORS[self as usize].1
    }

    pub const ALL: [LobbyColor; 4] = [
        LobbyColor::Blue,
        LobbyColor::Red,
        LobbyColor::Green,
        LobbyColor::Yellow,
    ];
}

#[derive(Clone, Debug)]
pub struct PlayerSlot {
    pub name: String,
    pub color: LobbyColor,
    pub ready: bool,
    pub is_host: bool,
}

#[derive(Clone, Debug)]
pub struct LobbyState {
    pub slots: Vec<PlayerSlot>,
    pub settings: GameSettings,
}

impl LobbyState {
    pub fn new_host(name: &str) -> Self {
        Self {
            slots: vec![PlayerSlot {
                name: name.to_string(),
                color: LobbyColor::Blue,
                ready: false,
                is_host: true,
            }],
            settings: GameSettings::default(),
        }
    }

    pub fn all_ready(&self) -> bool {
        self.slots.len() >= 2 && self.slots.iter().all(|s| s.ready)
    }

    pub fn color_taken(&self, color: LobbyColor, exclude_index: Option<usize>) -> bool {
        self.slots.iter().enumerate().any(|(i, s)| {
            s.color == color && exclude_index != Some(i)
        })
    }

    pub fn first_available_color(&self) -> Option<LobbyColor> {
        LobbyColor::ALL
            .iter()
            .find(|c| !self.color_taken(**c, None))
            .copied()
    }

    pub fn next_available_color(&self, current: LobbyColor, my_index: usize) -> LobbyColor {
        let start = current as usize;
        for offset in 1..=4 {
            let idx = (start + offset) % 4;
            let c = LobbyColor::from_u8(idx as u8).unwrap();
            if !self.color_taken(c, Some(my_index)) {
                return c;
            }
        }
        current
    }

    pub fn prev_available_color(&self, current: LobbyColor, my_index: usize) -> LobbyColor {
        let start = current as usize;
        for offset in 1..=4 {
            let idx = (start + 4 - offset) % 4;
            let c = LobbyColor::from_u8(idx as u8).unwrap();
            if !self.color_taken(c, Some(my_index)) {
                return c;
            }
        }
        current
    }
}
