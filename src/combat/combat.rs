use crate::combat::bullet::{Bullet, BULLET_DAMAGE, BULLET_GRAVITY};
use crate::level::platforms::Platform;
use crate::player::player::Player;

pub fn update_bullets(bullets: &mut Vec<Bullet>, players: &mut [Player], platforms: &[Platform], dt: f32) {
    for bullet in bullets.iter_mut() {
        bullet.prev_position = bullet.position;

        // Gravity
        bullet.velocity.y -= BULLET_GRAVITY * dt;

        // Move
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
                player.hp = (player.hp - BULLET_DAMAGE).max(0.0);
                bullet.lifetime = 0.0;
                break;
            }
        }
    }

    bullets.retain(|b| b.lifetime > 0.0);
}
