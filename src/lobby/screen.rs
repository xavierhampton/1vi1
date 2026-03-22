use raylib::prelude::*;

use crate::lobby::state::LobbyState;
use crate::menu::particles::MenuParticles;
use crate::menu::theme::Theme;

pub enum LobbyInput {
    None,
    ColorLeft,
    ColorRight,
    ToggleReady,
    Leave,
    CopyIP,
}

pub fn lobby_input(rl: &RaylibHandle) -> LobbyInput {
    if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
        return LobbyInput::Leave;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_LEFT) || rl.is_key_pressed(KeyboardKey::KEY_A) {
        return LobbyInput::ColorLeft;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) || rl.is_key_pressed(KeyboardKey::KEY_D) {
        return LobbyInput::ColorRight;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
        return LobbyInput::ToggleReady;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_C) {
        return LobbyInput::CopyIP;
    }
    // Back button click
    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        let w = rl.get_screen_width();
        let h = rl.get_screen_height();
        let (bx, by, bw, bh) = lobby_back_rect(w, h);
        let mx = rl.get_mouse_x();
        let my = rl.get_mouse_y();
        if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
            return LobbyInput::Leave;
        }
    }
    LobbyInput::None
}

fn lobby_back_rect(w: i32, h: i32) -> (i32, i32, i32, i32) {
    let btn_w = 120;
    let btn_h = 36;
    (w / 2 - btn_w / 2, h - 90, btn_w, btn_h)
}

pub fn draw_lobby(
    d: &mut RaylibDrawHandle,
    state: &LobbyState,
    my_index: usize,
    is_host: bool,
    host_addr: &str,
    theme: &Theme,
    time: f32,
    particles: &MenuParticles,
) {
    let w = d.get_screen_width();
    let h = d.get_screen_height();

    // Ambient particles
    particles.draw(d);

    // Background grid (same as menu)
    {
        let spacing = theme.bg_grid_spacing;
        let scroll = (time * 8.0) % spacing;
        let color = Color::new(
            theme.bg_grid_color.r,
            theme.bg_grid_color.g,
            theme.bg_grid_color.b,
            theme.bg_grid_alpha,
        );
        let mut x = -spacing + scroll;
        while x < w as f32 + spacing {
            d.draw_line(x as i32, 0, x as i32, h, color);
            x += spacing;
        }
        let mut y = -spacing + scroll;
        while y < h as f32 + spacing {
            d.draw_line(0, y as i32, w, y as i32, color);
            y += spacing;
        }
    }

    // Title
    let title = "LOBBY";
    let title_size = 60;
    let title_w = d.measure_text(title, title_size);
    let title_y = 30;
    d.draw_text(
        title,
        w / 2 - title_w / 2 + 3,
        title_y + 3,
        title_size,
        theme.title_shadow_color,
    );
    d.draw_text(
        title,
        w / 2 - title_w / 2,
        title_y,
        title_size,
        theme.title_color,
    );

    // Host address
    if is_host {
        let addr_text = format!("IP: {}", host_addr);
        let addr_size = 20;
        let addr_w = d.measure_text(&addr_text, addr_size);
        d.draw_text(
            &addr_text,
            w / 2 - addr_w / 2,
            title_y + title_size + 8,
            addr_size,
            theme.subtitle_color,
        );
    }

    // Accent line
    let line_y = title_y + title_size + 35;
    let line_w = 400;
    d.draw_rectangle(
        w / 2 - line_w / 2,
        line_y,
        line_w,
        theme.accent_height,
        theme.accent_color,
    );

    // Player slots
    let slot_start_y = line_y + 30;
    let slot_height = 70;
    let slot_width = 420;

    for i in 0..4 {
        let slot_y = slot_start_y + i as i32 * (slot_height + 10);
        let slot_x = w / 2 - slot_width / 2;
        let is_me = i == my_index;

        if i < state.slots.len() {
            let slot = &state.slots[i];

            // Slot background
            let bg_alpha = if is_me { 50 } else { 25 };
            let bg_color = Color::new(
                theme.selector_color.r,
                theme.selector_color.g,
                theme.selector_color.b,
                bg_alpha,
            );
            d.draw_rectangle(slot_x, slot_y, slot_width, slot_height, bg_color);

            // Border highlight for own slot
            if is_me {
                let pulse = ((time * theme.pulse_speed * 1.5).sin() * 30.0 + 200.0) as u8;
                let border_color = Color::new(
                    theme.selector_color.r,
                    theme.selector_color.g,
                    theme.selector_color.b,
                    pulse,
                );
                d.draw_rectangle_lines(slot_x, slot_y, slot_width, slot_height, border_color);
            }

            // Player color swatch
            let swatch_size = 40;
            let swatch_x = slot_x + 15;
            let swatch_y = slot_y + (slot_height - swatch_size) / 2;
            d.draw_rectangle(
                swatch_x,
                swatch_y,
                swatch_size,
                swatch_size,
                slot.color.to_color(),
            );

            // Color arrows for own slot
            if is_me {
                let arrow_size = 20;
                let arrow_color = theme.item_hover_color;
                let arrow_y = swatch_y + (swatch_size - arrow_size) / 2;
                d.draw_text("<", swatch_x - 14, arrow_y, arrow_size, arrow_color);
                d.draw_text(
                    ">",
                    swatch_x + swatch_size + 8,
                    arrow_y,
                    arrow_size,
                    arrow_color,
                );
            }

            // Player name
            let name_size = 28;
            let name_x = swatch_x + swatch_size + 30;
            let name_y = slot_y + 10;
            let name_color = if is_me {
                theme.item_hover_color
            } else {
                theme.item_color
            };
            d.draw_text(&slot.name, name_x, name_y, name_size, name_color);

            // Host badge
            if slot.is_host {
                let badge = "HOST";
                let badge_size = 14;
                let badge_w = d.measure_text(badge, badge_size);
                let badge_x = name_x + d.measure_text(&slot.name, name_size) + 12;
                d.draw_text(badge, badge_x, name_y + 4, badge_size, theme.selector_color);
                let _ = badge_w; // used for layout
            }

            // Color name
            let color_name = slot.color.name();
            let cn_size = 16;
            d.draw_text(
                color_name,
                name_x,
                name_y + name_size + 4,
                cn_size,
                theme.subtitle_color,
            );

            // Ready status
            let ready_x = slot_x + slot_width - 100;
            let ready_y = slot_y + (slot_height - 28) / 2;
            if slot.ready {
                d.draw_text(
                    "READY",
                    ready_x,
                    ready_y,
                    28,
                    Color::new(100, 230, 120, 255),
                );
            } else {
                d.draw_text(
                    "...",
                    ready_x + 20,
                    ready_y,
                    28,
                    Color::new(120, 120, 130, 200),
                );
            }
        } else {
            // Empty slot
            let bg_color = Color::new(30, 30, 40, 80);
            d.draw_rectangle(slot_x, slot_y, slot_width, slot_height, bg_color);
            let empty = "WAITING...";
            let empty_size = 22;
            let empty_w = d.measure_text(empty, empty_size);
            d.draw_text(
                empty,
                slot_x + slot_width / 2 - empty_w / 2,
                slot_y + (slot_height - empty_size) / 2,
                empty_size,
                Color::new(60, 60, 70, 200),
            );
        }
    }

    // Waiting / All Ready indicator
    let status_y = slot_start_y + 4 * (slot_height + 10) + 10;
    if state.all_ready() {
        let text = if is_host {
            "ALL READY - STARTING..."
        } else {
            "ALL READY!"
        };
        let size = 30;
        let tw = d.measure_text(text, size);
        let pulse = ((time * 4.0).sin() * 0.3 + 0.7) as f32;
        let c = Color::new(
            (100.0 * pulse) as u8,
            (230.0 * pulse) as u8,
            (120.0 * pulse) as u8,
            255,
        );
        d.draw_text(text, w / 2 - tw / 2, status_y, size, c);
    }

    // Back item
    let (bx, by, bw, bh) = lobby_back_rect(w, h);
    let back_label = "BACK";
    let back_size = 28;
    let back_tw = d.measure_text(back_label, back_size);
    let back_x = w / 2 - back_tw / 2;
    let back_y = by + (bh - back_size) / 2;
    let mx = d.get_mouse_x();
    let my = d.get_mouse_y();
    let hover = mx >= bx && mx <= bx + bw && my >= by && my <= by + bh;
    let back_color = if hover {
        theme.item_hover_color
    } else {
        theme.item_color
    };
    if hover {
        let bar_pulse = ((time * theme.pulse_speed * 1.5).sin() * 40.0 + 215.0) as u8;
        let bar_color = Color::new(
            theme.selector_color.r,
            theme.selector_color.g,
            theme.selector_color.b,
            bar_pulse,
        );
        let bar_x = back_x - 14 - 4;
        d.draw_rectangle(bar_x, back_y, 4, back_size, bar_color);
    }
    d.draw_text(back_label, back_x, back_y, back_size, back_color);

    // Footer
    let footer = "A/D to change color  |  Enter to ready up  |  C to copy IP";
    let footer_size = theme.footer_size;
    let footer_w = d.measure_text(footer, footer_size);
    d.draw_text(
        footer,
        w / 2 - footer_w / 2,
        h - footer_size - 12,
        footer_size,
        theme.footer_color,
    );
}
