use raylib::prelude::*;

use crate::game::state::GameState;
use crate::game::world::World;
use crate::menu::theme::Theme;
use crate::player::player::HIT_FLASH_DURATION;
use crate::render::cards::{self, CardPickAnim};
use crate::render::crt::CrtFilter;
use crate::render::hud;

pub fn draw_world(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    crt: &mut CrtFilter,
    world: &World,
    camera: Camera3D,
    render_w: i32,
    render_h: i32,
    theme: &Theme,
    time: f32,
    card_anim: &CardPickAnim,
    local_player: u8,
) {
    // Draw environment to render texture (CRT + aberration)
    {
        let mut d = rl.begin_texture_mode(thread, &mut crt.env_target);
        d.clear_background(theme.game_bg);

        // Scrolling asterisk grid behind everything
        {
            let spacing = 48.0_f32;
            let scroll = (time * 12.0) % spacing;
            let color = Color::new(
                theme.bg_grid_color.r,
                theme.bg_grid_color.g,
                theme.bg_grid_color.b,
                theme.bg_grid_alpha,
            );
            let font_size = 14;
            let mut gx = -spacing + scroll;
            while gx < render_w as f32 + spacing {
                let mut gy = -spacing + scroll;
                while gy < render_h as f32 + spacing {
                    d.draw_text("*", gx as i32, gy as i32, font_size, color);
                    gy += spacing;
                }
                gx += spacing;
            }
        }

        {
            let mut d3 = d.begin_mode3D(camera);

            for platform in &world.level.platforms {
                let c = platform.aabb.center();
                let s = platform.aabb.size();
                let base = if platform.is_wall { theme.game_wall_color } else { theme.game_platform_color };
                let wire = theme.game_wire_color;

                // Outer frame (the border)
                d3.draw_cube(c, s.x, s.y, s.z, wire);

                // Inner inset face (lighter core)
                let inset = 0.12_f32;
                let inner = Color::new(
                    base.r.saturating_add(20),
                    base.g.saturating_add(20),
                    base.b.saturating_add(20),
                    255,
                );
                let ix = (s.x - inset * 2.0).max(0.05);
                let iy = (s.y - inset * 2.0).max(0.05);
                let iz = (s.z - inset * 2.0).max(0.05);
                d3.draw_cube(c, ix, iy, iz, inner);

                // Outer wireframe for crispness
                d3.draw_cube_wires(c, s.x, s.y, s.z, Color::new(
                    wire.r.saturating_add(40),
                    wire.g.saturating_add(40),
                    wire.b.saturating_add(40),
                    255,
                ));
            }
        }
    }

    // Draw players to render texture (CRT scanlines, no aberration)
    {
        let mut d = rl.begin_texture_mode(thread, &mut crt.player_target);
        d.clear_background(Color::new(0, 0, 0, 0));
        {
            let mut d3 = d.begin_mode3D(camera);

            // Depth blockers
            for platform in &world.level.platforms {
                let c = platform.aabb.center();
                let s = platform.aabb.size();
                d3.draw_cube(c, s.x, s.y, s.z, Color::new(0, 0, 0, 0));
            }

            for player in &world.players {
                if !player.alive {
                    continue;
                }
                let px = player.position.x;
                let py = player.position.y;
                let pz = player.position.z;

                let render_color = if player.hit_flash_timer > 0.0 {
                    let t = (player.hit_flash_timer / HIT_FLASH_DURATION).min(1.0);
                    Color::new(
                        (player.color.r as f32 + (255.0 - player.color.r as f32) * t) as u8,
                        (player.color.g as f32 + (255.0 - player.color.g as f32) * t) as u8,
                        (player.color.b as f32 + (255.0 - player.color.b as f32) * t) as u8,
                        255,
                    )
                } else {
                    player.color
                };

                let body_r = 0.38;
                let head_r = 0.28;
                let body_center = Vector3::new(px, py + 0.5, pz);
                let head_center = Vector3::new(px, py + 1.15, pz);
                d3.draw_sphere(body_center, body_r, render_color);
                d3.draw_sphere(head_center, head_r, render_color);

                // Eyes
                let eye_r = 0.065;
                let eye_spread = 0.12;
                let ax = player.aim_dir.x;
                let ay = player.aim_dir.y;

                let cam_pos = camera.position;
                let fwd_xz_x = cam_pos.x - head_center.x;
                let fwd_xz_z = cam_pos.z - head_center.z;
                let fwd_xz_len = (fwd_xz_x * fwd_xz_x + fwd_xz_z * fwd_xz_z).sqrt();
                let (fwd_x, fwd_z) = if fwd_xz_len > 0.001 {
                    (fwd_xz_x / fwd_xz_len, fwd_xz_z / fwd_xz_len)
                } else {
                    (0.0, 1.0)
                };

                let right_x = fwd_z;
                let right_z = -fwd_x;

                let surf_r = head_r * 0.92;
                let base_x = head_center.x + surf_r * fwd_x;
                let base_y = head_center.y + 0.03;
                let base_z = head_center.z + surf_r * fwd_z;

                let look_shift = 0.08;
                let eye_cx = base_x + ax * look_shift * right_x;
                let eye_cy = base_y + ay * look_shift;
                let eye_cz = base_z + ax * look_shift * right_z;

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

                // Aim arrow
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
                let arrow_color =
                    Color::new(player.color.r, player.color.g, player.color.b, 160);
                d3.draw_cylinder_ex(arrow_start, shaft_end, 0.03, 0.03, 6, arrow_color);
                d3.draw_cylinder_ex(shaft_end, tip, 0.1, 0.0, 6, arrow_color);
            }

            // Bullets + tracers
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
                d3.draw_cylinder_ex(
                    trail_pos,
                    bullet.position,
                    0.02,
                    0.02,
                    4,
                    bullet.color,
                );
            }

            // Particles
            for p in &world.particles {
                let fade = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
                let c = Color::new(
                    (p.color.r as f32 * fade) as u8,
                    (p.color.g as f32 * fade) as u8,
                    (p.color.b as f32 * fade) as u8,
                    255,
                );
                d3.draw_sphere(p.position, p.size, c);
            }
        }
    }

    // Composite to screen
    {
        let mut d = rl.begin_drawing(thread);
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

        // Players with CRT scanlines only
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

        // HUD
        hud::draw_hud(&mut d, world, camera, render_w, render_h, local_player);

        // Card pick overlay
        if matches!(world.state, GameState::CardPick { .. }) {
            cards::draw_card_pick(&mut d, world, card_anim, render_w, render_h);
        }

        // Match over overlay
        if matches!(world.state, GameState::MatchOver { .. }) {
            cards::draw_match_over(&mut d, world, render_w, render_h);
        }

        d.draw_fps(10, 10);
    }
}

