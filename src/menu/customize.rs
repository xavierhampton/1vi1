use raylib::prelude::*;

use super::theme::Theme;
use crate::lobby::state::{LOBBY_COLORS, LOBBY_COLOR_COUNT};

// ── Accessory catalog ────────────────────────────────────────────────────────

pub const ACCESSORY_NONE: u8 = 0xFF;
pub const MAX_EQUIPPED: usize = 3;
pub const ACCESSORY_COUNT: usize = 8;

pub const ACCESSORY_NAMES: [&str; ACCESSORY_COUNT] = [
    "TOP HAT", "CROWN", "HALO", "BANDANA", "HORNS", "WINGS", "BOWTIE", "ANTENNA",
];

const ACCENT_PALETTE: [(u8, u8, u8); 10] = [
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

pub type Equipped = [(u8, u8, u8, u8); MAX_EQUIPPED]; // (id, r, g, b)

#[allow(dead_code)]
pub fn empty_equipped() -> Equipped {
    [(ACCESSORY_NONE, 255, 255, 255); MAX_EQUIPPED]
}

const PREVIEW_TEX_W: u32 = 280;
const PREVIEW_TEX_H: u32 = 340;

// ── Customize editor ─────────────────────────────────────────────────────────

pub struct CustomizeEditor {
    pub equipped: Equipped,
    pub name: String,
    name_focused: bool,
    grid_sel: usize,
    selected_slot: Option<usize>,
    color_sel: usize,
    pub preview_color: usize,
    grace_frames: u8,
    time: f32,
    preview_tex: Option<RenderTexture2D>,
    preview_shader: Option<Shader>,
}

impl CustomizeEditor {
    pub fn new(existing: &Equipped, preview_color: usize, name: String) -> Self {
        let sel = (0..MAX_EQUIPPED).find(|&i| existing[i].0 != ACCESSORY_NONE);
        let color_sel = sel
            .and_then(|s| {
                let (_, r, g, b) = existing[s];
                ACCENT_PALETTE
                    .iter()
                    .position(|&(pr, pg, pb)| pr == r && pg == g && pb == b)
            })
            .unwrap_or(0);
        Self {
            equipped: *existing,
            name,
            name_focused: false,
            grid_sel: 0,
            selected_slot: sel,
            color_sel,
            preview_color,
            grace_frames: 3,
            time: 0.0,
            preview_tex: None,
            preview_shader: None,
        }
    }

    // ── 3D preview (called before begin_drawing) ─────────────────────────────

    pub fn render_preview(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        screen_w: i32,
        screen_h: i32,
    ) {
        if self.preview_tex.is_none() {
            self.preview_tex = Some(
                rl.load_render_texture(thread, PREVIEW_TEX_W, PREVIEW_TEX_H)
                    .expect("preview texture"),
            );
            self.preview_shader = Some(rl.load_shader(thread, None, Some("assets/shaders/crt.fs")));
        }

        let player_color = LOBBY_COLORS[self.preview_color].0;
        let bob = (self.time * 2.0).sin() * 0.02;
        let equipped = self.equipped;

        let cursor_x = rl.get_mouse_x();
        let cursor_y = rl.get_mouse_y();
        let ly = Layout::new(screen_w, screen_h);

        let norm_x = (cursor_x - ly.preview_x) as f32 / ly.preview_w as f32 * 2.0 - 1.0;
        let norm_y = -((cursor_y - ly.preview_y) as f32 / PREVIEW_TEX_H as f32 * 2.0 - 1.0);
        let len = (norm_x * norm_x + norm_y * norm_y).sqrt();
        let (aim_x, aim_y) = if len < 0.1 {
            (0.0, 0.0)
        } else {
            (
                (norm_x / len).clamp(-1.0, 1.0),
                (norm_y / len).clamp(-1.0, 1.0),
            )
        };

        let camera = Camera3D::perspective(
            Vector3::new(0.0, 0.8, 2.6),
            Vector3::new(0.0, 0.8, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            40.0,
        );

        let tex = self.preview_tex.as_mut().unwrap();
        {
            let mut d = rl.begin_texture_mode(thread, tex);
            d.clear_background(Color::new(10, 10, 16, 255));
            {
                let mut d3 = d.begin_mode3D(camera);

                d3.draw_cube(
                    Vector3::new(0.0, -0.03, 0.0),
                    1.2,
                    0.02,
                    0.3,
                    Color::new(40, 40, 50, 255),
                );

                let py = bob;
                let body_r = 0.38_f32;
                let head_r = 0.28_f32;
                let body_center = Vector3::new(0.0, py + 0.5, 0.0);
                let head_center = Vector3::new(0.0, py + 1.15, 0.0);

                d3.draw_sphere(body_center, body_r, player_color);
                d3.draw_sphere(head_center, head_r, player_color);

                // Eyes
                let eye_r = 0.065;
                let eye_spread = 0.12;
                let cam_pos = camera.position;
                let fwd_xz_x = cam_pos.x - head_center.x;
                let fwd_xz_z = cam_pos.z - head_center.z;
                let fwd_len = (fwd_xz_x * fwd_xz_x + fwd_xz_z * fwd_xz_z).sqrt();
                let (fwd_x, fwd_z) = if fwd_len > 0.001 {
                    (fwd_xz_x / fwd_len, fwd_xz_z / fwd_len)
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
                let eye_cx = base_x + aim_x * look_shift * right_x;
                let eye_cy = base_y + aim_y * look_shift;
                let eye_cz = base_z + aim_x * look_shift * right_z;

                d3.draw_sphere(
                    Vector3::new(
                        eye_cx - right_x * eye_spread,
                        eye_cy,
                        eye_cz - right_z * eye_spread,
                    ),
                    eye_r,
                    Color::new(20, 20, 25, 255),
                );
                d3.draw_sphere(
                    Vector3::new(
                        eye_cx + right_x * eye_spread,
                        eye_cy,
                        eye_cz + right_z * eye_spread,
                    ),
                    eye_r,
                    Color::new(20, 20, 25, 255),
                );

                // Accessories
                for &(id, r, g, b) in equipped.iter() {
                    if id != ACCESSORY_NONE {
                        let ac = Color::new(r, g, b, 255);
                        crate::render::game::draw_accessory_3d(
                            &mut d3,
                            id,
                            ac,
                            head_center,
                            body_center,
                            head_r,
                            body_r,
                            1.0,
                            fwd_x,
                            fwd_z,
                            right_x,
                            right_z,
                        );
                    }
                }
            }
        }
    }

    // ── Update ───────────────────────────────────────────────────────────────

    pub fn update(&mut self, rl: &mut RaylibHandle, dt: f32, render_w: i32, render_h: i32) -> bool {
        self.time += dt;

        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            return true;
        }
        if self.grace_frames > 0 {
            self.grace_frames -= 1;
            return false;
        }

        let ly = Layout::new(render_w, render_h);

        // ── Name input ───────────────────────────────────────────────────────
        if self.name_focused {
            // Paste
            if (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                && rl.is_key_pressed(KeyboardKey::KEY_V)
            {
                let clip = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    rl.get_clipboard_text()
                }));
                if let Ok(Ok(text)) = clip {
                    for c in text.chars() {
                        if (c.is_alphanumeric() || c == '_') && self.name.len() < 12 {
                            self.name.push(c);
                        }
                    }
                }
            }
            loop {
                match rl.get_char_pressed() {
                    Some(c) if (c.is_alphanumeric() || c == '_') && self.name.len() < 12 => {
                        self.name.push(c);
                    }
                    None => break,
                    _ => {}
                }
            }
            if (rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                || rl.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE))
                && !self.name.is_empty()
            {
                self.name.pop();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_TAB)
            {
                self.name_focused = false;
            }
        } else {
            // Player color (A/D)
            if rl.is_key_pressed(KeyboardKey::KEY_LEFT) || rl.is_key_pressed(KeyboardKey::KEY_A) {
                self.preview_color = (self.preview_color + LOBBY_COLOR_COUNT - 1) % LOBBY_COLOR_COUNT;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) || rl.is_key_pressed(KeyboardKey::KEY_D) {
                self.preview_color = (self.preview_color + 1) % LOBBY_COLOR_COUNT;
            }
        }

        // ── Mouse (always active) ────────────────────────────────────────────
        let mx = rl.get_mouse_x();
        let my = rl.get_mouse_y();

        if !self.name_focused {
            for i in 0..ACCESSORY_COUNT {
                let (ix, iy, iw, ih) = ly.catalog_item(i);
                if mx >= ix && mx < ix + iw && my >= iy && my < iy + ih {
                    self.grid_sel = i;
                }
            }
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            // Name box
            let (nx, ny, nw, nh) = ly.name_box;
            if mx >= nx && mx < nx + nw && my >= ny && my < ny + nh {
                self.name_focused = true;
            } else {
                self.name_focused = false;
            }

            // Back button
            let bw = 200;
            let bh = 34;
            let bx = render_w / 2 - bw / 2;
            if mx >= bx && mx < bx + bw && my >= ly.back_y && my < ly.back_y + bh {
                return true;
            }

            // Catalog
            for i in 0..ACCESSORY_COUNT {
                let (ix, iy, iw, ih) = ly.catalog_item(i);
                if mx >= ix && mx < ix + iw && my >= iy && my < iy + ih {
                    self.toggle_equip(i as u8);
                }
            }
            // Equipped slots
            for i in 0..MAX_EQUIPPED {
                let (sx, sy, sw, sh) = ly.equipped_slot(i);
                if mx >= sx && mx < sx + sw && my >= sy && my < sy + sh {
                    if self.equipped[i].0 != ACCESSORY_NONE {
                        self.selected_slot = Some(i);
                        let (_, r, g, b) = self.equipped[i];
                        if let Some(ci) = ACCENT_PALETTE
                            .iter()
                            .position(|&(pr, pg, pb)| pr == r && pg == g && pb == b)
                        {
                            self.color_sel = ci;
                        }
                    }
                }
            }
            // Palette
            for i in 0..ACCENT_PALETTE.len() {
                let (px, py, ps) = ly.palette_swatch(i);
                if mx >= px && mx < px + ps && my >= py && my < py + ps {
                    self.color_sel = i;
                    self.apply_color_to_selected();
                }
            }
            // Player color arrows
            let px = ly.preview_x;
            let pw = ly.preview_w;
            if mx >= px && mx < px + pw && my >= ly.color_sel_y && my < ly.color_sel_y + 26 {
                if mx < px + pw / 2 {
                    self.preview_color = (self.preview_color + LOBBY_COLOR_COUNT - 1) % LOBBY_COLOR_COUNT;
                } else {
                    self.preview_color = (self.preview_color + 1) % LOBBY_COLOR_COUNT;
                }
            }
        }

        false
    }

    fn apply_color_to_selected(&mut self) {
        if let Some(slot) = self.selected_slot {
            if self.equipped[slot].0 != ACCESSORY_NONE {
                let (r, g, b) = ACCENT_PALETTE[self.color_sel];
                self.equipped[slot].1 = r;
                self.equipped[slot].2 = g;
                self.equipped[slot].3 = b;
            }
        }
    }

    fn toggle_equip(&mut self, id: u8) {
        for i in 0..MAX_EQUIPPED {
            if self.equipped[i].0 == id {
                self.unequip(i);
                return;
            }
        }
        let (r, g, b) = ACCENT_PALETTE[self.color_sel];
        self.equip(id, r, g, b);
    }

    fn equip(&mut self, id: u8, r: u8, g: u8, b: u8) {
        for (i, slot) in self.equipped.iter_mut().enumerate() {
            if slot.0 == ACCESSORY_NONE {
                *slot = (id, r, g, b);
                self.selected_slot = Some(i);
                return;
            }
        }
        self.equipped[0] = self.equipped[1];
        self.equipped[1] = self.equipped[2];
        self.equipped[2] = (id, r, g, b);
        self.selected_slot = Some(2);
    }

    fn unequip(&mut self, slot: usize) {
        for i in slot..MAX_EQUIPPED - 1 {
            self.equipped[i] = self.equipped[i + 1];
        }
        self.equipped[MAX_EQUIPPED - 1] = (ACCESSORY_NONE, 255, 255, 255);
        match self.selected_slot {
            Some(s) if s == slot => self.selected_slot = None,
            Some(s) if s > slot => self.selected_slot = Some(s - 1),
            _ => {}
        }
    }

    // ── Drawing (own full screen — bg drawn by Menu) ─────────────────────────

    pub fn draw(&mut self, d: &mut RaylibDrawHandle, theme: &Theme, render_w: i32, render_h: i32) {
        let ly = Layout::new(render_w, render_h);

        // Title (matches settings screen style)
        let title = "CUSTOMIZE";
        let title_size = 60;
        let title_w = d.measure_text(title, title_size);
        let cx = render_w / 2;
        d.draw_text(
            title,
            cx - title_w / 2 + 3,
            ly.title_y + 3,
            title_size,
            theme.title_shadow_color,
        );
        d.draw_text(
            title,
            cx - title_w / 2,
            ly.title_y,
            title_size,
            theme.title_color,
        );
        let line_w = title_w + 40;
        let line_y = ly.title_y + title_size + 10;
        d.draw_rectangle(
            cx - line_w / 2,
            line_y,
            line_w,
            theme.accent_height,
            theme.accent_color,
        );

        self.draw_preview(d, theme, &ly);
        self.draw_name(d, theme, &ly);
        self.draw_catalog(d, theme, &ly);
        self.draw_equipped_bar(d, theme, &ly);
        self.draw_palette(d, theme, &ly);

        // Back button
        let back = "BACK";
        let back_size = 28;
        let back_w = d.measure_text(back, back_size);
        let back_x = render_w / 2 - back_w / 2;
        let back_y = ly.back_y;
        let mx = d.get_mouse_x();
        let my = d.get_mouse_y();
        let hit_w = 200;
        let hit_h = back_size + 8;
        let hover = mx >= render_w / 2 - hit_w / 2
            && mx <= render_w / 2 + hit_w / 2
            && my >= back_y
            && my <= back_y + hit_h;
        let color = if hover {
            theme.item_hover_color
        } else {
            theme.item_color
        };
        if hover {
            let bar_x = back_x - theme.selector_gap - theme.selector_width;
            let bar_pulse = ((self.time * theme.pulse_speed * 1.5).sin() * 40.0 + 215.0) as u8;
            let bar_color = Color::new(
                theme.selector_color.r,
                theme.selector_color.g,
                theme.selector_color.b,
                bar_pulse,
            );
            d.draw_rectangle(bar_x, back_y, theme.selector_width, back_size, bar_color);
        }
        d.draw_text(back, back_x, back_y, back_size, color);

        // Footer
        let footer = "A/D player color  |  Esc to go back";
        let footer_size = theme.footer_size;
        let footer_w = d.measure_text(footer, footer_size);
        d.draw_text(
            footer,
            render_w / 2 - footer_w / 2,
            render_h - footer_size - 20,
            footer_size,
            theme.footer_color,
        );
    }

    fn draw_preview(&mut self, d: &mut RaylibDrawHandle, theme: &Theme, ly: &Layout) {
        let px = ly.preview_x;
        let py = ly.preview_y;
        let pw = ly.preview_w;
        let full_h = ly.preview_h;

        if let Some(ref tex) = self.preview_tex {
            if let Some(ref mut shader) = self.preview_shader {
                let mut s = d.begin_shader_mode(shader);
                s.draw_texture_rec(
                    tex.texture(),
                    Rectangle::new(0.0, 0.0, PREVIEW_TEX_W as f32, -(PREVIEW_TEX_H as f32)),
                    Vector2::new(px as f32, py as f32),
                    Color::WHITE,
                );
            } else {
                d.draw_texture_rec(
                    tex.texture(),
                    Rectangle::new(0.0, 0.0, PREVIEW_TEX_W as f32, -(PREVIEW_TEX_H as f32)),
                    Vector2::new(px as f32, py as f32),
                    Color::WHITE,
                );
            }
        } else {
            d.draw_rectangle(
                px,
                py,
                pw,
                PREVIEW_TEX_H as i32,
                Color::new(10, 10, 16, 255),
            );
        }

        d.draw_rectangle_lines_ex(
            Rectangle::new(px as f32, py as f32, pw as f32, full_h as f32),
            2.0,
            theme.selector_color,
        );

        // Player color selector
        let player_color = LOBBY_COLORS[self.preview_color].0;
        let color_name = LOBBY_COLORS[self.preview_color].1.to_uppercase();
        let label_size = 18;
        let label_w = d.measure_text(&color_name, label_size);
        let center_x = px + pw / 2;
        let sel_y = ly.color_sel_y;
        d.draw_text(
            "<",
            center_x - label_w / 2 - 16,
            sel_y,
            label_size,
            theme.item_hover_color,
        );
        d.draw_text(
            &color_name,
            center_x - label_w / 2,
            sel_y,
            label_size,
            player_color,
        );
        d.draw_text(
            ">",
            center_x + label_w / 2 + 8,
            sel_y,
            label_size,
            theme.item_hover_color,
        );
    }

    fn draw_name(&self, d: &mut RaylibDrawHandle, theme: &Theme, ly: &Layout) {
        let (bx, by, bw, bh) = ly.name_box;
        d.draw_text("NAME", ly.catalog_x, by - 20, 18, theme.item_color);
        d.draw_rectangle(bx, by, bw, bh, Color::new(20, 20, 30, 220));
        let border = if self.name_focused {
            theme.selector_color
        } else {
            Color::new(60, 60, 70, 200)
        };
        d.draw_rectangle_lines(bx, by, bw, bh, border);
        let display = if self.name_focused {
            format!("{}_", self.name)
        } else {
            self.name.clone()
        };
        let text_color = if self.name_focused {
            theme.item_hover_color
        } else {
            theme.item_color
        };
        d.draw_text(&display, bx + 10, by + (bh - 22) / 2, 22, text_color);
    }

    fn draw_catalog(&self, d: &mut RaylibDrawHandle, theme: &Theme, ly: &Layout) {
        d.draw_text(
            "ACCESSORIES",
            ly.catalog_x,
            ly.catalog_y - 24,
            20,
            theme.item_color,
        );

        for i in 0..ACCESSORY_COUNT {
            let (ix, iy, iw, ih) = ly.catalog_item(i);
            let is_hovered = i == self.grid_sel;
            let is_equipped = self.equipped.iter().any(|s| s.0 == i as u8);

            let bg = if is_equipped {
                Color::new(
                    theme.selector_color.r / 4,
                    theme.selector_color.g / 4,
                    theme.selector_color.b / 4,
                    200,
                )
            } else {
                Color::new(16, 16, 24, 220)
            };
            d.draw_rectangle(ix, iy, iw, ih, bg);

            let border = if is_hovered {
                theme.item_hover_color
            } else if is_equipped {
                theme.selector_color
            } else {
                Color::new(50, 50, 60, 200)
            };
            d.draw_rectangle_lines_ex(
                Rectangle::new(ix as f32, iy as f32, iw as f32, ih as f32),
                if is_hovered { 2.0 } else { 1.0 },
                border,
            );

            let name = ACCESSORY_NAMES[i];
            let name_size = 16;
            let name_w = d.measure_text(name, name_size);
            let name_color = if is_equipped {
                let eq = self.equipped.iter().find(|s| s.0 == i as u8).unwrap();
                Color::new(eq.1, eq.2, eq.3, 255)
            } else if is_hovered {
                theme.item_hover_color
            } else {
                theme.item_color
            };
            d.draw_text(
                name,
                ix + iw / 2 - name_w / 2,
                iy + ih / 2 - name_size / 2,
                name_size,
                name_color,
            );
        }
    }

    fn draw_equipped_bar(&self, d: &mut RaylibDrawHandle, theme: &Theme, ly: &Layout) {
        let has_sel = self
            .selected_slot
            .map(|s| self.equipped[s].0 != ACCESSORY_NONE)
            .unwrap_or(false);
        let header = if has_sel {
            let slot = self.selected_slot.unwrap();
            let name = ACCESSORY_NAMES[self.equipped[slot].0 as usize];
            format!("EQUIPPED:  recoloring {}", name)
        } else {
            "EQUIPPED:  click a slot to recolor".to_string()
        };
        d.draw_text(&header, ly.equipped_x, ly.equipped_y, 16, theme.item_color);

        for i in 0..MAX_EQUIPPED {
            let (sx, sy, sw, sh) = ly.equipped_slot(i);
            let slot = &self.equipped[i];
            let is_selected = self.selected_slot == Some(i) && slot.0 != ACCESSORY_NONE;

            if slot.0 != ACCESSORY_NONE {
                let color = Color::new(slot.1, slot.2, slot.3, 255);
                d.draw_rectangle(
                    sx,
                    sy,
                    sw,
                    sh,
                    Color::new(slot.1 / 5, slot.2 / 5, slot.3 / 5, 220),
                );
                let border_color = if is_selected {
                    let pulse = (self.time * 4.0).sin() * 0.3 + 0.7;
                    Color::new(
                        (color.r as f32 * pulse) as u8,
                        (color.g as f32 * pulse) as u8,
                        (color.b as f32 * pulse) as u8,
                        255,
                    )
                } else {
                    color
                };
                d.draw_rectangle_lines_ex(
                    Rectangle::new(sx as f32, sy as f32, sw as f32, sh as f32),
                    if is_selected { 3.0 } else { 1.0 },
                    border_color,
                );
                let name = ACCESSORY_NAMES[slot.0 as usize];
                let name_size = 14;
                let name_w = d.measure_text(name, name_size);
                d.draw_text(
                    name,
                    sx + sw / 2 - name_w / 2,
                    sy + sh / 2 - name_size / 2,
                    name_size,
                    color,
                );
            } else {
                d.draw_rectangle(sx, sy, sw, sh, Color::new(16, 16, 24, 180));
                d.draw_rectangle_lines_ex(
                    Rectangle::new(sx as f32, sy as f32, sw as f32, sh as f32),
                    1.0,
                    Color::new(50, 50, 60, 150),
                );
                let ew = d.measure_text("EMPTY", 13);
                d.draw_text(
                    "EMPTY",
                    sx + sw / 2 - ew / 2,
                    sy + sh / 2 - 6,
                    13,
                    Color::new(60, 60, 70, 200),
                );
            }
        }
    }

    fn draw_palette(&self, d: &mut RaylibDrawHandle, theme: &Theme, ly: &Layout) {
        let has_sel = self
            .selected_slot
            .map(|s| self.equipped[s].0 != ACCESSORY_NONE)
            .unwrap_or(false);
        let label_color = if has_sel {
            theme.item_color
        } else {
            Color::new(80, 80, 90, 200)
        };
        d.draw_text("ITEM COLOR", ly.palette_x, ly.palette_y, 16, label_color);

        for i in 0..ACCENT_PALETTE.len() {
            let (px, py, ps) = ly.palette_swatch(i);
            let (r, g, b) = ACCENT_PALETTE[i];
            let alpha = if has_sel { 255u8 } else { 90 };
            d.draw_rectangle(px, py, ps, ps, Color::new(r, g, b, alpha));
            if i == self.color_sel && has_sel {
                d.draw_rectangle_lines_ex(
                    Rectangle::new(
                        (px - 2) as f32,
                        (py - 2) as f32,
                        (ps + 4) as f32,
                        (ps + 4) as f32,
                    ),
                    2.0,
                    theme.item_hover_color,
                );
            }
        }
    }
}

// ── Layout (centered both axes) ──────────────────────────────────────────────

struct Layout {
    title_y: i32,
    preview_x: i32,
    preview_y: i32,
    preview_w: i32,
    preview_h: i32,
    name_box: (i32, i32, i32, i32),
    catalog_x: i32,
    catalog_y: i32,
    item_w: i32,
    item_h: i32,
    item_gap: i32,
    equipped_x: i32,
    equipped_y: i32,
    palette_x: i32,
    palette_y: i32,
    color_sel_y: i32,
    back_y: i32,
}

impl Layout {
    fn new(w: i32, h: i32) -> Self {
        let preview_w = PREVIEW_TEX_W as i32;
        let preview_3d_h = PREVIEW_TEX_H as i32;
        let preview_h = preview_3d_h + 40;
        let item_w = 110;
        let item_h = 50;
        let item_gap = 8;
        let gap = 40;
        let cols = 4;
        let right_w = cols * item_w + (cols - 1) * item_gap;

        let total_w = preview_w + gap + right_w;
        let base_x = (w - total_w) / 2;

        // Right panel heights
        let name_h = 90;
        let cat_label = 24;
        let cat_rows = 2 * (item_h + item_gap) - item_gap;
        let cat_total = cat_label + cat_rows;
        let eq_gap = 24;
        let eq_label = 22;
        let eq_slot_h = 48;
        let eq_total = eq_label + eq_slot_h;
        let pal_gap = 24;
        let pal_label = 18;
        let pal_swatch = 26;
        let pal_total = pal_label + pal_swatch;
        let right_total = name_h + cat_total + eq_gap + eq_total + pal_gap + pal_total;

        let content_h = preview_h.max(right_total);
        let back_y = h - 90;

        // Title at 12% of screen (matches settings screen)
        let title_y = (h as f32 * 0.12) as i32;
        let title_bottom = title_y + 60 + 14;

        // Center content between title area and back button
        let available = (back_y - 20) - title_bottom;
        let content_y = title_bottom + (available - content_h).max(0) / 2;

        let catalog_x = base_x + preview_w + gap;

        // Name box at top of right panel
        let name_box_y = content_y + 18;
        let name_box = (catalog_x, name_box_y, right_w.min(360), 36);

        // Catalog below name (with generous gap)
        let catalog_y = content_y + name_h + cat_label;

        let eq_y = catalog_y + cat_rows + eq_gap;
        let pal_y = eq_y + eq_total + pal_gap;
        let color_sel_y = content_y + preview_3d_h + 8;

        Self {
            title_y,
            preview_x: base_x,
            preview_y: content_y,
            preview_w,
            preview_h,
            name_box,
            catalog_x,
            catalog_y,
            item_w,
            item_h,
            item_gap,
            equipped_x: catalog_x,
            equipped_y: eq_y,
            palette_x: catalog_x,
            palette_y: pal_y,
            color_sel_y,
            back_y,
        }
    }

    fn catalog_item(&self, index: usize) -> (i32, i32, i32, i32) {
        let col = index % 4;
        let row = index / 4;
        (
            self.catalog_x + col as i32 * (self.item_w + self.item_gap),
            self.catalog_y + row as i32 * (self.item_h + self.item_gap),
            self.item_w,
            self.item_h,
        )
    }

    fn equipped_slot(&self, index: usize) -> (i32, i32, i32, i32) {
        let sw = 140;
        let sh = 48;
        let gap = 12;
        (
            self.equipped_x + index as i32 * (sw + gap),
            self.equipped_y + 20,
            sw,
            sh,
        )
    }

    fn palette_swatch(&self, index: usize) -> (i32, i32, i32) {
        let ps = 26;
        let gap = 6;
        (
            self.palette_x + index as i32 * (ps + gap),
            self.palette_y + 18,
            ps,
        )
    }
}
