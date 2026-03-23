use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use raylib::prelude::*;

use crate::combat::particles::update_particles;
use crate::game::net;
use crate::game::world::World;
use crate::lobby::client::GameClientParts;
use crate::lobby::protocol::ServerIncoming;
use crate::player::input;

pub struct GameClient {
    pub world: World,
    write_stream: TcpStream,
    incoming_rx: Receiver<ServerIncoming>,
    shutdown: Arc<AtomicBool>,
    pub my_index: u8,
    // Extrapolation: track base positions from last snapshot + time since
    snap_positions: Vec<(Vector3, Vector3)>, // (base_pos, velocity) per player
    time_since_snap: f32,
    time_scale: f32, // from server: 1.0 normal, 0.25 slow-mo, 0.0 frozen
}

impl GameClient {
    pub fn new(world: World, parts: GameClientParts) -> Self {
        let snap_positions = world.players.iter()
            .map(|p| (p.position, p.velocity))
            .collect();
        Self {
            world,
            write_stream: parts.write_stream,
            incoming_rx: parts.incoming_rx,
            shutdown: parts.shutdown,
            my_index: parts.my_index,
            snap_positions,
            time_since_snap: 0.0,
            time_scale: 1.0,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, camera: &Camera3D, dt: f32) {
        let my_idx = self.my_index as usize;

        // 1. Read local input and send to server
        if my_idx < self.world.players.len() {
            let center = Vector2::new(
                self.world.players[my_idx].position.x,
                self.world.players[my_idx].position.y + self.world.players[my_idx].size.y / 2.0,
            );
            let local_input = input::read_input(rl, camera, center);
            let data = net::encode_game_input(&local_input);
            let _ = self.write_stream.write_all(&data);
        }

        // 2. Drain snapshots, apply only the latest
        let mut got_snapshot = false;
        let mut latest_snapshot = None;
        for incoming in self.incoming_rx.try_iter() {
            if let ServerIncoming::Snapshot(snap) = incoming {
                latest_snapshot = Some(snap);
            }
        }

        if let Some(snap) = latest_snapshot {
            self.time_scale = snap.time_scale;
            self.world.apply_snapshot(&snap);
            // Store base positions + velocities for extrapolation
            self.snap_positions.clear();
            for p in &self.world.players {
                self.snap_positions.push((p.position, p.velocity));
            }
            self.time_since_snap = 0.0;
            got_snapshot = true;
        }

        // 3. Extrapolate positions between snapshots for smooth rendering
        if !got_snapshot {
            self.time_since_snap += dt;
            // Cap extrapolation to avoid overshooting
            let t = self.time_since_snap.min(0.05) * self.time_scale;
            for (i, (base_pos, vel)) in self.snap_positions.iter().enumerate() {
                if i < self.world.players.len() {
                    self.world.players[i].position.x = base_pos.x + vel.x * t;
                    self.world.players[i].position.y = base_pos.y + vel.y * t;
                }
            }
        }

        // 4. Update particles locally (client-side cosmetic only)
        update_particles(&mut self.world.particles, dt);
    }
}

impl Drop for GameClient {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}
