use raylib::prelude::*;

const PARTICLE_GRAVITY: f32 = 22.0;

// ── Minimal xorshift64 RNG ───────────────────────────────────────────────────

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        let v = self.next();
        (v >> 11) as f32 / (1u64 << 53) as f32
    }

    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

// ── Particle ─────────────────────────────────────────────────────────────────

pub struct Particle {
    pub position: Vector3,
    pub vel_x: f32,
    pub vel_y: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: Color,
    pub size: f32,
}

pub fn update_particles(particles: &mut Vec<Particle>, dt: f32) {
    for p in particles.iter_mut() {
        p.vel_y -= PARTICLE_GRAVITY * dt;
        p.position.x += p.vel_x * dt;
        p.position.y += p.vel_y * dt;
        p.lifetime -= dt;
    }
    particles.retain(|p| p.lifetime > 0.0);
}

// ── Spawn helpers ─────────────────────────────────────────────────────────────

pub fn spawn_terrain_hit(particles: &mut Vec<Particle>, rng: &mut Rng, pos: Vector3, color: Color) {
    for _ in 0..8 {
        let angle = rng.range(0.0, std::f32::consts::TAU);
        let speed = rng.range(2.0, 5.5);
        let lifetime = rng.range(0.25, 0.55);
        particles.push(Particle {
            position: pos,
            vel_x: angle.cos() * speed,
            vel_y: angle.sin() * speed + 1.5, // slight upward bias
            lifetime,
            max_lifetime: lifetime,
            color,
            size: rng.range(0.03, 0.07),
        });
    }
}

pub fn spawn_player_hit(particles: &mut Vec<Particle>, rng: &mut Rng, pos: Vector3, color: Color) {
    for _ in 0..12 {
        let angle = rng.range(0.0, std::f32::consts::TAU);
        let speed = rng.range(3.0, 8.0);
        let lifetime = rng.range(0.3, 0.65);
        particles.push(Particle {
            position: pos,
            vel_x: angle.cos() * speed,
            vel_y: angle.sin().abs() * speed + 2.0, // upward burst
            lifetime,
            max_lifetime: lifetime,
            color,
            size: rng.range(0.05, 0.11),
        });
    }
}

pub fn spawn_from_events(
    events: &[crate::game::net::GameEvent],
    particles: &mut Vec<Particle>,
    rng: &mut Rng,
) {
    use crate::game::net::GameEvent;
    for ev in events {
        match ev {
            GameEvent::PlayerHit { x, y, z, r, g, b } => {
                spawn_player_hit(particles, rng, Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
            }
            GameEvent::PlayerDied { x, y, z, r, g, b } => {
                spawn_death_explosion(particles, rng, Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
            }
            GameEvent::TerrainHit { x, y, z, r, g, b } => {
                spawn_terrain_hit(particles, rng, Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
            }
            GameEvent::Explosion { x, y, z, r, g, b, radius } => {
                spawn_explosion(particles, rng, Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255), *radius);
            }
            GameEvent::BulletFired { .. } => {}
        }
    }
}

pub fn spawn_explosion(particles: &mut Vec<Particle>, rng: &mut Rng, pos: Vector3, color: Color, radius: f32) {
    let count = ((radius * 10.0) as i32).clamp(4, 50);
    for _ in 0..count {
        let angle = rng.range(0.0, std::f32::consts::TAU);
        let speed = rng.range(3.0, 8.0) * radius.sqrt();
        let lifetime = rng.range(0.3, 0.6) * radius.sqrt();
        particles.push(Particle {
            position: pos,
            vel_x: angle.cos() * speed,
            vel_y: angle.sin() * speed + 2.0,
            lifetime,
            max_lifetime: lifetime,
            color,
            size: rng.range(0.04, 0.10) * radius.max(0.5),
        });
    }
}

pub fn spawn_death_explosion(particles: &mut Vec<Particle>, rng: &mut Rng, pos: Vector3, color: Color) {
    // Big burst of player-colored particles
    for _ in 0..40 {
        let angle = rng.range(0.0, std::f32::consts::TAU);
        let speed = rng.range(4.0, 14.0);
        let lifetime = rng.range(0.5, 1.2);
        particles.push(Particle {
            position: pos,
            vel_x: angle.cos() * speed,
            vel_y: angle.sin() * speed + 3.0,
            lifetime,
            max_lifetime: lifetime,
            color,
            size: rng.range(0.06, 0.16),
        });
    }
    // White flash particles
    for _ in 0..15 {
        let angle = rng.range(0.0, std::f32::consts::TAU);
        let speed = rng.range(2.0, 8.0);
        let lifetime = rng.range(0.2, 0.5);
        particles.push(Particle {
            position: pos,
            vel_x: angle.cos() * speed,
            vel_y: angle.sin().abs() * speed + 5.0,
            lifetime,
            max_lifetime: lifetime,
            color: Color::new(255, 255, 255, 255),
            size: rng.range(0.04, 0.10),
        });
    }
}
