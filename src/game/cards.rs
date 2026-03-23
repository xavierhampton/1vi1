use crate::player::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CardId {
    // Abilities (active, right-click, have cooldown)
    Dash = 0,
    Reflect = 1,
    GravFlip = 2,
    Shotgun = 3,
    // Powerups (passive, always active)
    ExtraHp = 4,
    SpeedBoost = 5,
    DoubleJump = 6,
    FastReload = 7,
    HeavyBullets = 8,
    Placeholder9 = 9,
}

impl CardId {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Dash),
            1 => Some(Self::Reflect),
            2 => Some(Self::GravFlip),
            3 => Some(Self::Shotgun),
            4 => Some(Self::ExtraHp),
            5 => Some(Self::SpeedBoost),
            6 => Some(Self::DoubleJump),
            7 => Some(Self::FastReload),
            8 => Some(Self::HeavyBullets),
            9 => Some(Self::Placeholder9),
            _ => None,
        }
    }
}

/// What kind of card this is — determines how it behaves in-game
#[derive(Debug, Clone, Copy)]
pub enum CardKind {
    /// Active ability: right-click triggers it, then goes on cooldown
    Ability { cooldown: f32 },
    /// Passive powerup: always active once picked, modifies player stats
    Powerup,
}

pub struct CardDef {
    pub id: CardId,
    pub name: &'static str,
    pub description: &'static str,
    pub color: (u8, u8, u8),
    pub icon_glyph: char,
    pub kind: CardKind,
}

impl CardDef {
    /// Returns cooldown duration for abilities, 0 for powerups
    pub fn cooldown(&self) -> f32 {
        match self.kind {
            CardKind::Ability { cooldown } => cooldown,
            CardKind::Powerup => 0.0,
        }
    }

    pub fn is_ability(&self) -> bool {
        matches!(self.kind, CardKind::Ability { .. })
    }

    pub fn is_powerup(&self) -> bool {
        matches!(self.kind, CardKind::Powerup)
    }

    /// Whether this card is implemented and should appear in the card pool
    pub fn is_implemented(&self) -> bool {
        !matches!(self.id, CardId::Placeholder9)
    }
}

pub const CARD_CATALOG: &[CardDef] = &[
    // ── Abilities ──
    CardDef { id: CardId::Dash,        name: "Dash",          description: "Burst in aim direction",   color: (100, 255, 100), icon_glyph: '>',  kind: CardKind::Ability { cooldown: 3.0 } },
    CardDef { id: CardId::Reflect,     name: "Reflect",       description: "Parry bullets back",       color: (80, 180, 255),  icon_glyph: ')',  kind: CardKind::Ability { cooldown: 5.0 } },
    CardDef { id: CardId::GravFlip,    name: "Grav Flip",     description: "Reverse gravity briefly",  color: (180, 100, 255), icon_glyph: '^',  kind: CardKind::Ability { cooldown: 6.0 } },
    CardDef { id: CardId::Shotgun,     name: "Shotgun",       description: "5-bullet spread blast",    color: (255, 160, 60),  icon_glyph: '#',  kind: CardKind::Ability { cooldown: 5.0 } },
    // ── Powerups ──
    CardDef { id: CardId::ExtraHp,     name: "Extra HP",      description: "+30 max health",           color: (255, 80, 80),   icon_glyph: '+',  kind: CardKind::Powerup },
    CardDef { id: CardId::SpeedBoost,  name: "Speed Boost",   description: "+15% move speed",          color: (255, 255, 80),  icon_glyph: '!',  kind: CardKind::Powerup },
    CardDef { id: CardId::DoubleJump,  name: "Double Jump",   description: "+1 extra air jump",        color: (120, 220, 255), icon_glyph: 'W',  kind: CardKind::Powerup },
    CardDef { id: CardId::FastReload,  name: "Fast Reload",   description: "-30% reload time",         color: (200, 255, 160), icon_glyph: '%',  kind: CardKind::Powerup },
    CardDef { id: CardId::HeavyBullets,name: "Heavy Bullets",  description: "+10 bullet damage",       color: (255, 140, 40),  icon_glyph: '*',  kind: CardKind::Powerup },
    CardDef { id: CardId::Placeholder9,name: "???",            description: "Coming soon",             color: (120, 120, 120), icon_glyph: '?',  kind: CardKind::Powerup },
];

/// Pick `count` unique random card IDs from implemented cards only.
pub fn random_cards(rng_val: &mut u64, count: usize) -> Vec<u8> {
    let pool_ids: Vec<u8> = CARD_CATALOG.iter()
        .filter(|c| c.is_implemented())
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

// ── Stat modifiers from powerups ─────────────────────────────────────────────

/// Additive/multiplicative stat modifiers computed from held powerup cards
#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub max_hp_bonus: f32,       // additive
    pub move_speed_mult: f32,    // multiplicative (1.0 = normal)
    pub extra_air_jumps: i32,    // additive
    pub reload_time_mult: f32,   // multiplicative (1.0 = normal, lower = faster)
    pub bullet_damage_bonus: f32,// additive
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            max_hp_bonus: 0.0,
            move_speed_mult: 1.0,
            extra_air_jumps: 0,
            reload_time_mult: 1.0,
            bullet_damage_bonus: 0.0,
        }
    }
}

/// Recompute PlayerStats from a player's held cards. Call after cards change.
pub fn compute_stats(cards: &[(CardId, f32)]) -> PlayerStats {
    let mut stats = PlayerStats::default();
    for (card_id, _) in cards {
        match card_id {
            CardId::ExtraHp => stats.max_hp_bonus += 30.0,
            CardId::SpeedBoost => stats.move_speed_mult += 0.15,
            CardId::DoubleJump => stats.extra_air_jumps += 1,
            CardId::FastReload => stats.reload_time_mult *= 0.7,
            CardId::HeavyBullets => stats.bullet_damage_bonus += 10.0,
            _ => {} // abilities don't modify stats
        }
    }
    stats
}

/// Apply computed stats to a player (call after compute_stats, e.g. on round reset)
pub fn apply_stats(player: &mut Player, stats: &PlayerStats) {
    player.max_hp = 100.0 + stats.max_hp_bonus;
    // Heal to new max if current hp exceeds nothing (keep damage taken)
    if player.hp > player.max_hp {
        player.hp = player.max_hp;
    }
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
            def.cooldown()
        }
        // TODO: implement other abilities
        _ => def.cooldown(),
    }
}
