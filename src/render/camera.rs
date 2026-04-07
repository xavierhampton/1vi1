use raylib::prelude::*;

use crate::game::world::World;

pub fn game_camera(world: &World) -> Camera3D {
    // Only track alive players so dead corpses don't pull the camera off-center
    // (which skews aim raycasts and makes controls feel inverted).
    // Fall back to all players if none are alive (e.g. draw / round reset).
    let alive: Vec<&crate::player::player::Player> = world.players.iter().filter(|p| p.alive).collect();
    let tracked: &[&crate::player::player::Player] = if alive.is_empty() {
        &world.players.iter().collect::<Vec<_>>()
    } else {
        &alive
    };
    let count = tracked.len() as f32;
    let mut mid_x = 0.0;
    let mut mid_y = 0.0;
    for p in tracked {
        mid_x += p.position.x;
        mid_y += p.position.y;
    }
    mid_x /= count;
    mid_y = mid_y / count + 2.0;

    let mut spread = 0.0_f32;
    for p in tracked {
        let dx = (p.position.x - mid_x).abs();
        let dy = (p.position.y - (mid_y - 2.0)).abs();
        spread = spread.max(dx).max(dy);
    }
    // More players need a wider default view (use total player count for consistent zoom)
    let total = world.players.len() as f32;
    let base_zoom = 12.0 + (total - 2.0) * 2.0;
    let cam_z = (spread * 0.8 + base_zoom).clamp(15.0, 35.0);

    Camera3D::perspective(
        Vector3::new(mid_x, mid_y, cam_z),
        Vector3::new(mid_x, mid_y, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        60.0,
    )
}
