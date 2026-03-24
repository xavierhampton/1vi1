use crate::player::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CardId {
    // Abilities (active, right-click, have cooldown)
    Dash = 0,
    Blink = 1,
    Stomp = 2,
    Swap = 3,
    Screech = 21,
    Ghost = 25,
    Zip = 26,
    PhantomShot = 27,
    Rewind = 28,
    Overclock = 29,
    GravityWell = 30,
    Decoy = 31,
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
    Gatling = 17,
    QuickReload = 18,
    Steroids = 19,
    LaserBeam = 20,
    PoisonBullets = 22,
    Chunky = 23,
    Chud = 24,
    SplitShot = 32,
    Slug = 33,
    HotPotato = 34,
    LeechField = 35,
    Featherweight = 36,
    StickyBombs = 37,
    Loadout = 38,
    ChaosRounds = 39,
    Adrenaline = 40,
    Upsizer = 41,
    Juggernaut = 42,
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
            17 => Some(Self::Gatling),
            18 => Some(Self::QuickReload),
            19 => Some(Self::Steroids),
            20 => Some(Self::LaserBeam),
            21 => Some(Self::Screech),
            22 => Some(Self::PoisonBullets),
            23 => Some(Self::Chunky),
            24 => Some(Self::Chud),
            25 => Some(Self::Ghost),
            26 => Some(Self::Zip),
            27 => Some(Self::PhantomShot),
            28 => Some(Self::Rewind),
            29 => Some(Self::Overclock),
            30 => Some(Self::GravityWell),
            31 => Some(Self::Decoy),
            32 => Some(Self::SplitShot),
            33 => Some(Self::Slug),
            34 => Some(Self::HotPotato),
            35 => Some(Self::LeechField),
            36 => Some(Self::Featherweight),
            37 => Some(Self::StickyBombs),
            38 => Some(Self::Loadout),
            39 => Some(Self::ChaosRounds),
            40 => Some(Self::Adrenaline),
            41 => Some(Self::Upsizer),
            42 => Some(Self::Juggernaut),
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

    pub fn is_implemented(&self) -> bool {
        !matches!(self.id, CardId::Blink | CardId::Swap | CardId::ChaosRounds
            | CardId::LaserBeam | CardId::GravityWell | CardId::SplitShot | CardId::HotPotato)
    }
}

pub const CARD_CATALOG: &[CardDef] = &[
    // ── Abilities ──
    CardDef { id: CardId::Dash,    name: "Dash",    description: "Burst in aim direction",        color: (100, 255, 100), icon_glyph: '>', kind: CardKind::Ability { cooldown: 2.0 } },
    CardDef { id: CardId::Blink,   name: "Blink",   description: "Teleport short range forward",  color: (160, 120, 255), icon_glyph: '~', kind: CardKind::Ability { cooldown: 5.0 } },
    CardDef { id: CardId::Stomp,   name: "Stomp",   description: "Hop up then slam down, AoE",    color: (255, 160, 60),  icon_glyph: 'v', kind: CardKind::Ability { cooldown: 4.0 } },
    CardDef { id: CardId::Swap,    name: "Swap",    description: "Teleswap with nearest enemy",   color: (255, 200, 80),  icon_glyph: 'S', kind: CardKind::Ability { cooldown: 6.0 } },
    // ── Powerups ──
    CardDef { id: CardId::TripleJump,      name: "Triple Jump",       description: "+2 extra air jumps",            color: (120, 220, 255), icon_glyph: 'W', kind: CardKind::Powerup },
    CardDef { id: CardId::RubberBullets,   name: "Rubber Bullets",    description: "Bounce off walls 2x",           color: (80, 255, 180),  icon_glyph: ')', kind: CardKind::Powerup },
    CardDef { id: CardId::HomingBullets,   name: "Homing Bullets",    description: "Track nearest enemy",           color: (255, 80, 200),  icon_glyph: '@', kind: CardKind::Powerup },
    CardDef { id: CardId::ExplosiveBullets,name: "Explosive Bullets",  description: "Small AoE on impact",          color: (255, 120, 40),  icon_glyph: '*', kind: CardKind::Powerup },
    CardDef { id: CardId::BigBullets,      name: "Big Bullets",       description: "3x bullet hitbox",              color: (200, 160, 255), icon_glyph: 'O', kind: CardKind::Powerup },
    CardDef { id: CardId::PiercingBullets, name: "Piercing Bullets",  description: "Pass through players",          color: (255, 255, 200), icon_glyph: '|', kind: CardKind::Powerup },
    CardDef { id: CardId::GlassCannon,     name: "Glass Cannon",      description: "2x damage, half max HP",        color: (255, 40, 40),   icon_glyph: 'X', kind: CardKind::Powerup },
    CardDef { id: CardId::Vampire,         name: "Vampire",           description: "Heal 8 HP per hit",             color: (200, 40, 80),   icon_glyph: 'V', kind: CardKind::Powerup },
    CardDef { id: CardId::TripleShot,      name: "Triple Shot",       description: "3 bullets, 45\u{00b0} spread",  color: (80, 200, 255),  icon_glyph: 'T', kind: CardKind::Powerup },
    CardDef { id: CardId::RapidFire,       name: "Rapid Fire",        description: "2x fire rate, +2 ammo",         color: (255, 255, 100), icon_glyph: '!', kind: CardKind::Powerup },
    CardDef { id: CardId::Tiny,            name: "Tiny",              description: "50% smaller hitbox",             color: (180, 140, 255), icon_glyph: '.', kind: CardKind::Powerup },
    CardDef { id: CardId::Bounceback,      name: "Knockback",         description: "Getting hit launches you away",  color: (100, 255, 200), icon_glyph: '<', kind: CardKind::Powerup },
    CardDef { id: CardId::Shotgun,         name: "Shotgun",           description: "Dump all ammo in one blast",     color: (255, 160, 80),  icon_glyph: '#', kind: CardKind::Powerup },
    CardDef { id: CardId::Gatling,         name: "Gatling",           description: "+6 ammo capacity",               color: (180, 180, 60),  icon_glyph: 'G', kind: CardKind::Powerup },
    CardDef { id: CardId::QuickReload,     name: "Quick Reload",      description: "50% faster reload",              color: (100, 200, 255), icon_glyph: 'R', kind: CardKind::Powerup },
    CardDef { id: CardId::Steroids,        name: "Steroids",          description: "2x max HP, 1.5x size",           color: (255, 60, 120),  icon_glyph: '+', kind: CardKind::Powerup },
    CardDef { id: CardId::LaserBeam,       name: "Laser Beam",        description: "Continuous beam, drains ammo",   color: (255, 40, 40),   icon_glyph: '=', kind: CardKind::Powerup },
    CardDef { id: CardId::Screech,         name: "Screech",           description: "Blast all enemies away",         color: (255, 255, 60),  icon_glyph: '^', kind: CardKind::Ability { cooldown: 12.0 } },
    CardDef { id: CardId::PoisonBullets,   name: "Poison Bullets",    description: "3s damage over time on hit",     color: (80, 200, 40),   icon_glyph: '%', kind: CardKind::Powerup },
    CardDef { id: CardId::Chunky,          name: "Chunky",            description: "+50 max HP",                     color: (200, 140, 80),  icon_glyph: 'C', kind: CardKind::Powerup },
    CardDef { id: CardId::Chud,            name: "Chud",              description: "+3 HP/s regen",                  color: (80, 255, 120),  icon_glyph: 'H', kind: CardKind::Powerup },
    CardDef { id: CardId::Ghost,           name: "Ghost",             description: "Vanish for 3 seconds",           color: (180, 180, 220), icon_glyph: '?', kind: CardKind::Ability { cooldown: 10.0 } },
    CardDef { id: CardId::Zip,             name: "Zip",               description: "2x bullet speed, less gravity",  color: (200, 240, 255), icon_glyph: 'Z', kind: CardKind::Powerup },
    CardDef { id: CardId::PhantomShot,     name: "Phantom Shot",      description: "Bullets pass through terrain",   color: (160, 140, 200), icon_glyph: 'P', kind: CardKind::Powerup },
    CardDef { id: CardId::Rewind,          name: "Rewind",            description: "Snap back to 3s ago",            color: (180, 120, 255), icon_glyph: 'D', kind: CardKind::Ability { cooldown: 8.0 } },
    CardDef { id: CardId::Overclock,       name: "Overclock",         description: "4s 2x speed+fire, brief crash",  color: (255, 220, 60),  icon_glyph: 'K', kind: CardKind::Ability { cooldown: 10.0 } },
    CardDef { id: CardId::GravityWell,     name: "Gravity Well",      description: "Pull enemies to a point",        color: (100, 60, 200),  icon_glyph: 'o', kind: CardKind::Ability { cooldown: 12.0 } },
    CardDef { id: CardId::Decoy,           name: "Yoru",              description: "Fast hunter, big explosion",      color: (200, 200, 200), icon_glyph: '&', kind: CardKind::Ability { cooldown: 8.0 } },
    CardDef { id: CardId::SplitShot,       name: "Split Shot",        description: "Bullets split on hit",           color: (120, 200, 255), icon_glyph: 'Y', kind: CardKind::Powerup },
    CardDef { id: CardId::Slug,            name: "Slug",              description: "1 huge bullet, 3x damage",       color: (180, 160, 120), icon_glyph: '$', kind: CardKind::Powerup },
    CardDef { id: CardId::HotPotato,       name: "Hot Potato",        description: "Bullets speed up over time",     color: (255, 180, 60),  icon_glyph: '~', kind: CardKind::Powerup },
    CardDef { id: CardId::LeechField,      name: "Leech Field",       description: "Drain 2 HP/s nearby foes",       color: (160, 40, 80),   icon_glyph: 'L', kind: CardKind::Powerup },
    CardDef { id: CardId::Featherweight,   name: "Featherweight",     description: "+3 jumps, faster, fragile",      color: (200, 240, 255), icon_glyph: 'F', kind: CardKind::Powerup },
    CardDef { id: CardId::StickyBombs,     name: "Sticky Bombs",      description: "Bullets stick then explode",     color: (200, 180, 60),  icon_glyph: 'B', kind: CardKind::Powerup },
    CardDef { id: CardId::Loadout,         name: "Loadout",           description: "+3 ammo, fast reload, +10 HP",   color: (160, 200, 160), icon_glyph: 'Q', kind: CardKind::Powerup },
    CardDef { id: CardId::ChaosRounds,     name: "Chaos Rounds",      description: "Random effect per bullet",       color: (255, 100, 255), icon_glyph: 'N', kind: CardKind::Powerup },
    CardDef { id: CardId::Adrenaline,      name: "Adrenaline",        description: "Getting hit = speed boost",      color: (255, 80, 80),   icon_glyph: 'A', kind: CardKind::Powerup },
    CardDef { id: CardId::Upsizer,         name: "Upsizer",           description: "Hits make enemies bigger",       color: (255, 200, 100), icon_glyph: 'U', kind: CardKind::Powerup },
    CardDef { id: CardId::Juggernaut,      name: "Juggernaut",        description: "+40 HP, regen, no knockback",    color: (140, 140, 160), icon_glyph: 'J', kind: CardKind::Powerup },
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
    pub reload_time_mult: f32,
    pub laser: bool,
    pub poison: bool,
    pub hp_regen: f32,
    pub bullet_gravity_mult: f32,
    pub phantom: bool,
    pub move_speed_mult: f32,
    pub damage_taken_mult: f32,
    pub knockback_immune: bool,
    pub split_shot: bool,
    pub hot_potato: bool,
    pub leech_field: bool,
    pub sticky: bool,
    pub chaos_rounds: bool,
    pub adrenaline: bool,
    pub upsizer: bool,
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
            reload_time_mult: 1.0,
            laser: false,
            poison: false,
            hp_regen: 0.0,
            bullet_gravity_mult: 1.0,
            phantom: false,
            move_speed_mult: 1.0,
            damage_taken_mult: 1.0,
            knockback_immune: false,
            split_shot: false,
            hot_potato: false,
            leech_field: false,
            sticky: false,
            chaos_rounds: false,
            adrenaline: false,
            upsizer: false,
        }
    }
}

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
            CardId::Tiny => stats.size_mult *= 0.5,
            CardId::Bounceback => stats.bounceback = true,
            CardId::Shotgun => stats.shotgun = true,
            CardId::Gatling => stats.extra_ammo += 6,
            CardId::QuickReload => stats.reload_time_mult *= 0.5,
            CardId::Steroids => {
                stats.max_hp_mult *= 2.0;
                stats.size_mult *= 1.5;
            }
            CardId::LaserBeam => stats.laser = true,
            CardId::PoisonBullets => stats.poison = true,
            CardId::Chunky => stats.max_hp_bonus += 50.0,
            CardId::Chud => stats.hp_regen += 3.0,
            CardId::Zip => {
                stats.bullet_speed_mult *= 2.0;
                stats.bullet_gravity_mult *= 0.3;
            }
            CardId::PhantomShot => stats.phantom = true,
            CardId::SplitShot => stats.split_shot = true,
            CardId::Slug => {
                stats.bullet_damage_mult *= 3.0;
                stats.extra_ammo -= 2; // 3-2=1 bullet
                stats.reload_time_mult *= 2.0;
                stats.bullet_radius_mult *= 1.5;
            }
            CardId::HotPotato => stats.hot_potato = true,
            CardId::LeechField => stats.leech_field = true,
            CardId::Featherweight => {
                stats.extra_air_jumps += 3;
                stats.move_speed_mult *= 1.3;
                stats.damage_taken_mult *= 1.2;
            }
            CardId::StickyBombs => stats.sticky = true,
            CardId::Loadout => {
                stats.extra_ammo += 3;
                stats.reload_time_mult *= 0.75;
                stats.max_hp_bonus += 10.0;
            }
            CardId::ChaosRounds => stats.chaos_rounds = true,
            CardId::Adrenaline => stats.adrenaline = true,
            CardId::Upsizer => stats.upsizer = true,
            CardId::Juggernaut => {
                stats.max_hp_bonus += 40.0;
                stats.hp_regen += 1.0;
                stats.knockback_immune = true;
            }
            _ => {} // abilities don't modify stats
        }
    }
    stats
}

pub fn apply_stats(player: &mut Player, stats: &PlayerStats) {
    player.max_hp = (100.0 + stats.max_hp_bonus) * stats.max_hp_mult;
    if player.max_hp < 1.0 { player.max_hp = 1.0; }
    if player.hp > player.max_hp {
        player.hp = player.max_hp;
    }
    player.size.x = 0.6 * stats.size_mult;
    player.size.y = 1.6 * stats.size_mult;
    player.size.z = 0.6 * stats.size_mult;
}

// ── Ability activation ──────────────────────────────────────────────────────

const DASH_SPEED: f32 = 32.0;
const STOMP_HOP: f32 = 10.0;

/// Side effects that need world-level handling after ability activation
pub enum AbilityEffect {
    None,
    Screech,
    Ghost,
    GravityWell,
    Decoy,
}

pub fn activate_ability(card_id: CardId, player: &mut Player) -> (f32, AbilityEffect) {
    let def = &CARD_CATALOG[card_id as usize];
    match card_id {
        CardId::Dash => {
            player.velocity.x = player.aim_dir.x * DASH_SPEED;
            player.velocity.y = player.aim_dir.y * DASH_SPEED;
            (def.cooldown(), AbilityEffect::None)
        }
        CardId::Stomp => {
            player.velocity.y = STOMP_HOP;
            player.velocity.x *= 0.3;
            player.stomp_active = true;
            (def.cooldown(), AbilityEffect::None)
        }
        CardId::Screech => {
            (def.cooldown(), AbilityEffect::Screech)
        }
        CardId::Ghost => {
            player.ghost_timer = 3.0;
            (def.cooldown(), AbilityEffect::Ghost)
        }
        CardId::Rewind => {
            if !player.rewind_history.is_empty() {
                let (x, y, hp) = player.rewind_history[0];
                player.position.x = x;
                player.position.y = y;
                player.hp = hp.max(player.hp);
                player.velocity.x = 0.0;
                player.velocity.y = 0.0;
                player.velocity.z = 0.0;
                player.rewind_history.clear();
            }
            (def.cooldown(), AbilityEffect::None)
        }
        CardId::Overclock => {
            player.overclock_timer = 4.0;
            (def.cooldown(), AbilityEffect::None)
        }
        CardId::GravityWell => (def.cooldown(), AbilityEffect::GravityWell),
        CardId::Decoy => (def.cooldown(), AbilityEffect::Decoy),
        _ => (def.cooldown(), AbilityEffect::None),
    }
}
