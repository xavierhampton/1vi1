use raylib::prelude::{Color, Vector2, Vector3};

use crate::combat::bullet::{Bullet, BULLET_GRAVITY};
use crate::game::net::GameEvent;
use crate::level::platforms::Platform;
use crate::physics::collision::AABB;
use crate::player::player::{Player, HIT_FLASH_DURATION};

const EXPLOSIVE_RADIUS: f32 = 3.0;

pub struct StickyBombData {
    pub position: Vector3,
    pub owner: usize,
    pub damage: f32,
    pub stuck_to: Option<usize>,
    pub color: Color,
}

pub fn update_bullets(
    bullets: &mut Vec<Bullet>,
    players: &mut [Player],
    platforms: &[Platform],
    dt: f32,
) -> (Vec<GameEvent>, Vec<StickyBombData>) {
    let mut events = Vec::new();

    // Collect alive player positions for homing (immutable snapshot)
    let alive_targets: Vec<(usize, f32, f32)> = players.iter().enumerate()
        .filter(|(_, p)| p.alive && p.ghost_timer <= 0.0)
        .map(|(i, p)| (i, p.position.x, p.position.y + p.size.y / 2.0))
        .collect();

    // Deferred effects (applied after bullet loop to avoid borrow conflicts)
    let mut damage_queue: Vec<(usize, f32, f32, f32, bool, usize)> = Vec::new(); // (idx, dmg, bx, by, poison, owner)
    let mut heal_queue: Vec<(usize, f32)> = Vec::new();
    let mut explosion_queue: Vec<(f32, f32, usize, f32, Color)> = Vec::new();
    let mut sticky_queue: Vec<StickyBombData> = Vec::new();
    let mut new_bullets: Vec<Bullet> = Vec::new();

    for bullet in bullets.iter_mut() {
        bullet.prev_position = bullet.position;

        // Homing: steer toward nearest enemy
        if bullet.homing {
            let mut closest_dist = f32::MAX;
            let mut closest_dir = None;
            for &(idx, px, py) in &alive_targets {
                if idx == bullet.owner { continue; }
                let dx = px - bullet.position.x;
                let dy = py - bullet.position.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < closest_dist && dist < 15.0 {
                    closest_dist = dist;
                    let len = dist.max(0.01);
                    closest_dir = Some((dx / len, dy / len));
                }
            }
            if let Some((tx, ty)) = closest_dir {
                let speed = (bullet.velocity.x.powi(2) + bullet.velocity.y.powi(2)).sqrt();
                let turn = 3.0 * dt;
                let cx = bullet.velocity.x / speed.max(0.01);
                let cy = bullet.velocity.y / speed.max(0.01);
                let nx = cx + (tx - cx) * turn;
                let ny = cy + (ty - cy) * turn;
                let len = (nx * nx + ny * ny).sqrt().max(0.01);
                bullet.velocity.x = nx / len * speed;
                bullet.velocity.y = ny / len * speed;
            }
        }

        if bullet.hot_potato {
            let accel = 1.0 + dt * 0.8;
            bullet.velocity.x *= accel;
            bullet.velocity.y *= accel;
            bullet.velocity.y -= BULLET_GRAVITY * bullet.gravity_mult * dt * 0.2;
        } else {
            bullet.velocity.y -= BULLET_GRAVITY * bullet.gravity_mult * dt;
        }
        bullet.position.x += bullet.velocity.x * dt;
        bullet.position.y += bullet.velocity.y * dt;
        bullet.lifetime -= dt;

        if bullet.lifetime <= 0.0 { continue; }

        let baabb = bullet.aabb();

        // Swept AABB covers full bullet path to prevent tunneling through thin platforms
        let swept = AABB {
            min: Vector3::new(
                bullet.prev_position.x.min(bullet.position.x) - bullet.radius,
                bullet.prev_position.y.min(bullet.position.y) - bullet.radius,
                bullet.position.z - bullet.radius,
            ),
            max: Vector3::new(
                bullet.prev_position.x.max(bullet.position.x) + bullet.radius,
                bullet.prev_position.y.max(bullet.position.y) + bullet.radius,
                bullet.position.z + bullet.radius,
            ),
        };

        // Platform collision
        for platform in platforms {
            if !bullet.phantom && swept.overlaps(&platform.aabb) {
                if bullet.sticky {
                    sticky_queue.push(StickyBombData {
                        position: bullet.prev_position,
                        owner: bullet.owner,
                        damage: bullet.damage,
                        stuck_to: None,
                        color: bullet.color,
                    });
                    bullet.lifetime = 0.0;
                } else if bullet.bounces_remaining > 0 {
                    // Rubber: bounce off wall
                    bullet.bounces_remaining -= 1;
                    let pen_left = baabb.max.x - platform.aabb.min.x;
                    let pen_right = platform.aabb.max.x - baabb.min.x;
                    let pen_bottom = baabb.max.y - platform.aabb.min.y;
                    let pen_top = platform.aabb.max.y - baabb.min.y;
                    if pen_left.min(pen_right) < pen_bottom.min(pen_top) {
                        bullet.velocity.x = -bullet.velocity.x;
                        bullet.position.x = bullet.prev_position.x;
                    } else {
                        bullet.velocity.y = -bullet.velocity.y;
                        bullet.position.y = bullet.prev_position.y;
                    }
                    events.push(GameEvent::TerrainHit {
                        x: bullet.position.x, y: bullet.position.y, z: bullet.position.z,
                        r: bullet.color.r, g: bullet.color.g, b: bullet.color.b,
                    });
                } else {
                    events.push(GameEvent::TerrainHit {
                        x: bullet.prev_position.x, y: bullet.prev_position.y, z: bullet.prev_position.z,
                        r: bullet.color.r, g: bullet.color.g, b: bullet.color.b,
                    });
                    if bullet.explosive {
                        explosion_queue.push((
                            bullet.position.x, bullet.position.y,
                            bullet.owner, bullet.damage * 0.5, bullet.color,
                        ));
                    }
                    bullet.lifetime = 0.0;
                }
                break;
            }
        }

        if bullet.lifetime <= 0.0 { continue; }

        // Player collision (read-only iteration, defer mutations)
        let self_grace = bullet.lifetime > (crate::combat::bullet::BULLET_LIFETIME - 0.15);
        for (i, player) in players.iter().enumerate() {
            if !player.alive || player.ghost_timer > 0.0 { continue; }
            // Skip self-hit for a brief grace period after firing
            if i == bullet.owner && self_grace { continue; }

            if baabb.overlaps(&player.aabb()) {
                // Sticky: attach bomb to player instead of dealing damage
                if bullet.sticky {
                    sticky_queue.push(StickyBombData {
                        position: player.render_center(),
                        owner: bullet.owner,
                        damage: bullet.damage,
                        stuck_to: Some(i),
                        color: bullet.color,
                    });
                    bullet.lifetime = 0.0;
                    break;
                }

                let hit_pos = player.render_center();
                events.push(GameEvent::PlayerHit {
                    x: hit_pos.x, y: hit_pos.y, z: hit_pos.z,
                    r: player.color.r, g: player.color.g, b: player.color.b,
                });

                damage_queue.push((i, bullet.damage, bullet.position.x, bullet.position.y, bullet.poison, bullet.owner));

                // Vampire: heal attacker
                if bullet.owner < players.len() {
                    let vamp = players[bullet.owner].stats.vampire_heal;
                    if vamp > 0.0 {
                        heal_queue.push((bullet.owner, vamp));
                    }
                }

                // Explosive: AoE on player hit
                if bullet.explosive {
                    explosion_queue.push((
                        bullet.position.x, bullet.position.y,
                        bullet.owner, bullet.damage * 0.5, bullet.color,
                    ));
                }

                // Split shot: spawn 2 bullets at ±30°
                if bullet.split_on_hit {
                    let speed = (bullet.velocity.x.powi(2) + bullet.velocity.y.powi(2)).sqrt();
                    if speed > 0.01 {
                        let dx = bullet.velocity.x / speed;
                        let dy = bullet.velocity.y / speed;
                        let angle = std::f32::consts::PI / 6.0;
                        for &sign in &[-1.0_f32, 1.0] {
                            let a = sign * angle;
                            let nx = dx * a.cos() - dy * a.sin();
                            let ny = dx * a.sin() + dy * a.cos();
                            let mut split = bullet.clone();
                            split.position = bullet.position;
                            split.prev_position = bullet.position;
                            split.velocity = Vector2::new(nx * speed * 0.7, ny * speed * 0.7);
                            split.lifetime = bullet.lifetime.min(1.5);
                            split.damage *= 0.5;
                            split.split_on_hit = false;
                            new_bullets.push(split);
                        }
                    }
                }

                if !bullet.piercing {
                    bullet.lifetime = 0.0;
                }
                break;
            }
        }
    }

    // Apply deferred damage
    for (idx, dmg, bx, by, poison, owner) in &damage_queue {
        if *idx < players.len() && players[*idx].alive {
            let effective_dmg = dmg * players[*idx].stats.damage_taken_mult;
            players[*idx].hp = (players[*idx].hp - effective_dmg).max(0.0);
            players[*idx].hit_flash_timer = HIT_FLASH_DURATION;
            if players[*idx].stats.bounceback && !players[*idx].stats.knockback_immune {
                let dx = players[*idx].position.x - *bx;
                let dy = (players[*idx].position.y + players[*idx].size.y / 2.0) - *by;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let knockback = 20.0;
                players[*idx].velocity.x += (dx / dist) * knockback;
                players[*idx].velocity.y += (dy / dist) * knockback;
            }
            if *poison {
                players[*idx].poison_timer = 3.0;
            }
            if players[*idx].stats.adrenaline {
                players[*idx].adrenaline_timer = 2.0;
            }
            if *owner < players.len() && players[*owner].stats.upsizer {
                players[*idx].upsized_stacks += 1;
            }
        }
    }

    // Apply deferred heals
    for (idx, heal) in &heal_queue {
        if *idx < players.len() && players[*idx].alive {
            players[*idx].hp = (players[*idx].hp + heal).min(players[*idx].max_hp);
        }
    }

    // Apply explosive AoE
    for (ex, ey, owner, dmg, color) in &explosion_queue {
        for (i, player) in players.iter_mut().enumerate() {
            if i == *owner || !player.alive { continue; }
            let dx = player.position.x - *ex;
            let dy = (player.position.y + player.size.y / 2.0) - *ey;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < EXPLOSIVE_RADIUS {
                let falloff = 1.0 - (dist / EXPLOSIVE_RADIUS);
                player.hp = (player.hp - *dmg * falloff).max(0.0);
                player.hit_flash_timer = HIT_FLASH_DURATION;
                let hit_pos = player.render_center();
                events.push(GameEvent::PlayerHit {
                    x: hit_pos.x, y: hit_pos.y, z: hit_pos.z,
                    r: player.color.r, g: player.color.g, b: player.color.b,
                });
            }
        }
        events.push(GameEvent::Explosion {
            x: *ex, y: *ey, z: 0.0,
            r: color.r, g: color.g, b: color.b,
        });
    }

    bullets.extend(new_bullets);
    bullets.retain(|b| b.lifetime > 0.0);
    (events, sticky_queue)
}
