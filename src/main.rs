mod game;
mod level;
mod physics;
mod player;
mod render;

use game::world::World;
use raylib::prelude::*;
use render::crt::CrtFilter;

const SCREEN_WIDTH: i32 = 960;
const SCREEN_HEIGHT: i32 = 540;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("1VI1")
        .vsync()
        .resizable()
        .build();

    rl.set_target_fps(60);

    let mut crt = CrtFilter::new(&mut rl, &thread, SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut render_w = SCREEN_WIDTH;
    let mut render_h = SCREEN_HEIGHT;
    let mut world = World::new();

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // F11 fullscreen toggle
        if rl.is_key_pressed(KeyboardKey::KEY_F11) {
            rl.toggle_fullscreen();
        }

        // Resize render texture if window size changed
        let w = rl.get_screen_width();
        let h = rl.get_screen_height();
        if w != render_w || h != render_h {
            render_w = w;
            render_h = h;
            crt = CrtFilter::new(&mut rl, &thread, render_w, render_h);
        }

        // Camera first (1-frame lag, needed for mouse aim projection)
        let camera = render::camera::game_camera(&world);
        world.update(&rl, &camera, dt);

        // Draw scene to render texture
        {
            let mut d = rl.begin_texture_mode(&thread, &mut crt.target);
            d.clear_background(Color::new(15, 15, 20, 255));
            {
                let mut d3 = d.begin_mode3D(camera);

                // Platforms
                for platform in &world.level.platforms {
                    let c = platform.aabb.center();
                    let s = platform.aabb.size();
                    d3.draw_cube(c, s.x, s.y, s.z, platform.color);
                    d3.draw_cube_wires(c, s.x, s.y, s.z, Color::new(80, 80, 90, 255));
                }

                // Players
                for player in &world.players {
                    let px = player.position.x;
                    let py = player.position.y;
                    let pz = player.position.z;

                    // Body + head spheres
                    let body_r = 0.38;
                    let head_r = 0.28;
                    let body_center = Vector3::new(px, py + 0.5, pz);
                    let head_center = Vector3::new(px, py + 1.15, pz);
                    d3.draw_sphere(body_center, body_r, player.color);
                    d3.draw_sphere(head_center, head_r, player.color);

                    // Eyes that track aim direction
                    let eye_r = 0.065;
                    let eye_spread = 0.12;
                    let ax = player.aim_dir.x;
                    let ay = player.aim_dir.y;
                    let perp_x = -ay;
                    let perp_y = ax;
                    let look_x = ax * 0.08;
                    let look_y = ay * 0.08;
                    let eye_z = head_center.z + head_r * 0.85;
                    let eye_base_y = head_center.y + 0.03;
                    let left_eye = Vector3::new(
                        head_center.x + look_x - perp_x * eye_spread,
                        eye_base_y + look_y - perp_y * eye_spread,
                        eye_z,
                    );
                    let right_eye = Vector3::new(
                        head_center.x + look_x + perp_x * eye_spread,
                        eye_base_y + look_y + perp_y * eye_spread,
                        eye_z,
                    );
                    d3.draw_sphere(left_eye, eye_r, Color::new(20, 20, 25, 255));
                    d3.draw_sphere(right_eye, eye_r, Color::new(20, 20, 25, 255));

                    // Aim arrow from head edge
                    let arrow_start = Vector3::new(
                        head_center.x + ax * (head_r + 0.05),
                        head_center.y + ay * (head_r + 0.05),
                        pz,
                    );
                    let aim_len = 1.2;
                    let shaft_end = Vector3::new(
                        arrow_start.x + ax * (aim_len - 0.3),
                        arrow_start.y + ay * (aim_len - 0.3),
                        pz,
                    );
                    let tip = Vector3::new(
                        arrow_start.x + ax * aim_len,
                        arrow_start.y + ay * aim_len,
                        pz,
                    );
                    d3.draw_cylinder_ex(arrow_start, shaft_end, 0.03, 0.03, 6, player.color);
                    d3.draw_cylinder_ex(shaft_end, tip, 0.1, 0.0, 6, player.color);
                }
            }
        }

        // Draw render texture with CRT shader
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            {
                let mut s = d.begin_shader_mode(&mut crt.shader);
                s.draw_texture_rec(
                    crt.target.texture(),
                    Rectangle::new(
                        0.0,
                        0.0,
                        crt.target.texture().width as f32,
                        -(crt.target.texture().height as f32),
                    ),
                    Vector2::new(0.0, 0.0),
                    Color::WHITE,
                );
            }
            d.draw_fps(10, 10);
        }
    }
}
