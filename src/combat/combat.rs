use raylib::prelude::{Color, Vector3};

use crate::combat::bullet::{Bullet, BULLET_GRAVITY};
use crate::game::net::GameEvent;
use crate::level::platforms::Platform;
use crate::physics::collision::AABB;
use crate::player::player::{Player, HIT_FLASH_DURATION};

pub struct StickyBombData {
    pub position: Vector3,
    pub owner: usize,
    pub damage: f32,
    pub stuck_to: Option<usize>,
    pub color: Color,
}

/// Hit info returned per bullet-player collision for world-level processing
pub struct BulletHitInfo {
    pub target: usize,
    pub owner: usize,
    pub damage: f32,
    pub bullet_x: f32,
    pub bullet_y: f32,
    pub poison: bool,
    pub ice: bool,
    pub void_pull: bool,
}

pub fn update_bullets(
    bullets: &mut Vec<Bullet>,
    players: &mut [Player],
    platforms: &[Platform],
    dt: f32,
) -> (Vec<GameEvent>, Vec<StickyBombData>, Vec<BulletHitInfo>) {
    let mut events = Vec::new();

    // Collect alive player positions for homing (immutable snapshot)
    let alive_targets: Vec<(usize, f32, f32)> = players.iter().enumerate()
        .filter(|(_, p)| p.alive && p.ghost_timer <= 0.0)
        .map(|(i, p)| (i, p.position.x, p.position.y + p.size.y / 2.0))
        .collect();

    let mut damage_queue: Vec<BulletHitInfo> = Vec::new();
    let mut heal_queue: Vec<(usize, f32)> = Vec::new();
    let mut sticky_queue: Vec<StickyBombData> = Vec::new();

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

        bullet.velocity.y -= BULLET_GRAVITY * bullet.gravity_mult * dt;
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
            if swept.overlaps(&platform.aabb) {
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
            if i == bullet.owner && self_grace { continue; }

            if baabb.overlaps(&player.aabb()) {
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

                damage_queue.push(BulletHitInfo {
                    target: i,
                    owner: bullet.owner,
                    damage: bullet.damage,
                    bullet_x: bullet.position.x,
                    bullet_y: bullet.position.y,
                    poison: bullet.poison,
                    ice: bullet.ice,
                    void_pull: bullet.void_pull,
                });

                // Vampire: heal attacker
                if bullet.owner < players.len() {
                    let vamp = players[bullet.owner].stats.vampire_heal;
                    if vamp > 0.0 {
                        heal_queue.push((bullet.owner, vamp));
                    }
                }

                if !bullet.piercing {
                    bullet.lifetime = 0.0;
                }
                break;
            }
        }
    }

    // Apply deferred damage + on-hit effects
    for hit in &damage_queue {
        if hit.target < players.len() && players[hit.target].alive {
            let effective_dmg = hit.damage * players[hit.target].stats.damage_taken_mult;
            players[hit.target].hp = (players[hit.target].hp - effective_dmg).max(0.0);
            players[hit.target].hit_flash_timer = HIT_FLASH_DURATION;

            // Knockback (offensive: check OWNER's stats, push TARGET away from bullet)
            if hit.owner < players.len() && players[hit.owner].stats.knockback {
                let dx = players[hit.target].position.x - hit.bullet_x;
                let dy = (players[hit.target].position.y + players[hit.target].size.y / 2.0) - hit.bullet_y;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let knockback_force = 20.0;
                players[hit.target].velocity.x += (dx / dist) * knockback_force;
                players[hit.target].velocity.y += (dy / dist) * knockback_force;
            }

            // Poison
            if hit.poison {
                players[hit.target].poison_timer = 3.0;
            }

            // Ice: slow target
            if hit.ice {
                players[hit.target].slow_timer = 2.0;
            }

            // Void pull: suck target toward bullet impact point
            if hit.void_pull {
                let dx = hit.bullet_x - players[hit.target].position.x;
                let dy = hit.bullet_y - (players[hit.target].position.y + players[hit.target].size.y / 2.0);
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let pull_force = 12.0;
                players[hit.target].velocity.x += (dx / dist) * pull_force;
                players[hit.target].velocity.y += (dy / dist) * pull_force;
            }

            // Adrenaline: defender gets speed buff when hit
            if players[hit.target].stats.adrenaline {
                players[hit.target].adrenaline_timer = 3.0;
            }

            // Bloodthirsty: attacker gets speed+DMG buff when hitting
            if hit.owner < players.len() && players[hit.owner].stats.bloodthirsty {
                players[hit.owner].bloodthirsty_timer = 3.0;
            }

            // Upsize: attacker makes target bigger
            if hit.owner < players.len() && players[hit.owner].stats.upsize {
                players[hit.target].upsized_stacks += 1;
            }
        }
    }

    // Apply deferred heals
    for (idx, heal) in &heal_queue {
        if *idx < players.len() && players[*idx].alive {
            players[*idx].hp = (players[*idx].hp + heal).min(players[*idx].max_hp);
        }
    }

    bullets.retain(|b| b.lifetime > 0.0);
    (events, sticky_queue, damage_queue)
}
