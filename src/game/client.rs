use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use raylib::prelude::*;

use crate::combat::bullet::BULLET_GRAVITY;
use crate::combat::particles::update_particles;
use crate::game::net;
use crate::game::state::GameState;
use crate::game::world::World;
use crate::lobby::client::GameClientParts;
use crate::lobby::protocol::{self, ServerIncoming, ServerMsg};
use crate::player::input;
use crate::render::cards::card_slot_from_mouse;

pub struct GameClient {
    pub world: World,
    pub disconnect_message: Option<String>,
    write_stream: TcpStream,
    incoming_rx: Receiver<ServerIncoming>,
    shutdown: Arc<AtomicBool>,
    pub my_index: u8,
    // Extrapolation: track base state from last snapshot + time since
    snap_positions: Vec<(Vector3, Vector3)>, // (base_pos, velocity) per player
    snap_aim: Vec<Vector2>,                  // target aim_dir per player
    snap_bullets: Vec<(Vector3, Vector2)>,   // (base_pos, velocity) per bullet
    time_since_snap: f32,
    time_scale: f32, // from server: 1.0 normal, 0.25 slow-mo, 0.0 frozen
}

impl GameClient {
    pub fn new(world: World, parts: GameClientParts) -> Self {
        let snap_positions = world.players.iter()
            .map(|p| (p.position, p.velocity))
            .collect();
        let snap_aim = world.players.iter()
            .map(|p| p.aim_dir)
            .collect();
        let snap_bullets = world.bullets.iter()
            .map(|b| (b.position, b.velocity))
            .collect();
        Self {
            world,
            disconnect_message: None,
            write_stream: parts.write_stream,
            incoming_rx: parts.incoming_rx,
            shutdown: parts.shutdown,
            my_index: parts.my_index,
            snap_positions,
            snap_aim,
            snap_bullets,
            time_since_snap: 0.0,
            time_scale: 1.0,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, camera: &Camera3D, dt: f32) {
        let my_idx = self.my_index as usize;

        // 1. Read local input and send to server
        let mut local_hover = 0xFFu8;
        if my_idx < self.world.players.len() {
            let center = Vector2::new(
                self.world.players[my_idx].position.x,
                self.world.players[my_idx].position.y + self.world.players[my_idx].size.y / 2.0,
            );
            let mut local_input = input::read_input(rl, camera, center);
            // Store own cursor locally so it's not lagged by network round-trip
            if my_idx < self.world.cursor_positions.len() {
                self.world.cursor_positions[my_idx] = (local_input.cursor_x, local_input.cursor_y);
            }

            // Compute hover_card if we're the current picker
            if let GameState::CardPick { current_picker, chosen_card, phase_timer, .. } = &self.world.state {
                if *current_picker == self.my_index && chosen_card.is_none() && *phase_timer <= 0.0 {
                    let mouse = rl.get_mouse_position();
                    let sw = rl.get_screen_width() as f32;
                    let sh = rl.get_screen_height() as f32;
                    let hover = card_slot_from_mouse(mouse, sw, sh).unwrap_or(0xFF);
                    local_input.hover_card = hover;
                    local_hover = hover;

                    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                        if let Some(slot) = card_slot_from_mouse(mouse, sw, sh) {
                            let data = protocol::encode_card_choice(slot);
                            let _ = self.write_stream.write_all(&data);
                        }
                    }
                }
            }

            let data = net::encode_game_input(&local_input);
            let _ = self.write_stream.write_all(&data);
        }

        // 2. Drain snapshots, apply only the latest
        let mut got_snapshot = false;
        let mut latest_snapshot = None;
        for incoming in self.incoming_rx.try_iter() {
            match incoming {
                ServerIncoming::Snapshot(snap) => {
                    latest_snapshot = Some(snap);
                }
                ServerIncoming::Lobby(ServerMsg::PlayerLeft { name }) => {
                    self.disconnect_message = Some(format!("{} Left", name));
                }
                ServerIncoming::Disconnected => {
                    if self.disconnect_message.is_none() {
                        self.disconnect_message = Some("Host disconnected".to_string());
                    }
                }
                _ => {}
            }
        }

        if let Some(snap) = latest_snapshot {
            self.time_scale = snap.time_scale;
            self.world.apply_snapshot(&snap);
            // Store base positions + velocities + aim for extrapolation
            self.snap_positions.clear();
            self.snap_aim.clear();
            for p in &self.world.players {
                self.snap_positions.push((p.position, p.velocity));
                self.snap_aim.push(p.aim_dir);
            }
            self.snap_bullets.clear();
            for b in &self.world.bullets {
                self.snap_bullets.push((b.position, b.velocity));
            }
            self.time_since_snap = 0.0;
            got_snapshot = true;
        }

        // 3. If we're the picker, override card_hover AFTER snapshot to avoid stale overwrite
        if local_hover != 0xFF {
            self.world.card_hover = local_hover;
        }

        // 4. Apply local player's aim directly for instant responsiveness
        if my_idx < self.world.players.len() {
            let center = Vector2::new(
                self.world.players[my_idx].position.x,
                self.world.players[my_idx].position.y + self.world.players[my_idx].size.y / 2.0,
            );
            let local_input = input::read_input(rl, camera, center);
            self.world.players[my_idx].aim_dir = local_input.aim_dir;
        }

        // 5. Extrapolate positions + lerp remote aim between snapshots
        if !got_snapshot {
            self.time_since_snap += dt;
            let t = self.time_since_snap.min(0.05) * self.time_scale;
            for (i, (base_pos, vel)) in self.snap_positions.iter().enumerate() {
                if i < self.world.players.len() {
                    self.world.players[i].position.x = base_pos.x + vel.x * t;
                    self.world.players[i].position.y = base_pos.y + vel.y * t;
                }
            }
            // Extrapolate bullet positions with gravity
            for (i, (base_pos, vel)) in self.snap_bullets.iter().enumerate() {
                if i < self.world.bullets.len() {
                    self.world.bullets[i].position.x = base_pos.x + vel.x * t;
                    self.world.bullets[i].position.y = base_pos.y + (vel.y - BULLET_GRAVITY * t) * t;
                }
            }
        }
        // Lerp remote players' aim toward snapshot target
        let aim_lerp = (dt * 30.0).min(1.0);
        for (i, target_aim) in self.snap_aim.iter().enumerate() {
            if i != my_idx && i < self.world.players.len() {
                let cur = &mut self.world.players[i].aim_dir;
                cur.x += (target_aim.x - cur.x) * aim_lerp;
                cur.y += (target_aim.y - cur.y) * aim_lerp;
            }
        }

        // 5. Update particles locally (client-side cosmetic only)
        update_particles(&mut self.world.particles, dt);
    }
}

impl Drop for GameClient {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

