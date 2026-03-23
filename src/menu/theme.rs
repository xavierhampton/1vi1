use raylib::prelude::*;

/// All menu visuals in one place. Swap the preset to re-skin the entire menu.
pub struct Theme {
    pub name: &'static str,

    // Background
    pub bg: Color,
    pub bg_grid_color: Color,
    pub bg_grid_spacing: f32,
    pub bg_grid_alpha: u8,

    // Title
    pub title_color: Color,
    pub title_shadow_color: Color,
    pub title_size: i32,
    pub title_shadow_offset: i32,
    pub title_y_ratio: f32,

    // Subtitle
    pub subtitle_color: Color,
    pub subtitle_size: i32,

    // Menu items
    pub item_color: Color,
    pub item_hover_color: Color,
    pub item_size: i32,
    pub item_spacing: i32,
    pub item_y_start_ratio: f32,

    // Selection indicator
    pub selector_color: Color,
    pub selector_width: i32,
    pub selector_gap: i32,

    // Accent line
    pub accent_color: Color,
    pub accent_height: i32,

    // Footer
    pub footer_color: Color,
    pub footer_size: i32,

    // Animation
    pub hover_slide_speed: f32,
    pub pulse_speed: f32,

    // Particles
    pub particle_color_primary: Color,

    // In-game colors (derived from theme palette)
    pub game_bg: Color,
    pub game_wall_color: Color,
    pub game_platform_color: Color,
    pub game_wire_color: Color,
}

// Helper to build a theme from just a few accent colors + bg
fn make_theme(
    name: &'static str,
    bg: Color,
    accent: Color,
    text: Color,
    text_dim: Color,
    subtitle: Color,
) -> Theme {
    Theme {
        name,
        bg,
        bg_grid_color: accent,
        bg_grid_spacing: 40.0,
        bg_grid_alpha: 18,

        title_color: text,
        title_shadow_color: Color::new(accent.r, accent.g, accent.b, 80),
        title_size: 100,
        title_shadow_offset: 4,
        title_y_ratio: 0.18,

        subtitle_color: subtitle,
        subtitle_size: 22,

        item_color: text_dim,
        item_hover_color: text,
        item_size: 36,
        item_spacing: 60,
        item_y_start_ratio: 0.48,

        selector_color: accent,
        selector_width: 4,
        selector_gap: 16,

        accent_color: Color::new(accent.r, accent.g, accent.b, 100),
        accent_height: 2,

        footer_color: Color::new(text_dim.r / 2, text_dim.g / 2, text_dim.b / 2, 255),
        footer_size: 16,

        hover_slide_speed: 12.0,
        pulse_speed: 3.0,

        particle_color_primary: accent,

        // Game colors: walls and platforms are tinted toward the accent
        game_bg: bg,
        game_wall_color: Color::new(
            (bg.r as u16 + accent.r as u16 / 5).min(255) as u8 + 30,
            (bg.g as u16 + accent.g as u16 / 5).min(255) as u8 + 30,
            (bg.b as u16 + accent.b as u16 / 5).min(255) as u8 + 30,
            255,
        ),
        game_platform_color: Color::new(
            (bg.r as u16 + accent.r as u16 / 8).min(255) as u8 + 18,
            (bg.g as u16 + accent.g as u16 / 8).min(255) as u8 + 18,
            (bg.b as u16 + accent.b as u16 / 8).min(255) as u8 + 18,
            255,
        ),
        game_wire_color: Color::new(
            (accent.r / 3).saturating_add(60),
            (accent.g / 3).saturating_add(60),
            (accent.b / 3).saturating_add(60),
            255,
        ),
    }
}

pub const THEME_COUNT: usize = 16;

pub fn all_themes() -> [Theme; THEME_COUNT] {
    [
        // 0: Terminal (default)
        make_theme(
            "terminal",
            Color::new(0, 0, 0, 255),
            Color::new(0, 255, 65, 255),
            Color::new(0, 255, 65, 255),
            Color::new(0, 160, 40, 255),
            Color::new(0, 120, 30, 255),
        ),
        // 1: Crimson
        make_theme(
            "crimson",
            Color::new(18, 8, 10, 255),
            Color::new(220, 50, 50, 255),
            Color::new(255, 230, 230, 255),
            Color::new(180, 140, 140, 255),
            Color::new(150, 100, 100, 255),
        ),
        // 2: Toxic
        make_theme(
            "toxic",
            Color::new(8, 16, 8, 255),
            Color::new(80, 255, 80, 255),
            Color::new(220, 255, 220, 255),
            Color::new(130, 180, 130, 255),
            Color::new(90, 150, 90, 255),
        ),
        // 3: Sunset
        make_theme(
            "sunset",
            Color::new(20, 12, 10, 255),
            Color::new(255, 140, 50, 255),
            Color::new(255, 240, 220, 255),
            Color::new(190, 160, 140, 255),
            Color::new(160, 120, 90, 255),
        ),
        // 4: Violet
        make_theme(
            "violet",
            Color::new(14, 8, 20, 255),
            Color::new(180, 80, 255, 255),
            Color::new(240, 230, 255, 255),
            Color::new(170, 150, 190, 255),
            Color::new(130, 100, 160, 255),
        ),
        // 5: Arctic
        make_theme(
            "arctic",
            Color::new(10, 15, 20, 255),
            Color::new(140, 220, 255, 255),
            Color::new(230, 245, 255, 255),
            Color::new(160, 190, 200, 255),
            Color::new(120, 160, 180, 255),
        ),
        // 6: Gold
        make_theme(
            "gold",
            Color::new(16, 14, 8, 255),
            Color::new(255, 200, 50, 255),
            Color::new(255, 245, 220, 255),
            Color::new(190, 175, 130, 255),
            Color::new(160, 140, 90, 255),
        ),
        // 7: Sakura
        make_theme(
            "sakura",
            Color::new(18, 10, 14, 255),
            Color::new(255, 130, 170, 255),
            Color::new(255, 235, 240, 255),
            Color::new(190, 155, 165, 255),
            Color::new(160, 120, 135, 255),
        ),
        // 8: Monochrome
        make_theme(
            "monochrome",
            Color::new(10, 10, 10, 255),
            Color::new(200, 200, 200, 255),
            Color::new(240, 240, 240, 255),
            Color::new(140, 140, 140, 255),
            Color::new(100, 100, 100, 255),
        ),
        // 9: Ocean
        make_theme(
            "ocean",
            Color::new(6, 12, 18, 255),
            Color::new(30, 144, 200, 255),
            Color::new(200, 230, 250, 255),
            Color::new(120, 160, 185, 255),
            Color::new(80, 130, 160, 255),
        ),
        // 10: Midnight
        make_theme(
            "midnight",
            Color::new(12, 12, 18, 255),
            Color::new(80, 180, 255, 255),
            Color::new(240, 240, 250, 255),
            Color::new(160, 160, 175, 255),
            Color::new(120, 120, 140, 255),
        ),
        // 11: Lava
        make_theme(
            "lava",
            Color::new(20, 6, 2, 255),
            Color::new(255, 80, 20, 255),
            Color::new(255, 220, 180, 255),
            Color::new(200, 130, 90, 255),
            Color::new(170, 90, 50, 255),
        ),
        // 12: Frost
        make_theme(
            "frost",
            Color::new(15, 18, 22, 255),
            Color::new(100, 200, 240, 255),
            Color::new(220, 240, 255, 255),
            Color::new(150, 180, 200, 255),
            Color::new(110, 140, 170, 255),
        ),
        // 13: Bubblegum
        make_theme(
            "bubblegum",
            Color::new(20, 10, 18, 255),
            Color::new(255, 100, 200, 255),
            Color::new(255, 230, 245, 255),
            Color::new(200, 150, 180, 255),
            Color::new(170, 110, 150, 255),
        ),
        // 14: Wasabi
        make_theme(
            "wasabi",
            Color::new(12, 16, 10, 255),
            Color::new(180, 220, 60, 255),
            Color::new(240, 250, 220, 255),
            Color::new(165, 180, 130, 255),
            Color::new(130, 150, 90, 255),
        ),
        // 15: Ember
        make_theme(
            "ember",
            Color::new(14, 10, 8, 255),
            Color::new(240, 160, 80, 255),
            Color::new(255, 235, 210, 255),
            Color::new(185, 155, 130, 255),
            Color::new(150, 120, 90, 255),
        ),
    ]
}
