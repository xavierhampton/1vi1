use crate::level::platforms::Platform;
use crate::player::input::PlayerInput;
use crate::player::player::Player;

const GRAVITY: f32 = 30.0;
const MAX_FALL_SPEED: f32 = 20.0;
const MOVE_SPEED: f32 = 8.0;
const MOVE_ACCEL: f32 = 40.0;
const MOVE_DECEL: f32 = 35.0;
const AIR_ACCEL: f32 = 8.0;
const AIR_COUNTER_ACCEL: f32 = 22.0;
const MAX_BHOP_SPEED: f32 = 12.0;
const BHOP_SPEED_BOOST: f32 = 0.15;
const JUMP_VELOCITY: f32 = 11.6;
const JUMP_CUT_MULTIPLIER: f32 = 0.5;
const COYOTE_TIME: f32 = 0.08;
const MAX_AIR_JUMPS: i32 = 1;
const WALL_JUMP_H: f32 = 10.0;
const WALL_SLIDE_FALL: f32 = 4.0;

pub fn update(player: &mut Player, input: &PlayerInput, platforms: &[Platform], dt: f32, gravity_scale: f32) {
    update_with_speed(player, input, platforms, dt, 1.0, gravity_scale);
}

pub fn update_with_speed(player: &mut Player, input: &PlayerInput, platforms: &[Platform], dt: f32, speed_mult: f32, gravity_scale: f32) {
    let prev_grounded = player.grounded;
    let prev_wall = player.wall_dir;
    player.grounded = false;
    player.wall_dir = 0;
    let mut jumped = false;

    // Effective stats from powerups
    let eff_move_speed = MOVE_SPEED * speed_mult * player.stats.move_speed_mult;
    let eff_bhop_speed = MAX_BHOP_SPEED * speed_mult * player.stats.move_speed_mult;
    let eff_max_air_jumps = MAX_AIR_JUMPS + player.stats.extra_air_jumps;

    // Bhop: if holding jump on landing, skip ground friction entirely
    let is_bhop_frame = input.jump_held && prev_grounded;

    if prev_grounded && !is_bhop_frame {
        // Ground movement: full accel/decel with speed cap
        if input.move_dir != 0.0 {
            let vel_before = player.velocity.x;
            player.velocity.x += input.move_dir * MOVE_ACCEL * dt;
            // Soft cap: preserve momentum above move speed (e.g. from dash)
            let cap = eff_move_speed.max(vel_before.abs());
            player.velocity.x = player.velocity.x.clamp(-cap, cap);
        } else {
            let decel = MOVE_DECEL * dt;
            if player.velocity.x > 0.0 {
                player.velocity.x = (player.velocity.x - decel).max(0.0);
            } else if player.velocity.x < 0.0 {
                player.velocity.x = (player.velocity.x + decel).min(0.0);
            }
        }
    } else {
        // Air / bhop: momentum preserved, counter-strafe is stronger for direction changes
        if input.move_dir != 0.0 {
            let against = (input.move_dir > 0.0 && player.velocity.x < 0.0)
                || (input.move_dir < 0.0 && player.velocity.x > 0.0);
            let accel = if against { AIR_COUNTER_ACCEL } else { AIR_ACCEL };
            let vel_before = player.velocity.x;
            player.velocity.x += input.move_dir * accel * dt;
            // Soft cap: preserve momentum above bhop speed (e.g. from dash)
            let cap = eff_bhop_speed.max(vel_before.abs());
            player.velocity.x = player.velocity.x.clamp(-cap, cap);
        }
    }

    // Gravity
    player.velocity.y -= GRAVITY * gravity_scale * dt;
    if player.velocity.y < -MAX_FALL_SPEED * gravity_scale {
        player.velocity.y = -MAX_FALL_SPEED * gravity_scale;
    }

    // Jump (bunny hop: holding jump auto-jumps on landing)
    let jump_trigger = input.jump_pressed || (input.jump_held && prev_grounded);
    let can_ground_jump = prev_grounded || player.coyote_timer > 0.0;
    let can_air_jump = input.jump_pressed && player.air_jumps < eff_max_air_jumps;
    if jump_trigger && can_ground_jump {
        player.velocity.y = JUMP_VELOCITY;
        player.coyote_timer = 0.0;
        jumped = true;
        player.jump_cut_applied = false;
        player.air_jumps = 0;
        // Bhop: small speed boost in movement direction
        if input.move_dir != 0.0 && player.velocity.x.abs() < eff_bhop_speed {
            player.velocity.x += input.move_dir * BHOP_SPEED_BOOST;
        }
    } else if can_air_jump {
        player.velocity.y = JUMP_VELOCITY;
        player.air_jumps += 1;
        jumped = true;
        player.jump_cut_applied = false;
    } else if !jumped && !prev_grounded && prev_wall != 0 && input.jump_pressed {
        // Wall jump: kick off wall
        player.velocity.x = -(prev_wall as f32) * WALL_JUMP_H;
        player.velocity.y = JUMP_VELOCITY;
        jumped = true;
        player.jump_cut_applied = false;
        player.air_jumps = 0;
    }

    // Variable jump height: one-time cut when jump key released
    if !input.jump_held && player.velocity.y > 0.0 && !player.jump_cut_applied {
        player.velocity.y *= JUMP_CUT_MULTIPLIER;
        player.jump_cut_applied = true;
    }

    // Move both axes
    player.position.x += player.velocity.x * dt;
    player.position.y += player.velocity.y * dt;

    // Resolve collisions using minimum penetration
    resolve_collisions(player, platforms);

    // Wall slide: slow fall when hugging a wall
    if player.wall_dir != 0 && !player.grounded && player.velocity.y < -WALL_SLIDE_FALL {
        player.velocity.y = -WALL_SLIDE_FALL;
    }

    // Reset when grounded
    if player.grounded {
        player.jump_cut_applied = false;
        player.air_jumps = 0;
    }

    // Coyote time
    if prev_grounded && !player.grounded && !jumped {
        player.coyote_timer = COYOTE_TIME;
    }
    if player.grounded {
        player.coyote_timer = 0.0;
    }
    player.coyote_timer = (player.coyote_timer - dt).max(0.0);
}

fn resolve_collisions(player: &mut Player, platforms: &[Platform]) {
    for _ in 0..4 {
        let paabb = player.aabb();
        let mut resolved = false;

        for platform in platforms {
            if !paabb.overlaps(&platform.aabb) {
                continue;
            }

            let pen_left = paabb.max.x - platform.aabb.min.x;
            let pen_right = platform.aabb.max.x - paabb.min.x;
            let pen_bottom = paabb.max.y - platform.aabb.min.y;
            let pen_top = platform.aabb.max.y - paabb.min.y;

            let min_pen_x = pen_left.min(pen_right);
            let min_pen_y = pen_bottom.min(pen_top);

            if min_pen_x < min_pen_y {
                // Resolve horizontal
                if pen_left < pen_right {
                    player.position.x -= pen_left;
                    player.wall_dir = 1; // wall to the right
                } else {
                    player.position.x += pen_right;
                    player.wall_dir = -1; // wall to the left
                }
                player.velocity.x = 0.0;
            } else {
                // Resolve vertical
                if pen_top < pen_bottom {
                    player.position.y = platform.aabb.max.y;
                    player.grounded = true;
                } else {
                    player.position.y = platform.aabb.min.y - player.size.y;
                }
                player.velocity.y = 0.0;
            }

            resolved = true;
            break;
        }

        if !resolved {
            break;
        }
    }
}
