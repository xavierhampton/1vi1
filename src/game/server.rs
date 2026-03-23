use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use raylib::prelude::*;

use crate::combat::particles::{spawn_death_explosion, spawn_player_hit, spawn_terrain_hit, update_particles};
use crate::game::net::{self, GameEvent, WorldSnapshot};
use crate::game::world::World;
use crate::lobby::protocol::ClientIncoming;
use crate::lobby::server::{GameServerParts, ServerEvent};
use crate::player::input::{self, PlayerInput};

const BROADCAST_RATE: f32 = 1.0 / 60.0;

pub struct GameServer {
    pub world: World,
    client_streams: Vec<Option<TcpStream>>,
    event_rx: Receiver<ServerEvent>,
    shutdown: Arc<AtomicBool>,
    inputs: Vec<PlayerInput>,
    broadcast_accumulator: f32,
    pending_events: Vec<GameEvent>,
    // Maps client_id → player slot index (same logic as lobby)
    client_slot_map: Vec<Option<usize>>,
}

impl GameServer {
    pub fn new(world: World, parts: GameServerParts) -> Self {
        let player_count = world.players.len();
        let inputs = (0..player_count).map(|_| PlayerInput::empty()).collect();

        // Build client→slot map from the existing connected client IDs
        let mut client_slot_map = vec![None; parts.client_streams.len()];
        let mut slot = 1usize; // slot 0 = host
        for (cid, stream) in parts.client_streams.iter().enumerate() {
            if stream.is_some() {
                if slot < player_count {
                    client_slot_map[cid] = Some(slot);
                }
                slot += 1;
            }
        }

        Self {
            world,
            client_streams: parts.client_streams,
            event_rx: parts.event_rx,
            shutdown: parts.shutdown,
            inputs,
            broadcast_accumulator: 0.0,
            pending_events: Vec::new(),
            client_slot_map,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, camera: &Camera3D, dt: f32) {
        // 1. Read host input (player 0)
        if self.world.players[0].alive || !matches!(self.world.state, crate::game::state::GameState::Playing) {
            let center = Vector2::new(
                self.world.players[0].position.x,
                self.world.players[0].position.y + self.world.players[0].size.y / 2.0,
            );
            self.inputs[0] = input::read_input(rl, camera, center);
        }

        // 2. Drain client inputs (OR-accumulate one-shot fields)
        let events: Vec<_> = self.event_rx.try_iter().collect();
        for event in events {
            match event {
                ServerEvent::ClientMessage(cid, incoming) => {
                    if let ClientIncoming::GameInput(new_input) = incoming {
                        if let Some(&Some(slot)) = self.client_slot_map.get(cid) {
                            if slot < self.inputs.len() {
                                // OR-accumulate one-shot inputs
                                self.inputs[slot].move_dir = new_input.move_dir;
                                self.inputs[slot].aim_dir = new_input.aim_dir;
                                self.inputs[slot].jump_held = new_input.jump_held;
                                self.inputs[slot].jump_pressed |= new_input.jump_pressed;
                                self.inputs[slot].shoot_pressed |= new_input.shoot_pressed;
                            }
                        }
                    }
                    // Ignore lobby messages during game
                }
                ServerEvent::ClientDisconnected(_cid) => {
                    // Player disconnected — could handle later
                }
                ServerEvent::ClientConnected(_, _) => {
                    // Ignore new connections during game
                }
            }
        }

        // 3. Run physics every frame for smooth host rendering
        let game_events = self.world.server_update(&self.inputs, dt);

        // Spawn particles locally on host from events
        for ev in &game_events {
            match ev {
                GameEvent::PlayerHit { x, y, z, r, g, b } => {
                    spawn_player_hit(&mut self.world.particles, &mut self.world.rng,
                        Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
                }
                GameEvent::PlayerDied { x, y, z, r, g, b } => {
                    spawn_death_explosion(&mut self.world.particles, &mut self.world.rng,
                        Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
                }
                GameEvent::TerrainHit { x, y, z, r, g, b } => {
                    spawn_terrain_hit(&mut self.world.particles, &mut self.world.rng,
                        Vector3::new(*x, *y, *z), Color::new(*r, *g, *b, 255));
                }
                GameEvent::BulletFired { .. } => {}
            }
        }

        self.pending_events.extend(game_events);

        // Clear one-shot inputs after physics consumes them
        for inp in &mut self.inputs {
            inp.jump_pressed = false;
            inp.shoot_pressed = false;
        }

        // 4. Broadcast snapshots at fixed rate
        self.broadcast_accumulator += dt;
        while self.broadcast_accumulator >= BROADCAST_RATE {
            let events = std::mem::take(&mut self.pending_events);
            let snapshot = self.world.to_snapshot(events);
            self.broadcast_snapshot(&snapshot);
            self.broadcast_accumulator -= BROADCAST_RATE;
        }

        // 5. Update particles locally on the host (for rendering)
        update_particles(&mut self.world.particles, dt);
    }

    fn broadcast_snapshot(&mut self, snapshot: &WorldSnapshot) {
        let data = net::encode_snapshot(snapshot);
        for stream_opt in self.client_streams.iter_mut() {
            if let Some(stream) = stream_opt {
                let _ = stream.write_all(&data);
            }
        }
    }
}

impl Drop for GameServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}
