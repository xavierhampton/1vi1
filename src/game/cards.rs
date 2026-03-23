use crate::player::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CardId {
    Dash = 0,
    Reflect = 1,
    GravFlip = 2,
    Shotgun = 3,
    Placeholder4 = 4,
    Placeholder5 = 5,
    Placeholder6 = 6,
    Placeholder7 = 7,
    Placeholder8 = 8,
    Placeholder9 = 9,
}

impl CardId {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Dash),
            1 => Some(Self::Reflect),
            2 => Some(Self::GravFlip),
            3 => Some(Self::Shotgun),
            4 => Some(Self::Placeholder4),
            5 => Some(Self::Placeholder5),
            6 => Some(Self::Placeholder6),
            7 => Some(Self::Placeholder7),
            8 => Some(Self::Placeholder8),
            9 => Some(Self::Placeholder9),
            _ => None,
        }
    }
}

pub struct CardDef {
    pub id: CardId,
    pub name: &'static str,
    pub description: &'static str,
    pub color: (u8, u8, u8),
    pub icon_glyph: char,
    pub cooldown: f32, // 0.0 = passive/unimplemented, >0 = active ability
}

pub const CARD_CATALOG: &[CardDef] = &[
    CardDef { id: CardId::Dash,         name: "Dash",         description: "Burst in aim direction",  color: (100, 255, 100), icon_glyph: '>', cooldown: 3.0 },
    CardDef { id: CardId::Reflect,      name: "Reflect",      description: "Parry bullets back",      color: (80, 180, 255),  icon_glyph: ')', cooldown: 5.0 },
    CardDef { id: CardId::GravFlip,     name: "Grav Flip",    description: "Reverse gravity briefly", color: (180, 100, 255), icon_glyph: '^', cooldown: 6.0 },
    CardDef { id: CardId::Shotgun,      name: "Shotgun",      description: "5-bullet spread blast",   color: (255, 160, 60),  icon_glyph: '#', cooldown: 5.0 },
    CardDef { id: CardId::Placeholder4, name: "???",          description: "Coming soon",             color: (120, 120, 120), icon_glyph: '?', cooldown: 0.0 },
    CardDef { id: CardId::Placeholder5, name: "???",          description: "Coming soon",             color: (120, 120, 120), icon_glyph: '?', cooldown: 0.0 },
    CardDef { id: CardId::Placeholder6, name: "???",          description: "Coming soon",             color: (120, 120, 120), icon_glyph: '?', cooldown: 0.0 },
    CardDef { id: CardId::Placeholder7, name: "???",          description: "Coming soon",             color: (120, 120, 120), icon_glyph: '?', cooldown: 0.0 },
    CardDef { id: CardId::Placeholder8, name: "???",          description: "Coming soon",             color: (120, 120, 120), icon_glyph: '?', cooldown: 0.0 },
    CardDef { id: CardId::Placeholder9, name: "???",          description: "Coming soon",             color: (120, 120, 120), icon_glyph: '?', cooldown: 0.0 },
];

/// Pick `count` unique random card IDs from implemented cards only.
pub fn random_cards(rng_val: &mut u64, count: usize) -> Vec<u8> {
    // Only offer cards that are actually implemented (cooldown > 0)
    let pool_ids: Vec<u8> = CARD_CATALOG.iter()
        .filter(|c| c.cooldown > 0.0)
        .map(|c| c.id as u8)
        .collect();

    let mut pool = pool_ids;
    let mut result = Vec::with_capacity(count);

    for _ in 0..count.min(pool.len()) {
        if pool.is_empty() { break; }
        *rng_val ^= *rng_val << 13;
        *rng_val ^= *rng_val >> 7;
        *rng_val ^= *rng_val << 17;
        let idx = (*rng_val as usize) % pool.len();
        result.push(pool.swap_remove(idx));
    }

    result
}

// ── Ability activation ──────────────────────────────────────────────────────

const DASH_SPEED: f32 = 22.0;

/// Activate an ability on a player. Returns the cooldown to set.
pub fn activate_ability(card_id: CardId, player: &mut Player) -> f32 {
    let def = &CARD_CATALOG[card_id as usize];
    match card_id {
        CardId::Dash => {
            player.velocity.x = player.aim_dir.x * DASH_SPEED;
            player.velocity.y = player.aim_dir.y * DASH_SPEED;
            def.cooldown
        }
        // TODO: implement other abilities
        _ => def.cooldown,
    }
}
