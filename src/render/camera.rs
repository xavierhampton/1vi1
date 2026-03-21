use raylib::prelude::*;

use crate::game::world::World;

pub fn game_camera(world: &World) -> Camera3D {
    let count = world.players.len() as f32;
    let mut mid_x = 0.0;
    let mut mid_y = 0.0;
    for p in &world.players {
        mid_x += p.position.x;
        mid_y += p.position.y;
    }
    mid_x /= count;
    mid_y = mid_y / count + 2.0;

    let mut spread = 0.0_f32;
    for p in &world.players {
        let dx = (p.position.x - mid_x).abs();
        let dy = (p.position.y - (mid_y - 2.0)).abs();
        spread = spread.max(dx).max(dy);
    }
    // More players need a wider default view
    let base_zoom = 12.0 + (count - 2.0) * 2.0;
    let cam_z = (spread * 0.8 + base_zoom).clamp(15.0, 35.0);

    Camera3D::perspective(
        Vector3::new(mid_x, mid_y, cam_z),
        Vector3::new(mid_x, mid_y, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        50.0,
    )
}
