use raylib::prelude::*;

use crate::combat::particles::Rng;

const MENU_GRAVITY: f32 = 120.0;

pub struct MenuParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
}

pub struct MenuParticles {
    pub particles: Vec<MenuParticle>,
    pub rng: Rng,
    ambient_timer: f32,
}

impl MenuParticles {
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        Self {
            particles: Vec::new(),
            rng: Rng::new(seed),
            ambient_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, screen_w: i32, screen_h: i32, accent: Color) {
        // Ambient: occasional sparks from edges
        self.ambient_timer -= dt;
        if self.ambient_timer <= 0.0 {
            self.ambient_timer = self.rng.range(0.08, 0.25);
            let count = (self.rng.next_f32() * 2.0) as i32 + 1; // 1-3 per burst
            for _ in 0..count {
                let side = (self.rng.next_f32() * 4.0) as i32;
                let (x, y, vx, vy) = match side {
                    0 => (0.0, self.rng.range(0.0, screen_h as f32), self.rng.range(20.0, 60.0), self.rng.range(-30.0, 30.0)),
                    1 => (screen_w as f32, self.rng.range(0.0, screen_h as f32), self.rng.range(-60.0, -20.0), self.rng.range(-30.0, 30.0)),
                    2 => (self.rng.range(0.0, screen_w as f32), 0.0, self.rng.range(-30.0, 30.0), self.rng.range(20.0, 50.0)),
                    _ => (self.rng.range(0.0, screen_w as f32), screen_h as f32, self.rng.range(-30.0, 30.0), self.rng.range(-50.0, -20.0)),
                };
                let life = self.rng.range(1.5, 3.0);
                self.particles.push(MenuParticle {
                    x, y, vx, vy,
                    life,
                    max_life: life,
                    size: self.rng.range(2.0, 5.0),
                    color: Color::new(accent.r, accent.g, accent.b, 120),
                });
            }
        }

        for p in self.particles.iter_mut() {
            p.vy += MENU_GRAVITY * 0.15 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    /// Explosion burst at a screen position
    pub fn explode(&mut self, x: f32, y: f32, color: Color) {
        for _ in 0..30 {
            let angle = self.rng.range(0.0, std::f32::consts::TAU);
            let speed = self.rng.range(80.0, 300.0);
            let life = self.rng.range(0.4, 1.0);
            self.particles.push(MenuParticle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life,
                max_life: life,
                size: self.rng.range(2.0, 7.0),
                color,
            });
        }
        // White flash
        for _ in 0..10 {
            let angle = self.rng.range(0.0, std::f32::consts::TAU);
            let speed = self.rng.range(40.0, 180.0);
            let life = self.rng.range(0.15, 0.4);
            self.particles.push(MenuParticle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life,
                max_life: life,
                size: self.rng.range(3.0, 8.0),
                color: Color::new(255, 255, 255, 200),
            });
        }
    }

    /// Small pop burst (for selection changes)
    pub fn pop(&mut self, x: f32, y: f32, color: Color) {
        for _ in 0..8 {
            let angle = self.rng.range(0.0, std::f32::consts::TAU);
            let speed = self.rng.range(30.0, 100.0);
            let life = self.rng.range(0.2, 0.5);
            self.particles.push(MenuParticle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life,
                max_life: life,
                size: self.rng.range(2.0, 4.0),
                color,
            });
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        for p in &self.particles {
            let fade = (p.life / p.max_life).clamp(0.0, 1.0);
            let alpha = (p.color.a as f32 * fade) as u8;
            let c = Color::new(p.color.r, p.color.g, p.color.b, alpha);
            let s = p.size * (0.5 + fade * 0.5);
            d.draw_rectangle(
                (p.x - s / 2.0) as i32,
                (p.y - s / 2.0) as i32,
                s as i32,
                s as i32,
                c,
            );
        }
    }
}
