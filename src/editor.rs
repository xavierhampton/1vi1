use raylib::prelude::*;

use crate::level::level::{load_all_levels, save_all_levels, LevelDef, PlatformDef, BouncePadDef, LavaPoolDef, LaserBeamDef};

// ── Constants ───────────────────────────────────────────────────────────────

const GRID_SIZE: f32 = 0.5;
const SPAWN_RADIUS: f32 = 0.3;
const REQUIRED_SPAWNS: usize = 4;

const BG_COLOR: Color = Color::new(24, 24, 30, 255);
const GRID_COLOR: Color = Color::new(50, 50, 60, 255);
const GRID_ORIGIN_COLOR: Color = Color::new(80, 80, 100, 255);
const WALL_COLOR: Color = Color::new(100, 120, 160, 255);
const WALL_HOVER_COLOR: Color = Color::new(130, 150, 190, 255);
const PLATFORM_COLOR: Color = Color::new(80, 180, 120, 255);
const PLATFORM_HOVER_COLOR: Color = Color::new(110, 210, 150, 255);
const SPAWN_COLORS: [Color; 4] = [
    Color::new(80, 140, 255, 255),  // blue
    Color::new(255, 80, 80, 255),   // red
    Color::new(80, 220, 80, 255),   // green
    Color::new(255, 220, 60, 255),  // yellow
];
const SIDEBAR_BG: Color = Color::new(18, 18, 24, 255);
const SIDEBAR_WIDTH: i32 = 220;
const TEXT_COLOR: Color = Color::new(220, 220, 230, 255);
const DIM_TEXT: Color = Color::new(140, 140, 150, 255);
const SELECTED_BG: Color = Color::new(50, 50, 70, 255);
const BTN_COLOR: Color = Color::new(40, 40, 55, 255);
const BTN_HOVER: Color = Color::new(60, 60, 80, 255);
const DELETE_COLOR: Color = Color::new(180, 50, 50, 255);
const DELETE_HOVER: Color = Color::new(220, 70, 70, 255);
const PAD_COLOR: Color = Color::new(0, 200, 255, 255);
const PAD_HOVER_COLOR: Color = Color::new(80, 230, 255, 255);
const LAVA_COLOR: Color = Color::new(220, 60, 10, 255);
const LAVA_HOVER_COLOR: Color = Color::new(255, 100, 40, 255);
const LASER_COLOR: Color = Color::new(255, 40, 40, 255);
const LASER_HOVER_COLOR: Color = Color::new(255, 100, 100, 255);

// ── Tool ────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
enum Tool {
    Platform,
    Wall,
    Spawn,
    BouncePad,
    Lava,
    Laser,
    Erase,
}

impl Tool {
    fn label(&self) -> &'static str {
        match self {
            Tool::Platform => "Platform",
            Tool::Wall => "Wall",
            Tool::Spawn => "Spawn",
            Tool::BouncePad => "Bounce",
            Tool::Lava => "Lava",
            Tool::Laser => "Laser",
            Tool::Erase => "Erase",
        }
    }
    fn key_hint(&self) -> &'static str {
        match self {
            Tool::Platform => "[1]",
            Tool::Wall => "[2]",
            Tool::Spawn => "[3]",
            Tool::BouncePad => "[4]",
            Tool::Lava => "[5]",
            Tool::Laser => "[6]",
            Tool::Erase => "[7]",
        }
    }
}

// ── Editor State ────────────────────────────────────────────────────────────

pub struct Editor {
    levels: Vec<LevelDef>,
    current: usize,
    tool: Tool,
    camera_x: f32,
    camera_y: f32,
    zoom: f32,
    // Drag-to-create platform/wall
    drag_start: Option<[f32; 2]>,
    // Pan state
    panning: bool,
    pan_start_mouse: Vector2,
    pan_start_cam: [f32; 2],
    // Current spawn being placed (0-3)
    placing_spawn: usize,
    // Sidebar scroll
    list_scroll: i32,
    // Confirmation for delete
    confirm_delete: bool,
    // Bounce pad drag start (drag-to-create like platforms)
    pad_drag_start: Option<[f32; 2]>,
    // Lava pool drag start
    lava_drag_start: Option<[f32; 2]>,
    // Laser: first click sets start, second click sets end
    laser_start: Option<[f32; 2]>,
    // Dirty flag (unsaved changes)
    dirty: bool,
    // Status message
    status: String,
    status_timer: f32,
    // Name editing
    editing_name: bool,
    name_buf: String,
}

impl Editor {
    pub fn new() -> Self {
        let levels = load_all_levels();
        Self {
            current: 0,
            levels,
            tool: Tool::Platform,
            camera_x: 0.0,
            camera_y: 4.0,
            zoom: 30.0,
            drag_start: None,
            panning: false,
            pan_start_mouse: Vector2::zero(),
            pan_start_cam: [0.0, 0.0],
            placing_spawn: 0,
            pad_drag_start: None,
            lava_drag_start: None,
            laser_start: None,
            list_scroll: 0,
            confirm_delete: false,
            dirty: false,
            status: String::new(),
            status_timer: 0.0,
            editing_name: false,
            name_buf: String::new(),
        }
    }

    fn set_status(&mut self, msg: &str) {
        self.status = msg.to_string();
        self.status_timer = 3.0;
    }

    fn snap(v: f32) -> f32 {
        (v / GRID_SIZE).round() * GRID_SIZE
    }

    fn screen_to_world(&self, sx: f32, sy: f32, sw: i32, sh: i32) -> [f32; 2] {
        let canvas_w = sw - SIDEBAR_WIDTH;
        let cx = canvas_w as f32 / 2.0;
        let cy = sh as f32 / 2.0;
        let wx = (sx - cx) / self.zoom + self.camera_x;
        let wy = -(sy - cy) / self.zoom + self.camera_y; // Y is flipped
        [wx, wy]
    }

    fn world_to_screen(&self, wx: f32, wy: f32, sw: i32, sh: i32) -> (f32, f32) {
        let canvas_w = sw - SIDEBAR_WIDTH;
        let cx = canvas_w as f32 / 2.0;
        let cy = sh as f32 / 2.0;
        let sx = (wx - self.camera_x) * self.zoom + cx;
        let sy = -(wy - self.camera_y) * self.zoom + cy;
        (sx, sy)
    }

    /// Returns true when ESC is pressed (signal to quit).
    pub fn update(&mut self, rl: &mut RaylibHandle, dt: f32) -> bool {
        self.status_timer -= dt;

        let sw = rl.get_screen_width();
        let sh = rl.get_screen_height();
        let mx = rl.get_mouse_x() as f32;
        let my = rl.get_mouse_y() as f32;
        let in_canvas = mx < (sw - SIDEBAR_WIDTH) as f32;

        // ── Name editing ────────────────────────────────────────────────
        if self.editing_name {
            let ch = rl.get_char_pressed();
            if let Some(c) = ch {
                if c as u32 >= 32 && self.name_buf.len() < 24 {
                    self.name_buf.push(c);
                }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) && !self.name_buf.is_empty() {
                self.name_buf.pop();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                if !self.name_buf.is_empty() {
                    if let Some(lev) = self.levels.get_mut(self.current) {
                        lev.name = self.name_buf.clone();
                        self.dirty = true;
                    }
                }
                self.editing_name = false;
            }
            return false;
        }

        // ── Keyboard shortcuts ──────────────────────────────────────────
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            if self.confirm_delete {
                self.confirm_delete = false;
            } else if self.dirty {
                // Prompt to save? For now just exit.
                return true;
            } else {
                return true;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_ONE) { self.tool = Tool::Platform; }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) { self.tool = Tool::Wall; }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) { self.tool = Tool::Spawn; }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) { self.tool = Tool::BouncePad; }
        if rl.is_key_pressed(KeyboardKey::KEY_FIVE) { self.tool = Tool::Lava; }
        if rl.is_key_pressed(KeyboardKey::KEY_SIX) { self.tool = Tool::Laser; }
        if rl.is_key_pressed(KeyboardKey::KEY_SEVEN) { self.tool = Tool::Erase; }

        // Ctrl+S to save
        if (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
            && rl.is_key_pressed(KeyboardKey::KEY_S)
        {
            self.save();
        }

        // Ctrl+Z undo last platform (simple: remove last added platform)
        if (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
            && rl.is_key_pressed(KeyboardKey::KEY_Z)
        {
            if let Some(lev) = self.levels.get_mut(self.current) {
                if !lev.platforms.is_empty() {
                    lev.platforms.pop();
                    self.dirty = true;
                    self.set_status("Undone last platform");
                }
            }
        }

        // ── Zoom ────────────────────────────────────────────────────────
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 && in_canvas {
            self.zoom = (self.zoom * (1.0 + wheel * 0.1)).clamp(5.0, 200.0);
        }

        // ── Pan (middle mouse or right mouse) ──────────────────────────
        if in_canvas && (rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_MIDDLE)
            || (rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) && self.tool != Tool::Erase))
        {
            self.panning = true;
            self.pan_start_mouse = Vector2::new(mx, my);
            self.pan_start_cam = [self.camera_x, self.camera_y];
        }
        if self.panning {
            let dx = mx - self.pan_start_mouse.x;
            let dy = my - self.pan_start_mouse.y;
            self.camera_x = self.pan_start_cam[0] - dx / self.zoom;
            self.camera_y = self.pan_start_cam[1] + dy / self.zoom;
        }
        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_MIDDLE)
            || rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_RIGHT)
        {
            self.panning = false;
        }

        // ── Canvas interactions ─────────────────────────────────────────
        if in_canvas && !self.panning && !self.levels.is_empty() {
            let [wx, wy] = self.screen_to_world(mx, my, sw, sh);
            let snapped = [Self::snap(wx), Self::snap(wy)];

            match self.tool {
                Tool::Platform | Tool::Wall => {
                    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                        self.drag_start = Some(snapped);
                    }
                    if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                        if let Some(start) = self.drag_start.take() {
                            let end = snapped;
                            let min_x = start[0].min(end[0]);
                            let min_y = start[1].min(end[1]);
                            let max_x = start[0].max(end[0]);
                            let max_y = start[1].max(end[1]);
                            // Only create if there's actual area
                            if (max_x - min_x).abs() >= GRID_SIZE * 0.5
                                && (max_y - min_y).abs() >= GRID_SIZE * 0.5
                            {
                                let kind = if self.tool == Tool::Wall { "wall" } else { "platform" };
                                if let Some(lev) = self.levels.get_mut(self.current) {
                                    lev.platforms.push(PlatformDef {
                                        kind: kind.to_string(),
                                        min: [min_x, min_y],
                                        max: [max_x, max_y],
                                    });
                                    self.dirty = true;
                                }
                            }
                        }
                    }
                }
                Tool::Spawn => {
                    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                        if let Some(lev) = self.levels.get_mut(self.current) {
                            let pos = [Self::snap(wx), Self::snap(wy)];
                            if lev.spawn_points.len() < REQUIRED_SPAWNS {
                                lev.spawn_points.push(pos);
                                self.placing_spawn = lev.spawn_points.len();
                            } else {
                                // Replace the next spawn in rotation
                                let idx = self.placing_spawn % REQUIRED_SPAWNS;
                                lev.spawn_points[idx] = pos;
                                self.placing_spawn = idx + 1;
                            }
                            self.dirty = true;
                        }
                    }
                }
                Tool::BouncePad => {
                    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                        self.pad_drag_start = Some(snapped);
                    }
                    if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                        if let Some(start) = self.pad_drag_start.take() {
                            let end = snapped;
                            let min_x = start[0].min(end[0]);
                            let min_y = start[1].min(end[1]);
                            let max_x = start[0].max(end[0]);
                            let max_y = start[1].max(end[1]);
                            if (max_x - min_x).abs() >= GRID_SIZE * 0.5
                                && (max_y - min_y).abs() >= GRID_SIZE * 0.5
                            {
                                if let Some(lev) = self.levels.get_mut(self.current) {
                                    lev.bounce_pads.push(BouncePadDef {
                                        min: [min_x, min_y],
                                        max: [max_x, max_y],
                                        strength: 25.0,
                                    });
                                    self.dirty = true;
                                }
                            }
                        }
                    }
                }
                Tool::Lava => {
                    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                        self.lava_drag_start = Some(snapped);
                    }
                    if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                        if let Some(start) = self.lava_drag_start.take() {
                            let end = snapped;
                            let min_x = start[0].min(end[0]);
                            let min_y = start[1].min(end[1]);
                            let max_x = start[0].max(end[0]);
                            let max_y = start[1].max(end[1]);
                            if (max_x - min_x).abs() >= GRID_SIZE * 0.5
                                && (max_y - min_y).abs() >= GRID_SIZE * 0.5
                            {
                                if let Some(lev) = self.levels.get_mut(self.current) {
                                    lev.lava_pools.push(LavaPoolDef {
                                        min: [min_x, min_y],
                                        max: [max_x, max_y],
                                        dps: 40.0,
                                    });
                                    self.dirty = true;
                                }
                            }
                        }
                    }
                }
                Tool::Laser => {
                    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                        if let Some(start) = self.laser_start.take() {
                            // Second click: create the laser
                            let end = [Self::snap(wx), Self::snap(wy)];
                            if (start[0] - end[0]).abs() > GRID_SIZE * 0.5
                                || (start[1] - end[1]).abs() > GRID_SIZE * 0.5
                            {
                                if let Some(lev) = self.levels.get_mut(self.current) {
                                    lev.lasers.push(LaserBeamDef {
                                        start,
                                        end,
                                        on_time: 2.0,
                                        off_time: 2.0,
                                    });
                                    self.dirty = true;
                                }
                            }
                        } else {
                            // First click: set start point
                            self.laser_start = Some(snapped);
                        }
                    }
                }
                Tool::Erase => {
                    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                        if let Some(lev) = self.levels.get_mut(self.current) {
                            // Find platform under cursor and remove it
                            let mut found_plat = None;
                            for (i, p) in lev.platforms.iter().enumerate().rev() {
                                if wx >= p.min[0] && wx <= p.max[0]
                                    && wy >= p.min[1] && wy <= p.max[1]
                                {
                                    found_plat = Some(i);
                                    break;
                                }
                            }
                            if let Some(i) = found_plat {
                                lev.platforms.remove(i);
                                self.dirty = true;
                            } else {
                                // Check bounce pads (AABB hit test)
                                let mut found_pad = None;
                                for (i, b) in lev.bounce_pads.iter().enumerate().rev() {
                                    if wx >= b.min[0] && wx <= b.max[0]
                                        && wy >= b.min[1] && wy <= b.max[1]
                                    {
                                        found_pad = Some(i);
                                        break;
                                    }
                                }
                                if let Some(i) = found_pad {
                                    lev.bounce_pads.remove(i);
                                    self.dirty = true;
                                } else {
                                    // Check lava pools
                                    let mut found_lava = None;
                                    for (i, l) in lev.lava_pools.iter().enumerate().rev() {
                                        if wx >= l.min[0] && wx <= l.max[0]
                                            && wy >= l.min[1] && wy <= l.max[1]
                                        {
                                            found_lava = Some(i);
                                            break;
                                        }
                                    }
                                    if let Some(i) = found_lava {
                                        lev.lava_pools.remove(i);
                                        self.dirty = true;
                                    } else {
                                        // Check lasers (near either endpoint)
                                        let mut found_laser = None;
                                        for (i, l) in lev.lasers.iter().enumerate().rev() {
                                            let d0 = ((wx - l.start[0]).powi(2) + (wy - l.start[1]).powi(2)).sqrt();
                                            let d1 = ((wx - l.end[0]).powi(2) + (wy - l.end[1]).powi(2)).sqrt();
                                            if d0 < 0.5 || d1 < 0.5 {
                                                found_laser = Some(i);
                                                break;
                                            }
                                        }
                                        if let Some(i) = found_laser {
                                            lev.lasers.remove(i);
                                            self.dirty = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Right-click to erase spawn points
                    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
                        if let Some(lev) = self.levels.get_mut(self.current) {
                            let mut found = None;
                            for (i, s) in lev.spawn_points.iter().enumerate().rev() {
                                let dx = wx - s[0];
                                let dy = wy - s[1];
                                if (dx * dx + dy * dy).sqrt() < 0.6 {
                                    found = Some(i);
                                    break;
                                }
                            }
                            if let Some(i) = found {
                                lev.spawn_points.remove(i);
                                self.dirty = true;
                            }
                        }
                    }
                }
            }
        }

        // ── Sidebar buttons ─────────────────────────────────────────────
        // Handled in draw (we return button presses via checking mouse)
        // This is done in draw() to keep layout logic in one place.

        false
    }

    fn save(&mut self) {
        // Validate all levels have exactly 4 spawn points
        for (_i, lev) in self.levels.iter().enumerate() {
            if lev.spawn_points.len() != REQUIRED_SPAWNS {
                self.set_status(&format!(
                    "ERROR: '{}' has {} spawns (need {})",
                    lev.name,
                    lev.spawn_points.len(),
                    REQUIRED_SPAWNS
                ));
                return;
            }
        }
        save_all_levels(&self.levels);
        self.dirty = false;
        self.set_status("Saved!");
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        let sw = d.get_screen_width();
        let sh = d.get_screen_height();
        let canvas_w = sw - SIDEBAR_WIDTH;

        d.clear_background(BG_COLOR);

        // ── Draw grid ───────────────────────────────────────────────────
        self.draw_grid(d, canvas_w, sh);

        // ── Draw current level ──────────────────────────────────────────
        if let Some(level) = self.levels.get(self.current).cloned() {
            self.draw_level_content(d, &level, canvas_w, sh, true);
        }

        // ── Draw drag preview ───────────────────────────────────────────
        if let Some(start) = self.drag_start {
            let mx = d.get_mouse_x() as f32;
            let my = d.get_mouse_y() as f32;
            let [wx, wy] = self.screen_to_world(mx, my, sw, sh);
            let end = [Self::snap(wx), Self::snap(wy)];
            let min_x = start[0].min(end[0]);
            let min_y = start[1].min(end[1]);
            let max_x = start[0].max(end[0]);
            let max_y = start[1].max(end[1]);

            let (sx1, sy1) = self.world_to_screen(min_x, max_y, sw, sh);
            let (sx2, sy2) = self.world_to_screen(max_x, min_y, sw, sh);
            let rw = sx2 - sx1;
            let rh = sy2 - sy1;
            let col = if self.tool == Tool::Wall {
                Color::new(WALL_COLOR.r, WALL_COLOR.g, WALL_COLOR.b, 100)
            } else {
                Color::new(PLATFORM_COLOR.r, PLATFORM_COLOR.g, PLATFORM_COLOR.b, 100)
            };
            d.draw_rectangle(sx1 as i32, sy1 as i32, rw as i32, rh as i32, col);
            d.draw_rectangle_lines(sx1 as i32, sy1 as i32, rw as i32, rh as i32, Color::WHITE);
        }

        // ── Draw drag preview (bounce pads) ──────────────────────────
        if let Some(start) = self.pad_drag_start {
            let mx = d.get_mouse_x() as f32;
            let my = d.get_mouse_y() as f32;
            let [wx, wy] = self.screen_to_world(mx, my, sw, sh);
            let end = [Self::snap(wx), Self::snap(wy)];
            let min_x = start[0].min(end[0]);
            let min_y = start[1].min(end[1]);
            let max_x = start[0].max(end[0]);
            let max_y = start[1].max(end[1]);

            let (sx1, sy1) = self.world_to_screen(min_x, max_y, sw, sh);
            let (sx2, sy2) = self.world_to_screen(max_x, min_y, sw, sh);
            let rw = sx2 - sx1;
            let rh = sy2 - sy1;
            let col = Color::new(PAD_COLOR.r, PAD_COLOR.g, PAD_COLOR.b, 100);
            d.draw_rectangle(sx1 as i32, sy1 as i32, rw as i32, rh as i32, col);
            d.draw_rectangle_lines(sx1 as i32, sy1 as i32, rw as i32, rh as i32, Color::new(0, 255, 255, 200));
        }

        // ── Draw drag preview (lava pools) ───────────────────────────
        if let Some(start) = self.lava_drag_start {
            let mx = d.get_mouse_x() as f32;
            let my = d.get_mouse_y() as f32;
            let [wx, wy] = self.screen_to_world(mx, my, sw, sh);
            let end = [Self::snap(wx), Self::snap(wy)];
            let min_x = start[0].min(end[0]);
            let min_y = start[1].min(end[1]);
            let max_x = start[0].max(end[0]);
            let max_y = start[1].max(end[1]);

            let (sx1, sy1) = self.world_to_screen(min_x, max_y, sw, sh);
            let (sx2, sy2) = self.world_to_screen(max_x, min_y, sw, sh);
            let rw = sx2 - sx1;
            let rh = sy2 - sy1;
            let col = Color::new(LAVA_COLOR.r, LAVA_COLOR.g, LAVA_COLOR.b, 100);
            d.draw_rectangle(sx1 as i32, sy1 as i32, rw as i32, rh as i32, col);
            d.draw_rectangle_lines(sx1 as i32, sy1 as i32, rw as i32, rh as i32, Color::new(255, 80, 20, 200));
        }

        // ── Sidebar ─────────────────────────────────────────────────────
        self.draw_sidebar(d, sw, sh, canvas_w);

        // ── Top bar ─────────────────────────────────────────────────────
        self.draw_toolbar(d, canvas_w);

        // ── Status message ──────────────────────────────────────────────
        if self.status_timer > 0.0 {
            let alpha = (self.status_timer.min(1.0) * 255.0) as u8;
            let col = Color::new(255, 255, 100, alpha);
            let tw = d.measure_text(&self.status, 20);
            d.draw_text(&self.status, canvas_w / 2 - tw / 2, sh - 36, 20, col);
        }

        // ── Bottom hint ─────────────────────────────────────────────────
        let hint = if self.editing_name {
            "Type level name, press Enter to confirm"
        } else {
            "LMB: place/drag | RMB/MMB: pan | Scroll: zoom | Ctrl+S: save | Ctrl+Z: undo | ESC: quit"
        };
        d.draw_text(hint, 8, sh - 22, 14, DIM_TEXT);
    }

    fn draw_grid(&self, d: &mut RaylibDrawHandle, canvas_w: i32, sh: i32) {
        // Determine visible world range
        let [left, top] = self.screen_to_world(0.0, 0.0, canvas_w + SIDEBAR_WIDTH, sh);
        let [right, bottom] = self.screen_to_world(canvas_w as f32, sh as f32, canvas_w + SIDEBAR_WIDTH, sh);

        let grid_step = if self.zoom < 15.0 { 2.0 } else if self.zoom < 40.0 { 1.0 } else { GRID_SIZE };

        // Vertical lines
        let mut x = (left / grid_step).floor() * grid_step;
        while x <= right {
            let (sx, _) = self.world_to_screen(x, 0.0, canvas_w + SIDEBAR_WIDTH, sh);
            if sx >= 0.0 && sx < canvas_w as f32 {
                let col = if x.abs() < 0.01 { GRID_ORIGIN_COLOR } else { GRID_COLOR };
                d.draw_line(sx as i32, 0, sx as i32, sh, col);
            }
            x += grid_step;
        }

        // Horizontal lines
        let mut y = (bottom / grid_step).floor() * grid_step;
        while y <= top {
            let (_, sy) = self.world_to_screen(0.0, y, canvas_w + SIDEBAR_WIDTH, sh);
            if sy >= 0.0 && sy < sh as f32 {
                let col = if y.abs() < 0.01 { GRID_ORIGIN_COLOR } else { GRID_COLOR };
                d.draw_line(0, sy as i32, canvas_w, sy as i32, col);
            }
            y += grid_step;
        }
    }

    fn draw_level_content(
        &self,
        d: &mut RaylibDrawHandle,
        level: &LevelDef,
        canvas_w: i32,
        sh: i32,
        interactive: bool,
    ) {
        let sw = canvas_w + SIDEBAR_WIDTH;
        let mx = d.get_mouse_x() as f32;
        let my = d.get_mouse_y() as f32;
        let in_canvas = mx < canvas_w as f32;

        // Draw platforms
        for p in &level.platforms {
            let (sx1, sy1) = self.world_to_screen(p.min[0], p.max[1], sw, sh);
            let (sx2, sy2) = self.world_to_screen(p.max[0], p.min[1], sw, sh);
            let rw = (sx2 - sx1).max(1.0);
            let rh = (sy2 - sy1).max(1.0);

            let is_wall = p.kind == "wall";
            let mut hover = false;
            if interactive && in_canvas && self.tool == Tool::Erase {
                let [wx, wy] = self.screen_to_world(mx, my, sw, sh);
                if wx >= p.min[0] && wx <= p.max[0] && wy >= p.min[1] && wy <= p.max[1] {
                    hover = true;
                }
            }

            let col = match (is_wall, hover) {
                (true, false) => WALL_COLOR,
                (true, true) => WALL_HOVER_COLOR,
                (false, false) => PLATFORM_COLOR,
                (false, true) => PLATFORM_HOVER_COLOR,
            };

            d.draw_rectangle(sx1 as i32, sy1 as i32, rw as i32, rh as i32, col);

            // Border
            let border = if hover { Color::WHITE } else { Color::new(col.r / 2, col.g / 2, col.b / 2, 255) };
            d.draw_rectangle_lines(sx1 as i32, sy1 as i32, rw as i32, rh as i32, border);

            // Label for walls
            if is_wall && rw > 20.0 && rh > 14.0 {
                d.draw_text("W", sx1 as i32 + 3, sy1 as i32 + 2, 10, Color::new(255, 255, 255, 80));
            }
        }

        // Draw spawn points
        for (i, s) in level.spawn_points.iter().enumerate() {
            let (sx, sy) = self.world_to_screen(s[0], s[1], sw, sh);
            let r = (SPAWN_RADIUS * self.zoom) as i32;
            let col = SPAWN_COLORS[i % SPAWN_COLORS.len()];
            d.draw_circle(sx as i32, sy as i32, r as f32, Color::new(col.r, col.g, col.b, 120));
            d.draw_circle_lines(sx as i32, sy as i32, r as f32, col);
            let label = format!("{}", i + 1);
            let tw = d.measure_text(&label, 14);
            d.draw_text(&label, sx as i32 - tw / 2, sy as i32 - 7, 14, Color::WHITE);
        }

        // Draw bounce pads (rectangles like platforms, cyan colored)
        for (i, b) in level.bounce_pads.iter().enumerate() {
            let (sx1, sy1) = self.world_to_screen(b.min[0], b.max[1], sw, sh);
            let (sx2, sy2) = self.world_to_screen(b.max[0], b.min[1], sw, sh);
            let rw = (sx2 - sx1).max(1.0);
            let rh = (sy2 - sy1).max(1.0);

            let mut hover = false;
            if interactive && in_canvas && self.tool == Tool::Erase {
                let [cwx, cwy] = self.screen_to_world(mx, my, sw, sh);
                if cwx >= b.min[0] && cwx <= b.max[0] && cwy >= b.min[1] && cwy <= b.max[1] {
                    hover = true;
                }
            }
            let col = if hover { PAD_HOVER_COLOR } else { PAD_COLOR };

            d.draw_rectangle(sx1 as i32, sy1 as i32, rw as i32, rh as i32, Color::new(col.r, col.g, col.b, 100));
            d.draw_rectangle_lines(sx1 as i32, sy1 as i32, rw as i32, rh as i32, col);

            let label = format!("B{}", i + 1);
            let tw = d.measure_text(&label, 14);
            let cx = sx1 as i32 + rw as i32 / 2;
            let cy = sy1 as i32 + rh as i32 / 2;
            d.draw_text(&label, cx - tw / 2, cy - 7, 14, Color::WHITE);
        }

        // Draw lava pools (red rectangles)
        for (i, l) in level.lava_pools.iter().enumerate() {
            let (sx1, sy1) = self.world_to_screen(l.min[0], l.max[1], sw, sh);
            let (sx2, sy2) = self.world_to_screen(l.max[0], l.min[1], sw, sh);
            let rw = (sx2 - sx1).max(1.0);
            let rh = (sy2 - sy1).max(1.0);

            let mut hover = false;
            if interactive && in_canvas && self.tool == Tool::Erase {
                let [cwx, cwy] = self.screen_to_world(mx, my, sw, sh);
                if cwx >= l.min[0] && cwx <= l.max[0] && cwy >= l.min[1] && cwy <= l.max[1] {
                    hover = true;
                }
            }
            let col = if hover { LAVA_HOVER_COLOR } else { LAVA_COLOR };

            d.draw_rectangle(sx1 as i32, sy1 as i32, rw as i32, rh as i32, Color::new(col.r, col.g, col.b, 100));
            d.draw_rectangle_lines(sx1 as i32, sy1 as i32, rw as i32, rh as i32, col);

            let label = format!("L{}", i + 1);
            let tw = d.measure_text(&label, 14);
            let cx = sx1 as i32 + rw as i32 / 2;
            let cy = sy1 as i32 + rh as i32 / 2;
            d.draw_text(&label, cx - tw / 2, cy - 7, 14, Color::WHITE);
        }

        // Draw lasers (line between two points with emitter dots)
        for (i, l) in level.lasers.iter().enumerate() {
            let (sx0, sy0) = self.world_to_screen(l.start[0], l.start[1], sw, sh);
            let (sx1, sy1) = self.world_to_screen(l.end[0], l.end[1], sw, sh);

            let mut hover = false;
            if interactive && in_canvas && self.tool == Tool::Erase {
                let [cwx, cwy] = self.screen_to_world(mx, my, sw, sh);
                let d0 = ((cwx - l.start[0]).powi(2) + (cwy - l.start[1]).powi(2)).sqrt();
                let d1 = ((cwx - l.end[0]).powi(2) + (cwy - l.end[1]).powi(2)).sqrt();
                if d0 < 0.5 || d1 < 0.5 { hover = true; }
            }
            let col = if hover { LASER_HOVER_COLOR } else { LASER_COLOR };

            d.draw_line_ex(Vector2::new(sx0, sy0), Vector2::new(sx1, sy1), 2.0, col);
            d.draw_circle(sx0 as i32, sy0 as i32, 5.0, col);
            d.draw_circle(sx1 as i32, sy1 as i32, 5.0, col);

            let mx_l = (sx0 + sx1) * 0.5;
            let my_l = (sy0 + sy1) * 0.5;
            let label = format!("Z{}", i + 1);
            let tw = d.measure_text(&label, 14);
            d.draw_text(&label, mx_l as i32 - tw / 2, my_l as i32 - 7, 14, Color::WHITE);
        }

        // Draw cursor position indicator (spawn tool ghost)
        if interactive && in_canvas && self.tool == Tool::Spawn && self.drag_start.is_none() {
            let [wx, wy] = self.screen_to_world(mx, my, sw, sh);
            let pos = [Self::snap(wx), Self::snap(wy)];
            let (sx, sy) = self.world_to_screen(pos[0], pos[1], sw, sh);
            let r = (SPAWN_RADIUS * self.zoom) as i32;
            let next_idx = if level.spawn_points.len() < REQUIRED_SPAWNS {
                level.spawn_points.len()
            } else {
                self.placing_spawn % REQUIRED_SPAWNS
            };
            let col = SPAWN_COLORS[next_idx % SPAWN_COLORS.len()];
            d.draw_circle(sx as i32, sy as i32, r as f32, Color::new(col.r, col.g, col.b, 60));
            d.draw_circle_lines(sx as i32, sy as i32, r as f32, Color::new(col.r, col.g, col.b, 150));
        }

        // Bounce pad tool cursor crosshair (shows snapped position)
        if interactive && in_canvas && self.tool == Tool::BouncePad && self.pad_drag_start.is_none() {
            let [wx, wy] = self.screen_to_world(mx, my, sw, sh);
            let pos = [Self::snap(wx), Self::snap(wy)];
            let (gsx, gsy) = self.world_to_screen(pos[0], pos[1], sw, sh);
            let ghost = Color::new(PAD_COLOR.r, PAD_COLOR.g, PAD_COLOR.b, 120);
            d.draw_circle(gsx as i32, gsy as i32, 4.0, ghost);
        }

        // Lava tool cursor
        if interactive && in_canvas && self.tool == Tool::Lava && self.lava_drag_start.is_none() {
            let [wx, wy] = self.screen_to_world(mx, my, sw, sh);
            let pos = [Self::snap(wx), Self::snap(wy)];
            let (gsx, gsy) = self.world_to_screen(pos[0], pos[1], sw, sh);
            d.draw_circle(gsx as i32, gsy as i32, 4.0, Color::new(LAVA_COLOR.r, LAVA_COLOR.g, LAVA_COLOR.b, 120));
        }

        // Laser tool: show first point placed
        if interactive && in_canvas && self.tool == Tool::Laser {
            let [wx, wy] = self.screen_to_world(mx, my, sw, sh);
            let pos = [Self::snap(wx), Self::snap(wy)];
            let (gsx, gsy) = self.world_to_screen(pos[0], pos[1], sw, sh);
            if let Some(start) = self.laser_start {
                let (ssx, ssy) = self.world_to_screen(start[0], start[1], sw, sh);
                d.draw_circle(ssx as i32, ssy as i32, 5.0, LASER_COLOR);
                d.draw_line_ex(Vector2::new(ssx, ssy), Vector2::new(gsx, gsy), 2.0,
                    Color::new(LASER_COLOR.r, LASER_COLOR.g, LASER_COLOR.b, 100));
                d.draw_circle(gsx as i32, gsy as i32, 4.0,
                    Color::new(LASER_COLOR.r, LASER_COLOR.g, LASER_COLOR.b, 100));
            } else {
                d.draw_circle(gsx as i32, gsy as i32, 4.0,
                    Color::new(LASER_COLOR.r, LASER_COLOR.g, LASER_COLOR.b, 120));
            }
        }
    }

    fn draw_toolbar(&mut self, d: &mut RaylibDrawHandle, canvas_w: i32) {
        // Tool bar at top
        d.draw_rectangle(0, 0, canvas_w, 34, Color::new(30, 30, 40, 230));

        let mx = d.get_mouse_x();
        let my = d.get_mouse_y();
        let clicked = d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        let tools = [Tool::Platform, Tool::Wall, Tool::Spawn, Tool::BouncePad, Tool::Lava, Tool::Laser, Tool::Erase];
        let mut tx = 8;
        for t in &tools {
            let label = format!("{} {}", t.key_hint(), t.label());
            let tw = d.measure_text(&label, 16);
            let is_active = *t == self.tool;
            let hovered = mx >= tx - 4 && mx < tx + tw + 4 && my >= 4 && my < 30;
            let col = if is_active { Color::WHITE } else if hovered { Color::new(200, 200, 220, 255) } else { DIM_TEXT };
            if is_active {
                d.draw_rectangle(tx - 4, 4, tw + 8, 26, Color::new(70, 70, 90, 200));
            } else if hovered {
                d.draw_rectangle(tx - 4, 4, tw + 8, 26, Color::new(50, 50, 65, 200));
            }
            d.draw_text(&label, tx, 9, 16, col);
            if hovered && clicked {
                self.tool = *t;
            }
            tx += tw + 20;
        }

        // Dirty indicator
        if self.dirty {
            d.draw_text("*unsaved*", canvas_w - 90, 9, 16, Color::new(255, 200, 80, 200));
        }
    }

    fn draw_sidebar(&mut self, d: &mut RaylibDrawHandle, _sw: i32, sh: i32, canvas_w: i32) {
        let sx = canvas_w;
        d.draw_rectangle(sx, 0, SIDEBAR_WIDTH, sh, SIDEBAR_BG);
        d.draw_line(sx, 0, sx, sh, GRID_COLOR);

        let mut y = 8;
        d.draw_text("LEVELS", sx + 10, y, 20, TEXT_COLOR);
        y += 28;

        // Level list
        let item_h = 28;
        let list_h = (sh - 200).max(100);
        let _visible_items = list_h / item_h;

        let mx = d.get_mouse_x();
        let my = d.get_mouse_y();
        let clicked = d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        for (i, lev) in self.levels.iter().enumerate() {
            let iy = y + (i as i32 - self.list_scroll) * item_h;
            if iy < y - item_h || iy > y + list_h { continue; }

            let is_sel = i == self.current;
            let is_hover = mx >= sx && mx < sx + SIDEBAR_WIDTH && my >= iy && my < iy + item_h;

            if is_sel {
                d.draw_rectangle(sx + 2, iy, SIDEBAR_WIDTH - 4, item_h, SELECTED_BG);
            } else if is_hover {
                d.draw_rectangle(sx + 2, iy, SIDEBAR_WIDTH - 4, item_h, Color::new(35, 35, 50, 255));
            }

            let label = if lev.enabled {
                format!("{}. {}", i + 1, lev.name)
            } else {
                format!("{}. {} (off)", i + 1, lev.name)
            };
            let spawn_ok = lev.spawn_points.len() == REQUIRED_SPAWNS;
            let name_col = if !lev.enabled {
                Color::new(100, 100, 110, 160)
            } else if spawn_ok {
                TEXT_COLOR
            } else {
                Color::new(255, 180, 80, 255)
            };
            d.draw_text(&label, sx + 12, iy + 6, 16, name_col);

            if !spawn_ok {
                let warn = format!("{}sp", lev.spawn_points.len());
                let ww = d.measure_text(&warn, 12);
                d.draw_text(&warn, sx + SIDEBAR_WIDTH - ww - 10, iy + 8, 12,
                    Color::new(255, 120, 60, 200));
            }

            if is_hover && clicked {
                self.current = i;
                self.confirm_delete = false;
                self.placing_spawn = 0;
            }
        }

        y += list_h + 8;

        // ── Buttons ─────────────────────────────────────────────────────
        let btn_h = 28;
        let btn_w = SIDEBAR_WIDTH - 20;

        // New Level
        if self.draw_button(d, sx + 10, y, btn_w, btn_h, "New Level", BTN_COLOR, BTN_HOVER) {
            self.levels.push(LevelDef {
                name: format!("Level {}", self.levels.len() + 1),
                enabled: true,
                spawn_points: vec![
                    [-6.0, 0.0],
                    [6.0, 0.0],
                    [-10.0, 0.0],
                    [10.0, 0.0],
                ],
                platforms: vec![
                    PlatformDef {
                        kind: "wall".to_string(),
                        min: [-15.0, -1.0],
                        max: [15.0, 0.0],
                    },
                    PlatformDef {
                        kind: "wall".to_string(),
                        min: [-15.0, 0.0],
                        max: [-14.0, 12.0],
                    },
                    PlatformDef {
                        kind: "wall".to_string(),
                        min: [14.0, 0.0],
                        max: [15.0, 12.0],
                    },
                ],
                sawblades: vec![], // kept for TOML compat
                bounce_pads: vec![],
                lava_pools: vec![],
                lasers: vec![],
            });
            self.current = self.levels.len() - 1;
            self.dirty = true;
            self.set_status("New level created");
        }
        y += btn_h + 4;

        // Duplicate
        if self.draw_button(d, sx + 10, y, btn_w, btn_h, "Duplicate", BTN_COLOR, BTN_HOVER) {
            if let Some(lev) = self.levels.get(self.current).cloned() {
                let mut dup = lev;
                dup.name = format!("{} (copy)", dup.name);
                self.levels.insert(self.current + 1, dup);
                self.current += 1;
                self.dirty = true;
                self.set_status("Level duplicated");
            }
        }
        y += btn_h + 4;

        // Enable/Disable toggle
        {
            let is_enabled = self.levels.get(self.current).map_or(true, |l| l.enabled);
            let label = if is_enabled { "Enabled  [ON]" } else { "Disabled [OFF]" };
            let col = if is_enabled { Color::new(40, 70, 40, 255) } else { Color::new(70, 40, 40, 255) };
            let hov = if is_enabled { Color::new(50, 90, 50, 255) } else { Color::new(90, 50, 50, 255) };
            if self.draw_button(d, sx + 10, y, btn_w, btn_h, label, col, hov) {
                if let Some(lev) = self.levels.get_mut(self.current) {
                    lev.enabled = !lev.enabled;
                    self.dirty = true;
                }
            }
        }
        y += btn_h + 4;

        // Rename
        if self.draw_button(d, sx + 10, y, btn_w, btn_h, "Rename", BTN_COLOR, BTN_HOVER) {
            if let Some(lev) = self.levels.get(self.current) {
                self.name_buf = lev.name.clone();
                self.editing_name = true;
            }
        }
        y += btn_h + 4;

        // Delete
        if self.confirm_delete {
            if self.draw_button(d, sx + 10, y, btn_w, btn_h, "Confirm Delete?", DELETE_COLOR, DELETE_HOVER) {
                if !self.levels.is_empty() {
                    self.levels.remove(self.current);
                    if self.current >= self.levels.len() && self.current > 0 {
                        self.current -= 1;
                    }
                    self.dirty = true;
                    self.confirm_delete = false;
                    self.set_status("Level deleted");
                }
            }
        } else if self.draw_button(d, sx + 10, y, btn_w, btn_h, "Delete", DELETE_COLOR, DELETE_HOVER) {
            self.confirm_delete = true;
        }
        y += btn_h + 8;

        // Save
        let save_label = if self.dirty { "Save (Ctrl+S) *" } else { "Save (Ctrl+S)" };
        if self.draw_button(d, sx + 10, y, btn_w, btn_h, save_label, BTN_COLOR, BTN_HOVER) {
            self.save();
        }
        y += btn_h + 4;

        // Test
        let test_col = Color::new(40, 55, 70, 255);
        let test_hov = Color::new(55, 75, 95, 255);
        if self.draw_button(d, sx + 10, y, btn_w, btn_h, "Test Level", test_col, test_hov) {
            self.save();
            if let Ok(exe) = std::env::current_exe() {
                let _ = std::process::Command::new(exe)
                    .arg("--test")
                    .arg(self.current.to_string())
                    .spawn();
            }
        }
        y += btn_h + 12;

        // ── Level info ──────────────────────────────────────────────────
        if let Some(lev) = self.levels.get(self.current) {
            d.draw_text(&format!("Name: {}", lev.name), sx + 10, y, 14, DIM_TEXT);
            y += 18;
            d.draw_text(&format!("Platforms: {}", lev.platforms.len()), sx + 10, y, 14, DIM_TEXT);
            y += 18;
            if !lev.bounce_pads.is_empty() {
                d.draw_text(&format!("Bounce Pads: {}", lev.bounce_pads.len()), sx + 10, y, 14, PAD_COLOR);
                y += 18;
            }
            let spawn_col = if lev.spawn_points.len() == REQUIRED_SPAWNS {
                Color::new(80, 220, 80, 200)
            } else {
                Color::new(255, 120, 60, 200)
            };
            d.draw_text(
                &format!("Spawns: {}/{}", lev.spawn_points.len(), REQUIRED_SPAWNS),
                sx + 10, y, 14, spawn_col,
            );
        }

        // ── Name editing overlay ────────────────────────────────────────
        if self.editing_name {
            let bx = sx + 10;
            let by = sh / 2 - 30;
            let bw = SIDEBAR_WIDTH - 20;
            d.draw_rectangle(bx - 2, by - 2, bw + 4, 60 + 4, Color::new(0, 0, 0, 200));
            d.draw_rectangle(bx, by, bw, 60, Color::new(40, 40, 55, 255));
            d.draw_text("Level Name:", bx + 8, by + 6, 14, DIM_TEXT);
            let cursor = if (self.status_timer * 4.0) as i32 % 2 == 0 { "_" } else { "" };
            d.draw_text(
                &format!("{}{}", self.name_buf, cursor),
                bx + 8, by + 28, 18, Color::WHITE,
            );
        }
    }

    fn draw_button(
        &self,
        d: &mut RaylibDrawHandle,
        x: i32, y: i32, w: i32, h: i32,
        label: &str,
        bg: Color, hover_bg: Color,
    ) -> bool {
        let mx = d.get_mouse_x();
        let my = d.get_mouse_y();
        let hovered = mx >= x && mx < x + w && my >= y && my < y + h;
        let col = if hovered { hover_bg } else { bg };
        d.draw_rectangle(x, y, w, h, col);
        d.draw_rectangle_lines(x, y, w, h, Color::new(80, 80, 100, 255));
        let tw = d.measure_text(label, 14);
        d.draw_text(label, x + w / 2 - tw / 2, y + 7, 14, TEXT_COLOR);
        hovered && d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
    }
}
