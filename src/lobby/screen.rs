use raylib::prelude::*;

use crate::lobby::state::{GameSettings, LobbyState};
use crate::menu::particles::MenuParticles;
use crate::menu::theme::Theme;

pub enum LobbyInput {
    None,
    ColorLeft,
    ColorRight,
    ToggleReady,
    Leave,
    CopyIP,
    ToggleSettings,
    SettingsUp,
    SettingsDown,
    SettingsLeft,
    SettingsRight,
}

pub struct LobbySettingsState {
    pub open: bool,
    pub selected: usize,
    pub time: f32,
}

impl LobbySettingsState {
    pub fn new() -> Self {
        Self {
            open: false,
            selected: 0,
            time: 0.0,
        }
    }
}

const SETTINGS_COUNT: usize = 7;

const WINS_OPTIONS: &[i32] = &[1, 2, 3, 5, 7];
const SPAWN_INVULN_OPTIONS: &[f32] = &[0.0, 1.0, 2.0, 2.5, 3.0, 5.0];
const STARTING_HP_OPTIONS: &[f32] = &[50.0, 75.0, 100.0, 150.0, 200.0];
const GRAVITY_OPTIONS: &[f32] = &[0.5, 0.75, 1.0, 1.25, 1.5];
const TURBO_OPTIONS: &[f32] = &[1.0, 1.25, 1.5, 2.0];

fn cycle_i32(options: &[i32], current: i32, dir: i32) -> i32 {
    let idx = options.iter().position(|&v| v == current).unwrap_or(0);
    let new_idx = (idx as i32 + dir).rem_euclid(options.len() as i32) as usize;
    options[new_idx]
}

fn cycle_f32(options: &[f32], current: f32, dir: i32) -> f32 {
    let idx = options.iter().position(|&v| (v - current).abs() < 0.001).unwrap_or(0);
    let new_idx = (idx as i32 + dir).rem_euclid(options.len() as i32) as usize;
    options[new_idx]
}

pub fn apply_settings_change(settings: &mut GameSettings, selected: usize, dir: i32) {
    match selected {
        0 => settings.wins_to_match = cycle_i32(WINS_OPTIONS, settings.wins_to_match, dir),
        1 => settings.spawn_invuln = cycle_f32(SPAWN_INVULN_OPTIONS, settings.spawn_invuln, dir),
        2 => settings.starting_hp = cycle_f32(STARTING_HP_OPTIONS, settings.starting_hp, dir),
        3 => settings.gravity_scale = cycle_f32(GRAVITY_OPTIONS, settings.gravity_scale, dir),
        4 => settings.turbo_speed = cycle_f32(TURBO_OPTIONS, settings.turbo_speed, dir),
        5 => settings.sudden_death = !settings.sudden_death,
        6 => settings.everyone_picks = !settings.everyone_picks,
        _ => {}
    }
}

fn setting_label(idx: usize) -> &'static str {
    match idx {
        0 => "WINS TO MATCH",
        1 => "SPAWN INVULN",
        2 => "STARTING HP",
        3 => "GRAVITY SCALE",
        4 => "TURBO MODE",
        5 => "SUDDEN DEATH",
        6 => "WHO PICKS",
        _ => "",
    }
}

fn setting_value(settings: &GameSettings, idx: usize) -> String {
    match idx {
        0 => format!("{}", settings.wins_to_match),
        1 => {
            if settings.spawn_invuln == 0.0 { "OFF".to_string() }
            else { format!("{:.1}s", settings.spawn_invuln) }
        }
        2 => format!("{}", settings.starting_hp as i32),
        3 => format!("{:.2}x", settings.gravity_scale),
        4 => {
            if (settings.turbo_speed - 1.0).abs() < 0.001 { "OFF".to_string() }
            else { format!("{:.2}x", settings.turbo_speed) }
        }
        5 => if settings.sudden_death { "ON".to_string() } else { "OFF".to_string() },
        6 => if settings.everyone_picks { "EVERYONE".to_string() } else { "LOSER".to_string() },
        _ => String::new(),
    }
}

fn is_default_value(settings: &GameSettings, idx: usize) -> bool {
    let d = GameSettings::default();
    match idx {
        0 => settings.wins_to_match == d.wins_to_match,
        1 => (settings.spawn_invuln - d.spawn_invuln).abs() < 0.001,
        2 => (settings.starting_hp - d.starting_hp).abs() < 0.001,
        3 => (settings.gravity_scale - d.gravity_scale).abs() < 0.001,
        4 => (settings.turbo_speed - d.turbo_speed).abs() < 0.001,
        5 => settings.sudden_death == d.sudden_death,
        6 => settings.everyone_picks == d.everyone_picks,
        _ => true,
    }
}

pub fn lobby_input(rl: &RaylibHandle, settings_open: bool) -> LobbyInput {
    if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
        return LobbyInput::ToggleSettings;
    }
    if settings_open {
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            return LobbyInput::ToggleSettings; // close panel
        }
        if rl.is_key_pressed(KeyboardKey::KEY_UP) || rl.is_key_pressed(KeyboardKey::KEY_W) {
            return LobbyInput::SettingsUp;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) || rl.is_key_pressed(KeyboardKey::KEY_S) {
            return LobbyInput::SettingsDown;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) || rl.is_key_pressed(KeyboardKey::KEY_A) {
            return LobbyInput::SettingsLeft;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) || rl.is_key_pressed(KeyboardKey::KEY_D) {
            return LobbyInput::SettingsRight;
        }
        return LobbyInput::None;
    }
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
    settings_state: &LobbySettingsState,
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

    // Title (matches settings screen style)
    let title = "LOBBY";
    let title_size = 60;
    let title_w = d.measure_text(title, title_size);
    let title_y = (h as f32 * 0.12) as i32;
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

    // Accent line
    let line_w = title_w + 40;
    let line_y = title_y + title_size + 10;
    d.draw_rectangle(
        w / 2 - line_w / 2,
        line_y,
        line_w,
        theme.accent_height,
        theme.accent_color,
    );

    // Host address (fixed under title)
    let mut addr_bottom = line_y + theme.accent_height;
    if is_host {
        let addr_text = format!("IP: {}", host_addr);
        let addr_size = 20;
        let addr_w = d.measure_text(&addr_text, addr_size);
        let addr_y = addr_bottom + 8;
        d.draw_text(
            &addr_text,
            w / 2 - addr_w / 2,
            addr_y,
            addr_size,
            theme.subtitle_color,
        );
        let _ = addr_w;
        addr_bottom = addr_y + addr_size;
    }

    // Two-column layout: players (left) + settings (right)
    let slot_height = 70;
    let panel_h = 4 * (slot_height + 10) - 10; // 310

    // Center columns between addr/title area and back button
    let back_y_area = h - 90 - 20;
    let available = back_y_area - addr_bottom;
    let slot_start_y = addr_bottom + (available - panel_h).max(0) / 2;
    let slot_width = 420;
    let col_gap = 30;
    let total_col_w = slot_width + col_gap + slot_width;
    let col_base_x = w / 2 - total_col_w / 2;

    for i in 0..4 {
        let slot_y = slot_start_y + i as i32 * (slot_height + 10);
        let slot_x = col_base_x;
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

    // ── Settings panel (right column, same size as player slots) ─────────────
    {
        let sp_x = col_base_x + slot_width + col_gap;
        let focused = settings_state.open;
        let panel_h = 4 * (slot_height + 10) - 10; // match total slots height

        // Background
        let bg_alpha = if focused { 140u8 } else { 80 };
        d.draw_rectangle(
            sp_x, slot_start_y, slot_width, panel_h,
            Color::new(15, 15, 25, bg_alpha),
        );
        let border_color = if focused {
            Color::new(theme.accent_color.r, theme.accent_color.g, theme.accent_color.b, 120)
        } else {
            Color::new(50, 50, 60, 80)
        };
        d.draw_rectangle_lines_ex(
            Rectangle::new(sp_x as f32, slot_start_y as f32, slot_width as f32, panel_h as f32),
            1.0, border_color,
        );

        let content_w = 350;
        let content_x = sp_x + (slot_width - content_w) / 2;
        let inner_x = content_x;
        let inner_w = content_w;

        // Header
        let header = "SETTINGS";
        let header_size = 18;
        let header_tw = d.measure_text(header, header_size);
        let header_y = slot_start_y + 10;
        let header_color = if focused { theme.item_hover_color } else { theme.item_color };
        d.draw_text(header, sp_x + slot_width / 2 - header_tw / 2, header_y, header_size, header_color);
        let hl_y = header_y + header_size + 4;
        d.draw_rectangle(inner_x, hl_y, inner_w, 1, theme.accent_color);

        // Rows (sized to fill panel)
        let rows_y = hl_y + 6;
        let hint_reserve = 22;
        let row_h = (panel_h - (rows_y - slot_start_y) - hint_reserve) / SETTINGS_COUNT as i32;

        for i in 0..SETTINGS_COUNT {
            let ry = rows_y + i as i32 * row_h;
            let is_sel = focused && i == settings_state.selected;

            if is_sel {
                let ba = ((time * theme.pulse_speed * 1.5).sin() * 15.0 + 30.0) as u8;
                d.draw_rectangle(
                    inner_x - 4, ry, inner_w + 8, row_h,
                    Color::new(theme.selector_color.r, theme.selector_color.g, theme.selector_color.b, ba),
                );
                let bp = ((time * theme.pulse_speed * 1.5).sin() * 40.0 + 215.0) as u8;
                d.draw_rectangle(
                    inner_x, ry + 3, 3, row_h - 6,
                    Color::new(theme.selector_color.r, theme.selector_color.g, theme.selector_color.b, bp),
                );
            }

            let label = setting_label(i);
            let ls = 16;
            let lc = if is_sel { theme.item_hover_color } else { theme.item_color };
            d.draw_text(label, inner_x + 12, ry + (row_h - ls) / 2, ls, lc);

            let value = setting_value(&state.settings, i);
            let vs = 16;
            let vtw = d.measure_text(&value, vs);
            let vy = ry + (row_h - vs) / 2;
            let vc = if !is_default_value(&state.settings, i) {
                theme.selector_color
            } else {
                Color::new(theme.item_color.r, theme.item_color.g, theme.item_color.b, 200)
            };

            let va_x = inner_x + inner_w - 130;
            if is_host && is_sel {
                d.draw_text("<", va_x, vy, vs, theme.selector_color);
                let vx = va_x + 16 + (96 - vtw) / 2;
                d.draw_text(&value, vx, vy, vs, vc);
                d.draw_text(">", va_x + 112, vy, vs, theme.selector_color);
            } else {
                let vx = va_x + 16 + (96 - vtw) / 2;
                d.draw_text(&value, vx, vy, vs, vc);
            }
        }

        // Inline hint
        let hint_y = rows_y + SETTINGS_COUNT as i32 * row_h + 4;
        let hint = if focused && is_host {
            "W/S select  |  A/D adjust"
        } else if focused {
            "Host controls settings"
        } else {
            "Tab to edit"
        };
        let hint_size = 13;
        let hint_tw = d.measure_text(hint, hint_size);
        let hint_alpha = if focused && is_host { 255u8 } else { 150 };
        d.draw_text(
            hint, sp_x + slot_width / 2 - hint_tw / 2, hint_y, hint_size,
            Color::new(theme.footer_color.r, theme.footer_color.g, theme.footer_color.b, hint_alpha),
        );
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
    let footer = "A/D color  |  Enter ready  |  Tab settings  |  Esc to leave";
    let footer_size = theme.footer_size;
    let footer_w = d.measure_text(footer, footer_size);
    d.draw_text(
        footer,
        w / 2 - footer_w / 2,
        h - footer_size - 20,
        footer_size,
        theme.footer_color,
    );
}

