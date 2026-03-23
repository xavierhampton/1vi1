use raylib::prelude::*;

#[derive(Debug, Clone)]
pub struct PlayerInput {
    pub move_dir: f32,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub shoot_pressed: bool,
    pub ability_pressed: bool,
    pub aim_dir: Vector2,
    pub cursor_x: f32, // normalized 0..1 screen coords
    pub cursor_y: f32,
    pub hover_card: u8, // 0-2 = hovering card slot, 0xFF = none
}

impl PlayerInput {
    pub fn empty() -> Self {
        Self {
            move_dir: 0.0,
            jump_pressed: false,
            jump_held: false,
            shoot_pressed: false,
            ability_pressed: false,
            aim_dir: Vector2::new(1.0, 0.0),
            cursor_x: 0.5,
            cursor_y: 0.5,
            hover_card: 0xFF,
        }
    }
}

pub fn read_input(rl: &RaylibHandle, camera: &Camera3D, player_center: Vector2) -> PlayerInput {
    let mut move_dir = 0.0;
    if rl.is_key_down(KeyboardKey::KEY_A) {
        move_dir -= 1.0;
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        move_dir += 1.0;
    }

    let jump_pressed = rl.is_key_pressed(KeyboardKey::KEY_W)
        || rl.is_key_pressed(KeyboardKey::KEY_SPACE);
    let jump_held = rl.is_key_down(KeyboardKey::KEY_W)
        || rl.is_key_down(KeyboardKey::KEY_SPACE);

    let shoot_pressed = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
    let ability_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT);

    // Mouse aim: cast ray from screen to Z=0 plane
    let mouse_pos = rl.get_mouse_position();
    let ray = rl.get_screen_to_world_ray(mouse_pos, camera);
    let aim_target = if ray.direction.z.abs() > 0.0001 {
        let t = -ray.position.z / ray.direction.z;
        Vector2::new(
            ray.position.x + t * ray.direction.x,
            ray.position.y + t * ray.direction.y,
        )
    } else {
        Vector2::new(player_center.x + 1.0, player_center.y)
    };

    // Compute normalized aim direction
    let dx = aim_target.x - player_center.x;
    let dy = aim_target.y - player_center.y;
    let len = (dx * dx + dy * dy).sqrt();
    let aim_dir = if len > 0.001 {
        Vector2::new(dx / len, dy / len)
    } else {
        Vector2::new(1.0, 0.0)
    };

    // Normalized cursor position (0..1)
    let screen_w = rl.get_screen_width() as f32;
    let screen_h = rl.get_screen_height() as f32;
    let cursor_x = if screen_w > 0.0 { (mouse_pos.x / screen_w).clamp(0.0, 1.0) } else { 0.5 };
    let cursor_y = if screen_h > 0.0 { (mouse_pos.y / screen_h).clamp(0.0, 1.0) } else { 0.5 };

    PlayerInput {
        move_dir,
        jump_pressed,
        jump_held,
        shoot_pressed,
        ability_pressed,
        aim_dir,
        cursor_x,
        cursor_y,
        hover_card: 0xFF, // set by game server/client during card pick
    }
}
