use crate::player::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CardId {
    // Abilities (active, right-click, have cooldown)
    Dash = 0,
    Blink = 1,
    Stomp = 2,
    Swap = 3,
    // Powerups (passive, always active)
    TripleJump = 4,
    RubberBullets = 5,
    HomingBullets = 6,
    ExplosiveBullets = 7,
    BigBullets = 8,
    PiercingBullets = 9,
    GlassCannon = 10,
    Vampire = 11,
    TripleShot = 12,
    RapidFire = 13,
    Tiny = 14,
    Bounceback = 15,
    Shotgun = 16,
}

impl CardId {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Dash),
            1 => Some(Self::Blink),
            2 => Some(Self::Stomp),
            3 => Some(Self::Swap),
            4 => Some(Self::TripleJump),
            5 => Some(Self::RubberBullets),
            6 => Some(Self::HomingBullets),
            7 => Some(Self::ExplosiveBullets),
            8 => Some(Self::BigBullets),
            9 => Some(Self::PiercingBullets),
            10 => Some(Self::GlassCannon),
            11 => Some(Self::Vampire),
            12 => Some(Self::TripleShot),
            13 => Some(Self::RapidFire),
            14 => Some(Self::Tiny),
            15 => Some(Self::Bounceback),
            16 => Some(Self::Shotgun),
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
        true
    }
}

pub const CARD_CATALOG: &[CardDef] = &[
    // ── Abilities ──
    CardDef { id: CardId::Dash,    name: "Dash",    description: "Burst in aim direction",        color: (100, 255, 100), icon_glyph: '>', kind: CardKind::Ability { cooldown: 2.0 } },
    CardDef { id: CardId::Blink,   name: "Blink",   description: "Teleport short range forward",  color: (160, 120, 255), icon_glyph: '~', kind: CardKind::Ability { cooldown: 5.0 } },
    CardDef { id: CardId::Stomp,   name: "Stomp",   description: "Hop up then slam down, AoE",    color: (255, 160, 60),  icon_glyph: 'v', kind: CardKind::Ability { cooldown: 4.0 } },
    CardDef { id: CardId::Swap,    name: "Swap",    description: "Teleswap with nearest enemy",   color: (255, 200, 80),  icon_glyph: 'S', kind: CardKind::Ability { cooldown: 6.0 } },
    // ── Powerups ──
    CardDef { id: CardId::TripleJump,      name: "Triple Jump",      description: "+2 extra air jumps",           color: (120, 220, 255), icon_glyph: 'W', kind: CardKind::Powerup },
    CardDef { id: CardId::RubberBullets,   name: "Rubber Bullets",   description: "Bounce off walls 2x",          color: (80, 255, 180),  icon_glyph: ')', kind: CardKind::Powerup },
    CardDef { id: CardId::HomingBullets,   name: "Homing Bullets",   description: "Track nearest enemy",          color: (255, 80, 200),  icon_glyph: '@', kind: CardKind::Powerup },
    CardDef { id: CardId::ExplosiveBullets,name: "Explosive Bullets", description: "Small AoE on impact",         color: (255, 120, 40),  icon_glyph: '*', kind: CardKind::Powerup },
    CardDef { id: CardId::BigBullets,      name: "Big Bullets",      description: "3x hitbox, slower speed",      color: (200, 160, 255), icon_glyph: 'O', kind: CardKind::Powerup },
    CardDef { id: CardId::PiercingBullets, name: "Piercing Bullets", description: "Pass through players",         color: (255, 255, 200), icon_glyph: '|', kind: CardKind::Powerup },
    CardDef { id: CardId::GlassCannon,     name: "Glass Cannon",     description: "2x damage, half max HP",       color: (255, 40, 40),   icon_glyph: 'X', kind: CardKind::Powerup },
    CardDef { id: CardId::Vampire,         name: "Vampire",          description: "Heal 8 HP per hit",            color: (200, 40, 80),   icon_glyph: 'V', kind: CardKind::Powerup },
    CardDef { id: CardId::TripleShot,      name: "Triple Shot",      description: "3 bullets per shot, 45\u{00b0} spread", color: (80, 200, 255), icon_glyph: 'T', kind: CardKind::Powerup },
    CardDef { id: CardId::RapidFire,       name: "Rapid Fire",       description: "2x fire rate, +2 ammo",        color: (255, 255, 100), icon_glyph: '!', kind: CardKind::Powerup },
    CardDef { id: CardId::Tiny,            name: "Tiny",             description: "50% smaller, -30 max HP",       color: (180, 140, 255), icon_glyph: '.', kind: CardKind::Powerup },
    CardDef { id: CardId::Bounceback,      name: "Bounceback",       description: "Getting hit knocks you away",   color: (100, 255, 200), icon_glyph: '<', kind: CardKind::Powerup },
    CardDef { id: CardId::Shotgun,         name: "Shotgun",          description: "Dump all ammo in one blast",    color: (255, 160, 80),  icon_glyph: '#', kind: CardKind::Powerup },
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
    pub max_hp_mult: f32,
    pub max_hp_bonus: f32,
    pub size_mult: f32,
    pub extra_air_jumps: i32,
    pub bullet_damage_mult: f32,
    pub bullet_speed_mult: f32,
    pub bullet_radius_mult: f32,
    pub rubber_bounces: i32,
    pub homing: bool,
    pub explosive: bool,
    pub piercing: bool,
    pub vampire_heal: f32,
    pub triple_shot: bool,
    pub shoot_cooldown_mult: f32,
    pub extra_ammo: i32,
    pub bounceback: bool,
    pub shotgun: bool,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            max_hp_mult: 1.0,
            max_hp_bonus: 0.0,
            size_mult: 1.0,
            extra_air_jumps: 0,
            bullet_damage_mult: 1.0,
            bullet_speed_mult: 1.0,
            bullet_radius_mult: 1.0,
            rubber_bounces: 0,
            homing: false,
            explosive: false,
            piercing: false,
            vampire_heal: 0.0,
            triple_shot: false,
            shoot_cooldown_mult: 1.0,
            extra_ammo: 0,
            bounceback: false,
            shotgun: false,
        }
    }
}

/// Recompute PlayerStats from a player's held cards. Call after cards change.
pub fn compute_stats(cards: &[(CardId, f32)]) -> PlayerStats {
    let mut stats = PlayerStats::default();
    for (card_id, _) in cards {
        match card_id {
            CardId::TripleJump => stats.extra_air_jumps += 2,
            CardId::RubberBullets => stats.rubber_bounces += 2,
            CardId::HomingBullets => stats.homing = true,
            CardId::ExplosiveBullets => stats.explosive = true,
            CardId::BigBullets => {
                stats.bullet_radius_mult *= 3.0;
                stats.bullet_speed_mult *= 0.6;
            }
            CardId::PiercingBullets => stats.piercing = true,
            CardId::GlassCannon => {
                stats.bullet_damage_mult *= 2.0;
                stats.max_hp_mult *= 0.5;
            }
            CardId::Vampire => stats.vampire_heal += 8.0,
            CardId::TripleShot => stats.triple_shot = true,
            CardId::RapidFire => {
                stats.shoot_cooldown_mult *= 0.5;
                stats.extra_ammo += 2;
            }
            CardId::Tiny => {
                stats.size_mult *= 0.5;
                stats.max_hp_bonus -= 30.0;
            }
            CardId::Bounceback => stats.bounceback = true,
            CardId::Shotgun => stats.shotgun = true,
            _ => {} // abilities don't modify stats
        }
    }
    stats
}

/// Apply computed stats to a player (call after compute_stats, e.g. on round reset)
pub fn apply_stats(player: &mut Player, stats: &PlayerStats) {
    player.max_hp = (100.0 + stats.max_hp_bonus) * stats.max_hp_mult;
    if player.max_hp < 1.0 { player.max_hp = 1.0; }
    if player.hp > player.max_hp {
        player.hp = player.max_hp;
    }
    // Apply size multiplier
    player.size.x = 0.6 * stats.size_mult;
    player.size.y = 1.6 * stats.size_mult;
    player.size.z = 0.6 * stats.size_mult;
}

// ── Ability activation ──────────────────────────────────────────────────────

const DASH_SPEED: f32 = 32.0;
const BLINK_DISTANCE: f32 = 6.0;
const STOMP_HOP: f32 = 10.0;

/// Side effects that need world-level handling after ability activation
pub enum AbilityEffect {
    None,
    Swap,
}

/// Activate an ability on a player. Returns (cooldown, side_effect).
pub fn activate_ability(card_id: CardId, player: &mut Player) -> (f32, AbilityEffect) {
    let def = &CARD_CATALOG[card_id as usize];
    match card_id {
        CardId::Dash => {
            player.velocity.x = player.aim_dir.x * DASH_SPEED;
            player.velocity.y = player.aim_dir.y * DASH_SPEED;
            (def.cooldown(), AbilityEffect::None)
        }
        CardId::Blink => {
            player.position.x += player.aim_dir.x * BLINK_DISTANCE;
            player.position.y += player.aim_dir.y * BLINK_DISTANCE;
            (def.cooldown(), AbilityEffect::None)
        }
        CardId::Stomp => {
            player.velocity.y = STOMP_HOP;
            player.velocity.x *= 0.3;
            player.stomp_active = true;
            (def.cooldown(), AbilityEffect::None)
        }
        CardId::Swap => {
            // Position swap handled in world.rs
            (def.cooldown(), AbilityEffect::Swap)
        }
        _ => (def.cooldown(), AbilityEffect::None),
    }
}
