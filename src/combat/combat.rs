use raylib::prelude::Color;

use crate::combat::bullet::{Bullet, BULLET_GRAVITY};
use crate::game::net::GameEvent;
use crate::level::platforms::Platform;
use crate::player::player::{Player, HIT_FLASH_DURATION};

const EXPLOSIVE_RADIUS: f32 = 3.0;

pub fn update_bullets(
    bullets: &mut Vec<Bullet>,
    players: &mut [Player],
    platforms: &[Platform],
    dt: f32,
) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // Collect alive player positions for homing (immutable snapshot)
    let alive_targets: Vec<(usize, f32, f32)> = players.iter().enumerate()
        .filter(|(_, p)| p.alive)
        .map(|(i, p)| (i, p.position.x, p.position.y + p.size.y / 2.0))
        .collect();

    // Deferred effects (applied after bullet loop to avoid borrow conflicts)
    let mut damage_queue: Vec<(usize, f32, f32, f32)> = Vec::new(); // (idx, dmg, bullet_x, bullet_y)
    let mut heal_queue: Vec<(usize, f32)> = Vec::new();
    let mut explosion_queue: Vec<(f32, f32, usize, f32, Color)> = Vec::new();

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

        bullet.velocity.y -= BULLET_GRAVITY * dt;
        bullet.position.x += bullet.velocity.x * dt;
        bullet.position.y += bullet.velocity.y * dt;
        bullet.lifetime -= dt;

        if bullet.lifetime <= 0.0 { continue; }

        let baabb = bullet.aabb();

        // Platform collision
        for platform in platforms {
            if baabb.overlaps(&platform.aabb) {
                if bullet.bounces_remaining > 0 {
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
            if !player.alive { continue; }
            // Skip self-hit for a brief grace period after firing
            if i == bullet.owner && self_grace { continue; }

            if baabb.overlaps(&player.aabb()) {
                let hit_pos = player.render_center();
                events.push(GameEvent::PlayerHit {
                    x: hit_pos.x, y: hit_pos.y, z: hit_pos.z,
                    r: player.color.r, g: player.color.g, b: player.color.b,
                });

                damage_queue.push((i, bullet.damage, bullet.position.x, bullet.position.y));

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

                if !bullet.piercing {
                    bullet.lifetime = 0.0;
                }
                break;
            }
        }
    }

    // Apply deferred damage
    for (idx, dmg, bx, by) in &damage_queue {
        if *idx < players.len() && players[*idx].alive {
            players[*idx].hp = (players[*idx].hp - dmg).max(0.0);
            players[*idx].hit_flash_timer = HIT_FLASH_DURATION;
            // Bounceback: knockback away from bullet origin
            if players[*idx].stats.bounceback {
                let dx = players[*idx].position.x - *bx;
                let dy = (players[*idx].position.y + players[*idx].size.y / 2.0) - *by;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let knockback = 12.0;
                players[*idx].velocity.x += (dx / dist) * knockback;
                players[*idx].velocity.y += (dy / dist) * knockback;
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
        // Visual explosion (reuse death explosion particles)
        events.push(GameEvent::PlayerDied {
            x: *ex, y: *ey, z: 0.0,
            r: color.r, g: color.g, b: color.b,
        });
    }

    bullets.retain(|b| b.lifetime > 0.0);
    events
}
