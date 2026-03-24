use raylib::prelude::*;

use crate::game::cards::{CardDef, CardId, CARD_CATALOG};
use crate::game::state::GameState;
use crate::game::world::World;
use crate::menu::theme::Theme;
use crate::player::player::HIT_FLASH_DURATION;
use crate::render::cards::{self as render_cards, CardPickAnim};
use crate::render::crt::CrtFilter;
use crate::render::hud;

fn ray_aabb_t_render(ox: f32, oy: f32, dx: f32, dy: f32, aabb: &crate::physics::collision::AABB) -> Option<f32> {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    if dx.abs() > 1e-8 {
        let t1 = (aabb.min.x - ox) / dx;
        let t2 = (aabb.max.x - ox) / dx;
        tmin = tmin.max(t1.min(t2));
        tmax = tmax.min(t1.max(t2));
    } else if ox < aabb.min.x || ox > aabb.max.x {
        return None;
    }
    if dy.abs() > 1e-8 {
        let t1 = (aabb.min.y - oy) / dy;
        let t2 = (aabb.max.y - oy) / dy;
        tmin = tmin.max(t1.min(t2));
        tmax = tmax.min(t1.max(t2));
    } else if oy < aabb.min.y || oy > aabb.max.y {
        return None;
    }
    if tmin <= tmax && tmax > 0.0 {
        Some(if tmin > 0.0 { tmin } else { tmax })
    } else {
        None
    }
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
                } else {
                    base_color
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

                // Leech field aura
                if player.stats.leech_field {
                    let pulse = (time * 2.0).sin() * 0.15 + 0.85;
                    let r = 5.0 * pulse;
                    let center = Vector3::new(px, py + player.size.y / 2.0, pz);
                    d3.draw_sphere(center, r * 0.15, Color::new(160, 40, 80, 40));
                    d3.draw_sphere(center, r * 0.3, Color::new(160, 40, 80, 20));
                    // Inner ring markers
                    for angle_i in 0..8 {
                        let a = angle_i as f32 * std::f32::consts::PI / 4.0 + time * 1.5;
                        let rx = a.cos() * r * 0.4;
                        let ry = a.sin() * r * 0.4;
                        d3.draw_sphere(
                            Vector3::new(center.x + rx, center.y + ry, pz),
                            0.06,
                            Color::new(200, 60, 100, 120),
                        );
                    }
                }
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

            // Laser beams
            for (pi, player) in world.players.iter().enumerate() {
                if !player.alive || !player.laser_active { continue; }
                if player.ghost_timer > 0.0 && pi as u8 != local_player { continue; }
                let aim = player.aim_dir;
                let ox = player.position.x + aim.x * 0.5;
                let oy = player.position.y + 1.1 + aim.y * 0.5;
                let oz = player.position.z;
                let beam_r = 0.06 * (player.stats.bullet_radius_mult.sqrt().min(3.0));
                let core_r = 0.02 * (player.stats.bullet_radius_mult.sqrt().min(3.0));

                // Build beam directions (triple_shot = 3 beams)
                let mut aims: Vec<Vector2> = vec![aim];
                if player.stats.triple_shot {
                    let angle = std::f32::consts::PI / 12.0;
                    for &sign in &[-1.0_f32, 1.0] {
                        let a = sign * angle;
                        aims.push(Vector2::new(
                            aim.x * a.cos() - aim.y * a.sin(),
                            aim.x * a.sin() + aim.y * a.cos(),
                        ));
                    }
                }

                for beam_aim in &aims {
                    let mut max_t = 50.0_f32;
                    if !player.stats.phantom {
                        for platform in &world.level.platforms {
                            if let Some(t) = ray_aabb_t_render(ox, oy, beam_aim.x, beam_aim.y, &platform.aabb) {
                                if t > 0.0 && t < max_t { max_t = t; }
                            }
                        }
                    }
                    if !player.stats.piercing {
                        for (pj, other) in world.players.iter().enumerate() {
                            if pj == pi || !other.alive || other.ghost_timer > 0.0 { continue; }
                            if let Some(t) = ray_aabb_t_render(ox, oy, beam_aim.x, beam_aim.y, &other.aabb()) {
                                if t > 0.0 && t < max_t { max_t = t; }
                            }
                        }
                    }
                    let start = Vector3::new(ox, oy, oz);
                    let end = Vector3::new(ox + beam_aim.x * max_t, oy + beam_aim.y * max_t, oz);
                    let laser_color = Color::new(player.color.r, player.color.g, player.color.b, 200);
                    d3.draw_cylinder_ex(start, end, beam_r, beam_r, 4, laser_color);
                    d3.draw_cylinder_ex(start, end, core_r, core_r, 4, Color::WHITE);
                }
            }

            // Gravity wells
            for well in &world.gravity_wells {
                let pulse = (time * 4.0).sin() * 0.3 + 0.7;
                let alpha = ((well.lifetime / 4.0) * 255.0).min(255.0) as u8;
                let r = 6.0 * pulse;
                d3.draw_sphere(well.position, 0.3, Color::new(100, 60, 200, alpha));
                // Pulsing ring effect (rendered as thin sphere wireframe)
                d3.draw_sphere(well.position, r * 0.3, Color::new(120, 80, 220, alpha / 3));
            }

            // Clones (rendered as smaller copies of their owner)
            for clone in &world.clones {
                let fade = (clone.lifetime / 5.0).min(1.0);
                let c = Color::new(
                    (clone.color.r as f32 * fade) as u8,
                    (clone.color.g as f32 * fade) as u8,
                    (clone.color.b as f32 * fade) as u8,
                    255,
                );
                d3.draw_sphere(clone.position, 0.3, c);
                d3.draw_sphere(
                    Vector3::new(clone.position.x, clone.position.y + 0.5, clone.position.z),
                    0.22,
                    c,
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

        hud::draw_hud(&mut d, world, camera, render_w, render_h, local_player);

        if matches!(world.state, GameState::CardPick { .. }) {
            render_cards::draw_card_pick(&mut d, world, card_anim, render_w, render_h);
        }

        if matches!(world.state, GameState::MatchOver { .. }) {
            render_cards::draw_match_over(&mut d, world, render_w, render_h);
        }

        if dev_overlay {
            let held: Vec<CardId> = world.players.get(local_player as usize)
                .map(|p| p.cards.iter().map(|(id, _)| *id).collect())
                .unwrap_or_default();
            draw_dev_overlay(&mut d, render_w, render_h, &held);
        }

        d.draw_fps(10, 10);
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
