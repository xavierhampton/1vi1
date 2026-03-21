use raylib::prelude::*;

pub struct PlayerInput {
    pub move_dir: f32,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub shoot_pressed: bool,
    pub aim_target: Vector2,
}

pub fn read_input(rl: &RaylibHandle, camera: &Camera3D) -> PlayerInput {
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
        Vector2::new(0.0, 0.0)
    };

    PlayerInput {
        move_dir,
        jump_pressed,
        jump_held,
        shoot_pressed,
        aim_target,
    }
}
