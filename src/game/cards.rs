#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CardId {
    ExtraHp = 0,
    FastReload = 1,
    ExtraAmmo = 2,
    SpeedBoost = 3,
    DoubleJump = 4,
    BulletSpeed = 5,
    DamageUp = 6,
    ArmorPlating = 7,
    BigBullets = 8,
    Lightweight = 9,
}

impl CardId {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::ExtraHp),
            1 => Some(Self::FastReload),
            2 => Some(Self::ExtraAmmo),
            3 => Some(Self::SpeedBoost),
            4 => Some(Self::DoubleJump),
            5 => Some(Self::BulletSpeed),
            6 => Some(Self::DamageUp),
            7 => Some(Self::ArmorPlating),
            8 => Some(Self::BigBullets),
            9 => Some(Self::Lightweight),
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
}

pub const CARD_CATALOG: &[CardDef] = &[
    CardDef { id: CardId::ExtraHp,      name: "Vitality",     description: "+30 Max HP",           color: (220, 60, 60),   icon_glyph: '+' },
    CardDef { id: CardId::FastReload,    name: "Quick Hands",  description: "Reload 30% faster",    color: (60, 200, 220),  icon_glyph: '>' },
    CardDef { id: CardId::ExtraAmmo,     name: "Deep Pockets", description: "+1 Bullet capacity",   color: (200, 180, 60),  icon_glyph: '#' },
    CardDef { id: CardId::SpeedBoost,    name: "Adrenaline",   description: "+15% Move speed",      color: (100, 255, 100), icon_glyph: '~' },
    CardDef { id: CardId::DoubleJump,    name: "Rocket Boots", description: "+1 Air jump",          color: (180, 100, 255), icon_glyph: '^' },
    CardDef { id: CardId::BulletSpeed,   name: "Rifling",      description: "+25% Bullet speed",    color: (255, 200, 80),  icon_glyph: '!' },
    CardDef { id: CardId::DamageUp,      name: "Sharpened",    description: "+20% Damage",          color: (255, 80, 80),   icon_glyph: '*' },
    CardDef { id: CardId::ArmorPlating,  name: "Iron Skin",    description: "-15% Damage taken",    color: (160, 170, 180), icon_glyph: '=' },
    CardDef { id: CardId::BigBullets,    name: "Cannonball",   description: "+50% Bullet size",     color: (220, 140, 60),  icon_glyph: 'O' },
    CardDef { id: CardId::Lightweight,   name: "Featherfall",  description: "-20% Gravity",         color: (200, 220, 255), icon_glyph: '?' },
];

/// Pick `count` unique random card IDs from the catalog.
pub fn random_cards(rng_val: &mut u64, count: usize) -> Vec<u8> {
    let catalog_len = CARD_CATALOG.len();
    let mut pool: Vec<u8> = (0..catalog_len as u8).collect();
    let mut result = Vec::with_capacity(count);

    for _ in 0..count.min(catalog_len) {
        if pool.is_empty() { break; }
        // xorshift64
        *rng_val ^= *rng_val << 13;
        *rng_val ^= *rng_val >> 7;
        *rng_val ^= *rng_val << 17;
        let idx = (*rng_val as usize) % pool.len();
        result.push(pool.swap_remove(idx));
    }

    result
}
