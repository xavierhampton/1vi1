use crate::combat::bullet::{Bullet, BULLET_DAMAGE, BULLET_GRAVITY};
use crate::combat::particles::{spawn_player_hit, spawn_terrain_hit, Particle, Rng};
use crate::level::platforms::Platform;
use crate::player::player::{Player, HIT_FLASH_DURATION};

pub fn update_bullets(
    bullets: &mut Vec<Bullet>,
    players: &mut [Player],
    platforms: &[Platform],
    particles: &mut Vec<Particle>,
    rng: &mut Rng,
    dt: f32,
) {
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
                spawn_terrain_hit(particles, rng, bullet.prev_position, bullet.color);
                bullet.lifetime = 0.0;
                break;
            }
        }

        if bullet.lifetime <= 0.0 {
            continue;
        }

        // Player collision
        for (i, player) in players.iter_mut().enumerate() {
            if i == bullet.owner {
                continue;
            }
            if baabb.overlaps(&player.aabb()) {
                let hit_pos = player.render_center();
                spawn_player_hit(particles, rng, hit_pos, player.color);
                player.hit_flash_timer = HIT_FLASH_DURATION;
                player.hp = (player.hp - BULLET_DAMAGE).max(0.0);
                bullet.lifetime = 0.0;
                break;
            }
        }
    }

    bullets.retain(|b| b.lifetime > 0.0);
}
