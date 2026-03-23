use raylib::prelude::*;

use crate::level::platforms::Platform;

pub struct Level {
    pub platforms: Vec<Platform>,
    pub spawn_points: Vec<Vector3>,
    pub id: u8,
}

pub const MAP_COUNT: u8 = 6;

pub fn random_level(rng_val: u64) -> Level {
    let id = (rng_val % MAP_COUNT as u64) as u8;
    level_by_id(id)
}

pub fn level_by_id(id: u8) -> Level {
    match id {
        0 => classic(),
        1 => towers(),
        2 => pit(),
        3 => scaffold(),
        4 => bunkers(),
        5 => bridges(),
        _ => classic(),
    }
}

// ── Map 0: Classic ──────────────────────────────────────────────────────────
// Symmetric with two mid platforms and a high center platform.
fn classic() -> Level {
    let platforms = vec![
        // Floor
        Platform::wall(
            Vector3::new(-15.0, -1.0, -2.0),
            Vector3::new(15.0, 0.0, 2.0),
        ),
        // Left wall
        Platform::wall(
            Vector3::new(-15.0, 0.0, -2.0),
            Vector3::new(-14.0, 12.0, 2.0),
        ),
        // Right wall
        Platform::wall(
            Vector3::new(14.0, 0.0, -2.0),
            Vector3::new(15.0, 12.0, 2.0),
        ),
        // Left mid platform
        Platform::platform(
            Vector3::new(-8.0, 3.5, -2.0),
            Vector3::new(-3.0, 4.0, 2.0),
        ),
        // Right mid platform
        Platform::platform(
            Vector3::new(3.0, 3.5, -2.0),
            Vector3::new(8.0, 4.0, 2.0),
        ),
        // Center high platform
        Platform::platform(
            Vector3::new(-2.5, 7.0, -2.0),
            Vector3::new(2.5, 7.5, 2.0),
        ),
    ];

    Level {
        platforms,
        spawn_points: vec![
            Vector3::new(-6.0, 0.0, 0.0),
            Vector3::new(6.0, 0.0, 0.0),
            Vector3::new(-10.0, 0.0, 0.0),
            Vector3::new(10.0, 0.0, 0.0),
        ],
        id: 0,
    }
}

// ── Map 1: Towers ───────────────────────────────────────────────────────────
// Two tall narrow pillars with small ledges at varying heights.
fn towers() -> Level {
    let platforms = vec![
        // Floor
        Platform::wall(
            Vector3::new(-15.0, -1.0, -2.0),
            Vector3::new(15.0, 0.0, 2.0),
        ),
        // Left wall
        Platform::wall(
            Vector3::new(-15.0, 0.0, -2.0),
            Vector3::new(-14.0, 12.0, 2.0),
        ),
        // Right wall
        Platform::wall(
            Vector3::new(14.0, 0.0, -2.0),
            Vector3::new(15.0, 12.0, 2.0),
        ),
        // Left tower
        Platform::wall(
            Vector3::new(-7.0, 0.0, -2.0),
            Vector3::new(-5.5, 6.0, 2.0),
        ),
        // Right tower
        Platform::wall(
            Vector3::new(5.5, 0.0, -2.0),
            Vector3::new(7.0, 6.0, 2.0),
        ),
        // Left low ledge
        Platform::platform(
            Vector3::new(-12.0, 2.5, -2.0),
            Vector3::new(-8.0, 3.0, 2.0),
        ),
        // Right low ledge
        Platform::platform(
            Vector3::new(8.0, 2.5, -2.0),
            Vector3::new(12.0, 3.0, 2.0),
        ),
        // Center floating platform
        Platform::platform(
            Vector3::new(-2.0, 4.5, -2.0),
            Vector3::new(2.0, 5.0, 2.0),
        ),
        // Left high ledge (on tower)
        Platform::platform(
            Vector3::new(-9.0, 6.0, -2.0),
            Vector3::new(-5.5, 6.5, 2.0),
        ),
        // Right high ledge (on tower)
        Platform::platform(
            Vector3::new(5.5, 6.0, -2.0),
            Vector3::new(9.0, 6.5, 2.0),
        ),
        // Top bridge
        Platform::platform(
            Vector3::new(-1.5, 8.5, -2.0),
            Vector3::new(1.5, 9.0, 2.0),
        ),
    ];

    Level {
        platforms,
        spawn_points: vec![
            Vector3::new(-10.0, 0.0, 0.0),
            Vector3::new(10.0, 0.0, 0.0),
            Vector3::new(-2.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
        ],
        id: 1,
    }
}

// ── Map 2: Pit ──────────────────────────────────────────────────────────────
// Floor has a deadly gap in the center. Platforms bridge the sides.
fn pit() -> Level {
    let platforms = vec![
        // Left floor
        Platform::wall(
            Vector3::new(-15.0, -1.0, -2.0),
            Vector3::new(-4.0, 0.0, 2.0),
        ),
        // Right floor
        Platform::wall(
            Vector3::new(4.0, -1.0, -2.0),
            Vector3::new(15.0, 0.0, 2.0),
        ),
        // Left wall
        Platform::wall(
            Vector3::new(-15.0, 0.0, -2.0),
            Vector3::new(-14.0, 12.0, 2.0),
        ),
        // Right wall
        Platform::wall(
            Vector3::new(14.0, 0.0, -2.0),
            Vector3::new(15.0, 12.0, 2.0),
        ),
        // Low bridge over pit (narrow)
        Platform::platform(
            Vector3::new(-1.5, 2.0, -2.0),
            Vector3::new(1.5, 2.5, 2.0),
        ),
        // Left shelf
        Platform::platform(
            Vector3::new(-10.0, 3.5, -2.0),
            Vector3::new(-5.0, 4.0, 2.0),
        ),
        // Right shelf
        Platform::platform(
            Vector3::new(5.0, 3.5, -2.0),
            Vector3::new(10.0, 4.0, 2.0),
        ),
        // High center
        Platform::platform(
            Vector3::new(-3.0, 6.5, -2.0),
            Vector3::new(3.0, 7.0, 2.0),
        ),
        // Left high
        Platform::platform(
            Vector3::new(-12.0, 7.0, -2.0),
            Vector3::new(-8.0, 7.5, 2.0),
        ),
        // Right high
        Platform::platform(
            Vector3::new(8.0, 7.0, -2.0),
            Vector3::new(12.0, 7.5, 2.0),
        ),
    ];

    Level {
        platforms,
        spawn_points: vec![
            Vector3::new(-8.0, 0.0, 0.0),
            Vector3::new(8.0, 0.0, 0.0),
            Vector3::new(-12.0, 0.0, 0.0),
            Vector3::new(12.0, 0.0, 0.0),
        ],
        id: 2,
    }
}

// ── Map 3: Scaffold ─────────────────────────────────────────────────────────
// Grid of small floating platforms at varying heights. Vertical gameplay.
fn scaffold() -> Level {
    let platforms = vec![
        // Floor
        Platform::wall(
            Vector3::new(-15.0, -1.0, -2.0),
            Vector3::new(15.0, 0.0, 2.0),
        ),
        // Left wall
        Platform::wall(
            Vector3::new(-15.0, 0.0, -2.0),
            Vector3::new(-14.0, 12.0, 2.0),
        ),
        // Right wall
        Platform::wall(
            Vector3::new(14.0, 0.0, -2.0),
            Vector3::new(15.0, 12.0, 2.0),
        ),
        // Bottom row
        Platform::platform(
            Vector3::new(-10.0, 2.0, -2.0),
            Vector3::new(-7.0, 2.5, 2.0),
        ),
        Platform::platform(
            Vector3::new(-1.5, 2.0, -2.0),
            Vector3::new(1.5, 2.5, 2.0),
        ),
        Platform::platform(
            Vector3::new(7.0, 2.0, -2.0),
            Vector3::new(10.0, 2.5, 2.0),
        ),
        // Mid row (offset)
        Platform::platform(
            Vector3::new(-12.0, 4.5, -2.0),
            Vector3::new(-9.0, 5.0, 2.0),
        ),
        Platform::platform(
            Vector3::new(-5.0, 4.5, -2.0),
            Vector3::new(-2.0, 5.0, 2.0),
        ),
        Platform::platform(
            Vector3::new(2.0, 4.5, -2.0),
            Vector3::new(5.0, 5.0, 2.0),
        ),
        Platform::platform(
            Vector3::new(9.0, 4.5, -2.0),
            Vector3::new(12.0, 5.0, 2.0),
        ),
        // Top row
        Platform::platform(
            Vector3::new(-8.0, 7.0, -2.0),
            Vector3::new(-5.0, 7.5, 2.0),
        ),
        Platform::platform(
            Vector3::new(-1.5, 7.5, -2.0),
            Vector3::new(1.5, 8.0, 2.0),
        ),
        Platform::platform(
            Vector3::new(5.0, 7.0, -2.0),
            Vector3::new(8.0, 7.5, 2.0),
        ),
    ];

    Level {
        platforms,
        spawn_points: vec![
            Vector3::new(-8.5, 0.0, 0.0),
            Vector3::new(8.5, 0.0, 0.0),
            Vector3::new(-3.0, 0.0, 0.0),
            Vector3::new(3.0, 0.0, 0.0),
        ],
        id: 3,
    }
}

// ── Map 4: Bunkers ──────────────────────────────────────────────────────────
// Two enclosed bunker structures with openings, connected by upper platforms.
fn bunkers() -> Level {
    let platforms = vec![
        // Floor
        Platform::wall(
            Vector3::new(-15.0, -1.0, -2.0),
            Vector3::new(15.0, 0.0, 2.0),
        ),
        // Left wall
        Platform::wall(
            Vector3::new(-15.0, 0.0, -2.0),
            Vector3::new(-14.0, 12.0, 2.0),
        ),
        // Right wall
        Platform::wall(
            Vector3::new(14.0, 0.0, -2.0),
            Vector3::new(15.0, 12.0, 2.0),
        ),
        // Left bunker floor (raised)
        Platform::platform(
            Vector3::new(-13.0, 2.5, -2.0),
            Vector3::new(-7.0, 3.0, 2.0),
        ),
        // Left bunker roof
        Platform::platform(
            Vector3::new(-13.0, 6.0, -2.0),
            Vector3::new(-7.5, 6.5, 2.0),
        ),
        // Left bunker inner wall
        Platform::wall(
            Vector3::new(-7.0, 0.0, -2.0),
            Vector3::new(-6.5, 5.0, 2.0),
        ),
        // Right bunker floor (raised)
        Platform::platform(
            Vector3::new(7.0, 2.5, -2.0),
            Vector3::new(13.0, 3.0, 2.0),
        ),
        // Right bunker roof
        Platform::platform(
            Vector3::new(7.5, 6.0, -2.0),
            Vector3::new(13.0, 6.5, 2.0),
        ),
        // Right bunker inner wall
        Platform::wall(
            Vector3::new(6.5, 0.0, -2.0),
            Vector3::new(7.0, 5.0, 2.0),
        ),
        // Center low platform
        Platform::platform(
            Vector3::new(-2.5, 1.5, -2.0),
            Vector3::new(2.5, 2.0, 2.0),
        ),
        // Center high bridge
        Platform::platform(
            Vector3::new(-3.0, 5.0, -2.0),
            Vector3::new(3.0, 5.5, 2.0),
        ),
        // Top center
        Platform::platform(
            Vector3::new(-1.5, 8.5, -2.0),
            Vector3::new(1.5, 9.0, 2.0),
        ),
    ];

    Level {
        platforms,
        spawn_points: vec![
            Vector3::new(-10.0, 3.0, 0.0),
            Vector3::new(10.0, 3.0, 0.0),
            Vector3::new(-3.0, 0.0, 0.0),
            Vector3::new(3.0, 0.0, 0.0),
        ],
        id: 4,
    }
}

// ── Map 5: Bridges ──────────────────────────────────────────────────────────
// Three thin bridges at different heights spanning the arena.
fn bridges() -> Level {
    let platforms = vec![
        // Floor
        Platform::wall(
            Vector3::new(-15.0, -1.0, -2.0),
            Vector3::new(15.0, 0.0, 2.0),
        ),
        // Left wall
        Platform::wall(
            Vector3::new(-15.0, 0.0, -2.0),
            Vector3::new(-14.0, 12.0, 2.0),
        ),
        // Right wall
        Platform::wall(
            Vector3::new(14.0, 0.0, -2.0),
            Vector3::new(15.0, 12.0, 2.0),
        ),
        // Low bridge (left-leaning)
        Platform::platform(
            Vector3::new(-10.0, 2.5, -2.0),
            Vector3::new(2.0, 3.0, 2.0),
        ),
        // Mid bridge (right-leaning)
        Platform::platform(
            Vector3::new(-2.0, 5.5, -2.0),
            Vector3::new(10.0, 6.0, 2.0),
        ),
        // High bridge (centered, short)
        Platform::platform(
            Vector3::new(-5.0, 8.5, -2.0),
            Vector3::new(5.0, 9.0, 2.0),
        ),
        // Left pillar (connects low to mid)
        Platform::wall(
            Vector3::new(-11.0, 0.0, -2.0),
            Vector3::new(-10.0, 5.5, 2.0),
        ),
        // Right pillar (connects mid to high)
        Platform::wall(
            Vector3::new(10.0, 0.0, -2.0),
            Vector3::new(11.0, 3.5, 2.0),
        ),
        // Small left high ledge
        Platform::platform(
            Vector3::new(-13.0, 5.0, -2.0),
            Vector3::new(-11.0, 5.5, 2.0),
        ),
        // Small right low ledge
        Platform::platform(
            Vector3::new(11.0, 3.0, -2.0),
            Vector3::new(13.0, 3.5, 2.0),
        ),
    ];

    Level {
        platforms,
        spawn_points: vec![
            Vector3::new(-6.0, 0.0, 0.0),
            Vector3::new(6.0, 0.0, 0.0),
            Vector3::new(-12.0, 0.0, 0.0),
            Vector3::new(12.0, 0.0, 0.0),
        ],
        id: 5,
    }
}
