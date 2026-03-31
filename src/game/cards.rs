use crate::player::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CardId {
    // Abilities (active, right-click, have cooldown)
    Dash = 0,
    Overclock = 1,
    Rewind = 2,
    Sage = 3,
    Ghost = 4,
    Screech = 5,
    BulletManip = 6,
    CaseOh = 7,
    // Basic Powerups
    GatlingGunner = 8,
    BottomlessBag = 9,
    BigBullets = 10,
    Caffeine = 11,
    IronSkin = 12,
    Tough = 13,
    LastClip = 14,
    Vampire = 15,
    LeechShot = 16,
    HeavyHitter = 17,
    TracerRounds = 18,
    Sniper = 19,
    Precise = 20,
    GlassCannon = 21,
    FastMag = 22,
    Tiny = 23,
    RapidFire = 24,
    Zip = 25,
    Knockback = 26,
    Smol = 27,
    Chunky = 28,
    Humongous = 29,
    Chud = 30,
    HappyThoughts = 31,
    Free = 32,
    Breeze = 33,
    Bird = 34,
    FatRounds = 35,
    CursedMag = 36,
    // Special Powerups
    EchoShot = 37,
    SoulSiphon = 38,
    Adrenaline = 39,
    Bloodthirsty = 40,
    Shotgun = 41,
    RearShot = 42,
    TriShot = 43,
    DoubleVision = 44,
    DoppelGanger = 45,
    VoidShots = 46,
    Gambler = 47,
    Confusion = 48,
    Featherweight = 49,
    Upsize = 50,
    PoisonBullets = 51,
    StickyBombs = 52,
    PiercingShots = 53,
    BouncyBullets = 54,
    HomingShots = 55,
    GunEmDown = 56,
    IceShots = 57,
    Steroids = 58,
    AngryBlind = 59,
}

impl CardId {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Dash),
            1 => Some(Self::Overclock),
            2 => Some(Self::Rewind),
            3 => Some(Self::Sage),
            4 => Some(Self::Ghost),
            5 => Some(Self::Screech),
            6 => Some(Self::BulletManip),
            7 => Some(Self::CaseOh),
            8 => Some(Self::GatlingGunner),
            9 => Some(Self::BottomlessBag),
            10 => Some(Self::BigBullets),
            11 => Some(Self::Caffeine),
            12 => Some(Self::IronSkin),
            13 => Some(Self::Tough),
            14 => Some(Self::LastClip),
            15 => Some(Self::Vampire),
            16 => Some(Self::LeechShot),
            17 => Some(Self::HeavyHitter),
            18 => Some(Self::TracerRounds),
            19 => Some(Self::Sniper),
            20 => Some(Self::Precise),
            21 => Some(Self::GlassCannon),
            22 => Some(Self::FastMag),
            23 => Some(Self::Tiny),
            24 => Some(Self::RapidFire),
            25 => Some(Self::Zip),
            26 => Some(Self::Knockback),
            27 => Some(Self::Smol),
            28 => Some(Self::Chunky),
            29 => Some(Self::Humongous),
            30 => Some(Self::Chud),
            31 => Some(Self::HappyThoughts),
            32 => Some(Self::Free),
            33 => Some(Self::Breeze),
            34 => Some(Self::Bird),
            35 => Some(Self::FatRounds),
            36 => Some(Self::CursedMag),
            37 => Some(Self::EchoShot),
            38 => Some(Self::SoulSiphon),
            39 => Some(Self::Adrenaline),
            40 => Some(Self::Bloodthirsty),
            41 => Some(Self::Shotgun),
            42 => Some(Self::RearShot),
            43 => Some(Self::TriShot),
            44 => Some(Self::DoubleVision),
            45 => Some(Self::DoppelGanger),
            46 => Some(Self::VoidShots),
            47 => Some(Self::Gambler),
            48 => Some(Self::Confusion),
            49 => Some(Self::Featherweight),
            50 => Some(Self::Upsize),
            51 => Some(Self::PoisonBullets),
            52 => Some(Self::StickyBombs),
            53 => Some(Self::PiercingShots),
            54 => Some(Self::BouncyBullets),
            55 => Some(Self::HomingShots),
            56 => Some(Self::GunEmDown),
            57 => Some(Self::IceShots),
            58 => Some(Self::Steroids),
            59 => Some(Self::AngryBlind),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CardKind {
    Ability { cooldown: f32 },
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
    pub fn is_ability(&self) -> bool { matches!(self.kind, CardKind::Ability { .. }) }
    pub fn is_powerup(&self) -> bool { matches!(self.kind, CardKind::Powerup) }
    pub fn is_implemented(&self) -> bool { true }
}

pub const CARD_CATALOG: &[CardDef] = &[
    // ── Abilities ──
    CardDef { id: CardId::Dash,        name: "Dash",              description: "Burst in aim direction",              color: (100, 255, 100), icon_glyph: '>', kind: CardKind::Ability { cooldown: 2.0 } },
    CardDef { id: CardId::Overclock,   name: "Overclock",         description: "2x speed+fire, then crash",           color: (255, 220, 60),  icon_glyph: 'K', kind: CardKind::Ability { cooldown: 10.0 } },
    CardDef { id: CardId::Rewind,      name: "Rewind",            description: "Snap back to 3s ago",                 color: (180, 120, 255), icon_glyph: 'D', kind: CardKind::Ability { cooldown: 8.0 } },
    CardDef { id: CardId::Sage,        name: "Sage",              description: "Healing zone, anyone can use, 5s",    color: (100, 255, 200), icon_glyph: '+', kind: CardKind::Ability { cooldown: 12.0 } },
    CardDef { id: CardId::Ghost,       name: "Ghost",             description: "Vanish completely for 3s",            color: (180, 180, 220), icon_glyph: '?', kind: CardKind::Ability { cooldown: 10.0 } },
    CardDef { id: CardId::Screech,     name: "Screech",           description: "Blast all enemies away",              color: (255, 255, 60),  icon_glyph: '^', kind: CardKind::Ability { cooldown: 12.0 } },
    CardDef { id: CardId::BulletManip, name: "Bullet Manipulation", description: "All bullets become yours + homing", color: (60, 160, 255),  icon_glyph: '@', kind: CardKind::Ability { cooldown: 15.0 } },
    CardDef { id: CardId::CaseOh,      name: "CaseOh",            description: "EARTHQUAKE! 3s of violent shaking",   color: (255, 80, 80),   icon_glyph: 'Q', kind: CardKind::Ability { cooldown: 14.0 } },
    // ── Basic Powerups ──
    CardDef { id: CardId::GatlingGunner,  name: "Gatling Gunner",   description: "+6 Ammo Capacity",                   color: (180, 180, 60),  icon_glyph: 'G', kind: CardKind::Powerup },
    CardDef { id: CardId::BottomlessBag,  name: "Bottomless Bag",   description: "Infinite Ammo, -30% DMG",            color: (160, 140, 100), icon_glyph: 'B', kind: CardKind::Powerup },
    CardDef { id: CardId::BigBullets,     name: "Big Bullets",      description: "2x Bullet Size, +20% DMG",           color: (200, 160, 255), icon_glyph: 'O', kind: CardKind::Powerup },
    CardDef { id: CardId::Caffeine,       name: "Caffeine",         description: "+30% Firerate +30% Speed -10HP",      color: (200, 140, 80),  icon_glyph: 'C', kind: CardKind::Powerup },
    CardDef { id: CardId::IronSkin,       name: "Iron Skin",        description: "Take 25% less DMG",                   color: (160, 170, 180), icon_glyph: 'I', kind: CardKind::Powerup },
    CardDef { id: CardId::Tough,          name: "Tough",            description: "+50 Max HP, -10% Movement",           color: (180, 140, 100), icon_glyph: 'T', kind: CardKind::Powerup },
    CardDef { id: CardId::LastClip,       name: "Last Clip",        description: "+20 Ammo, can't reload",              color: (255, 200, 60),  icon_glyph: 'L', kind: CardKind::Powerup },
    CardDef { id: CardId::Vampire,        name: "Vampire",          description: "+8 HP per hit",                       color: (200, 40, 80),   icon_glyph: 'V', kind: CardKind::Powerup },
    CardDef { id: CardId::LeechShot,      name: "Leech Shot",       description: "Steal 5 HP per hit",                  color: (160, 60, 100),  icon_glyph: 'l', kind: CardKind::Powerup },
    CardDef { id: CardId::HeavyHitter,    name: "Heavy Hitter",     description: "+50% DMG",                            color: (255, 140, 60),  icon_glyph: 'H', kind: CardKind::Powerup },
    CardDef { id: CardId::TracerRounds,   name: "Tracer Rounds",    description: "2x Bullet Speed, -15 DMG",            color: (255, 255, 180), icon_glyph: '-', kind: CardKind::Powerup },
    CardDef { id: CardId::Sniper,         name: "Sniper",           description: "2x DMG, +300% Reload Time",           color: (100, 120, 160), icon_glyph: 'S', kind: CardKind::Powerup },
    CardDef { id: CardId::Precise,        name: "Precise",          description: "-50% Fire Rate, +75% DMG",            color: (200, 200, 220), icon_glyph: 'P', kind: CardKind::Powerup },
    CardDef { id: CardId::GlassCannon,    name: "Glass Cannon",     description: "-50% Max HP, 2x DMG",                 color: (255, 40, 40),   icon_glyph: 'X', kind: CardKind::Powerup },
    CardDef { id: CardId::FastMag,        name: "Fast Mag",         description: "67% Faster Reload",                   color: (100, 200, 255), icon_glyph: 'R', kind: CardKind::Powerup },
    CardDef { id: CardId::Tiny,           name: "Tiny",             description: "-50% Size, -10 Max HP",                color: (180, 140, 255), icon_glyph: '.', kind: CardKind::Powerup },
    CardDef { id: CardId::RapidFire,      name: "Rapid Fire",       description: "2x Fire Rate",                         color: (255, 255, 100), icon_glyph: '!', kind: CardKind::Powerup },
    CardDef { id: CardId::Zip,            name: "Zip",              description: "x1.5 Bullet Speed, less gravity",      color: (200, 240, 255), icon_glyph: 'Z', kind: CardKind::Powerup },
    CardDef { id: CardId::Knockback,      name: "Knockback",        description: "Shots launch enemies",                 color: (100, 255, 200), icon_glyph: '<', kind: CardKind::Powerup },
    CardDef { id: CardId::Smol,           name: "Smol",             description: "-40% Size, -10% DMG",                  color: (200, 180, 255), icon_glyph: ',', kind: CardKind::Powerup },
    CardDef { id: CardId::Chunky,         name: "Chunky",           description: "+30 Max HP",                           color: (200, 140, 80),  icon_glyph: 'c', kind: CardKind::Powerup },
    CardDef { id: CardId::Humongous,      name: "Humongous",        description: "+75 Max HP, +20% Size",                color: (220, 160, 100), icon_glyph: 'M', kind: CardKind::Powerup },
    CardDef { id: CardId::Chud,           name: "Chud",             description: "+50 Max HP, +2 HP/s regen",            color: (80, 255, 120),  icon_glyph: 'h', kind: CardKind::Powerup },
    CardDef { id: CardId::HappyThoughts,  name: "Happy Thoughts",   description: "+3 HP/s regen",                        color: (255, 200, 220), icon_glyph: 'J', kind: CardKind::Powerup },
    CardDef { id: CardId::Free,           name: "Free",             description: "+75% Movement Speed",                   color: (120, 255, 180), icon_glyph: 'F', kind: CardKind::Powerup },
    CardDef { id: CardId::Breeze,         name: "Breeze",           description: "+50% Speed, +1 Air Jump",               color: (180, 240, 255), icon_glyph: 'b', kind: CardKind::Powerup },
    CardDef { id: CardId::Bird,           name: "Bird",             description: "+4 Air Jumps",                          color: (120, 220, 255), icon_glyph: 'W', kind: CardKind::Powerup },
    CardDef { id: CardId::FatRounds,      name: "Fat Rounds",       description: "x4 Size, +50% DMG, very slow",          color: (200, 180, 140), icon_glyph: '0', kind: CardKind::Powerup },
    CardDef { id: CardId::CursedMag,      name: "Cursed Mag",       description: "Random -30% to +30% DMG per shot",      color: (255, 100, 255), icon_glyph: 'N', kind: CardKind::Powerup },
    // ── Special Powerups ──
    CardDef { id: CardId::EchoShot,       name: "Echo Shot",        description: "Ghost bullet copy 0.3s later",          color: (180, 200, 255), icon_glyph: 'E', kind: CardKind::Powerup },
    CardDef { id: CardId::SoulSiphon,     name: "Soul Siphon",      description: "Kills give +5 permanent Max HP",        color: (180, 60, 200),  icon_glyph: 'w', kind: CardKind::Powerup },
    CardDef { id: CardId::Adrenaline,     name: "Adrenaline",       description: "Hit = 3s speed+fire+reload boost",      color: (255, 80, 80),   icon_glyph: 'A', kind: CardKind::Powerup },
    CardDef { id: CardId::Bloodthirsty,   name: "Bloodthirsty",     description: "Hitting enemy = 3s speed+DMG boost",    color: (200, 40, 40),   icon_glyph: 'x', kind: CardKind::Powerup },
    CardDef { id: CardId::Shotgun,        name: "Shotgun",          description: "+50% Reload, dump all ammo at once",     color: (255, 160, 80),  icon_glyph: '#', kind: CardKind::Powerup },
    CardDef { id: CardId::RearShot,       name: "Rear Shot",        description: "Bullets also fire behind you",           color: (160, 200, 120), icon_glyph: '=', kind: CardKind::Powerup },
    CardDef { id: CardId::TriShot,        name: "Tri-Shot",         description: "Shoot 3 bullets, 45 spread",             color: (80, 200, 255),  icon_glyph: 'Y', kind: CardKind::Powerup },
    CardDef { id: CardId::DoubleVision,   name: "Double Vision",    description: "Double shot",                            color: (200, 200, 255), icon_glyph: '"', kind: CardKind::Powerup },
    CardDef { id: CardId::DoppelGanger,   name: "Doppel Ganger",    description: "Clone auto-shoots with your bullets",    color: (200, 200, 200), icon_glyph: '&', kind: CardKind::Powerup },
    CardDef { id: CardId::VoidShots,      name: "Void Shots",       description: "Bullets suck enemies on hit",            color: (80, 40, 160),   icon_glyph: 'o', kind: CardKind::Powerup },
    CardDef { id: CardId::Gambler,        name: "Gambler",          description: "Receive 2 random powerups",              color: (255, 215, 0),   icon_glyph: '$', kind: CardKind::Powerup },
    CardDef { id: CardId::Confusion,      name: "Confusion",        description: "Opponents have inverted controls",       color: (255, 120, 200), icon_glyph: '~', kind: CardKind::Powerup },
    CardDef { id: CardId::Featherweight,  name: "Featherweight",    description: "+3 Jumps, +25% Speed",                   color: (200, 240, 255), icon_glyph: 'f', kind: CardKind::Powerup },
    CardDef { id: CardId::Upsize,         name: "Upsize",           description: "Hits make enemies bigger",               color: (255, 200, 100), icon_glyph: 'U', kind: CardKind::Powerup },
    CardDef { id: CardId::PoisonBullets,  name: "Poison Bullets",   description: "3s damage over time on hit",             color: (80, 200, 40),   icon_glyph: '%', kind: CardKind::Powerup },
    CardDef { id: CardId::StickyBombs,    name: "Sticky Bombs",     description: "Terrain shots explode 1s later",         color: (200, 180, 60),  icon_glyph: '*', kind: CardKind::Powerup },
    CardDef { id: CardId::PiercingShots,  name: "Piercing Shots",   description: "Bullets pass through players",           color: (255, 255, 200), icon_glyph: '|', kind: CardKind::Powerup },
    CardDef { id: CardId::BouncyBullets,  name: "Bouncy Bullets",   description: "Bounce off walls 2x",                   color: (80, 255, 180),  icon_glyph: ')', kind: CardKind::Powerup },
    CardDef { id: CardId::HomingShots,    name: "Homing Shots",     description: "Bullets chase enemies",                  color: (255, 80, 200),  icon_glyph: '@', kind: CardKind::Powerup },
    CardDef { id: CardId::GunEmDown,      name: "Gun em' Down",     description: "-50% DMG, 2x Fire Rate, 2x Ammo",       color: (200, 200, 100), icon_glyph: 'g', kind: CardKind::Powerup },
    CardDef { id: CardId::IceShots,       name: "Ice Shots",        description: "Bullets slow enemies 2s",                color: (140, 200, 255), icon_glyph: 'i', kind: CardKind::Powerup },
    CardDef { id: CardId::Steroids,       name: "Steroids",         description: "2x Max HP, 1.5x Bigger",                color: (255, 60, 120),  icon_glyph: '+', kind: CardKind::Powerup },
    CardDef { id: CardId::AngryBlind,     name: "Angry & Blind",    description: "4x DMG, random bullet direction",        color: (255, 40, 40),   icon_glyph: '!', kind: CardKind::Powerup },
];

pub fn random_cards(rng_val: &mut u64, count: usize) -> Vec<u8> {
    let pool: Vec<u8> = CARD_CATALOG.iter()
        .filter(|c| c.is_implemented())
        .map(|c| c.id as u8)
        .collect();
    let mut available = pool;
    let mut result = Vec::with_capacity(count);
    for _ in 0..count.min(available.len()) {
        if available.is_empty() { break; }
        *rng_val ^= *rng_val << 13;
        *rng_val ^= *rng_val >> 7;
        *rng_val ^= *rng_val << 17;
        let idx = (*rng_val as usize) % available.len();
        result.push(available.swap_remove(idx));
    }
    result
}

// ── Stat modifiers from powerups ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PlayerStats {
    // HP / size / movement
    pub max_hp_mult: f32,
    pub max_hp_bonus: f32,
    pub size_mult: f32,
    pub extra_air_jumps: i32,
    pub move_speed_mult: f32,
    pub damage_taken_mult: f32,
    pub hp_regen: f32,
    // Bullet modifiers
    pub bullet_damage_mult: f32,
    pub bullet_damage_flat: f32,
    pub bullet_speed_mult: f32,
    pub bullet_radius_mult: f32,
    pub bullet_gravity_mult: f32,
    pub rubber_bounces: i32,
    pub homing: bool,
    pub piercing: bool,
    pub poison: bool,
    pub sticky: bool,
    pub ice_shots: bool,
    pub void_shots: bool,
    pub cursed_mag: bool,
    pub knockback: bool,
    pub vampire_heal: f32,
    // Shooting modifiers
    pub shoot_cooldown_mult: f32,
    pub extra_ammo: i32,
    pub reload_time_mult: f32,
    pub infinite_ammo: bool,
    pub no_reload: bool,
    pub shotgun: bool,
    pub tri_shot: bool,
    pub double_shot: bool,
    pub rear_shot: bool,
    pub angry_blind: bool,
    pub echo_shot: bool,
    // Passive effects
    pub adrenaline: bool,
    pub bloodthirsty: bool,
    pub upsize: bool,
    pub soul_siphon: bool,
    pub confusion: bool,
    pub doppelganger: bool,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            max_hp_mult: 1.0,
            max_hp_bonus: 0.0,
            size_mult: 1.0,
            extra_air_jumps: 0,
            move_speed_mult: 1.0,
            damage_taken_mult: 1.0,
            hp_regen: 0.0,
            bullet_damage_mult: 1.0,
            bullet_damage_flat: 0.0,
            bullet_speed_mult: 1.0,
            bullet_radius_mult: 1.0,
            bullet_gravity_mult: 1.0,
            rubber_bounces: 0,
            homing: false,
            piercing: false,
            poison: false,
            sticky: false,
            ice_shots: false,
            void_shots: false,
            cursed_mag: false,
            knockback: false,
            vampire_heal: 0.0,
            shoot_cooldown_mult: 1.0,
            extra_ammo: 0,
            reload_time_mult: 1.0,
            infinite_ammo: false,
            no_reload: false,
            shotgun: false,
            tri_shot: false,
            double_shot: false,
            rear_shot: false,
            angry_blind: false,
            echo_shot: false,
            adrenaline: false,
            bloodthirsty: false,
            upsize: false,
            soul_siphon: false,
            confusion: false,
            doppelganger: false,
        }
    }
}

pub fn compute_stats(cards: &[(CardId, f32)]) -> PlayerStats {
    let mut s = PlayerStats::default();
    for (card_id, _) in cards {
        match card_id {
            // Basic Powerups
            CardId::GatlingGunner  => s.extra_ammo += 6,
            CardId::BottomlessBag  => { s.infinite_ammo = true; s.bullet_damage_mult *= 0.7; }
            CardId::BigBullets     => { s.bullet_radius_mult *= 2.0; s.bullet_damage_mult *= 1.2; }
            CardId::Caffeine       => { s.shoot_cooldown_mult *= 0.7; s.move_speed_mult *= 1.3; s.max_hp_bonus -= 10.0; }
            CardId::IronSkin       => s.damage_taken_mult *= 0.75,
            CardId::Tough          => { s.max_hp_bonus += 50.0; s.move_speed_mult *= 0.9; }
            CardId::LastClip       => { s.extra_ammo += 20; s.no_reload = true; }
            CardId::Vampire        => s.vampire_heal += 8.0,
            CardId::LeechShot      => s.vampire_heal += 5.0,
            CardId::HeavyHitter    => s.bullet_damage_mult *= 1.5,
            CardId::TracerRounds   => { s.bullet_speed_mult *= 2.0; s.bullet_damage_flat -= 15.0; }
            CardId::Sniper         => { s.bullet_damage_mult *= 2.0; s.reload_time_mult *= 4.0; }
            CardId::Precise        => { s.shoot_cooldown_mult *= 2.0; s.bullet_damage_mult *= 1.75; }
            CardId::GlassCannon    => { s.max_hp_mult *= 0.5; s.bullet_damage_mult *= 2.0; }
            CardId::FastMag        => s.reload_time_mult *= 0.33,
            CardId::Tiny           => { s.size_mult *= 0.5; s.max_hp_bonus -= 10.0; }
            CardId::RapidFire      => s.shoot_cooldown_mult *= 0.5,
            CardId::Zip            => { s.bullet_speed_mult *= 1.5; s.bullet_gravity_mult *= 0.5; }
            CardId::Knockback      => s.knockback = true,
            CardId::Smol           => { s.size_mult *= 0.6; s.bullet_damage_mult *= 0.9; }
            CardId::Chunky         => s.max_hp_bonus += 30.0,
            CardId::Humongous      => { s.max_hp_bonus += 75.0; s.size_mult *= 1.2; }
            CardId::Chud           => { s.max_hp_bonus += 50.0; s.hp_regen += 2.0; }
            CardId::HappyThoughts  => s.hp_regen += 3.0,
            CardId::Free           => s.move_speed_mult *= 1.75,
            CardId::Breeze         => { s.move_speed_mult *= 1.5; s.extra_air_jumps += 1; }
            CardId::Bird           => s.extra_air_jumps += 4,
            CardId::FatRounds      => { s.bullet_radius_mult *= 4.0; s.bullet_damage_mult *= 1.5; s.bullet_speed_mult *= 0.25; }
            CardId::CursedMag      => s.cursed_mag = true,
            // Special Powerups
            CardId::EchoShot       => s.echo_shot = true,
            CardId::SoulSiphon     => s.soul_siphon = true,
            CardId::Adrenaline     => s.adrenaline = true,
            CardId::Bloodthirsty   => s.bloodthirsty = true,
            CardId::Shotgun        => { s.shotgun = true; s.reload_time_mult *= 1.5; }
            CardId::RearShot       => s.rear_shot = true,
            CardId::TriShot        => s.tri_shot = true,
            CardId::DoubleVision   => s.double_shot = true,
            CardId::DoppelGanger   => s.doppelganger = true,
            CardId::VoidShots      => s.void_shots = true,
            CardId::Gambler        => {} // special: resolved at pick time
            CardId::Confusion      => s.confusion = true,
            CardId::Featherweight  => { s.extra_air_jumps += 3; s.move_speed_mult *= 1.25; }
            CardId::Upsize         => s.upsize = true,
            CardId::PoisonBullets  => s.poison = true,
            CardId::StickyBombs    => s.sticky = true,
            CardId::PiercingShots  => s.piercing = true,
            CardId::BouncyBullets  => s.rubber_bounces += 2,
            CardId::HomingShots    => s.homing = true,
            CardId::GunEmDown      => { s.bullet_damage_mult *= 0.5; s.shoot_cooldown_mult *= 0.5; s.extra_ammo += 3; }
            CardId::IceShots       => s.ice_shots = true,
            CardId::Steroids       => { s.max_hp_mult *= 2.0; s.size_mult *= 1.5; }
            CardId::AngryBlind     => { s.bullet_damage_mult *= 4.0; s.angry_blind = true; }
            _ => {} // abilities don't modify stats
        }
    }
    s
}

pub fn apply_stats(player: &mut Player, stats: &PlayerStats, base_hp: f32) {
    player.max_hp = (base_hp + stats.max_hp_bonus) * stats.max_hp_mult;
    if player.max_hp < 1.0 { player.max_hp = 1.0; }
    if player.hp > player.max_hp { player.hp = player.max_hp; }
    player.size.x = 0.6 * stats.size_mult;
    player.size.y = 1.6 * stats.size_mult;
    player.size.z = 0.6 * stats.size_mult;
}

// ── Ability activation ──────────────────────────────────────────────────────

const DASH_SPEED: f32 = 32.0;

pub enum AbilityEffect {
    None,
    Screech,
    Ghost,
    Sage,
    BulletManip,
    CaseOh,
}

pub fn activate_ability(card_id: CardId, player: &mut Player) -> (f32, AbilityEffect) {
    let def = &CARD_CATALOG[card_id as usize];
    match card_id {
        CardId::Dash => {
            player.velocity.x = player.aim_dir.x * DASH_SPEED;
            player.velocity.y = player.aim_dir.y * DASH_SPEED;
            (def.cooldown(), AbilityEffect::None)
        }
        CardId::Overclock => {
            player.overclock_timer = 4.0;
            (def.cooldown(), AbilityEffect::None)
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
        CardId::Sage => (def.cooldown(), AbilityEffect::Sage),
        CardId::Ghost => {
            player.ghost_timer = 3.0;
            (def.cooldown(), AbilityEffect::Ghost)
        }
        CardId::Screech => (def.cooldown(), AbilityEffect::Screech),
        CardId::BulletManip => (def.cooldown(), AbilityEffect::BulletManip),
        CardId::CaseOh => (def.cooldown(), AbilityEffect::CaseOh),
        _ => (def.cooldown(), AbilityEffect::None),
    }
}
