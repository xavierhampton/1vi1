use crate::combat::bullet::{Bullet, BULLET_DAMAGE, BULLET_GRAVITY};
use crate::game::net::GameEvent;
use crate::level::platforms::Platform;
use crate::player::player::{Player, HIT_FLASH_DURATION};

pub fn update_bullets(
    bullets: &mut Vec<Bullet>,
    players: &mut [Player],
    platforms: &[Platform],
    dt: f32,
) -> Vec<GameEvent> {
    let mut events = Vec::new();

    for bullet in bullets.iter_mut() {
        bullet.prev_position = bullet.position;

        bullet.velocity.y -= BULLET_GRAVITY * dt;
        bullet.position.x += bullet.velocity.x * dt;
        bullet.position.y += bullet.velocity.y * dt;
        bullet.lifetime -= dt;

        if bullet.lifetime <= 0.0 {
            continue;
        }

        let baabb = bullet.aabb();

        // Platform collision
        for platform in platforms {
            if baabb.overlaps(&platform.aabb) {
                events.push(GameEvent::TerrainHit {
                    x: bullet.prev_position.x,
                    y: bullet.prev_position.y,
                    z: bullet.prev_position.z,
                    r: bullet.color.r,
                    g: bullet.color.g,
                    b: bullet.color.b,
                });
                bullet.lifetime = 0.0;
                break;
            }
        }

        if bullet.lifetime <= 0.0 {
            continue;
        }

        // Player collision
        let damage_bonus = if bullet.owner < players.len() {
            players[bullet.owner].stats.bullet_damage_bonus
        } else { 0.0 };
        for (i, player) in players.iter_mut().enumerate() {
            if i == bullet.owner || !player.alive {
                continue;
            }
            if baabb.overlaps(&player.aabb()) {
                let hit_pos = player.render_center();
                events.push(GameEvent::PlayerHit {
                    x: hit_pos.x,
                    y: hit_pos.y,
                    z: hit_pos.z,
                    r: player.color.r,
                    g: player.color.g,
                    b: player.color.b,
                });
                player.hit_flash_timer = HIT_FLASH_DURATION;
                player.hp = (player.hp - (BULLET_DAMAGE + damage_bonus)).max(0.0);
                bullet.lifetime = 0.0;
                break;
            }
        }
    }

    bullets.retain(|b| b.lifetime > 0.0);
    events
}
