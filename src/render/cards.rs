use raylib::prelude::*;

use crate::game::cards::CARD_CATALOG;
use crate::game::state::GameState;
use crate::game::world::World;

const CARD_W: f32 = 220.0;
const CARD_H: f32 = 310.0;
const CARD_GAP: f32 = 44.0;

const ENTRANCE_DURATION: f32 = 0.8;
const EXIT_DURATION: f32 = 0.8;

/// Client-only animation state for card pick phase — ticks locally for smooth rendering
pub struct CardPickAnim {
    pub entrance_t: f32,     // 0..ENTRANCE_DURATION, ticked locally
    pub hover_index: Option<u8>,
    pub slam_t: f32,         // 0..EXIT_DURATION, ticked locally after card chosen
    pub prev_picker: u8,
    pub chose_seen: bool,    // whether we've started the slam anim
    pub toast_timer: f32,
    pub toast_text: String,
}

impl CardPickAnim {
    pub fn new() -> Self {
        Self {
            entrance_t: 0.0,
            hover_index: None,
            slam_t: 0.0,
            prev_picker: 0xFF,
            chose_seen: false,
            toast_timer: 0.0,
            toast_text: String::new(),
        }
    }

    pub fn update(&mut self, world: &World, dt: f32) {
        if let GameState::CardPick { current_picker, chosen_card, .. } = &world.state {
            // Reset anim when picker changes
            if *current_picker != self.prev_picker {
                self.entrance_t = 0.0;
                self.slam_t = 0.0;
                self.hover_index = None;
                self.chose_seen = false;
                self.prev_picker = *current_picker;
            }

            // Tick entrance locally
            if self.entrance_t < ENTRANCE_DURATION {
                self.entrance_t = (self.entrance_t + dt).min(ENTRANCE_DURATION);
            }

            // Hover: read directly from server-broadcast card_hover
            let entrance_done = self.entrance_t >= ENTRANCE_DURATION;
            if chosen_card.is_none() && entrance_done && world.card_hover < 3 {
                self.hover_index = Some(world.card_hover);
            } else if chosen_card.is_none() {
                self.hover_index = None;
            }

            // Slam + exit: tick locally once server says a card was chosen
            if chosen_card.is_some() {
                if !self.chose_seen {
                    self.chose_seen = true;
                    self.slam_t = 0.0;
                    self.hover_index = None;
                }
                self.slam_t = (self.slam_t + dt).min(EXIT_DURATION);
            }

            // Toast
            if self.toast_timer > 0.0 {
                self.toast_timer = (self.toast_timer - dt).max(0.0);
            }
        } else {
            // Not in card pick — reset
            if self.prev_picker != 0xFF {
                self.prev_picker = 0xFF;
                self.entrance_t = 0.0;
                self.chose_seen = false;
            }
        }
    }
}

fn ease_out(t: f32) -> f32 {
    t * (2.0 - t)
}

fn card_rects(screen_w: f32, screen_h: f32) -> [(f32, f32); 3] {
    let total_w = 3.0 * CARD_W + 2.0 * CARD_GAP;
    let start_x = (screen_w - total_w) / 2.0;
    let card_y = screen_h * 0.35;
    [
        (start_x, card_y),
        (start_x + CARD_W + CARD_GAP, card_y),
        (start_x + 2.0 * (CARD_W + CARD_GAP), card_y),
    ]
}

pub fn card_slot_from_mouse(mouse: Vector2, screen_w: f32, screen_h: f32) -> Option<u8> {
    let rects = card_rects(screen_w, screen_h);
    for i in 0..3u8 {
        let (cx, cy) = rects[i as usize];
        if mouse.x >= cx && mouse.x <= cx + CARD_W && mouse.y >= cy && mouse.y <= cy + CARD_H {
            return Some(i);
        }
    }
    None
}

pub fn draw_card_pick(
    d: &mut RaylibDrawHandle,
    world: &World,
    anim: &CardPickAnim,
    screen_w: i32,
    screen_h: i32,
) {
    let (current_picker, offered_cards, chosen_card) =
        if let GameState::CardPick { current_picker, offered_cards, chosen_card, .. } = &world.state {
            (*current_picker, *offered_cards, *chosen_card)
        } else {
            return;
        };

    let sw = screen_w as f32;
    let sh = screen_h as f32;

    // Dim overlay
    d.draw_rectangle(0, 0, screen_w, screen_h, Color::new(0, 0, 0, 160));

    // Banner text — player name in their color
    let picker = world.players.get(current_picker as usize);
    let picker_name = picker.map(|p| p.name.as_str()).unwrap_or("???");
    let picker_color = picker.map(|p| p.color).unwrap_or(Color::WHITE);

    let banner_size = 32;
    let banner_y = (sh * 0.25) as i32;

    let (suffix, suffix_color) = if chosen_card.is_some() {
        let slot = chosen_card.unwrap() as usize;
        let card_id = offered_cards.get(slot).copied().unwrap_or(0) as usize;
        let card_def = CARD_CATALOG.get(card_id);
        let card_name = card_def.map(|c| c.name).unwrap_or("???");
        let (cr, cg, cb) = card_def.map(|c| c.color).unwrap_or((255, 255, 255));
        (format!(" took {}", card_name), Color::new(cr, cg, cb, 255))
    } else {
        (" is choosing...".to_string(), Color::WHITE)
    };

    let name_w = d.measure_text(picker_name, banner_size);
    let suffix_w = d.measure_text(&suffix, banner_size);
    let total_w = name_w + suffix_w;
    let start_x = screen_w / 2 - total_w / 2;

    d.draw_text(picker_name, start_x, banner_y, banner_size, picker_color);
    d.draw_text(&suffix, start_x + name_w, banner_y, banner_size, suffix_color);

    // Draw the 3 cards
    let rects = card_rects(sw, sh);
    let offscreen_y = sh + 50.0;

    for i in 0..3u8 {
        let card_id = offered_cards[i as usize] as usize;
        let card_def = match CARD_CATALOG.get(card_id) {
            Some(c) => c,
            None => continue,
        };

        let (base_x, base_y) = rects[i as usize];

        // Entrance: slide up from below, staggered per card
        let stagger_sec = i as f32 * 0.12;
        let card_elapsed = (anim.entrance_t - stagger_sec).max(0.0);
        let card_dur = ENTRANCE_DURATION - stagger_sec;
        let card_entrance = if card_dur > 0.0 { (card_elapsed / card_dur).min(1.0) } else { 1.0 };
        let entrance_ease = ease_out(card_entrance);
        let mut card_y = offscreen_y + (base_y - offscreen_y) * entrance_ease;

        // Scale and offset for hover/chosen/unchosen
        let mut scale = 1.0_f32;
        let mut alpha = 255_u8;

        let is_hovered = anim.hover_index == Some(i);
        let is_chosen = chosen_card == Some(i);

        if chosen_card.is_some() {
            let progress = (anim.slam_t / EXIT_DURATION).clamp(0.0, 1.0);
            if is_chosen {
                // Slam: scale bump then settle
                scale = 1.0 + 0.15 * (1.0 - progress).max(0.0);
            } else {
                // Unchosen: shrink + fade
                scale = 1.0 - progress * 0.6;
                alpha = (255.0 * (1.0 - progress).max(0.0)) as u8;
            }
        } else if is_hovered {
            scale = 1.08;
            card_y -= 15.0;
        }

        if alpha == 0 { continue; }

        let scaled_w = CARD_W * scale;
        let scaled_h = CARD_H * scale;
        let draw_x = base_x + (CARD_W - scaled_w) / 2.0;
        let draw_y = card_y + (CARD_H - scaled_h) / 2.0;

        // Hover glow
        if is_hovered && chosen_card.is_none() {
            let picker_color = world.players.get(current_picker as usize)
                .map(|p| p.color)
                .unwrap_or(Color::WHITE);
            let glow = Color::new(picker_color.r, picker_color.g, picker_color.b, 40);
            d.draw_rectangle(
                (draw_x - 6.0) as i32, (draw_y - 6.0) as i32,
                (scaled_w + 12.0) as i32, (scaled_h + 12.0) as i32,
                glow,
            );
        }

        // Card background (dark, theme-tinted)
        let (cr, cg, cb) = card_def.color;
        let bg = Color::new(cr / 8 + 15, cg / 8 + 15, cb / 8 + 15, alpha);
        d.draw_rectangle(draw_x as i32, draw_y as i32, scaled_w as i32, scaled_h as i32, bg);

        // Inner bevel (slightly lighter inset)
        let inset = 6.0 * scale;
        let inner_bg = Color::new(cr / 6 + 25, cg / 6 + 25, cb / 6 + 25, alpha);
        d.draw_rectangle(
            (draw_x + inset) as i32, (draw_y + inset) as i32,
            (scaled_w - inset * 2.0) as i32, (scaled_h - inset * 2.0) as i32,
            inner_bg,
        );

        // Border
        let border = Color::new(cr / 2 + 60, cg / 2 + 60, cb / 2 + 60, alpha);
        d.draw_rectangle_lines_ex(
            Rectangle::new(draw_x, draw_y, scaled_w, scaled_h),
            2.0 * scale,
            border,
        );

        // Icon glyph (large, centered)
        let glyph = format!("{}", card_def.icon_glyph);
        let glyph_size = (64.0 * scale) as i32;
        let glyph_w = d.measure_text(&glyph, glyph_size);
        let glyph_x = draw_x as i32 + scaled_w as i32 / 2 - glyph_w / 2;
        let glyph_y = (draw_y + scaled_h * 0.18) as i32;
        let glyph_color = Color::new(cr, cg, cb, alpha);
        d.draw_text(&glyph, glyph_x, glyph_y, glyph_size, glyph_color);

        // Card name
        let name_size = (22.0 * scale) as i32;
        let name_w = d.measure_text(card_def.name, name_size);
        let name_x = draw_x as i32 + scaled_w as i32 / 2 - name_w / 2;
        let name_y = (draw_y + scaled_h * 0.55) as i32;
        let name_color = Color::new(240, 240, 240, alpha);
        d.draw_text(card_def.name, name_x, name_y, name_size, name_color);

        // Description
        let desc_size = (16.0 * scale) as i32;
        let desc_w = d.measure_text(card_def.description, desc_size);
        let desc_x = draw_x as i32 + scaled_w as i32 / 2 - desc_w / 2;
        let desc_y = (draw_y + scaled_h * 0.68) as i32;
        let desc_color = Color::new(180, 180, 180, alpha);
        d.draw_text(card_def.description, desc_x, desc_y, desc_size, desc_color);

        // Chosen flash overlay
        let slam_progress = (anim.slam_t / EXIT_DURATION).clamp(0.0, 1.0);
        if is_chosen && slam_progress < 0.5 {
            let flash_alpha = ((1.0 - slam_progress / 0.5) * 120.0) as u8;
            d.draw_rectangle(
                draw_x as i32, draw_y as i32,
                scaled_w as i32, scaled_h as i32,
                Color::new(255, 255, 255, flash_alpha),
            );
        }
    }

    // Toast (fading text after pick)
    if anim.toast_timer > 0.0 && !anim.toast_text.is_empty() {
        let ta = (anim.toast_timer * 255.0 / 1.5).min(255.0) as u8;
        let toast_size = 24;
        let tw = d.measure_text(&anim.toast_text, toast_size);
        d.draw_text(
            &anim.toast_text,
            screen_w / 2 - tw / 2,
            (sh * 0.88) as i32,
            toast_size,
            Color::new(255, 255, 255, ta),
        );
    }
}

pub fn draw_match_over(
    d: &mut RaylibDrawHandle,
    world: &World,
    screen_w: i32,
    screen_h: i32,
) {
    if let GameState::MatchOver { winner_index, timer } = &world.state {
        // Dim overlay
        d.draw_rectangle(0, 0, screen_w, screen_h, Color::new(0, 0, 0, 180));

        let winner = world.players.get(*winner_index as usize);
        let name = winner.map(|p| p.name.as_str()).unwrap_or("???");
        let color = winner.map(|p| p.color).unwrap_or(Color::WHITE);

        // Big win text
        let text = format!("{} WINS THE MATCH!", name);
        let font_size = 72;
        let text_w = d.measure_text(&text, font_size);
        d.draw_text(
            &text,
            screen_w / 2 - text_w / 2,
            screen_h / 2 - font_size - 20,
            font_size,
            color,
        );

        // Final scores
        let score_font = 28;
        let mut score_parts: Vec<String> = Vec::new();
        for (i, player) in world.players.iter().enumerate() {
            score_parts.push(format!("{}: {}", player.name, world.scores[i]));
        }
        let score_text = score_parts.join("    ");
        let score_w = d.measure_text(&score_text, score_font);
        d.draw_text(
            &score_text,
            screen_w / 2 - score_w / 2,
            screen_h / 2 + 30,
            score_font,
            Color::new(200, 200, 200, 220),
        );

        // "Press ESC" hint
        if *timer <= 2.0 {
            let hint = "Press ESC to return to menu";
            let hint_size = 20;
            let hint_w = d.measure_text(hint, hint_size);
            d.draw_text(
                hint,
                screen_w / 2 - hint_w / 2,
                screen_h / 2 + 80,
                hint_size,
                Color::new(150, 150, 150, 180),
            );
        }
    }
}
