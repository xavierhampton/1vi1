use raylib::prelude::*;

use super::particles::MenuParticles;
use super::theme::{all_themes, Theme, THEME_COUNT};

// ── Menu screens ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Main,
    Settings,
}

#[derive(Clone, Copy, PartialEq)]
enum MainItem {
    Play,
    Settings,
    Quit,
}
const MAIN_ITEMS: &[MainItem] = &[MainItem::Play, MainItem::Settings, MainItem::Quit];

#[derive(Clone, Copy, PartialEq)]
enum SettingsItem {
    Theme,
    Volume,
    Back,
}
const SETTINGS_ITEMS: &[SettingsItem] = &[
    SettingsItem::Theme,
    SettingsItem::Volume,
    SettingsItem::Back,
];

pub enum MenuAction {
    None,
    StartGame,
    Quit,
}

// ── Menu state ───────────────────────────────────────────────────────────────

pub struct Menu {
    themes: Vec<Theme>,
    pub theme_index: usize,
    pub volume: f32, // 0.0 – 1.0

    screen: Screen,
    main_sel: usize,
    settings_sel: usize,
    hover_offsets: [f32; 4],
    prev_sel: Option<usize>,
    time: f32,
    pub fx: MenuParticles,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            themes: all_themes().into(),
            theme_index: 0,
            volume: 0.8,

            screen: Screen::Main,
            main_sel: 0,
            settings_sel: 0,
            hover_offsets: [0.0; 4],
            prev_sel: None,
            time: 0.0,
            fx: MenuParticles::new(),
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.themes[self.theme_index]
    }

    fn item_y(&self, i: usize, h: i32) -> i32 {
        let start_y = (h as f32 * self.theme().item_y_start_ratio) as i32;
        start_y + i as i32 * self.theme().item_spacing
    }

    fn item_center(&self, i: usize, _w: i32, h: i32) -> (f32, f32) {
        let y = self.item_y(i, h) + self.theme().item_size / 2;
        (_w as f32 / 2.0, y as f32)
    }

    // ── Update ───────────────────────────────────────────────────────────────

    pub fn update(&mut self, rl: &RaylibHandle, dt: f32) -> MenuAction {
        self.time += dt;

        let w = rl.get_screen_width();
        let h = rl.get_screen_height();
        let accent = self.theme().particle_color_primary;
        self.fx.update(dt, w, h, accent);

        match self.screen {
            Screen::Main => self.update_main(rl, dt, w, h),
            Screen::Settings => {
                self.update_settings(rl, dt, w, h);
                MenuAction::None
            }
        }
    }

    fn update_main(&mut self, rl: &RaylibHandle, dt: f32, w: i32, h: i32) -> MenuAction {
        let count = MAIN_ITEMS.len();
        let mut sel = self.main_sel;

        nav_keys(rl, &mut sel, count);
        mouse_hover(rl, &mut sel, count, h, self.theme());
        self.main_sel = sel;

        // Animate offsets
        let speed = self.theme().hover_slide_speed;
        animate_offsets(&mut self.hover_offsets, dt, sel, count, speed);

        // Selection changed -> pop particles
        if self.prev_sel != Some(sel) {
            if self.prev_sel.is_some() {
                let (px, py) = self.item_center(sel, w, h);
                let c = self.theme().selector_color;
                self.fx.pop(px, py, c);
            }
            self.prev_sel = Some(sel);
        }

        if confirm_pressed(rl) {
            let (px, py) = self.item_center(sel, w, h);
            let c = self.theme().selector_color;
            self.fx.explode(px, py, c);
            return match MAIN_ITEMS[sel] {
                MainItem::Play => MenuAction::StartGame,
                MainItem::Settings => {
                    self.screen = Screen::Settings;
                    self.settings_sel = 0;
                    self.prev_sel = None;
                    MenuAction::None
                }
                MainItem::Quit => MenuAction::Quit,
            };
        }

        MenuAction::None
    }

    fn update_settings(&mut self, rl: &RaylibHandle, dt: f32, w: i32, h: i32) {
        let count = SETTINGS_ITEMS.len();
        let mut sel = self.settings_sel;

        nav_keys(rl, &mut sel, count);
        mouse_hover(rl, &mut sel, count, h, self.theme());
        self.settings_sel = sel;

        let speed = self.theme().hover_slide_speed;
        animate_offsets(&mut self.hover_offsets, dt, sel, count, speed);

        if self.prev_sel != Some(sel) {
            if self.prev_sel.is_some() {
                let (px, py) = self.item_center(sel, w, h);
                let c = self.theme().selector_color;
                self.fx.pop(px, py, c);
            }
            self.prev_sel = Some(sel);
        }

        match SETTINGS_ITEMS[sel] {
            SettingsItem::Theme => {
                let changed = if rl.is_key_pressed(KeyboardKey::KEY_RIGHT)
                    || rl.is_key_pressed(KeyboardKey::KEY_D)
                {
                    self.theme_index = (self.theme_index + 1) % THEME_COUNT;
                    true
                } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT)
                    || rl.is_key_pressed(KeyboardKey::KEY_A)
                {
                    self.theme_index = (self.theme_index + THEME_COUNT - 1) % THEME_COUNT;
                    true
                } else {
                    false
                };
                if changed {
                    let (px, py) = self.item_center(sel, w, h);
                    let c = self.theme().selector_color;
                    self.fx.explode(px, py, c);
                }
            }
            SettingsItem::Volume => {
                if rl.is_key_pressed(KeyboardKey::KEY_RIGHT)
                    || rl.is_key_pressed(KeyboardKey::KEY_D)
                {
                    self.volume = (self.volume + 0.1).min(1.0);
                }
                if rl.is_key_pressed(KeyboardKey::KEY_LEFT)
                    || rl.is_key_pressed(KeyboardKey::KEY_A)
                {
                    self.volume = (self.volume - 0.1).max(0.0);
                }
                // Mouse drag on volume bar
                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    let bar_w = 200;
                    let bar_x = w / 2 + 60;
                    let item_size = self.theme().item_size;
                    let item_y = self.item_y(sel, h);
                    let mx = rl.get_mouse_x();
                    let my = rl.get_mouse_y();
                    if my >= item_y - 5
                        && my <= item_y + item_size + 5
                        && mx >= bar_x
                        && mx <= bar_x + bar_w
                    {
                        self.volume = ((mx - bar_x) as f32 / bar_w as f32).clamp(0.0, 1.0);
                    }
                }
            }
            SettingsItem::Back => {}
        }

        if confirm_pressed(rl) && SETTINGS_ITEMS[sel] == SettingsItem::Back {
            let (px, py) = self.item_center(sel, w, h);
            let c = self.theme().selector_color;
            self.fx.explode(px, py, c);
            self.screen = Screen::Main;
            self.prev_sel = None;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            self.screen = Screen::Main;
            self.prev_sel = None;
        }
    }

    // ── Drawing ──────────────────────────────────────────────────────────────

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let w = d.get_screen_width();
        let h = d.get_screen_height();
        let th = self.theme();

        d.clear_background(th.bg);
        self.draw_grid(d, w, h);
        self.fx.draw(d);

        match self.screen {
            Screen::Main => self.draw_main(d, w, h),
            Screen::Settings => self.draw_settings(d, w, h),
        }
    }

    fn draw_main(&self, d: &mut RaylibDrawHandle, w: i32, h: i32) {
        let th = self.theme();
        self.draw_title(d, w, h);

        let labels = ["PLAY", "SETTINGS", "QUIT"];
        for (i, label) in labels.iter().enumerate() {
            self.draw_item(d, label, i, self.main_sel, h, th);
        }

        self.draw_footer(d, "W/S to navigate  |  Enter to select", w, h);
    }

    fn draw_settings(&self, d: &mut RaylibDrawHandle, w: i32, h: i32) {
        let th = self.theme();

        // Settings title
        let title = "SETTINGS";
        let title_size = 60;
        let title_w = d.measure_text(title, title_size);
        let title_y = (h as f32 * 0.12) as i32;
        d.draw_text(
            title,
            w / 2 - title_w / 2 + 3,
            title_y + 3,
            title_size,
            th.title_shadow_color,
        );
        d.draw_text(title, w / 2 - title_w / 2, title_y, title_size, th.title_color);

        // Accent line
        let line_w = title_w + 40;
        let line_y = title_y + title_size + 10;
        d.draw_rectangle(w / 2 - line_w / 2, line_y, line_w, th.accent_height, th.accent_color);

        for (i, item) in SETTINGS_ITEMS.iter().enumerate() {
            let is_selected = i == self.settings_sel;
            let item_y = self.item_y(i, h);
            let slide = self.hover_offsets[i] as i32;

            let color = if is_selected { th.item_hover_color } else { th.item_color };

            match item {
                SettingsItem::Theme => {
                    let label = "THEME";
                    let label_w = d.measure_text(label, th.item_size);
                    let label_x = w / 2 - label_w - 30 + slide;
                    d.draw_text(label, label_x, item_y, th.item_size, color);

                    if is_selected {
                        draw_selector(d, label_x, item_y, th, self.time);
                    }

                    // Theme name with arrows
                    let name = th.name.to_uppercase();
                    let val_x = w / 2 + 30;
                    let arrow_color = if is_selected { th.selector_color } else { th.item_color };
                    d.draw_text("<", val_x, item_y, th.item_size, arrow_color);
                    let name_x = val_x + 30;
                    d.draw_text(&name, name_x, item_y, th.item_size, th.selector_color);
                    let name_w = d.measure_text(&name, th.item_size);
                    d.draw_text(">", name_x + name_w + 10, item_y, th.item_size, arrow_color);
                }
                SettingsItem::Volume => {
                    let label = "VOLUME";
                    let label_w = d.measure_text(label, th.item_size);
                    let label_x = w / 2 - label_w - 30 + slide;
                    d.draw_text(label, label_x, item_y, th.item_size, color);

                    if is_selected {
                        draw_selector(d, label_x, item_y, th, self.time);
                    }

                    // Volume slider
                    let bar_x = w / 2 + 60;
                    let bar_w = 200;
                    let bar_h = 8;
                    let bar_y = item_y + th.item_size / 2 - bar_h / 2;
                    let fill_w = (bar_w as f32 * self.volume) as i32;

                    d.draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(40, 40, 50, 200));
                    d.draw_rectangle(bar_x, bar_y, fill_w, bar_h, th.selector_color);

                    // Knob
                    let knob_x = bar_x + fill_w;
                    let knob_r = 10;
                    d.draw_rectangle(
                        knob_x - knob_r / 2,
                        bar_y - (knob_r - bar_h) / 2,
                        knob_r,
                        knob_r,
                        th.item_hover_color,
                    );

                    let pct = format!("{}%", (self.volume * 100.0) as i32);
                    d.draw_text(&pct, bar_x + bar_w + 16, item_y, th.item_size, color);
                }
                SettingsItem::Back => {
                    let label = "BACK";
                    let label_w = d.measure_text(label, th.item_size);
                    let label_x = w / 2 - label_w / 2 + slide;
                    d.draw_text(label, label_x, item_y, th.item_size, color);

                    if is_selected {
                        draw_selector(d, label_x, item_y, th, self.time);
                    }
                }
            }
        }

        // Theme preview swatches
        let swatch_size = 16;
        let swatch_gap = 6;
        let total_w = THEME_COUNT as i32 * (swatch_size + swatch_gap) - swatch_gap;
        let swatch_x_start = w / 2 - total_w / 2;
        let swatch_y = h - 70;
        let themes = all_themes();
        for (i, t) in themes.iter().enumerate() {
            let sx = swatch_x_start + i as i32 * (swatch_size + swatch_gap);
            if i == self.theme_index {
                d.draw_rectangle(sx - 2, swatch_y - 2, swatch_size + 4, swatch_size + 4, th.item_hover_color);
            }
            d.draw_rectangle(sx, swatch_y, swatch_size, swatch_size, t.selector_color);
        }

        self.draw_footer(d, "A/D or Arrows to adjust  |  Enter/Esc to go back", w, h);
    }

    // ── Shared draw helpers ──────────────────────────────────────────────────

    fn draw_title(&self, d: &mut RaylibDrawHandle, w: i32, h: i32) {
        let th = self.theme();

        let title = "1VI1";
        let title_y = (h as f32 * th.title_y_ratio) as i32;
        let title_w = d.measure_text(title, th.title_size);
        let title_x = w / 2 - title_w / 2;

        d.draw_text(
            title,
            title_x + th.title_shadow_offset,
            title_y + th.title_shadow_offset,
            th.title_size,
            th.title_shadow_color,
        );
        d.draw_text(title, title_x, title_y, th.title_size, th.title_color);

        // Subtitle
        let sub = "ARENA COMBAT";
        let sub_w = d.measure_text(sub, th.subtitle_size);
        let sub_y = title_y + th.title_size + 6;
        let pulse = ((self.time * th.pulse_speed).sin() * 0.3 + 0.7) as f32;
        let sub_color = Color::new(
            (th.subtitle_color.r as f32 * pulse) as u8,
            (th.subtitle_color.g as f32 * pulse) as u8,
            (th.subtitle_color.b as f32 * pulse) as u8,
            th.subtitle_color.a,
        );
        d.draw_text(sub, w / 2 - sub_w / 2, sub_y, th.subtitle_size, sub_color);

        // Accent line
        let line_w = title_w + 40;
        let line_y = sub_y + th.subtitle_size + 12;
        d.draw_rectangle(w / 2 - line_w / 2, line_y, line_w, th.accent_height, th.accent_color);
    }

    fn draw_item(
        &self,
        d: &mut RaylibDrawHandle,
        label: &str,
        i: usize,
        selected: usize,
        h: i32,
        th: &Theme,
    ) {
        let w = d.get_screen_width();
        let is_selected = i == selected;
        let slide = self.hover_offsets[i] as i32;
        let color = if is_selected { th.item_hover_color } else { th.item_color };

        let text_w = d.measure_text(label, th.item_size);
        let item_x = w / 2 - text_w / 2 + slide;
        let item_y = self.item_y(i, h);

        if is_selected {
            draw_selector(d, item_x, item_y, th, self.time);
        }

        d.draw_text(label, item_x, item_y, th.item_size, color);
    }

    fn draw_footer(&self, d: &mut RaylibDrawHandle, text: &str, w: i32, h: i32) {
        let th = self.theme();
        let fw = d.measure_text(text, th.footer_size);
        d.draw_text(text, w / 2 - fw / 2, h - th.footer_size - 20, th.footer_size, th.footer_color);
    }

    fn draw_grid(&self, d: &mut RaylibDrawHandle, w: i32, h: i32) {
        let th = self.theme();
        let spacing = th.bg_grid_spacing;
        let scroll = (self.time * 8.0) % spacing;
        let color = Color::new(th.bg_grid_color.r, th.bg_grid_color.g, th.bg_grid_color.b, th.bg_grid_alpha);

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
}

// ── Free functions ───────────────────────────────────────────────────────────

fn draw_selector(d: &mut RaylibDrawHandle, item_x: i32, item_y: i32, th: &Theme, time: f32) {
    let bar_x = item_x - th.selector_gap - th.selector_width;
    let bar_h = th.item_size;
    let bar_pulse = ((time * th.pulse_speed * 1.5).sin() * 40.0 + 215.0) as u8;
    let bar_color = Color::new(
        th.selector_color.r,
        th.selector_color.g,
        th.selector_color.b,
        bar_pulse,
    );
    d.draw_rectangle(bar_x, item_y, th.selector_width, bar_h, bar_color);
}

fn nav_keys(rl: &RaylibHandle, sel: &mut usize, count: usize) {
    if rl.is_key_pressed(KeyboardKey::KEY_DOWN) || rl.is_key_pressed(KeyboardKey::KEY_S) {
        *sel = (*sel + 1) % count;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_UP) || rl.is_key_pressed(KeyboardKey::KEY_W) {
        *sel = (*sel + count - 1) % count;
    }
}

fn mouse_hover(rl: &RaylibHandle, sel: &mut usize, count: usize, h: i32, th: &Theme) {
    let mx = rl.get_mouse_x();
    let my = rl.get_mouse_y();
    let w = rl.get_screen_width();
    let start_y = (h as f32 * th.item_y_start_ratio) as i32;
    for i in 0..count {
        let item_y = start_y + i as i32 * th.item_spacing;
        let hit_x = w / 2 - 150;
        let hit_w = 300;
        let hit_y = item_y - 4;
        let hit_h = th.item_size + 8;
        if mx >= hit_x && mx <= hit_x + hit_w && my >= hit_y && my <= hit_y + hit_h {
            *sel = i;
        }
    }
}

fn animate_offsets(offsets: &mut [f32; 4], dt: f32, selected: usize, count: usize, speed: f32) {
    for i in 0..count {
        let target = if i == selected { 12.0 } else { 0.0 };
        offsets[i] += (target - offsets[i]) * speed * dt;
    }
}

fn confirm_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KeyboardKey::KEY_ENTER)
        || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
        || rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
}
