use raylib::prelude::*;

use crate::game::world::World;

pub fn game_camera(world: &World) -> Camera3D {
    let p1 = &world.players[0].position;
    let p2 = &world.players[1].position;

    let mid_x = (p1.x + p2.x) / 2.0;
    let mid_y = (p1.y + p2.y) / 2.0 + 2.0;

    let dx = (p1.x - p2.x).abs();
    let dy = (p1.y - p2.y).abs();
    let spread = dx.max(dy);
    let cam_z = (spread * 0.8 + 12.0).clamp(15.0, 30.0);

    Camera3D::perspective(
        Vector3::new(mid_x, mid_y, cam_z),
        Vector3::new(mid_x, mid_y, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        50.0,
    )
}
