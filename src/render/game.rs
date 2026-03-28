use raylib::prelude::*;

use crate::game::cards::{CardDef, CardId, CARD_CATALOG};
use crate::game::state::GameState;
use crate::game::world::World;
use crate::menu::theme::Theme;
use crate::player::player::HIT_FLASH_DURATION;
use crate::render::cards::{self as render_cards, CardPickAnim};
use crate::render::crt::CrtFilter;
use crate::render::hud;

pub struct MatchOverButtons<'a> {
    pub selected: usize,
    pub waiting: bool,
    pub theme: &'a Theme,
    pub time: f32,
}

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
    dev_overlay: bool,
    match_over_btns: Option<&MatchOverButtons>,
) {
    if render_w < 100 || render_h < 100 { return; }
    // Compute screen shake offset from local player
    let (shake_x, shake_y) = if let Some(local_p) = world.players.get(local_player as usize) {
        if local_p.shake_timer > 0.0 {
            let intensity = local_p.shake_timer.min(0.4) * 4.0;
            (
                (time * 47.0).sin() * intensity,
                (time * 61.0).cos() * intensity,
            )
        } else { (0.0, 0.0) }
    } else { (0.0, 0.0) };

    // Apply shake to camera
    let mut camera = camera;
    camera.position.x += shake_x * 0.05;
    camera.position.y += shake_y * 0.05;
    camera.target.x += shake_x * 0.05;
    camera.target.y += shake_y * 0.05;

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

                d3.draw_cube(c, s.x, s.y, s.z, wire);

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

                d3.draw_cube_wires(c, s.x, s.y, s.z, Color::new(
                    wire.r.saturating_add(40),
                    wire.g.saturating_add(40),
                    wire.b.saturating_add(40),
                    255,
                ));
            }

            // Bounce pads — glowing wall block
            for pad in &world.level.bounce_pads {
                let c = pad.aabb.center();
                let s = pad.aabb.size();
                let glow_pulse = (time * 4.0).sin() * 0.2 + 0.8;
                let ga = (glow_pulse * 255.0) as u8;
                let pad_color = Color::new(0, (180.0 * glow_pulse) as u8, (255.0 * glow_pulse) as u8, ga);
                let bright = Color::new(60, (230.0 * glow_pulse) as u8, 255, ga);

                // Outer cube
                d3.draw_cube(c, s.x, s.y, s.z, pad_color);
                // Brighter inner cube
                let inset = 0.08;
                let ix = (s.x - inset * 2.0).max(0.05);
                let iy = (s.y - inset * 2.0).max(0.05);
                let iz = (s.z - inset * 2.0).max(0.05);
                d3.draw_cube(c, ix, iy, iz, bright);
                // Wireframe
                d3.draw_cube_wires(c, s.x, s.y, s.z, Color::new(150, 255, 255, ga));
            }

            // Lava pools — animated glowing hazard
            for (pi, pool) in world.level.lava_pools.iter().enumerate() {
                let c = pool.aabb.center();
                let s = pool.aabb.size();
                let phase_offset = pi as f32 * 1.7; // stagger per pool
                let pulse = (time * 3.0 + phase_offset).sin() * 0.15 + 0.85;
                let ripple = (time * 5.0 + phase_offset).sin() * 0.1 + 0.9;

                // Dark base layer
                let base_color = Color::new(
                    (160.0 * pulse) as u8,
                    (30.0 * pulse) as u8,
                    5,
                    230,
                );
                d3.draw_cube(c, s.x, s.y, s.z, base_color);

                // Hot inner core — shifts between orange and yellow
                let hot_g = (80.0 + 60.0 * (time * 2.0 + phase_offset).sin()) * ripple;
                let hot = Color::new(255, hot_g as u8, 15, (240.0 * pulse) as u8);
                let inset = 0.06;
                let ix = (s.x - inset * 2.0).max(0.05);
                let iy = (s.y - inset * 2.0).max(0.05);
                let iz = (s.z - inset * 2.0).max(0.05);
                d3.draw_cube(c, ix, iy, iz, hot);

                // Bright surface stripe (simulates flowing surface)
                let stripe_y = c.y + s.y * 0.25;
                let stripe_center = Vector3::new(c.x, stripe_y, c.z);
                let stripe_a = ((time * 4.0 + phase_offset).sin() * 0.5 + 0.5) * 180.0;
                d3.draw_cube(stripe_center, s.x * 0.9, s.y * 0.15, s.z * 0.8,
                    Color::new(255, 200, 40, stripe_a as u8));

                // Wireframe border — bright warning outline
                d3.draw_cube_wires(c, s.x, s.y, s.z,
                    Color::new(255, 80, 20, (220.0 * pulse) as u8));
            }

            // Laser beams — glowing cylinder between two emitter cubes, toggled on/off
            for laser in &world.level.lasers {
                let cycle = laser.on_time + laser.off_time;
                let phase = if cycle > 0.0 { world.elapsed_time % cycle } else { 0.0 };
                let is_on = phase < laser.on_time;

                let emitter_size = 0.25;
                let emitter_color = if is_on {
                    Color::new(255, 40, 40, 255)
                } else {
                    Color::new(100, 30, 30, 255)
                };

                // Emitter cubes
                d3.draw_cube(laser.start, emitter_size, emitter_size, emitter_size, emitter_color);
                d3.draw_cube_wires(laser.start, emitter_size, emitter_size, emitter_size, Color::new(255, 100, 100, 200));
                d3.draw_cube(laser.end, emitter_size, emitter_size, emitter_size, emitter_color);
                d3.draw_cube_wires(laser.end, emitter_size, emitter_size, emitter_size, Color::new(255, 100, 100, 200));

                if is_on {
                    let flicker = (time * 20.0).sin() * 0.1 + 0.9;
                    let beam_color = Color::new(255, (40.0 * flicker) as u8, (40.0 * flicker) as u8, (220.0 * flicker) as u8);
                    // Main beam
                    d3.draw_cylinder_ex(laser.start, laser.end, 0.08, 0.08, 6, beam_color);
                    // Bright core
                    d3.draw_cylinder_ex(laser.start, laser.end, 0.03, 0.03, 6, Color::new(255, 200, 200, (255.0 * flicker) as u8));
                }

                // Warning glow when about to turn on (last 0.5s of off phase)
                if !is_on && cycle > 0.0 {
                    let time_until_on = cycle - phase;
                    if time_until_on < 0.5 {
                        let warn = (world.elapsed_time * 12.0).sin().abs() * 0.4;
                        let warn_a = (warn * 150.0) as u8;
                        d3.draw_cylinder_ex(laser.start, laser.end, 0.04, 0.04, 6, Color::new(255, 60, 60, warn_a));
                    }
                }
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

            for (pi, player) in world.players.iter().enumerate() {
                if !player.alive {
                    continue;
                }
                // Ghost: invisible to other players, semi-transparent to self
                if player.ghost_timer > 0.0 && pi as u8 != local_player {
                    continue;
                }

                let px = player.position.x;
                let py = player.position.y;
                let pz = player.position.z;

                let base_color = if player.hit_flash_timer > 0.0 {
                    let t = (player.hit_flash_timer / HIT_FLASH_DURATION).min(1.0);
                    Color::new(
                        (player.color.r as f32 + (255.0 - player.color.r as f32) * t) as u8,
                        (player.color.g as f32 + (255.0 - player.color.g as f32) * t) as u8,
                        (player.color.b as f32 + (255.0 - player.color.b as f32) * t) as u8,
                        255,
                    )
                } else if player.poison_timer > 0.0 {
                    Color::new(
                        player.color.r / 2,
                        ((player.color.g as u16 + 200) / 2).min(255) as u8,
                        player.color.b / 2,
                        255,
                    )
                } else {
                    player.color
                };

                let render_color = if player.ghost_timer > 0.0 {
                    Color::new(base_color.r / 3, base_color.g / 3, base_color.b / 3, 255)
                } else if player.overclock_timer > 0.0 {
                    // Overclock boost: golden tint
                    Color::new(
                        (base_color.r as u16 + 200).min(255) as u8 / 2 + base_color.r / 2,
                        (base_color.g as u16 + 180).min(255) as u8 / 2 + base_color.g / 2,
                        base_color.b / 2,
                        255,
                    )
                } else if player.overclock_crash_timer > 0.0 {
                    // Overclock crash: darkened
                    Color::new(base_color.r / 3, base_color.g / 3, base_color.b / 3, 255)
                } else if player.adrenaline_timer > 0.0 {
                    // Adrenaline: red tint
                    Color::new(
                        ((base_color.r as u16 + 255) / 2).min(255) as u8,
                        base_color.g / 2,
                        base_color.b / 2,
                        255,
                    )
                } else if player.bloodthirsty_timer > 0.0 {
                    // Bloodthirsty: deep red glow
                    Color::new(
                        ((base_color.r as u16 + 200) / 2).min(255) as u8,
                        base_color.g / 3,
                        base_color.b / 3,
                        255,
                    )
                } else if player.slow_timer > 0.0 {
                    // Ice: blue tint
                    Color::new(
                        base_color.r / 2,
                        base_color.g / 2,
                        ((base_color.b as u16 + 255) / 2).min(255) as u8,
                        255,
                    )
                } else {
                    base_color
                };

                // Spawn invulnerability: gentle pulse between normal and slightly washed out
                let render_color = if player.invuln_timer > 0.0 {
                    let pulse = (player.invuln_timer * 3.5).sin() * 0.5 + 0.5;
                    let white_mix = 0.05 + pulse * 0.1; // 0.05..0.15 — very subtle
                    Color::new(
                        (render_color.r as f32 + (255.0 - render_color.r as f32) * white_mix) as u8,
                        (render_color.g as f32 + (255.0 - render_color.g as f32) * white_mix) as u8,
                        (render_color.b as f32 + (255.0 - render_color.b as f32) * white_mix) as u8,
                        (230.0 - pulse * 20.0) as u8, // 210..230 alpha
                    )
                } else {
                    render_color
                };

                let size_scale = player.size.x / 0.6;
                let body_r = 0.38 * size_scale;
                let head_r = 0.28 * size_scale;
                let body_center = Vector3::new(px, py + 0.5 * size_scale, pz);
                let head_center = Vector3::new(px, py + 1.15 * size_scale, pz);
                d3.draw_sphere(body_center, body_r, render_color);
                d3.draw_sphere(head_center, head_r, render_color);

                let eye_r = 0.065 * size_scale;
                let eye_spread = 0.12 * size_scale;
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

                let look_shift = 0.08 * size_scale;
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

                // Accessories
                for &(id, r, g, b) in &player.accessories {
                    let ac = Color::new(r, g, b, 255);
                    draw_accessory_3d(&mut d3, id, ac, head_center, body_center, head_r, body_r, size_scale, fwd_x, fwd_z, right_x, right_z);
                }
            }

            // DoppelGanger ghosts (semi-transparent delayed copies)
            for player in &world.players {
                if !player.alive || !player.stats.doppelganger { continue; }
                let (gx, gy, gax, gay) = player.doppel_ghost;
                if gx == 0.0 && gy == 0.0 { continue; }
                let gz = player.position.z;
                let ghost_color = Color::new(
                    player.color.r / 2 + 60,
                    player.color.g / 2 + 60,
                    player.color.b / 2 + 60,
                    120,
                );
                let size_scale = player.size.x / 0.6;
                let body_r = 0.38 * size_scale;
                let head_r = 0.28 * size_scale;
                let g_body = Vector3::new(gx, gy + 0.5 * size_scale, gz);
                let g_head = Vector3::new(gx, gy + 1.15 * size_scale, gz);
                d3.draw_sphere(g_body, body_r, ghost_color);
                d3.draw_sphere(g_head, head_r, ghost_color);

                // Ghost aim arrow
                let g_arrow_start = Vector3::new(
                    g_head.x + gax * (head_r + 0.05),
                    g_head.y + gay * (head_r + 0.05),
                    gz,
                );
                let g_tip = Vector3::new(
                    g_arrow_start.x + gax * 0.9,
                    g_arrow_start.y + gay * 0.9,
                    gz,
                );
                d3.draw_cylinder_ex(g_arrow_start, g_tip, 0.03, 0.0, 6, ghost_color);
            }

            // Bullets + tracers (radius scales with big bullets)
            for bullet in &world.bullets {
                let vlen = (bullet.velocity.x.powi(2) + bullet.velocity.y.powi(2)).sqrt();
                let trail_len = 0.8;
                let trail_pos = if vlen > 0.001 {
                    Vector3::new(
                        bullet.position.x - bullet.velocity.x / vlen * trail_len,
                        bullet.position.y - bullet.velocity.y / vlen * trail_len,
                        bullet.position.z,
                    )
                } else {
                    bullet.position
                };
                let tracer_r = (bullet.radius / 0.08) * 0.02; // scale tracer with bullet size
                d3.draw_cylinder_ex(
                    trail_pos,
                    bullet.position,
                    tracer_r,
                    tracer_r,
                    4,
                    bullet.color,
                );
                // Big bullets: draw a sphere at the tip
                if bullet.radius > 0.1 {
                    d3.draw_sphere(bullet.position, bullet.radius * 0.5, bullet.color);
                }
            }

            // Healing zones
            for zone in &world.healing_zones {
                let fade = (zone.lifetime / 5.0).min(1.0);
                let pulse = (time * 3.0).sin() * 0.15 + 0.85;
                let r = 3.0 * pulse;
                let alpha = (fade * 100.0) as u8;
                let center = zone.position;

                // Inner glow sphere
                d3.draw_sphere(center, r * 0.3, Color::new(100, 255, 200, alpha));
                d3.draw_sphere(center, r * 0.15, Color::new(100, 255, 200, alpha / 2));

                // Thick green ring outline on the ground
                let ring_alpha = (fade * 220.0) as u8;
                let ring_color = Color::new(50, 230, 150, ring_alpha);
                let ring_r = r * 0.95;
                let segments = 32;
                let thickness = 0.12;
                for seg in 0..segments {
                    let a0 = (seg as f32 / segments as f32) * std::f32::consts::TAU;
                    let a1 = ((seg + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                    let p0 = Vector3::new(
                        center.x + a0.cos() * ring_r,
                        center.y - 0.3,
                        center.z + a0.sin() * ring_r,
                    );
                    let p1 = Vector3::new(
                        center.x + a1.cos() * ring_r,
                        center.y - 0.3,
                        center.z + a1.sin() * ring_r,
                    );
                    d3.draw_cylinder_ex(p0, p1, thickness, thickness, 4, ring_color);
                }

                // Second smaller ring for depth
                let inner_ring_r = r * 0.5;
                let inner_alpha = (fade * 140.0) as u8;
                let inner_color = Color::new(100, 255, 200, inner_alpha);
                for seg in 0..segments {
                    let a0 = (seg as f32 / segments as f32) * std::f32::consts::TAU;
                    let a1 = ((seg + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                    let p0 = Vector3::new(
                        center.x + a0.cos() * inner_ring_r,
                        center.y - 0.3,
                        center.z + a0.sin() * inner_ring_r,
                    );
                    let p1 = Vector3::new(
                        center.x + a1.cos() * inner_ring_r,
                        center.y - 0.3,
                        center.z + a1.sin() * inner_ring_r,
                    );
                    d3.draw_cylinder_ex(p0, p1, thickness * 0.7, thickness * 0.7, 4, inner_color);
                }

                // Cross marker
                let cross_r = 0.4;
                let cross_alpha = (fade * 230.0) as u8;
                d3.draw_cylinder_ex(
                    Vector3::new(center.x - cross_r, center.y, center.z),
                    Vector3::new(center.x + cross_r, center.y, center.z),
                    0.1, 0.1, 4, Color::new(100, 255, 200, cross_alpha),
                );
                d3.draw_cylinder_ex(
                    Vector3::new(center.x, center.y - cross_r, center.z),
                    Vector3::new(center.x, center.y + cross_r, center.z),
                    0.1, 0.1, 4, Color::new(100, 255, 200, cross_alpha),
                );
            }

            // Sticky bombs
            for bomb in &world.sticky_bombs {
                let urgency = (1.0 - bomb.fuse / 2.0).max(0.0);
                let flash = if urgency > 0.5 { ((time * 10.0 * urgency).sin() > 0.0) as u8 } else { 0 };
                let c = if flash == 1 {
                    Color::new(255, 255, 255, 255)
                } else {
                    Color::new(
                        bomb.color.r.saturating_add(80),
                        bomb.color.g.min(100),
                        bomb.color.b.min(60),
                        255,
                    )
                };
                d3.draw_sphere(bomb.position, 0.15, c);
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

        let tex_rect = |t: &RenderTexture2D| -> Rectangle {
            Rectangle::new(0.0, 0.0, t.texture().width as f32, -(t.texture().height as f32))
        };

        // Environment pass (CRT + aberration)
        {
            let mut s = d.begin_shader_mode(&mut crt.shader);
            s.draw_texture_rec(
                crt.env_target.texture(), tex_rect(&crt.env_target),
                Vector2::new(0.0, 0.0), Color::WHITE,
            );
        }

        // Player pass (CRT scanlines, no aberration)
        {
            let mut s = d.begin_shader_mode(&mut crt.shader_no_aberration);
            s.draw_texture_rec(
                crt.player_target.texture(), tex_rect(&crt.player_target),
                Vector2::new(0.0, 0.0), Color::WHITE,
            );
        }

        // Pre-compute screen positions while we still have the main draw handle
        let hud_pre = hud::precompute_hud(&d, world, camera, render_w, render_h);
        let fps_text = format!("{} FPS", d.get_fps());

        // All UI into ui_target → CRT scanlines, no vignette
        {
            let mut t = d.begin_texture_mode(thread, &mut crt.ui_target);
            t.clear_background(Color::new(0, 0, 0, 0));

            hud::draw_hud(&mut *t, world, &hud_pre, render_w, render_h, local_player);

            if matches!(world.state, GameState::CardPick { .. }) {
                render_cards::draw_card_pick(&mut *t, world, card_anim, render_w, render_h);
            }

            if matches!(world.state, GameState::MatchOver { .. }) {
                render_cards::draw_match_over(&mut *t, world, render_w, render_h);
                if let Some(btns) = match_over_btns {
                    render_cards::draw_match_over_buttons(&mut *t, render_w, render_h, btns.selected, btns.waiting, btns.theme, btns.time);
                }
            }

            t.draw_text(&fps_text, 10, 10, 20, theme.item_color);
        }
        {
            let mut s = d.begin_shader_mode(&mut crt.shader_ui);
            s.draw_texture_rec(
                crt.ui_target.texture(), tex_rect(&crt.ui_target),
                Vector2::new(0.0, 0.0), Color::WHITE,
            );
        }

        // Dev overlay: NO CRT (user requested)
        if dev_overlay {
            let held: Vec<CardId> = world.players.get(local_player as usize)
                .map(|p| p.cards.iter().map(|(id, _)| *id).collect())
                .unwrap_or_default();
            draw_dev_overlay(&mut d, render_w, render_h, &held);
        }
    }
}

// ── Dev mode overlay ─────────────────────────────────────────────────────────

const DEV_COLS: i32 = 7;
const DEV_CARD_W: i32 = 120;
const DEV_CARD_H: i32 = 50;
const DEV_GAP: i32 = 6;

fn dev_card_rect(idx: usize, screen_w: i32, screen_h: i32) -> (i32, i32) {
    let col = idx as i32 % DEV_COLS;
    let row = idx as i32 / DEV_COLS;
    let total_w = DEV_COLS * (DEV_CARD_W + DEV_GAP) - DEV_GAP;
    let start_x = (screen_w - total_w) / 2;
    let start_y = screen_h / 2 - 120;
    (start_x + col * (DEV_CARD_W + DEV_GAP), start_y + row * (DEV_CARD_H + DEV_GAP))
}

pub fn draw_dev_overlay(d: &mut RaylibDrawHandle, screen_w: i32, screen_h: i32, held_cards: &[CardId]) {
    d.draw_rectangle(0, 0, screen_w, screen_h, Color::new(0, 0, 0, 180));

    let title = "DEV: Click to toggle cards (TAB to close)";
    let title_size = 20;
    let tw = d.measure_text(title, title_size);
    d.draw_text(title, screen_w / 2 - tw / 2, screen_h / 2 - 160, title_size, Color::new(255, 80, 80, 220));

    let mouse = d.get_mouse_position();

    let implemented: Vec<&CardDef> = CARD_CATALOG.iter().filter(|c| c.is_implemented()).collect();
    for (i, card_def) in implemented.iter().enumerate() {
        let (cx, cy) = dev_card_rect(i, screen_w, screen_h);
        let (cr, cg, cb) = card_def.color;
        let active = held_cards.contains(&card_def.id);

        let hovered = mouse.x >= cx as f32 && mouse.x <= (cx + DEV_CARD_W) as f32
            && mouse.y >= cy as f32 && mouse.y <= (cy + DEV_CARD_H) as f32;

        // Active cards get a brighter, colored background
        let bg = if active {
            Color::new(cr / 3 + 30, cg / 3 + 30, cb / 3 + 30, if hovered { 230 } else { 200 })
        } else {
            Color::new(cr / 6 + 15, cg / 6 + 15, cb / 6 + 15, if hovered { 200 } else { 100 })
        };
        d.draw_rectangle(cx, cy, DEV_CARD_W, DEV_CARD_H, bg);

        let border_w = if active { 2.0 } else if hovered { 2.0 } else { 1.0 };
        let border_color = if active {
            Color::new(cr, cg, cb, 255)
        } else if hovered {
            Color::new(cr, cg, cb, 200)
        } else {
            Color::new(cr / 2 + 40, cg / 2 + 40, cb / 2 + 40, 120)
        };
        d.draw_rectangle_lines_ex(
            Rectangle::new(cx as f32, cy as f32, DEV_CARD_W as f32, DEV_CARD_H as f32),
            border_w,
            border_color,
        );

        let tag = if card_def.is_ability() { "A" } else { "P" };
        let tag_color = Color::new(cr / 2 + 80, cg / 2 + 80, cb / 2 + 80, 200);
        d.draw_text(tag, cx + 4, cy + 4, 12, tag_color);

        // Active checkmark
        if active {
            let check_color = Color::new(100, 255, 100, 255);
            d.draw_text("ON", cx + DEV_CARD_W - 24, cy + 4, 12, check_color);
        }

        let name_alpha = if active { 255 } else { 160 };
        let name_size = 14;
        let nw = d.measure_text(card_def.name, name_size);
        d.draw_text(card_def.name, cx + DEV_CARD_W / 2 - nw / 2, cy + 8, name_size,
            Color::new(cr, cg, cb, name_alpha));

        let desc_size = 10;
        let dw = d.measure_text(card_def.description, desc_size);
        d.draw_text(card_def.description, cx + DEV_CARD_W / 2 - dw / 2, cy + 28, desc_size,
            Color::new(180, 180, 180, if active { 220 } else { 140 }));
    }
}

pub fn dev_overlay_click(mouse: Vector2, screen_w: i32, screen_h: i32) -> Option<CardId> {
    let implemented: Vec<&CardDef> = CARD_CATALOG.iter().filter(|c| c.is_implemented()).collect();
    for (i, card_def) in implemented.iter().enumerate() {
        let (cx, cy) = dev_card_rect(i, screen_w, screen_h);
        if mouse.x >= cx as f32 && mouse.x <= (cx + DEV_CARD_W) as f32
            && mouse.y >= cy as f32 && mouse.y <= (cy + DEV_CARD_H) as f32
        {
            return Some(card_def.id);
        }
    }
    None
}

// ── 3D accessory rendering ──────────────────────────────────────────────────

pub fn draw_accessory_3d(
    d3: &mut RaylibMode3D<'_, RaylibTextureMode<'_, RaylibHandle>>,
    id: u8, color: Color,
    head: Vector3, body: Vector3,
    head_r: f32, body_r: f32, _ss: f32,
    fwd_x: f32, fwd_z: f32, right_x: f32, right_z: f32,
) {
    match id {
        0 => { // Top Hat
            let brim_bot = Vector3::new(head.x, head.y + head_r * 0.55, head.z);
            let brim_top = Vector3::new(head.x, head.y + head_r * 0.65, head.z);
            d3.draw_cylinder_ex(brim_bot, brim_top, head_r * 1.1, head_r * 1.1, 12, color);
            let hat_bot = brim_top;
            let hat_top = Vector3::new(head.x, head.y + head_r * 1.6, head.z);
            d3.draw_cylinder_ex(hat_bot, hat_top, head_r * 0.7, head_r * 0.65, 12, color);
        }
        1 => { // Crown
            let base_y = head.y + head_r * 0.5;
            let crown_r = head_r * 0.85;
            let crown_h = head_r * 0.5;
            d3.draw_cylinder_ex(
                Vector3::new(head.x, base_y, head.z),
                Vector3::new(head.x, base_y + crown_h * 0.3, head.z),
                crown_r, crown_r, 12, color,
            );
            for i in 0..5 {
                let angle = i as f32 * std::f32::consts::TAU / 5.0;
                let px = head.x + angle.cos() * crown_r * 0.8;
                let pz = head.z + angle.sin() * crown_r * 0.8;
                let bot = Vector3::new(px, base_y + crown_h * 0.2, pz);
                let top = Vector3::new(px, base_y + crown_h, pz);
                d3.draw_cylinder_ex(bot, top, head_r * 0.12, 0.0, 4, color);
            }
        }
        2 => { // Halo
            let halo_y = head.y + head_r * 1.15;
            let halo_r = head_r * 0.9;
            for i in 0..16 {
                let angle = i as f32 * std::f32::consts::TAU / 16.0;
                let px = head.x + angle.cos() * halo_r;
                let pz = head.z + angle.sin() * halo_r;
                d3.draw_sphere(Vector3::new(px, halo_y, pz), head_r * 0.08, color);
            }
        }
        3 => { // Bandana (thick)
            let band_y = head.y + head_r * 0.15;
            d3.draw_cylinder_ex(
                Vector3::new(head.x, band_y - head_r * 0.12, head.z),
                Vector3::new(head.x, band_y + head_r * 0.12, head.z),
                head_r * 1.06, head_r * 1.06, 12, color,
            );
            let tail_start = Vector3::new(
                head.x - right_x * head_r, band_y, head.z - right_z * head_r,
            );
            let tail_end = Vector3::new(
                tail_start.x - right_x * head_r * 0.6,
                band_y - head_r * 0.5,
                tail_start.z - right_z * head_r * 0.6,
            );
            d3.draw_cylinder_ex(tail_start, tail_end, head_r * 0.12, head_r * 0.06, 4, color);
        }
        4 => { // Horns
            let base_y = head.y + head_r * 0.3;
            let spread = head_r * 0.7;
            let horn_h = head_r * 0.9;
            let lb = Vector3::new(head.x - right_x * spread, base_y, head.z - right_z * spread);
            let lt = Vector3::new(lb.x - right_x * head_r * 0.3, base_y + horn_h, lb.z - right_z * head_r * 0.3);
            d3.draw_cylinder_ex(lb, lt, head_r * 0.15, 0.0, 6, color);
            let rb = Vector3::new(head.x + right_x * spread, base_y, head.z + right_z * spread);
            let rt = Vector3::new(rb.x + right_x * head_r * 0.3, base_y + horn_h, rb.z + right_z * head_r * 0.3);
            d3.draw_cylinder_ex(rb, rt, head_r * 0.15, 0.0, 6, color);
        }
        5 => { // Wings
            let wing_y = body.y + body_r * 0.2;
            let wing_span = body_r * 1.2;
            let wing_h = body_r * 0.7;
            for i in 0..3 {
                let t = i as f32 / 2.0;
                let base = Vector3::new(body.x - right_x * body_r * 0.8, wing_y + t * wing_h * 0.3, body.z - right_z * body_r * 0.8);
                let tip = Vector3::new(base.x - right_x * wing_span * (1.0 - t * 0.3), wing_y + wing_h * (0.5 - t * 0.3), base.z - right_z * wing_span * (1.0 - t * 0.3));
                d3.draw_cylinder_ex(base, tip, body_r * 0.06, 0.0, 4, color);
            }
            for i in 0..3 {
                let t = i as f32 / 2.0;
                let base = Vector3::new(body.x + right_x * body_r * 0.8, wing_y + t * wing_h * 0.3, body.z + right_z * body_r * 0.8);
                let tip = Vector3::new(base.x + right_x * wing_span * (1.0 - t * 0.3), wing_y + wing_h * (0.5 - t * 0.3), base.z + right_z * wing_span * (1.0 - t * 0.3));
                d3.draw_cylinder_ex(base, tip, body_r * 0.06, 0.0, 4, color);
            }
        }
        6 => { // Bowtie
            let bow_y = body.y + body_r * 0.8;
            let bow_fwd = body_r * 0.9;
            let bow_center = Vector3::new(body.x + fwd_x * bow_fwd, bow_y, body.z + fwd_z * bow_fwd);
            d3.draw_sphere(bow_center, body_r * 0.08, color);
            let ll = Vector3::new(bow_center.x - right_x * body_r * 0.3, bow_y, bow_center.z - right_z * body_r * 0.3);
            d3.draw_sphere(ll, body_r * 0.15, color);
            let rl = Vector3::new(bow_center.x + right_x * body_r * 0.3, bow_y, bow_center.z + right_z * body_r * 0.3);
            d3.draw_sphere(rl, body_r * 0.15, color);
        }
        7 => { // Antenna
            let base = Vector3::new(head.x, head.y + head_r * 0.8, head.z);
            let top = Vector3::new(head.x, head.y + head_r * 2.0, head.z);
            d3.draw_cylinder_ex(base, top, head_r * 0.04, head_r * 0.03, 4, color);
            d3.draw_sphere(top, head_r * 0.15, color);
        }
        _ => {}
    }
}
