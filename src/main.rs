mod combat;
mod game;
mod level;
mod physics;
mod player;
mod render;

use game::world::{World, MAX_BULLETS, RELOAD_TIME};
use raylib::prelude::*;
use render::crt::CrtFilter;

const SCREEN_WIDTH: i32 = 960;
const SCREEN_HEIGHT: i32 = 540;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("1VI1")
        .resizable()
        .build();

    rl.set_target_fps(144);

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

        // Draw environment to render texture (CRT + aberration)
        {
            let mut d = rl.begin_texture_mode(&thread, &mut crt.env_target);
            d.clear_background(Color::new(15, 15, 20, 255));
            {
                let mut d3 = d.begin_mode3D(camera);

                for platform in &world.level.platforms {
                    let c = platform.aabb.center();
                    let s = platform.aabb.size();
                    d3.draw_cube(c, s.x, s.y, s.z, platform.color);
                    d3.draw_cube_wires(c, s.x, s.y, s.z, Color::new(80, 80, 90, 255));
                }
            }
        }

        // Draw players to render texture (CRT scanlines, no aberration)
        {
            let mut d = rl.begin_texture_mode(&thread, &mut crt.player_target);
            d.clear_background(Color::new(0, 0, 0, 0));
            {
                let mut d3 = d.begin_mode3D(camera);

                // Draw platforms as invisible depth blockers so they occlude players
                for platform in &world.level.platforms {
                    let c = platform.aabb.center();
                    let s = platform.aabb.size();
                    d3.draw_cube(c, s.x, s.y, s.z, Color::new(0, 0, 0, 0));
                }

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

                    // Eyes that track aim direction (billboarded to camera plane)
                    let eye_r = 0.065;
                    let eye_spread = 0.12;
                    let ax = player.aim_dir.x;
                    let ay = player.aim_dir.y;

                    // Forward = head to camera in XZ (for base placement on sphere front)
                    let cam_pos = camera.position;
                    let fwd_xz_x = cam_pos.x - head_center.x;
                    let fwd_xz_z = cam_pos.z - head_center.z;
                    let fwd_xz_len = (fwd_xz_x * fwd_xz_x + fwd_xz_z * fwd_xz_z).sqrt();
                    let (fwd_x, fwd_z) = if fwd_xz_len > 0.001 {
                        (fwd_xz_x / fwd_xz_len, fwd_xz_z / fwd_xz_len)
                    } else {
                        (0.0, 1.0)
                    };

                    // Right = perpendicular to forward in XZ plane
                    let right_x = fwd_z;
                    let right_z = -fwd_x;

                    // Base eye position: front of sphere facing camera (fixed depth)
                    let surf_r = head_r * 0.92;
                    let base_x = head_center.x + surf_r * fwd_x;
                    let base_y = head_center.y + 0.03;
                    let base_z = head_center.z + surf_r * fwd_z;

                    // Aim shifts eyes in screen plane only (right + Y), no depth change
                    let look_shift = 0.08;
                    let eye_cx = base_x + ax * look_shift * right_x;
                    let eye_cy = base_y + ay * look_shift;
                    let eye_cz = base_z + ax * look_shift * right_z;

                    // Spread along camera-right (no depth change)
                    let left_eye = Vector3::new(
                        eye_cx - right_x * eye_spread,
                        eye_cy,
                        eye_cz - right_z * eye_spread,
                    );
                    let right_eye = Vector3::new(
                        eye_cx + right_x * eye_spread,
                        eye_cy,
                        eye_cz + right_z * eye_spread,
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
                    let arrow_color = Color::new(player.color.r, player.color.g, player.color.b, 160);
                    d3.draw_cylinder_ex(arrow_start, shaft_end, 0.03, 0.03, 6, arrow_color);
                    d3.draw_cylinder_ex(shaft_end, tip, 0.1, 0.0, 6, arrow_color);
                }

                // Draw bullets + tracers
                for bullet in &world.bullets {
                    let vlen = (bullet.velocity.x.powi(2) + bullet.velocity.y.powi(2)).sqrt();
                    let trail_pos = if vlen > 0.001 {
                        let t = 0.8;
                        Vector3::new(
                            bullet.position.x - bullet.velocity.x / vlen * t,
                            bullet.position.y - bullet.velocity.y / vlen * t,
                            bullet.position.z,
                        )
                    } else {
                        bullet.position
                    };
                    d3.draw_cylinder_ex(trail_pos, bullet.position, 0.02, 0.02, 4, bullet.color);
                }
            }
        }

        // Composite to screen
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            // Environment with CRT + aberration
            {
                let mut s = d.begin_shader_mode(&mut crt.shader);
                s.draw_texture_rec(
                    crt.env_target.texture(),
                    Rectangle::new(
                        0.0,
                        0.0,
                        crt.env_target.texture().width as f32,
                        -(crt.env_target.texture().height as f32),
                    ),
                    Vector2::new(0.0, 0.0),
                    Color::WHITE,
                );
            }

            // Players with CRT scanlines only (no aberration)
            {
                let mut s = d.begin_shader_mode(&mut crt.shader_no_aberration);
                s.draw_texture_rec(
                    crt.player_target.texture(),
                    Rectangle::new(
                        0.0,
                        0.0,
                        crt.player_target.texture().width as f32,
                        -(crt.player_target.texture().height as f32),
                    ),
                    Vector2::new(0.0, 0.0),
                    Color::WHITE,
                );
            }

            // HUD: player names + HP bars
            for player in &world.players {
                let above_head = Vector3::new(
                    player.position.x,
                    player.position.y + player.size.y + 0.15,
                    player.position.z,
                );
                let screen_pos = d.get_world_to_screen(above_head, camera);
                let sx = screen_pos.x as i32;
                let sy = screen_pos.y as i32;

                let font_size = 20;
                let text_w = d.measure_text(&player.name, font_size);
                d.draw_text(&player.name, sx - text_w / 2, sy - font_size, font_size, Color::WHITE);

                let bar_w = 56;
                let bar_h = 5;
                let bar_x = sx - bar_w / 2;
                let bar_y = sy + 2;
                let hp_ratio = (player.hp / player.max_hp).clamp(0.0, 1.0);
                let fill_w = (bar_w as f32 * hp_ratio) as i32;
                d.draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(40, 40, 40, 200));
                d.draw_rectangle(bar_x, bar_y, fill_w, bar_h, player.color);

                // Bullet pips
                let pip_size = 7;
                let pip_gap = 4;
                let total_pip_w = MAX_BULLETS * (pip_size + pip_gap) - pip_gap;
                let pip_x = sx - total_pip_w / 2;
                let pip_y = bar_y + bar_h + 5;
                for i in 0..MAX_BULLETS {
                    let px = pip_x + i * (pip_size + pip_gap);
                    let pip_color = if i < player.bullets_remaining {
                        player.color
                    } else {
                        Color::new(40, 40, 40, 200)
                    };
                    d.draw_rectangle(px, pip_y, pip_size, pip_size, pip_color);
                }

                // Reload bar: overlays the pip row when reloading
                if player.reload_timer > 0.0 {
                    let reload_ratio = 1.0 - (player.reload_timer / RELOAD_TIME);
                    let reload_fill = (total_pip_w as f32 * reload_ratio) as i32;
                    d.draw_rectangle(pip_x, pip_y, total_pip_w, pip_size, Color::new(40, 40, 40, 200));
                    d.draw_rectangle(pip_x, pip_y, reload_fill, pip_size, Color::new(200, 200, 200, 220));
                }
            }

            d.draw_fps(10, 10);
        }
    }
}
