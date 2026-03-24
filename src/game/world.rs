use raylib::prelude::*;

use crate::combat::bullet::{Bullet, SHOOT_COOLDOWN};
use crate::combat::combat::update_bullets;
use crate::combat::particles::{spawn_from_events, Particle, Rng};
use crate::game::cards;
use crate::game::net::{BulletSnapshot, CloneSnapshot, GameEvent, GravityWellSnapshot, PlayerSnapshot, StickyBombSnapshot, WorldSnapshot};
use crate::game::state::GameState;
use crate::level::level::{self, Level};
use crate::lobby::state::LobbyState;
use crate::player::input::PlayerInput;
use crate::player::movement;
use crate::player::player::{Player, HIT_FLASH_DURATION};

pub const MAX_BULLETS: i32 = 3;
pub const RELOAD_TIME: f32 = 1.5;

pub const COUNTDOWN_DURATION: f32 = 3.0;
const ROUND_END_DURATION: f32 = 3.5;
const SLOW_MO_FACTOR: f32 = 0.25;
pub const WINS_TO_MATCH: i32 = 3;

const CARD_ENTRANCE_DURATION: f32 = 0.8;
const CARD_EXIT_DURATION: f32 = 0.8;
const MATCH_OVER_DURATION: f32 = 5.0;

const STOMP_RADIUS: f32 = 3.0;
const STOMP_DAMAGE: f32 = 35.0;
const STOMP_SLAM_SPEED: f32 = 35.0;

const GRAVITY_WELL_RADIUS: f32 = 6.0;
const GRAVITY_WELL_LIFETIME: f32 = 4.0;
const GRAVITY_WELL_FORCE: f32 = 15.0;
const CLONE_SPEED: f32 = 10.0;
const CLONE_LIFETIME: f32 = 6.0;
const CLONE_DAMAGE: f32 = 50.0;
const CLONE_EXPLODE_RADIUS: f32 = 3.5;
const STICKY_FUSE: f32 = 2.0;
const STICKY_EXPLODE_RADIUS: f32 = 3.0;
const LEECH_FIELD_RADIUS: f32 = 5.0;
const LEECH_FIELD_DPS: f32 = 2.0;

pub struct GravityWellEntity {
    pub position: Vector3,
    pub owner: usize,
    pub lifetime: f32,
}

pub struct CloneEntity {
    pub position: Vector3,
    pub velocity: Vector3,
    pub owner: usize,
    pub lifetime: f32,
    pub color: Color,
}

pub struct StickyBomb {
    pub position: Vector3,
    pub owner: usize,
    pub damage: f32,
    pub fuse: f32,
    pub stuck_to: Option<usize>,
    pub color: Color,
}

pub struct World {
    pub players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    pub particles: Vec<Particle>,
    pub level: Level,
    pub state: GameState,
    pub scores: Vec<i32>,
    pub rng: Rng,
    pub cursor_positions: Vec<(f32, f32)>,
    pub card_hover: u8,
    pub gravity_wells: Vec<GravityWellEntity>,
    pub clones: Vec<CloneEntity>,
    pub sticky_bombs: Vec<StickyBomb>,
}

/// 2D ray-AABB intersection (XY plane). Returns t of first hit, or None.
fn ray_aabb_t(ox: f32, oy: f32, dx: f32, dy: f32, aabb: &crate::physics::collision::AABB) -> Option<f32> {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;

    if dx.abs() > 1e-8 {
        let t1 = (aabb.min.x - ox) / dx;
        let t2 = (aabb.max.x - ox) / dx;
        tmin = tmin.max(t1.min(t2));
        tmax = tmax.min(t1.max(t2));
    } else if ox < aabb.min.x || ox > aabb.max.x {
        return None;
    }

    if dy.abs() > 1e-8 {
        let t1 = (aabb.min.y - oy) / dy;
        let t2 = (aabb.max.y - oy) / dy;
        tmin = tmin.max(t1.min(t2));
        tmax = tmax.min(t1.max(t2));
    } else if oy < aabb.min.y || oy > aabb.max.y {
        return None;
    }

    if tmin <= tmax && tmax > 0.0 {
        Some(if tmin > 0.0 { tmin } else { tmax })
    } else {
        None
    }
}

impl World {
    pub fn from_lobby(lobby: &LobbyState) -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        let mut rng = Rng::new(seed);
        let level = level::random_level(rng.next());
        let count = lobby.slots.len().clamp(2, 4);
        let players = lobby.slots.iter().enumerate().take(count).map(|(i, slot)| {
            Player::new(
                level.spawn_points[i],
                Vector3::new(0.6, 1.6, 0.6),
                slot.color.to_color(),
                &slot.name,
            )
        }).collect();

        Self {
            cursor_positions: vec![(0.5, 0.5); count],
            players,
            bullets: Vec::new(),
            particles: Vec::new(),
            level,
            state: GameState::RoundStart { timer: COUNTDOWN_DURATION },
            scores: vec![0; count],
            rng,
            card_hover: 0xFF,
            gravity_wells: Vec::new(),
            clones: Vec::new(),
            sticky_bombs: Vec::new(),
        }
    }

    fn reset_round(&mut self) {
        self.level = level::random_level(self.rng.next());

        for (i, player) in self.players.iter_mut().enumerate() {
            player.position = self.level.spawn_points[i];
            player.velocity = Vector3::new(0.0, 0.0, 0.0);
            player.alive = true;
            player.hit_flash_timer = 0.0;
            player.reload_timer = 0.0;
            player.shoot_cooldown = 0.0;
            player.aim_dir = Vector2::new(if i % 2 == 0 { 1.0 } else { -1.0 }, 0.0);
            player.stomp_active = false;
            player.laser_active = false;
            player.poison_timer = 0.0;
            player.ghost_timer = 0.0;
            player.overclock_timer = 0.0;
            player.overclock_crash_timer = 0.0;
            player.adrenaline_timer = 0.0;
            player.upsized_stacks = 0;
            player.rewind_history.clear();
            player.rewind_sample_timer = 0.0;
            for (_, cd) in player.cards.iter_mut() {
                *cd = 0.0;
            }
            player.stats = cards::compute_stats(&player.cards);
            cards::apply_stats(player, &player.stats.clone());
            player.hp = player.max_hp;
            player.bullets_remaining = MAX_BULLETS + player.stats.extra_ammo;
        }
        self.bullets.clear();
        self.particles.clear();
        self.gravity_wells.clear();
        self.clones.clear();
        self.sticky_bombs.clear();
        self.state = GameState::RoundStart { timer: COUNTDOWN_DURATION };
    }

    fn kill_player_server(&mut self, idx: usize, events: &mut Vec<GameEvent>) {
        self.players[idx].alive = false;
        let pos = self.players[idx].render_center();
        let c = self.players[idx].color;
        events.push(GameEvent::PlayerDied {
            x: pos.x, y: pos.y, z: pos.z,
            r: c.r, g: c.g, b: c.b,
        });
    }

    fn alive_count(&self) -> usize {
        self.players.iter().filter(|p| p.alive).count()
    }

    fn last_alive(&self) -> Option<usize> {
        let alive: Vec<usize> = self.players.iter().enumerate()
            .filter(|(_, p)| p.alive)
            .map(|(i, _)| i)
            .collect();
        if alive.len() == 1 { Some(alive[0]) } else { None }
    }

    fn winner_index(&self) -> u8 {
        match &self.state {
            GameState::RoundEnd { winner_index, .. } => *winner_index,
            GameState::CardPick { winner_index, .. } => *winner_index,
            GameState::MatchOver { winner_index, .. } => *winner_index,
            _ => 0,
        }
    }

    fn build_pick_order(&self, winner_idx: u8) -> Vec<u8> {
        (0..self.players.len() as u8)
            .filter(|&i| i != winner_idx)
            .collect()
    }

    fn enter_card_pick(&mut self, winner_idx: u8) {
        let mut pick_order = self.build_pick_order(winner_idx);
        if pick_order.is_empty() {
            self.reset_round();
            return;
        }
        let current_picker = pick_order.remove(0);
        let mut seed = self.rng.next();
        let offered = cards::random_cards(&mut seed, 3);
        self.state = GameState::CardPick {
            winner_index: winner_idx,
            current_picker,
            offered_cards: [
                *offered.get(0).unwrap_or(&0),
                *offered.get(1).unwrap_or(&1),
                *offered.get(2).unwrap_or(&2),
            ],
            pick_order,
            phase_timer: CARD_ENTRANCE_DURATION,
            chosen_card: None,
            exit_timer: 0.0,
        };
    }

    pub fn process_card_choice(&mut self, player_index: u8, card_slot: u8) {
        if let GameState::CardPick { current_picker, chosen_card, .. } = &mut self.state {
            if player_index == *current_picker && chosen_card.is_none() && card_slot < 3 {
                *chosen_card = Some(card_slot);
            }
        }
    }

    /// Dev mode: toggle a card on/off for a player
    pub fn dev_toggle_card(&mut self, player_idx: usize, card_id: cards::CardId) {
        if player_idx < self.players.len() {
            let p = &mut self.players[player_idx];
            if let Some(pos) = p.cards.iter().position(|(id, _)| *id == card_id) {
                p.cards.remove(pos);
            } else {
                p.cards.push((card_id, 0.0));
            }
            p.stats = cards::compute_stats(&p.cards);
            cards::apply_stats(p, &p.stats.clone());
        }
    }

    // ── Server-authoritative update (processes ALL players) ──────────────────

    pub fn server_update(&mut self, inputs: &[PlayerInput], dt: f32) -> Vec<GameEvent> {
        let mut events = Vec::new();

        for (i, inp) in inputs.iter().enumerate() {
            if i < self.cursor_positions.len() {
                self.cursor_positions[i] = (inp.cursor_x, inp.cursor_y);
            }
        }

        if let GameState::CardPick { current_picker, .. } = &self.state {
            let pi = *current_picker as usize;
            self.card_hover = inputs.get(pi).map(|inp| inp.hover_card).unwrap_or(0xFF);
        } else {
            self.card_hover = 0xFF;
        }

        match &self.state {
            GameState::RoundStart { timer } => {
                let new_timer = *timer - dt;
                // Players can move during countdown but not shoot or use abilities
                for (i, inp) in inputs.iter().enumerate() {
                    if i < self.players.len() && self.players[i].alive {
                        movement::update(&mut self.players[i], &inp, &self.level.platforms, dt);
                        self.players[i].aim_dir = inp.aim_dir;
                    }
                }
                if new_timer <= 0.0 {
                    self.state = GameState::Playing;
                } else {
                    self.state = GameState::RoundStart { timer: new_timer };
                }
            }
            GameState::Playing => {
                self.server_update_playing(inputs, dt, &mut events);
            }
            GameState::RoundEnd { winner_index, timer, .. } => {
                let wi = *winner_index;
                let timer = *timer;
                let slow_dt = dt * SLOW_MO_FACTOR;

                if let Some(inp) = inputs.get(wi as usize) {
                    if (wi as usize) < self.players.len() && self.players[wi as usize].alive {
                        movement::update(&mut self.players[wi as usize], inp, &self.level.platforms, slow_dt);
                        self.players[wi as usize].aim_dir = inp.aim_dir;
                    }
                }

                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    if self.scores.get(wi as usize).copied().unwrap_or(0) >= WINS_TO_MATCH {
                        self.state = GameState::MatchOver {
                            winner_index: wi,
                            timer: MATCH_OVER_DURATION,
                        };
                    } else {
                        self.enter_card_pick(wi);
                    }
                } else {
                    let name = if let GameState::RoundEnd { winner_name, .. } = &self.state {
                        winner_name.clone()
                    } else { String::new() };
                    let color = if let GameState::RoundEnd { winner_color, .. } = &self.state {
                        *winner_color
                    } else { (255, 255, 255) };
                    self.state = GameState::RoundEnd { winner_index: wi, winner_name: name, winner_color: color, timer: new_timer };
                }
            }
            GameState::CardPick { .. } => {
                self.server_update_card_pick(dt);
            }
            GameState::MatchOver { timer, winner_index, .. } => {
                let new_timer = *timer - dt;
                let wi = *winner_index;
                if new_timer <= 0.0 {
                    self.state = GameState::MatchOver { winner_index: wi, timer: 0.0 };
                } else {
                    self.state = GameState::MatchOver { winner_index: wi, timer: new_timer };
                }
            }
        }

        events
    }

    fn server_update_card_pick(&mut self, dt: f32) {
        let (winner_index, current_picker, offered_cards, pick_order, phase_timer, chosen_card, exit_timer) =
            if let GameState::CardPick {
                winner_index, current_picker, offered_cards, pick_order, phase_timer, chosen_card, exit_timer,
            } = &self.state {
                (*winner_index, *current_picker, *offered_cards, pick_order.clone(), *phase_timer, *chosen_card, *exit_timer)
            } else {
                return;
            };

        if chosen_card.is_some() {
            let new_exit = exit_timer + dt;
            if new_exit >= CARD_EXIT_DURATION {
                if let Some(slot) = chosen_card {
                    let card_id_u8 = offered_cards[slot as usize];
                    if let Some(card_id) = cards::CardId::from_u8(card_id_u8) {
                        let picker_idx = current_picker as usize;
                        if picker_idx < self.players.len() {
                            self.players[picker_idx].cards.push((card_id, 0.0));
                            let p = &mut self.players[picker_idx];
                            p.stats = cards::compute_stats(&p.cards);
                            cards::apply_stats(p, &p.stats.clone());
                        }
                    }
                }

                if pick_order.is_empty() {
                    self.reset_round();
                } else {
                    let next_picker = pick_order[0];
                    let mut remaining = pick_order.clone();
                    remaining.remove(0);
                    let mut seed = self.rng.next();
                    let new_offered = cards::random_cards(&mut seed, 3);
                    self.state = GameState::CardPick {
                        winner_index,
                        current_picker: next_picker,
                        offered_cards: [
                            *new_offered.get(0).unwrap_or(&0),
                            *new_offered.get(1).unwrap_or(&1),
                            *new_offered.get(2).unwrap_or(&2),
                        ],
                        pick_order: remaining,
                        phase_timer: CARD_ENTRANCE_DURATION,
                        chosen_card: None,
                        exit_timer: 0.0,
                    };
                }
            } else {
                self.state = GameState::CardPick {
                    winner_index, current_picker, offered_cards, pick_order,
                    phase_timer, chosen_card, exit_timer: new_exit,
                };
            }
        } else {
            let new_phase = (phase_timer - dt).max(0.0);
            self.state = GameState::CardPick {
                winner_index, current_picker, offered_cards, pick_order,
                phase_timer: new_phase, chosen_card, exit_timer,
            };
        }
    }

    fn server_update_playing(&mut self, inputs: &[PlayerInput], dt: f32, events: &mut Vec<GameEvent>) {
        for i in 0..self.players.len() {
            if !self.players[i].alive {
                continue;
            }
            let inp = inputs.get(i).cloned().unwrap_or_else(PlayerInput::empty);

            // Upsized stacks → bigger hitbox
            let upsized_mult = 1.0 + 0.05 * self.players[i].upsized_stacks as f32;
            self.players[i].size.x = 0.6 * self.players[i].stats.size_mult * upsized_mult;
            self.players[i].size.y = 1.6 * self.players[i].stats.size_mult * upsized_mult;
            self.players[i].size.z = 0.6 * self.players[i].stats.size_mult * upsized_mult;

            // Compute speed multiplier from active buffs/debuffs
            let mut speed_mult = 1.0_f32;
            if self.players[i].overclock_timer > 0.0 { speed_mult *= 2.0; }
            if self.players[i].overclock_crash_timer > 0.0 { speed_mult *= 0.6; }
            if self.players[i].adrenaline_timer > 0.0 { speed_mult *= 1.4; }

            let was_airborne = !self.players[i].grounded;
            movement::update_with_speed(&mut self.players[i], &inp, &self.level.platforms, dt, speed_mult);
            self.players[i].aim_dir = inp.aim_dir;

            // Stomp: force slam when falling after hop
            if self.players[i].stomp_active && self.players[i].velocity.y < 0.0 {
                self.players[i].velocity.y = -STOMP_SLAM_SPEED;
            }

            // Stomp: check for landing AoE
            if self.players[i].stomp_active && was_airborne && self.players[i].grounded {
                self.players[i].stomp_active = false;
                let stomp_pos = self.players[i].position;
                for j in 0..self.players.len() {
                    if j == i || !self.players[j].alive { continue; }
                    let dx = self.players[j].position.x - stomp_pos.x;
                    let dy = self.players[j].position.y - stomp_pos.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < STOMP_RADIUS {
                        let falloff = 1.0 - (dist / STOMP_RADIUS);
                        self.players[j].hp = (self.players[j].hp - STOMP_DAMAGE * falloff).max(0.0);
                        self.players[j].hit_flash_timer = HIT_FLASH_DURATION;
                        let hit_pos = self.players[j].render_center();
                        events.push(GameEvent::PlayerHit {
                            x: hit_pos.x, y: hit_pos.y, z: hit_pos.z,
                            r: self.players[j].color.r, g: self.players[j].color.g, b: self.players[j].color.b,
                        });
                    }
                }
                events.push(GameEvent::PlayerDied {
                    x: stomp_pos.x, y: stomp_pos.y, z: stomp_pos.z,
                    r: 255, g: 160, b: 60,
                });
            }

            // Reload
            let max_ammo = MAX_BULLETS + self.players[i].stats.extra_ammo;
            if self.players[i].reload_timer > 0.0 {
                self.players[i].reload_timer = (self.players[i].reload_timer - dt).max(0.0);
                if self.players[i].reload_timer <= 0.0 {
                    self.players[i].bullets_remaining = max_ammo;
                }
            }

            // Shoot / Laser
            if self.players[i].stats.laser {
                self.players[i].laser_active = false;
                if inp.shoot_held
                    && self.players[i].bullets_remaining > 0
                    && self.players[i].reload_timer <= 0.0
                {
                    self.players[i].laser_active = true;
                    let stats = self.players[i].stats.clone();
                    let drain_interval = 0.4 * stats.shoot_cooldown_mult;
                    self.players[i].shoot_cooldown += dt;
                    if self.players[i].shoot_cooldown >= drain_interval {
                        self.players[i].shoot_cooldown -= drain_interval;
                        self.players[i].bullets_remaining -= 1;
                        if self.players[i].bullets_remaining <= 0 {
                            self.players[i].bullets_remaining = 0;
                            self.players[i].reload_timer = RELOAD_TIME * stats.reload_time_mult;
                            self.players[i].laser_active = false;
                        }
                    }
                    let aim = self.players[i].aim_dir;
                    let ox = self.players[i].position.x + aim.x * 0.5;
                    let oy = self.players[i].position.y + 1.1 + aim.y * 0.5;
                    let dps = 40.0 * stats.bullet_damage_mult;
                    let beam_width = 0.08 * stats.bullet_radius_mult;

                    // Build beam directions (triple_shot = 3 beams)
                    let mut aims: Vec<Vector2> = vec![aim];
                    if stats.triple_shot {
                        let angle = std::f32::consts::PI / 12.0;
                        for &sign in &[-1.0_f32, 1.0] {
                            let a = sign * angle;
                            aims.push(Vector2::new(
                                aim.x * a.cos() - aim.y * a.sin(),
                                aim.x * a.sin() + aim.y * a.cos(),
                            ));
                        }
                    }

                    for beam_aim in &aims {
                        let mut max_t = 50.0_f32;
                        // Platform collision (phantom = pass through)
                        if !stats.phantom {
                            for platform in &self.level.platforms {
                                if let Some(t) = ray_aabb_t(ox, oy, beam_aim.x, beam_aim.y, &platform.aabb) {
                                    if t > 0.0 && t < max_t { max_t = t; }
                                }
                            }
                        }
                        // Hit players (piercing = hit all in line)
                        let mut hit_players: Vec<usize> = Vec::new();
                        for j in 0..self.players.len() {
                            if j == i || !self.players[j].alive || self.players[j].ghost_timer > 0.0 { continue; }
                            let mut paabb = self.players[j].aabb();
                            paabb.min.x -= beam_width;
                            paabb.min.y -= beam_width;
                            paabb.max.x += beam_width;
                            paabb.max.y += beam_width;
                            if let Some(t) = ray_aabb_t(ox, oy, beam_aim.x, beam_aim.y, &paabb) {
                                if t > 0.0 && t < max_t {
                                    hit_players.push(j);
                                    if !stats.piercing { max_t = t; }
                                }
                            }
                        }
                        for j in &hit_players {
                            self.players[*j].hp = (self.players[*j].hp - dps * dt).max(0.0);
                            self.players[*j].hit_flash_timer = HIT_FLASH_DURATION;
                            if stats.poison { self.players[*j].poison_timer = 3.0; }
                            if stats.bounceback {
                                let dx = self.players[*j].position.x - self.players[i].position.x;
                                let dy = self.players[*j].position.y - self.players[i].position.y;
                                let d = (dx * dx + dy * dy).sqrt().max(0.01);
                                self.players[*j].velocity.x += (dx / d) * 8.0 * dt;
                                self.players[*j].velocity.y += (dy / d) * 8.0 * dt;
                            }
                        }
                        if stats.vampire_heal > 0.0 && !hit_players.is_empty() {
                            self.players[i].hp = (self.players[i].hp + stats.vampire_heal * dt * 2.0).min(self.players[i].max_hp);
                        }
                    }
                } else {
                    self.players[i].shoot_cooldown = 0.0;
                }
            } else {
                self.players[i].laser_active = false;
                self.players[i].shoot_cooldown = (self.players[i].shoot_cooldown - dt).max(0.0);
                if inp.shoot_pressed
                    && self.players[i].shoot_cooldown <= 0.0
                    && self.players[i].bullets_remaining > 0
                    && self.players[i].reload_timer <= 0.0
                {
                    let aim = self.players[i].aim_dir;
                    let color = self.players[i].color;
                    let spawn = Vector3::new(
                        self.players[i].position.x + aim.x * 0.5,
                        self.players[i].position.y + 1.1 + aim.y * 0.5,
                        self.players[i].position.z,
                    );
                    let stats = self.players[i].stats.clone();

                    if stats.shotgun {
                        let count = self.players[i].bullets_remaining;
                        let total_spread = std::f32::consts::PI / 6.0;
                        for s in 0..count {
                            let t = if count > 1 {
                                (s as f32 / (count - 1) as f32) - 0.5
                            } else {
                                0.0
                            };
                            let angle = t * total_spread;
                            let cos_a = angle.cos();
                            let sin_a = angle.sin();
                            let rotated = Vector2::new(
                                aim.x * cos_a - aim.y * sin_a,
                                aim.x * sin_a + aim.y * cos_a,
                            );
                            self.bullets.push(Bullet::new_with_stats(spawn, rotated, i, color, &stats));
                        }
                        self.players[i].bullets_remaining = 0;
                    } else {
                        self.bullets.push(Bullet::new_with_stats(spawn, aim, i, color, &stats));
                        self.players[i].bullets_remaining -= 1;

                        if stats.triple_shot {
                            let angle = std::f32::consts::PI / 4.0;
                            for &sign in &[-1.0_f32, 1.0] {
                                let a = sign * angle;
                                let cos_a = a.cos();
                                let sin_a = a.sin();
                                let rotated = Vector2::new(
                                    aim.x * cos_a - aim.y * sin_a,
                                    aim.x * sin_a + aim.y * cos_a,
                                );
                                self.bullets.push(Bullet::new_with_stats(spawn, rotated, i, color, &stats));
                            }
                        }
                    }

                    let mut cd = SHOOT_COOLDOWN * stats.shoot_cooldown_mult;
                    if self.players[i].overclock_timer > 0.0 { cd *= 0.5; }
                    self.players[i].shoot_cooldown = cd;
                    if self.players[i].bullets_remaining <= 0 {
                        self.players[i].bullets_remaining = 0;
                        self.players[i].reload_timer = RELOAD_TIME * self.players[i].stats.reload_time_mult;
                    }
                }
            }

            // Tick ability cooldowns
            for (card_id, cd) in self.players[i].cards.iter_mut() {
                if cards::CARD_CATALOG[*card_id as u8 as usize].is_ability() {
                    *cd = (*cd - dt).max(0.0);
                }
            }

            // Activate abilities on right-click
            if inp.ability_pressed {
                let to_activate: Vec<(usize, cards::CardId)> = self.players[i].cards.iter()
                    .enumerate()
                    .filter(|(_, (card_id, cd))| {
                        cards::CARD_CATALOG[*card_id as u8 as usize].is_ability() && *cd <= 0.0
                    })
                    .map(|(j, (card_id, _))| (j, *card_id))
                    .collect();
                for (j, card_id) in to_activate {
                    let (cd, effect) = cards::activate_ability(card_id, &mut self.players[i]);
                    self.players[i].cards[j].1 = cd;
                    match effect {
                        cards::AbilityEffect::Screech => {
                            let my_pos = self.players[i].render_center();
                            for k in 0..self.players.len() {
                                if k == i || !self.players[k].alive { continue; }
                                let other_pos = self.players[k].render_center();
                                let dx = other_pos.x - my_pos.x;
                                let dy = other_pos.y - my_pos.y;
                                let dist = (dx * dx + dy * dy).sqrt();
                                let nx = dx / dist.max(0.01);
                                let ny = dy / dist.max(0.01);
                                self.players[k].velocity.x += nx * 30.0;
                                self.players[k].velocity.y += ny * 30.0;
                            }
                        }
                        cards::AbilityEffect::GravityWell => {
                            let aim = self.players[i].aim_dir;
                            let spawn = Vector3::new(
                                self.players[i].position.x + aim.x * 5.0,
                                self.players[i].position.y + 1.0 + aim.y * 5.0,
                                self.players[i].position.z,
                            );
                            self.gravity_wells.push(GravityWellEntity {
                                position: spawn,
                                owner: i,
                                lifetime: GRAVITY_WELL_LIFETIME,
                            });
                        }
                        cards::AbilityEffect::Decoy => {
                            let aim = self.players[i].aim_dir;
                            self.clones.push(CloneEntity {
                                position: self.players[i].render_center(),
                                velocity: Vector3::new(aim.x * CLONE_SPEED, aim.y * CLONE_SPEED, 0.0),
                                owner: i,
                                lifetime: CLONE_LIFETIME,
                                color: self.players[i].color,
                            });
                        }
                        cards::AbilityEffect::Ghost | cards::AbilityEffect::None => {}
                    }
                }
            }
        }

        // Tick hit flash timers
        for player in self.players.iter_mut() {
            player.hit_flash_timer = (player.hit_flash_timer - dt).max(0.0);
        }

        // Tick poison (10 DPS)
        for player in self.players.iter_mut() {
            if player.alive && player.poison_timer > 0.0 {
                player.poison_timer = (player.poison_timer - dt).max(0.0);
                player.hp = (player.hp - 10.0 * dt).max(0.0);
            }
        }

        // HP regen
        for player in self.players.iter_mut() {
            if player.alive && player.stats.hp_regen > 0.0 {
                player.hp = (player.hp + player.stats.hp_regen * dt).min(player.max_hp);
            }
        }

        // Tick ghost timer
        for player in self.players.iter_mut() {
            if player.ghost_timer > 0.0 {
                player.ghost_timer = (player.ghost_timer - dt).max(0.0);
            }
        }

        // Tick overclock: boost → crash transition
        for player in self.players.iter_mut() {
            if player.overclock_timer > 0.0 {
                player.overclock_timer = (player.overclock_timer - dt).max(0.0);
                if player.overclock_timer <= 0.0 {
                    player.overclock_crash_timer = 1.5;
                }
            }
            if player.overclock_crash_timer > 0.0 {
                player.overclock_crash_timer = (player.overclock_crash_timer - dt).max(0.0);
            }
        }

        // Tick adrenaline
        for player in self.players.iter_mut() {
            if player.adrenaline_timer > 0.0 {
                player.adrenaline_timer = (player.adrenaline_timer - dt).max(0.0);
            }
        }

        // Rewind history sampling (every 0.1s, max 30 entries = 3s)
        for player in self.players.iter_mut() {
            if !player.alive { continue; }
            let has_rewind = player.cards.iter().any(|(c, _)| *c == cards::CardId::Rewind);
            if has_rewind {
                player.rewind_sample_timer += dt;
                if player.rewind_sample_timer >= 0.1 {
                    player.rewind_sample_timer = 0.0;
                    player.rewind_history.push((player.position.x, player.position.y, player.hp));
                    if player.rewind_history.len() > 30 {
                        player.rewind_history.remove(0);
                    }
                }
            }
        }

        // Leech field: drain nearby enemies, heal self
        let positions: Vec<(f32, f32, bool, bool)> = self.players.iter()
            .map(|p| (p.position.x, p.position.y + p.size.y / 2.0, p.alive, p.stats.leech_field))
            .collect();
        for i in 0..self.players.len() {
            if !positions[i].2 || !positions[i].3 { continue; }
            let mut heal_total = 0.0_f32;
            for j in 0..self.players.len() {
                if j == i || !positions[j].2 { continue; }
                let dx = positions[j].0 - positions[i].0;
                let dy = positions[j].1 - positions[i].1;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < LEECH_FIELD_RADIUS {
                    let drain = LEECH_FIELD_DPS * dt;
                    self.players[j].hp = (self.players[j].hp - drain).max(0.0);
                    heal_total += drain;
                }
            }
            if heal_total > 0.0 {
                self.players[i].hp = (self.players[i].hp + heal_total).min(self.players[i].max_hp);
            }
        }

        // Update bullets
        let (bullet_events, sticky_datas) = update_bullets(&mut self.bullets, &mut self.players, &self.level.platforms, dt);
        events.extend(bullet_events);

        // Convert sticky bomb data to entities
        for sd in sticky_datas {
            self.sticky_bombs.push(StickyBomb {
                position: sd.position,
                owner: sd.owner,
                damage: sd.damage,
                fuse: STICKY_FUSE,
                stuck_to: sd.stuck_to,
                color: sd.color,
            });
        }

        // Tick gravity wells: pull enemies + bullets toward center
        for well in self.gravity_wells.iter_mut() {
            well.lifetime -= dt;
            for player in self.players.iter_mut() {
                if !player.alive || player.ghost_timer > 0.0 { continue; }
                let dx = well.position.x - player.position.x;
                let dy = well.position.y - (player.position.y + player.size.y / 2.0);
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < GRAVITY_WELL_RADIUS && dist > 0.5 {
                    let force = GRAVITY_WELL_FORCE * (1.0 - dist / GRAVITY_WELL_RADIUS);
                    player.velocity.x += (dx / dist) * force * dt;
                    player.velocity.y += (dy / dist) * force * dt;
                }
            }
            for bullet in self.bullets.iter_mut() {
                let dx = well.position.x - bullet.position.x;
                let dy = well.position.y - bullet.position.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < GRAVITY_WELL_RADIUS && dist > 0.3 {
                    let force = GRAVITY_WELL_FORCE * 2.0 * (1.0 - dist / GRAVITY_WELL_RADIUS);
                    bullet.velocity.x += (dx / dist) * force * dt;
                    bullet.velocity.y += (dy / dist) * force * dt;
                }
            }
        }
        self.gravity_wells.retain(|w| w.lifetime > 0.0);

        // Tick clones: chase nearest enemy, explode on contact
        for clone in self.clones.iter_mut() {
            clone.lifetime -= dt;
            // Find nearest enemy to chase
            let mut closest_dist = f32::MAX;
            let mut chase_dir = (0.0_f32, 0.0_f32);
            for (pi, player) in self.players.iter().enumerate() {
                if pi == clone.owner || !player.alive || player.ghost_timer > 0.0 { continue; }
                let dx = player.position.x - clone.position.x;
                let dy = (player.position.y + player.size.y / 2.0) - clone.position.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < closest_dist {
                    closest_dist = dist;
                    let d = dist.max(0.01);
                    chase_dir = (dx / d, dy / d);
                }
            }
            // Steer toward target
            let spd = CLONE_SPEED;
            clone.velocity.x = clone.velocity.x * 0.9 + chase_dir.0 * spd * 0.1;
            clone.velocity.y = clone.velocity.y * 0.9 + chase_dir.1 * spd * 0.1;
            let vel_len = (clone.velocity.x * clone.velocity.x + clone.velocity.y * clone.velocity.y + clone.velocity.z * clone.velocity.z).sqrt();
            if vel_len > spd {
                clone.velocity.x = clone.velocity.x / vel_len * spd;
                clone.velocity.y = clone.velocity.y / vel_len * spd;
            }
            clone.position.x += clone.velocity.x * dt;
            clone.position.y += clone.velocity.y * dt;

            // Explode on contact with enemy
            if closest_dist < CLONE_EXPLODE_RADIUS {
                for player in self.players.iter_mut() {
                    if !player.alive { continue; }
                    let dx = player.position.x - clone.position.x;
                    let dy = (player.position.y + player.size.y / 2.0) - clone.position.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < CLONE_EXPLODE_RADIUS {
                        let falloff = 1.0 - (dist / CLONE_EXPLODE_RADIUS);
                        player.hp = (player.hp - CLONE_DAMAGE * falloff).max(0.0);
                        player.hit_flash_timer = HIT_FLASH_DURATION;
                    }
                }
                events.push(GameEvent::Explosion {
                    x: clone.position.x, y: clone.position.y, z: clone.position.z,
                    r: clone.color.r, g: clone.color.g, b: clone.color.b,
                });
                clone.lifetime = 0.0;
            }
        }
        self.clones.retain(|c| c.lifetime > 0.0);

        // Tick sticky bombs: count down fuse, track stuck player, explode
        for bomb in self.sticky_bombs.iter_mut() {
            // Track stuck player position
            if let Some(pi) = bomb.stuck_to {
                if pi < self.players.len() && self.players[pi].alive {
                    bomb.position = self.players[pi].render_center();
                }
            }
            bomb.fuse -= dt;
        }
        // Explode expired sticky bombs
        let mut bomb_explosions: Vec<(Vector3, usize, f32, Color)> = Vec::new();
        self.sticky_bombs.retain(|b| {
            if b.fuse <= 0.0 {
                bomb_explosions.push((b.position, b.owner, b.damage, b.color));
                false
            } else {
                true
            }
        });
        for (pos, _owner, damage, color) in &bomb_explosions {
            for player in self.players.iter_mut() {
                if !player.alive { continue; }
                let dx = player.position.x - pos.x;
                let dy = (player.position.y + player.size.y / 2.0) - pos.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < STICKY_EXPLODE_RADIUS {
                    let falloff = 1.0 - (dist / STICKY_EXPLODE_RADIUS);
                    player.hp = (player.hp - damage * falloff).max(0.0);
                    player.hit_flash_timer = HIT_FLASH_DURATION;
                }
            }
            events.push(GameEvent::Explosion {
                x: pos.x, y: pos.y, z: pos.z,
                r: color.r, g: color.g, b: color.b,
            });
        }

        // Check for deaths
        for i in 0..self.players.len() {
            if !self.players[i].alive {
                continue;
            }
            if self.players[i].hp <= 0.0 || self.players[i].position.y < -10.0 {
                self.kill_player_server(i, events);
            }
        }

        // Check for round end
        if self.alive_count() <= 1 {
            if let Some(winner_idx) = self.last_alive() {
                let name = self.players[winner_idx].name.clone();
                let c = self.players[winner_idx].color;
                self.scores[winner_idx] += 1;
                self.state = GameState::RoundEnd {
                    winner_index: winner_idx as u8,
                    winner_name: name,
                    winner_color: (c.r, c.g, c.b),
                    timer: ROUND_END_DURATION,
                };
            } else {
                self.reset_round();
            }
        }
    }

    // ── Snapshot generation (server) ─────────────────────────────────────────

    pub fn to_snapshot(&self, events: Vec<GameEvent>) -> WorldSnapshot {
        let (state_tag, state_timer, time_scale) = match &self.state {
            GameState::RoundStart { timer } => (0, *timer, 0.0),
            GameState::Playing => (1, 0.0, 1.0),
            GameState::RoundEnd { timer, .. } => (2, *timer, SLOW_MO_FACTOR),
            GameState::CardPick { .. } => (3, 0.0, 0.0),
            GameState::MatchOver { timer, .. } => (4, *timer, 0.0),
        };

        let players: Vec<PlayerSnapshot> = self.players.iter().enumerate().map(|(i, p)| {
            let (cx, cy) = self.cursor_positions.get(i).copied().unwrap_or((0.5, 0.5));
            PlayerSnapshot {
                pos_x: p.position.x,
                pos_y: p.position.y,
                vel_x: p.velocity.x,
                vel_y: p.velocity.y,
                aim_x: p.aim_dir.x,
                aim_y: p.aim_dir.y,
                hp: p.hp,
                hit_flash: p.hit_flash_timer,
                reload_timer: p.reload_timer,
                shoot_cooldown: p.shoot_cooldown,
                bullets_remaining: p.bullets_remaining as i8,
                alive: p.alive,
                cursor_x: cx,
                cursor_y: cy,
                cards: p.cards.iter().map(|(c, cd)| (*c as u8, *cd)).collect(),
                laser_active: p.laser_active,
                poison_timer: p.poison_timer,
                ghost_timer: p.ghost_timer,
                overclock_timer: p.overclock_timer,
                overclock_crash_timer: p.overclock_crash_timer,
                adrenaline_timer: p.adrenaline_timer,
                upsized_stacks: p.upsized_stacks,
            }
        }).collect();

        let bullets: Vec<BulletSnapshot> = self.bullets.iter().map(|b| BulletSnapshot {
            pos_x: b.position.x,
            pos_y: b.position.y,
            pos_z: b.position.z,
            prev_x: b.prev_position.x,
            prev_y: b.prev_position.y,
            prev_z: b.prev_position.z,
            vel_x: b.velocity.x,
            vel_y: b.velocity.y,
            owner: b.owner as u8,
            lifetime: b.lifetime,
            radius: b.radius,
        }).collect();

        // Card pick fields
        let (card_current_picker, card_offered, card_remaining_pickers, card_phase_timer, card_chosen, card_exit_timer, card_hover) =
            if let GameState::CardPick { current_picker, offered_cards, pick_order, phase_timer, chosen_card, exit_timer, .. } = &self.state {
                (*current_picker, *offered_cards, pick_order.len() as u8, *phase_timer,
                 chosen_card.unwrap_or(0xFF), *exit_timer, self.card_hover)
            } else {
                (0, [0, 0, 0], 0, 0.0, 0xFF, 0.0, 0xFF)
            };

        let (match_winner, match_timer) = if let GameState::MatchOver { winner_index, timer } = &self.state {
            (*winner_index, *timer)
        } else {
            (0, 0.0)
        };

        WorldSnapshot {
            state_tag,
            state_timer,
            time_scale,
            level_id: self.level.id,
            winner_index: self.winner_index(),
            player_count: self.players.len() as u8,
            players,
            scores: self.scores.clone(),
            bullets,
            events,
            card_current_picker,
            card_offered,
            card_remaining_pickers,
            card_phase_timer,
            card_chosen,
            card_exit_timer,
            card_hover,
            match_winner,
            match_timer,
            gravity_wells: self.gravity_wells.iter().map(|w| GravityWellSnapshot {
                x: w.position.x, y: w.position.y,
                owner: w.owner as u8, lifetime: w.lifetime,
            }).collect(),
            clones: self.clones.iter().map(|c| CloneSnapshot {
                x: c.position.x, y: c.position.y,
                vel_x: c.velocity.x, vel_y: c.velocity.y,
                owner: c.owner as u8, lifetime: c.lifetime,
            }).collect(),
            sticky_bombs: self.sticky_bombs.iter().map(|s| StickyBombSnapshot {
                x: s.position.x, y: s.position.y,
                owner: s.owner as u8, fuse: s.fuse,
                stuck_to: s.stuck_to.map(|i| i as u8).unwrap_or(0xFF),
            }).collect(),
        }
    }

    // ── Snapshot application (client) ────────────────────────────────────────

    pub fn apply_snapshot(&mut self, snap: &WorldSnapshot) {
        if snap.level_id != self.level.id {
            self.level = level::level_by_id(snap.level_id);
        }

        let names: Vec<String> = self.players.iter().map(|p| p.name.clone()).collect();
        let colors: Vec<Color> = self.players.iter().map(|p| p.color).collect();
        self.state = snap.game_state(&names, &colors);
        self.card_hover = snap.card_hover;

        for (i, ps) in snap.players.iter().enumerate() {
            if i >= self.players.len() { break; }
            let p = &mut self.players[i];
            p.position.x = ps.pos_x;
            p.position.y = ps.pos_y;
            p.velocity.x = ps.vel_x;
            p.velocity.y = ps.vel_y;
            p.aim_dir.x = ps.aim_x;
            p.aim_dir.y = ps.aim_y;
            p.hp = ps.hp;
            p.hit_flash_timer = ps.hit_flash;
            p.reload_timer = ps.reload_timer;
            p.shoot_cooldown = ps.shoot_cooldown;
            p.bullets_remaining = ps.bullets_remaining as i32;
            p.alive = ps.alive;
            if i < self.cursor_positions.len() {
                self.cursor_positions[i] = (ps.cursor_x, ps.cursor_y);
            }
            p.cards.clear();
            for (card_id_u8, cooldown) in &ps.cards {
                if let Some(card_id) = cards::CardId::from_u8(*card_id_u8) {
                    p.cards.push((card_id, *cooldown));
                }
            }
            p.stats = cards::compute_stats(&p.cards);
            p.max_hp = (100.0 + p.stats.max_hp_bonus) * p.stats.max_hp_mult;
            p.laser_active = ps.laser_active;
            p.poison_timer = ps.poison_timer;
            p.ghost_timer = ps.ghost_timer;
            p.overclock_timer = ps.overclock_timer;
            p.overclock_crash_timer = ps.overclock_crash_timer;
            p.adrenaline_timer = ps.adrenaline_timer;
            p.upsized_stacks = ps.upsized_stacks;
            // Apply upsized size on client
            let upsized_mult = 1.0 + 0.05 * p.upsized_stacks as f32;
            p.size.x = 0.6 * p.stats.size_mult * upsized_mult;
            p.size.y = 1.6 * p.stats.size_mult * upsized_mult;
            p.size.z = 0.6 * p.stats.size_mult * upsized_mult;
        }

        for (i, &s) in snap.scores.iter().enumerate() {
            if i < self.scores.len() {
                self.scores[i] = s;
            }
        }

        self.bullets.clear();
        for bs in &snap.bullets {
            let owner = bs.owner as usize;
            let color = if owner < self.players.len() {
                self.players[owner].color
            } else {
                Color::WHITE
            };
            self.bullets.push(Bullet {
                position: Vector3::new(bs.pos_x, bs.pos_y, bs.pos_z),
                prev_position: Vector3::new(bs.prev_x, bs.prev_y, bs.prev_z),
                velocity: Vector2::new(bs.vel_x, bs.vel_y),
                owner,
                lifetime: bs.lifetime,
                color,
                radius: bs.radius,
                damage: 0.0,
                bounces_remaining: 0,
                homing: false,
                piercing: false,
                explosive: false,
                poison: false,
                gravity_mult: 1.0,
                phantom: false,
                split_on_hit: false,
                hot_potato: false,
                sticky: false,
            });
        }

        // Reconstruct entities from snapshot
        self.gravity_wells.clear();
        for w in &snap.gravity_wells {
            self.gravity_wells.push(GravityWellEntity {
                position: Vector3::new(w.x, w.y, 0.0),
                owner: w.owner as usize,
                lifetime: w.lifetime,
            });
        }
        self.clones.clear();
        for c in &snap.clones {
            let color = if (c.owner as usize) < self.players.len() {
                self.players[c.owner as usize].color
            } else {
                Color::WHITE
            };
            self.clones.push(CloneEntity {
                position: Vector3::new(c.x, c.y, 0.0),
                velocity: Vector3::new(c.vel_x, c.vel_y, 0.0),
                owner: c.owner as usize,
                lifetime: c.lifetime,
                color,
            });
        }
        self.sticky_bombs.clear();
        for s in &snap.sticky_bombs {
            let color = if (s.owner as usize) < self.players.len() {
                self.players[s.owner as usize].color
            } else {
                Color::WHITE
            };
            self.sticky_bombs.push(StickyBomb {
                position: Vector3::new(s.x, s.y, 0.0),
                owner: s.owner as usize,
                damage: 0.0,
                fuse: s.fuse,
                stuck_to: if s.stuck_to == 0xFF { None } else { Some(s.stuck_to as usize) },
                color,
            });
        }

        spawn_from_events(&snap.events, &mut self.particles, &mut self.rng);
    }
}
