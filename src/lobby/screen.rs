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
    let footer = "A/D color  |  Enter ready  |  Tab settings  |  C copy IP";
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

pub fn draw_settings_panel(
    d: &mut RaylibDrawHandle,
    settings: &GameSettings,
    panel_state: &LobbySettingsState,
    theme: &Theme,
    is_host: bool,
) {
    let w = d.get_screen_width();
    let h = d.get_screen_height();
    let time = panel_state.time;

    // Dim overlay
    d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, 160));

    // Panel dimensions
    let panel_w = 450;
    let row_h = 36;
    let title_h = 50;
    let footer_h = 30;
    let padding = 20;
    let panel_h = title_h + SETTINGS_COUNT as i32 * row_h + footer_h + padding * 2;
    let panel_x = w / 2 - panel_w / 2;
    let panel_y = h / 2 - panel_h / 2;

    // Panel background
    d.draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::new(15, 15, 25, 240));
    // Border
    let border_rect = Rectangle::new(panel_x as f32, panel_y as f32, panel_w as f32, panel_h as f32);
    d.draw_rectangle_lines_ex(border_rect, 2.0, theme.accent_color);

    // Title
    let title = "GAME SETTINGS";
    let title_size = 32;
    let title_tw = d.measure_text(title, title_size);
    let title_x = w / 2 - title_tw / 2;
    let title_y = panel_y + padding;
    d.draw_text(title, title_x + 2, title_y + 2, title_size, theme.title_shadow_color);
    d.draw_text(title, title_x, title_y, title_size, theme.title_color);

    // Accent line under title
    let accent_y = title_y + title_size + 6;
    let accent_w = title_tw + 30;
    d.draw_rectangle(w / 2 - accent_w / 2, accent_y, accent_w, 2, theme.accent_color);

    // Setting rows
    let rows_start_y = accent_y + 14;
    let label_x = panel_x + padding;
    let value_area_x = panel_x + panel_w - padding - 160;

    for i in 0..SETTINGS_COUNT {
        let row_y = rows_start_y + i as i32 * row_h;
        let is_selected = i == panel_state.selected;

        // Selected row highlight
        if is_selected {
            let bg_alpha = ((time * theme.pulse_speed * 1.5).sin() * 15.0 + 30.0) as u8;
            d.draw_rectangle(
                panel_x + 4, row_y - 2,
                panel_w - 8, row_h,
                Color::new(theme.selector_color.r, theme.selector_color.g, theme.selector_color.b, bg_alpha),
            );
            // Selector bar on left
            let bar_pulse = ((time * theme.pulse_speed * 1.5).sin() * 40.0 + 215.0) as u8;
            let bar_color = Color::new(
                theme.selector_color.r,
                theme.selector_color.g,
                theme.selector_color.b,
                bar_pulse,
            );
            d.draw_rectangle(panel_x + 8, row_y + 2, 4, row_h - 8, bar_color);
        }

        // Label
        let label = setting_label(i);
        let label_size = 20;
        let label_color = if is_selected { theme.item_hover_color } else { theme.item_color };
        d.draw_text(label, label_x + 18, row_y + (row_h - label_size) / 2, label_size, label_color);

        // Value with arrows
        let value = setting_value(settings, i);
        let val_size = 20;
        let val_tw = d.measure_text(&value, val_size);
        let val_y = row_y + (row_h - val_size) / 2;

        // Color non-default values differently
        let val_color = if !is_default_value(settings, i) {
            theme.selector_color
        } else {
            Color::new(theme.item_color.r, theme.item_color.g, theme.item_color.b, 200)
        };

        if is_host && is_selected {
            let arrow_color = theme.selector_color;
            d.draw_text("<", value_area_x, val_y, val_size, arrow_color);
            let val_x = value_area_x + 20 + (120 - val_tw) / 2;
            d.draw_text(&value, val_x, val_y, val_size, val_color);
            d.draw_text(">", value_area_x + 140, val_y, val_size, arrow_color);
        } else {
            let val_x = value_area_x + 20 + (120 - val_tw) / 2;
            d.draw_text(&value, val_x, val_y, val_size, val_color);
        }
    }

    // Footer
    let footer = if is_host {
        "Tab to close  |  Up/Down select  |  Left/Right adjust"
    } else {
        "Tab to close  |  Host controls settings"
    };
    let footer_size = 14;
    let footer_tw = d.measure_text(footer, footer_size);
    d.draw_text(
        footer,
        w / 2 - footer_tw / 2,
        panel_y + panel_h - padding - footer_size + 4,
        footer_size,
        theme.footer_color,
    );
}
