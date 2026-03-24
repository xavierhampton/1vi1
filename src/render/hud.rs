use raylib::prelude::*;

use crate::game::cards::CARD_CATALOG;
use crate::game::state::GameState;
use crate::game::world::{World, COUNTDOWN_DURATION, MAX_BULLETS, RELOAD_TIME, WINS_TO_MATCH};

pub fn draw_hud(
    d: &mut RaylibDrawHandle, world: &World, camera: Camera3D,
    render_w: i32, render_h: i32, local_player: u8,
) {
    let mouse = d.get_mouse_position();

    // Tooltip to draw last (on top of everything)
    let mut tooltip: Option<(&str, &str, (u8, u8, u8), f32, f32)> = None;

    // ── Per-player floating HUD (name, HP, ammo — above head) ───────────
    for player in &world.players {
        if !player.alive { continue; }
        let above_head = Vector3::new(
            player.position.x,
            player.position.y + player.size.y + 0.15,
            player.position.z,
        );
        let screen_pos = d.get_world_to_screen(above_head, camera);
        let sx = screen_pos.x as i32;
        let sy = screen_pos.y as i32;

        // Name
        let font_size = 20;
        let text_w = d.measure_text(&player.name, font_size);
        d.draw_text(&player.name, sx - text_w / 2, sy - font_size, font_size, Color::WHITE);

        // HP bar
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
        let max_ammo = MAX_BULLETS + player.stats.extra_ammo;
        let total_pip_w = max_ammo * (pip_size + pip_gap) - pip_gap;
        let pip_x = sx - total_pip_w / 2;
        let pip_y = bar_y + bar_h + 5;
        for i in 0..max_ammo {
            let px = pip_x + i * (pip_size + pip_gap);
            let pip_color = if i < player.bullets_remaining {
                player.color
            } else {
                Color::new(40, 40, 40, 200)
            };
            d.draw_rectangle(px, pip_y, pip_size, pip_size, pip_color);
        }

        // Reload bar
        if player.reload_timer > 0.0 {
            let eff_reload = RELOAD_TIME;
            let reload_ratio = 1.0 - (player.reload_timer / eff_reload);
            let reload_fill = (total_pip_w as f32 * reload_ratio) as i32;
            d.draw_rectangle(pip_x, pip_y, total_pip_w, pip_size, Color::new(40, 40, 40, 200));
            d.draw_rectangle(pip_x, pip_y, reload_fill, pip_size, Color::new(200, 200, 200, 220));
        }
    }

    // ── Local player HUD (bottom center) ─────────────────────────────────
    if let Some(local_p) = world.players.get(local_player as usize) {
        let abilities: Vec<_> = local_p.cards.iter()
            .filter(|(c, _)| CARD_CATALOG[*c as u8 as usize].is_ability())
            .collect();
        let powerups: Vec<_> = local_p.cards.iter()
            .filter(|(c, _)| CARD_CATALOG[*c as u8 as usize].is_powerup())
            .collect();

        let icon_radius: f32 = 30.0;
        let ring_thick: f32 = 3.0;

        // ── Ability circles (bottom center) ──
        if !abilities.is_empty() {
            let spacing: f32 = 72.0;
            let count = abilities.len();
            let total_w = (count - 1) as f32 * spacing;
            let base_cx = render_w as f32 / 2.0 - total_w / 2.0;
            let base_cy = render_h as f32 - 48.0;

            // Connecting rail
            if count > 1 {
                d.draw_rectangle(
                    base_cx as i32, (base_cy - 1.0) as i32,
                    total_w as i32, 2,
                    Color::new(255, 255, 255, 18),
                );
            }

            for (j, (card_id, cooldown)) in abilities.iter().enumerate() {
                let def = &CARD_CATALOG[*card_id as u8 as usize];
                let (cr, cg, cb) = def.color;
                let cx = base_cx + j as f32 * spacing;
                let cy = base_cy;
                let center = Vector2::new(cx, cy);
                let ready = *cooldown <= 0.0;

                d.draw_circle(cx as i32, cy as i32, icon_radius,
                    Color::new(cr / 8 + 8, cg / 8 + 8, cb / 8 + 8, 210));

                if ready {
                    d.draw_circle(cx as i32, cy as i32, icon_radius - ring_thick,
                        Color::new(cr, cg, cb, 35));
                    d.draw_ring(center, icon_radius - ring_thick, icon_radius, 0.0, 360.0, 36,
                        Color::new(cr, cg, cb, 200));

                    let glyph = format!("{}", def.icon_glyph);
                    let g_font = 28;
                    let g_w = d.measure_text(&glyph, g_font);
                    d.draw_text(&glyph, cx as i32 - g_w / 2, cy as i32 - g_font / 2,
                        g_font, Color::new(255, 255, 255, 240));
                } else {
                    let ratio = 1.0 - (*cooldown / def.cooldown()).clamp(0.0, 1.0);
                    let sweep = ratio * 360.0;
                    let start_angle = 90.0 - sweep;
                    let end_angle = 90.0;

                    d.draw_ring(center, icon_radius - ring_thick, icon_radius, 0.0, 360.0, 36,
                        Color::new(cr, cg, cb, 20));
                    if sweep > 0.5 {
                        d.draw_ring(center, icon_radius - ring_thick, icon_radius, start_angle, end_angle, 36,
                            Color::new(cr, cg, cb, 140));
                    }

                    let cd_text = format!("{:.0}", cooldown.ceil());
                    let cd_font = 22;
                    let cd_w = d.measure_text(&cd_text, cd_font);
                    d.draw_text(&cd_text, cx as i32 - cd_w / 2, cy as i32 - cd_font / 2,
                        cd_font, Color::new(cr, cg, cb, 160));
                }

                // Hover check for tooltip
                let dx = mouse.x - cx;
                let dy = mouse.y - cy;
                if dx * dx + dy * dy <= icon_radius * icon_radius {
                    tooltip = Some((def.name, def.description, def.color, cx, cy));
                }
            }
        }

        // ── Powerup icons (bottom left, vertical stack) ──
        if !powerups.is_empty() {
            let spacing_y: f32 = 72.0;
            let count = powerups.len();
            let total_h = (count - 1) as f32 * spacing_y;
            let cx = 48.0;
            let base_cy = render_h as f32 - 48.0 - total_h;

            // Connecting rail (vertical)
            if count > 1 {
                d.draw_rectangle(
                    (cx - 1.0) as i32, base_cy as i32,
                    2, total_h as i32,
                    Color::new(255, 255, 255, 18),
                );
            }

            for (j, (card_id, _)) in powerups.iter().enumerate() {
                let def = &CARD_CATALOG[*card_id as u8 as usize];
                let (cr, cg, cb) = def.color;
                let cy = base_cy + j as f32 * spacing_y;
                let center = Vector2::new(cx, cy);

                d.draw_circle(cx as i32, cy as i32, icon_radius,
                    Color::new(cr / 8 + 8, cg / 8 + 8, cb / 8 + 8, 210));
                d.draw_circle(cx as i32, cy as i32, icon_radius - ring_thick,
                    Color::new(cr, cg, cb, 35));
                d.draw_ring(center, icon_radius - ring_thick, icon_radius, 0.0, 360.0, 36,
                    Color::new(cr, cg, cb, 200));

                let glyph = format!("{}", def.icon_glyph);
                let g_font = 28;
                let g_w = d.measure_text(&glyph, g_font);
                d.draw_text(&glyph, cx as i32 - g_w / 2, cy as i32 - g_font / 2,
                    g_font, Color::new(255, 255, 255, 240));

                // Hover check for tooltip
                let dx = mouse.x - cx;
                let dy = mouse.y - cy;
                if dx * dx + dy * dy <= icon_radius * icon_radius {
                    tooltip = Some((def.name, def.description, def.color, cx, cy));
                }
            }
        }
    }

    // ── Score pips (top right) — squares with thick outlines ───────────
    {
        let wins_needed = WINS_TO_MATCH;
        let pip_size: i32 = 14;
        let pip_gap: i32 = 6;
        let row_gap: i32 = 22;
        let margin_r: i32 = 16;
        let margin_t: i32 = 12;
        let border: f32 = 2.0;
        let row_w = wins_needed * (pip_size + pip_gap) - pip_gap;

        for (i, player) in world.players.iter().enumerate() {
            let score = world.scores.get(i).copied().unwrap_or(0);
            let row_y = margin_t + i as i32 * row_gap;
            let row_x = render_w - margin_r - row_w;

            for w in 0..wins_needed {
                let px = row_x + w * (pip_size + pip_gap);
                let filled = w < score;
                let rect = Rectangle::new(px as f32, row_y as f32, pip_size as f32, pip_size as f32);
                if filled {
                    d.draw_rectangle(px, row_y, pip_size, pip_size, player.color);
                }
                d.draw_rectangle_lines_ex(rect, border,
                    if filled { player.color }
                    else { Color::new(player.color.r / 2, player.color.g / 2, player.color.b / 2, 120) });
            }
        }
    }

    // ── Countdown dim + text ─────────────────────────────────────────────
    if let GameState::RoundStart { timer } = &world.state {
        let fade = (*timer / COUNTDOWN_DURATION).clamp(0.0, 1.0);
        let alpha = (fade * 140.0) as u8;
        d.draw_rectangle(0, 0, render_w, render_h, Color::new(0, 0, 0, alpha));

        let num = timer.ceil() as i32;
        let text = format!("{}", num.max(1));
        let font_size = 120;
        let text_w = d.measure_text(&text, font_size);
        d.draw_text(&text, render_w / 2 - text_w / 2, render_h / 2 - font_size / 2,
            font_size, Color::WHITE);
    }

    // ── Win text ────────────────────────────────────────────────────────
    if let GameState::RoundEnd { winner_name, winner_color, .. } = &world.state {
        let text = format!("{} Wins!", winner_name);
        let font_size = 80;
        let text_w = d.measure_text(&text, font_size);
        let color = Color::new(winner_color.0, winner_color.1, winner_color.2, 255);
        d.draw_text(&text, render_w / 2 - text_w / 2, render_h / 2 - font_size / 2,
            font_size, color);
    }

    // ── Tooltip (drawn last so it's on top) ──────────────────────────────
    if let Some((name, desc, (cr, cg, cb), icon_cx, icon_cy)) = tooltip {
        let pad_x: i32 = 10;
        let pad_y: i32 = 8;
        let name_font = 18;
        let desc_font = 14;
        let name_w = d.measure_text(name, name_font);
        let desc_w = d.measure_text(desc, desc_font);
        let inner_w = name_w.max(desc_w);
        let box_w = inner_w + pad_x * 2;
        let box_h = name_font + desc_font + pad_y * 3;

        // Position above the icon, centered horizontally, clamped to screen
        let mut tx = icon_cx as i32 - box_w / 2;
        let mut ty = icon_cy as i32 - 44 - box_h;
        tx = tx.clamp(4, render_w - box_w - 4);
        ty = ty.clamp(4, render_h - box_h - 4);

        // Background
        d.draw_rectangle(tx, ty, box_w, box_h, Color::new(12, 12, 14, 230));
        // Border
        d.draw_rectangle_lines_ex(
            Rectangle::new(tx as f32, ty as f32, box_w as f32, box_h as f32),
            1.0, Color::new(cr / 2 + 40, cg / 2 + 40, cb / 2 + 40, 180),
        );
        // Name
        d.draw_text(name, tx + pad_x, ty + pad_y, name_font,
            Color::new(cr, cg, cb, 255));
        // Description
        d.draw_text(desc, tx + pad_x, ty + pad_y + name_font + pad_y, desc_font,
            Color::new(180, 180, 180, 220));
    }
}
