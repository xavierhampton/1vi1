use raylib::prelude::*;

use crate::lobby::state::LOBBY_COLORS;
use crate::menu::customize::ACCESSORY_COUNT;
use crate::render::crt::CrtFilter;

const DEMO_W: i32 = 1920;
const DEMO_H: i32 = 1080;
const ACCESSORY_SWAP_INTERVAL: f32 = 1.0;

struct DemoPlayer {
    x: f32,
    y: f32,
    color: Color,
    aim_x: f32,
    aim_y: f32,
    accessories: Vec<(u8, u8, u8, u8)>,
}

const ACCENT_COLORS: [(u8, u8, u8); 10] = [
    (255, 255, 255),
    (190, 190, 200),
    (255, 210, 50),
    (255, 70, 70),
    (70, 160, 255),
    (90, 220, 110),
    (255, 140, 40),
    (170, 80, 255),
    (255, 100, 170),
    (30, 30, 35),
];

fn xorshift(state: &mut u64) -> u64 {
    let mut s = *state;
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    *state = s;
    s
}

fn random_accessories(rng: &mut u64) -> Vec<(u8, u8, u8, u8)> {
    let mut acc = Vec::new();
    let count = (xorshift(rng) % 3) as usize + 1; // 1-3 accessories
    let mut used = [false; ACCESSORY_COUNT];
    for _ in 0..count {
        let mut id;
        loop {
            id = (xorshift(rng) % ACCESSORY_COUNT as u64) as u8;
            if !used[id as usize] { break; }
        }
        used[id as usize] = true;
        let ci = (xorshift(rng) % ACCENT_COLORS.len() as u64) as usize;
        let (r, g, b) = ACCENT_COLORS[ci];
        acc.push((id, r, g, b));
    }
    acc
}

pub fn run_demo() {
    let (mut rl, thread) = raylib::init()
        .size(DEMO_W, DEMO_H)
        .title("1VI1 Demo")
        .log_level(raylib::callbacks::TraceLogLevel::LOG_WARNING)
        .build();
    rl.set_target_fps(60);

    let mut crt = CrtFilter::new(&mut rl, &thread, DEMO_W, DEMO_H);

    let mut rng: u64 = 0xDEADBEEF;

    // 4 players evenly spaced on a flat platform
    let spacing = 3.0;
    let start_x = -spacing * 1.5;
    let floor_y = 0.0;
    let mut players: Vec<DemoPlayer> = (0..4).map(|i| {
        DemoPlayer {
            x: start_x + i as f32 * spacing,
            y: floor_y,
            color: LOBBY_COLORS[i].0,
            aim_x: 0.0,
            aim_y: 1.0,
            accessories: random_accessories(&mut rng),
        }
    }).collect();

    let mut time: f32 = 0.0;
    let mut prev_swap: u32 = 0;

    let themes = crate::menu::theme::all_themes();
    let theme = &themes[0];

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time += dt;

        // Swap accessories every interval
        let swap_idx = (time / ACCESSORY_SWAP_INTERVAL) as u32;
        if swap_idx != prev_swap {
            for p in &mut players {
                p.accessories = random_accessories(&mut rng);
            }
            prev_swap = swap_idx;
        }

        // Aim animation: sweep side to side with a bounce at each edge
        // Cycle: sweep right (1s) → bounce at right (0.5s) → sweep left (1s) → bounce at left (0.5s)
        let cycle = 3.0_f32;
        let phase = time % cycle;
        let (ax, ay) = if phase < 1.0 {
            // Sweep from left to right
            let t = phase;
            let sweep = -1.0 + t * 2.0; // -1 to 1
            let angle = std::f32::consts::FRAC_PI_2 + sweep * 1.2;
            (angle.cos(), angle.sin())
        } else if phase < 1.5 {
            // Bounce up/down at right edge
            let t = (phase - 1.0) / 0.5;
            let bounce_y = (t * std::f32::consts::TAU).sin() * 0.4;
            let angle = std::f32::consts::FRAC_PI_2 + 1.2;
            (angle.cos(), angle.sin() + bounce_y)
        } else if phase < 2.5 {
            // Sweep from right to left
            let t = phase - 1.5;
            let sweep = 1.0 - t * 2.0; // 1 to -1
            let angle = std::f32::consts::FRAC_PI_2 + sweep * 1.2;
            (angle.cos(), angle.sin())
        } else {
            // Bounce up/down at left edge
            let t = (phase - 2.5) / 0.5;
            let bounce_y = (t * std::f32::consts::TAU).sin() * 0.4;
            let angle = std::f32::consts::FRAC_PI_2 - 1.2;
            (angle.cos(), angle.sin() + bounce_y)
        };
        // Normalize
        let len = (ax * ax + ay * ay).sqrt();
        let (ax, ay) = (ax / len, ay / len);
        for p in &mut players {
            p.aim_x = ax;
            p.aim_y = ay;
        }

        // Camera
        let mid_x = players.iter().map(|p| p.x).sum::<f32>() / 4.0;
        let mid_y = floor_y + 2.0;
        let cam_z = 16.0;
        let camera = Camera3D::perspective(
            Vector3::new(mid_x, mid_y, cam_z),
            Vector3::new(mid_x, mid_y, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            60.0,
        );

        // ── ENV pass ──
        {
            let mut d = rl.begin_texture_mode(&thread, &mut crt.env_target);
            d.clear_background(theme.game_bg);

            // Asterisk grid
            {
                let spacing_g = 48.0_f32;
                let scroll = (time * 12.0) % spacing_g;
                let color = Color::new(
                    theme.bg_grid_color.r,
                    theme.bg_grid_color.g,
                    theme.bg_grid_color.b,
                    theme.bg_grid_alpha,
                );
                let mut gx = -spacing_g + scroll;
                while gx < DEMO_W as f32 + spacing_g {
                    let mut gy = -spacing_g + scroll;
                    while gy < DEMO_H as f32 + spacing_g {
                        d.draw_text("*", gx as i32, gy as i32, 14, color);
                        gy += spacing_g;
                    }
                    gx += spacing_g;
                }
            }

            {
                let mut d3 = d.begin_mode3D(camera);
                // Floor platform
                let fc = Vector3::new(0.0, -0.5, 0.0);
                let fs = Vector3::new(20.0, 1.0, 4.0);
                let wire = theme.game_wire_color;
                d3.draw_cube(fc, fs.x, fs.y, fs.z, wire);
                let inset = 0.12;
                let inner = Color::new(
                    theme.game_platform_color.r.saturating_add(20),
                    theme.game_platform_color.g.saturating_add(20),
                    theme.game_platform_color.b.saturating_add(20),
                    255,
                );
                d3.draw_cube(fc, fs.x - inset * 2.0, fs.y - inset * 2.0, fs.z - inset * 2.0, inner);
                d3.draw_cube_wires(fc, fs.x, fs.y, fs.z, Color::new(
                    wire.r.saturating_add(40),
                    wire.g.saturating_add(40),
                    wire.b.saturating_add(40),
                    255,
                ));
            }
        }

        // ── PLAYER pass ──
        {
            let mut d = rl.begin_texture_mode(&thread, &mut crt.player_target);
            d.clear_background(Color::new(0, 0, 0, 0));
            {
                let mut d3 = d.begin_mode3D(camera);

                // Depth blocker for floor
                d3.draw_cube(Vector3::new(0.0, -0.5, 0.0), 20.0, 1.0, 4.0, Color::new(0, 0, 0, 0));

                for p in &players {
                    let px = p.x;
                    let py = p.y;
                    let pz = 0.0_f32;
                    let size_scale = 1.0_f32;
                    let body_r = 0.38;
                    let head_r = 0.28;
                    let body_center = Vector3::new(px, py + 0.5, pz);
                    let head_center = Vector3::new(px, py + 1.15, pz);
                    d3.draw_sphere(body_center, body_r, p.color);
                    d3.draw_sphere(head_center, head_r, p.color);

                    // Eyes
                    let eye_r = 0.065;
                    let eye_spread = 0.12;
                    let fwd_xz_x = camera.position.x - head_center.x;
                    let fwd_xz_z = camera.position.z - head_center.z;
                    let fwd_xz_len = (fwd_xz_x * fwd_xz_x + fwd_xz_z * fwd_xz_z).sqrt();
                    let (fwd_x, fwd_z) = if fwd_xz_len > 0.001 {
                        (fwd_xz_x / fwd_xz_len, fwd_xz_z / fwd_xz_len)
                    } else { (0.0, 1.0) };
                    let right_x = fwd_z;
                    let right_z = -fwd_x;

                    let surf_r = head_r * 0.92;
                    let base_x = head_center.x + surf_r * fwd_x;
                    let base_y = head_center.y + 0.03;
                    let base_z = head_center.z + surf_r * fwd_z;

                    let look_shift = 0.08;
                    let eye_cx = base_x + p.aim_x * look_shift * right_x;
                    let eye_cy = base_y + p.aim_y * look_shift;
                    let eye_cz = base_z + p.aim_x * look_shift * right_z;

                    d3.draw_sphere(
                        Vector3::new(eye_cx - right_x * eye_spread, eye_cy, eye_cz - right_z * eye_spread),
                        eye_r, Color::new(20, 20, 25, 255),
                    );
                    d3.draw_sphere(
                        Vector3::new(eye_cx + right_x * eye_spread, eye_cy, eye_cz + right_z * eye_spread),
                        eye_r, Color::new(20, 20, 25, 255),
                    );

                    // Aim arrow
                    let arrow_start = Vector3::new(
                        head_center.x + p.aim_x * (head_r + 0.05),
                        head_center.y + p.aim_y * (head_r + 0.05),
                        pz,
                    );
                    let aim_len = 1.2;
                    let shaft_end = Vector3::new(
                        arrow_start.x + p.aim_x * (aim_len - 0.3),
                        arrow_start.y + p.aim_y * (aim_len - 0.3),
                        pz,
                    );
                    let tip = Vector3::new(
                        arrow_start.x + p.aim_x * aim_len,
                        arrow_start.y + p.aim_y * aim_len,
                        pz,
                    );
                    let arrow_color = Color::new(p.color.r, p.color.g, p.color.b, 160);
                    d3.draw_cylinder_ex(arrow_start, shaft_end, 0.03, 0.03, 6, arrow_color);
                    d3.draw_cylinder_ex(shaft_end, tip, 0.1, 0.0, 6, arrow_color);

                    // Accessories
                    for &(id, r, g, b) in &p.accessories {
                        let ac = Color::new(r, g, b, 255);
                        crate::render::game::draw_accessory_3d(
                            &mut d3, id, ac,
                            head_center, body_center, head_r, body_r, size_scale,
                            fwd_x, fwd_z, right_x, right_z,
                        );
                    }
                }
            }
        }

        // ── Composite ──
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            let tex_rect = |t: &RenderTexture2D| -> Rectangle {
                Rectangle::new(0.0, 0.0, t.texture().width as f32, -(t.texture().height as f32))
            };

            {
                let mut s = d.begin_shader_mode(&mut crt.shader);
                s.draw_texture_rec(
                    crt.env_target.texture(), tex_rect(&crt.env_target),
                    Vector2::new(0.0, 0.0), Color::WHITE,
                );
            }
            {
                let mut s = d.begin_shader_mode(&mut crt.shader_no_aberration);
                s.draw_texture_rec(
                    crt.player_target.texture(), tex_rect(&crt.player_target),
                    Vector2::new(0.0, 0.0), Color::WHITE,
                );
            }

            let title = "1VI1";
            let title_size = 60;
            let tw = d.measure_text(title, title_size);
            d.draw_text(title, DEMO_W / 2 - tw / 2, 20, title_size, theme.item_hover_color);
        }
    }
}
