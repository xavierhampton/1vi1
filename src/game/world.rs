use raylib::prelude::*;

use crate::combat::bullet::{Bullet, SHOOT_COOLDOWN};
use crate::combat::combat::update_bullets;
use crate::combat::particles::{spawn_from_events, Particle, Rng};
use crate::game::cards;
use crate::game::net::{BulletSnapshot, GameEvent, HealingZoneSnapshot, PlayerSnapshot, StickyBombSnapshot, WorldSnapshot};
use crate::game::state::GameState;
use crate::level::level::{self, Level, LevelQueue};
use crate::lobby::state::{GameSettings, LobbyState};
use crate::player::input::PlayerInput;
use crate::player::movement;
use crate::player::player::{Player, HIT_FLASH_DURATION};

pub const MAX_BULLETS: i32 = 3;
pub const RELOAD_TIME: f32 = 1.5;

pub const COUNTDOWN_DURATION: f32 = 3.0;
const ROUND_END_DURATION: f32 = 3.5;
const SLOW_MO_FACTOR: f32 = 0.25;

const CARD_ENTRANCE_DURATION: f32 = 0.8;
const CARD_EXIT_DURATION: f32 = 0.8;
const MATCH_OVER_DURATION: f32 = 5.0;

const STICKY_FUSE: f32 = 2.0;
const STICKY_EXPLODE_RADIUS: f32 = 3.0;
const HEALING_ZONE_RADIUS: f32 = 3.0;
const HEALING_ZONE_HPS: f32 = 15.0;
const HEALING_ZONE_LIFETIME: f32 = 5.0;
const ECHO_DELAY: f32 = 0.3;

pub struct StickyBomb {
    pub position: Vector3,
    pub owner: usize,
    pub damage: f32,
    pub fuse: f32,
    pub stuck_to: Option<usize>,
    pub color: Color,
}

pub struct HealingZone {
    pub position: Vector3,
    pub owner: usize,
    pub lifetime: f32,
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
    pub sticky_bombs: Vec<StickyBomb>,
    pub healing_zones: Vec<HealingZone>,
    pub echo_queue: Vec<(f32, Bullet)>, // (delay_remaining, bullet_to_spawn)
    pub game_settings: GameSettings,
    pub latest_events: Vec<GameEvent>, // last frame's events for audio
    pub elapsed_time: f32,
    pub level_queue: LevelQueue,
}

impl World {
    pub fn from_lobby(lobby: &LobbyState) -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        let mut rng = Rng::new(seed);
        let mut level_queue = LevelQueue::new();
        let level = level_queue.next(rng.next());
        let count = lobby.slots.len().clamp(2, 4);
        let settings = lobby.settings.clone();
        let starting_hp = if settings.sudden_death { 1.0 } else { settings.starting_hp };
        let players: Vec<Player> = lobby.slots.iter().enumerate().take(count).map(|(i, slot)| {
            let mut p = Player::new(
                level.spawn_points[i],
                Vector3::new(0.6, 1.6, 0.6),
                slot.color.to_color(),
                &slot.name,
            );
            p.hp = starting_hp;
            p.max_hp = starting_hp;
            p.accessories = slot.accessories.clone();
            p
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
            sticky_bombs: Vec::new(),
            healing_zones: Vec::new(),
            echo_queue: Vec::new(),
            game_settings: settings,
            latest_events: Vec::new(),
            elapsed_time: 0.0,
            level_queue,
        }
    }

    fn base_hp(&self) -> f32 {
        if self.game_settings.sudden_death { 1.0 } else { self.game_settings.starting_hp }
    }

    fn reset_round(&mut self) {
        self.level = self.level_queue.next(self.rng.next());
        let base_hp = self.base_hp();

        for (i, player) in self.players.iter_mut().enumerate() {
            player.position = self.level.spawn_points[i];
            player.velocity = Vector3::new(0.0, 0.0, 0.0);
            player.alive = true;
            player.hit_flash_timer = 0.0;
            player.reload_timer = 0.0;
            player.shoot_cooldown = 0.0;
            player.aim_dir = Vector2::new(if i % 2 == 0 { 1.0 } else { -1.0 }, 0.0);
            player.poison_timer = 0.0;
            player.ghost_timer = 0.0;
            player.overclock_timer = 0.0;
            player.overclock_crash_timer = 0.0;
            player.adrenaline_timer = 0.0;
            player.bloodthirsty_timer = 0.0;
            player.slow_timer = 0.0;
            player.shake_timer = 0.0;
            player.doppel_history.clear();
            player.doppel_ghost = (0.0, 0.0, 0.0, 0.0);
            player.upsized_stacks = 0;
            player.rewind_history.clear();
            player.rewind_sample_timer = 0.0;
            for (card_id, cd) in player.cards.iter_mut() {
                *cd = match card_id {
                    cards::CardId::Screech => 5.0,
                    _ => 0.0,
                };
            }
            player.stats = cards::compute_stats(&player.cards);
            cards::apply_stats(player, &player.stats.clone(), base_hp);
            player.hp = player.max_hp;
            player.bullets_remaining = MAX_BULLETS + player.stats.extra_ammo;
            player.invuln_timer = 0.0;
        }
        self.bullets.clear();
        self.particles.clear();
        self.sticky_bombs.clear();
        self.healing_zones.clear();
        self.echo_queue.clear();
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
        if self.game_settings.everyone_picks {
            // Everyone picks: losers first, then winner
            let mut order: Vec<u8> = (0..self.players.len() as u8)
                .filter(|&i| i != winner_idx)
                .collect();
            order.push(winner_idx);
            order
        } else {
            // Losers only
            (0..self.players.len() as u8)
                .filter(|&i| i != winner_idx)
                .collect()
        }
    }

    fn enter_card_pick(&mut self, winner_idx: u8) {
        let mut pick_order = self.build_pick_order(winner_idx);
        if pick_order.is_empty() {
            self.reset_round();
            return;
        }
        let current_picker = pick_order.remove(0);
        let mut seed = self.rng.next();
        let held = &self.players[current_picker as usize].cards;
        let offered = cards::random_cards(&mut seed, 3, held);
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

    pub fn dev_toggle_card(&mut self, player_idx: usize, card_id: cards::CardId) {
        let base_hp = self.base_hp();
        if player_idx < self.players.len() {
            let p = &mut self.players[player_idx];
            if let Some(pos) = p.cards.iter().position(|(id, _)| *id == card_id) {
                p.cards.remove(pos);
            } else {
                p.cards.push((card_id, 0.0));
            }
            p.stats = cards::compute_stats(&p.cards);
            cards::apply_stats(p, &p.stats.clone(), base_hp);
        }
    }

    // ── Composable bullet creation (synergy system) ─────────────────────────

    /// Create all bullets for a single shot, applying TriShot, DoubleVision,
    /// RearShot multiplicatively. Returns bullets to spawn.
    fn create_shot_bullets(
        &mut self,
        owner: usize,
        aim: Vector2,
        spawn: Vector3,
        color: Color,
        stats: &cards::PlayerStats,
    ) -> Vec<Bullet> {
        let mut aim_dirs: Vec<Vector2> = vec![aim];

        // TriShot: add 2 bullets at ±45° spread
        if stats.tri_shot {
            let angle = std::f32::consts::PI / 4.0;
            let mut extra = Vec::new();
            for base_aim in &aim_dirs {
                for &sign in &[-1.0_f32, 1.0] {
                    let a = sign * angle;
                    extra.push(Vector2::new(
                        base_aim.x * a.cos() - base_aim.y * a.sin(),
                        base_aim.x * a.sin() + base_aim.y * a.cos(),
                    ));
                }
            }
            aim_dirs.extend(extra);
        }

        // DoubleVision: duplicate all bullets with slight offset
        if stats.double_shot {
            let mut extra = Vec::new();
            for base_aim in &aim_dirs {
                let offset_angle: f32 = 0.08; // ~4.5° offset
                extra.push(Vector2::new(
                    base_aim.x * offset_angle.cos() - base_aim.y * offset_angle.sin(),
                    base_aim.x * offset_angle.sin() + base_aim.y * offset_angle.cos(),
                ));
            }
            aim_dirs.extend(extra);
        }

        // RearShot: duplicate everything but aimed backward
        if stats.rear_shot {
            let rear: Vec<Vector2> = aim_dirs.iter()
                .map(|a| Vector2::new(-a.x, -a.y))
                .collect();
            aim_dirs.extend(rear);
        }

        // Angry & Blind: randomize all bullet directions
        if stats.angry_blind {
            for a in aim_dirs.iter_mut() {
                let rand_angle = (self.rng.next() as f32 / u64::MAX as f32) * std::f32::consts::TAU;
                *a = Vector2::new(rand_angle.cos(), rand_angle.sin());
            }
        }

        // Cursed Mag: random -30% to +30% damage per bullet
        let mut result = Vec::with_capacity(aim_dirs.len());
        for dir in &aim_dirs {
            let mut bullet = Bullet::new_with_stats(spawn, *dir, owner, color, stats);
            if stats.cursed_mag {
                let rand_val = (self.rng.next() as f32 / u64::MAX as f32) * 0.6 - 0.3;
                bullet.damage *= 1.0 + rand_val;
            }
            result.push(bullet);
        }
        result
    }

    // ── Server-authoritative update ─────────────────────────────────────────

    pub fn server_update(&mut self, inputs: &[PlayerInput], raw_dt: f32) -> Vec<GameEvent> {
        let dt = raw_dt * self.game_settings.turbo_speed;
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
                // Frozen during countdown — only aim updates
                for (i, inp) in inputs.iter().enumerate() {
                    if i < self.players.len() && self.players[i].alive {
                        self.players[i].aim_dir = inp.aim_dir;
                    }
                }
                if new_timer <= 0.0 {
                    // Grant invulnerability when round actually starts
                    for player in self.players.iter_mut() {
                        player.invuln_timer = self.game_settings.spawn_invuln;
                    }
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
                        movement::update(&mut self.players[wi as usize], inp, &self.level.platforms, slow_dt, self.game_settings.gravity_scale);
                        self.players[wi as usize].aim_dir = inp.aim_dir;
                    }
                }

                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    if wi == 0xFF {
                        // Draw — no winner, skip card pick, just reset
                        self.reset_round();
                    } else if self.scores.get(wi as usize).copied().unwrap_or(0) >= self.game_settings.wins_to_match {
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
                // Update aim for all players during card pick (taunting)
                for (i, inp) in inputs.iter().enumerate() {
                    if i < self.players.len() {
                        self.players[i].aim_dir = inp.aim_dir;
                    }
                }
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
        let base_hp = self.base_hp();
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
                            // Gambler: add 2 random powerups instead
                            if card_id == cards::CardId::Gambler {
                                for _ in 0..2 {
                                    let mut seed = self.rng.next();
                                    let pool: Vec<u8> = cards::CARD_CATALOG.iter()
                                        .filter(|c| c.is_powerup() && c.id != cards::CardId::Gambler)
                                        .map(|c| c.id as u8)
                                        .collect();
                                    if !pool.is_empty() {
                                        seed ^= seed << 13;
                                        seed ^= seed >> 7;
                                        seed ^= seed << 17;
                                        let idx = (seed as usize) % pool.len();
                                        if let Some(random_id) = cards::CardId::from_u8(pool[idx]) {
                                            self.players[picker_idx].cards.push((random_id, 0.0));
                                        }
                                    }
                                }
                            } else {
                                self.players[picker_idx].cards.push((card_id, 0.0));
                            }
                            let p = &mut self.players[picker_idx];
                            p.stats = cards::compute_stats(&p.cards);
                            cards::apply_stats(p, &p.stats.clone(), base_hp);
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
                    let held = &self.players[next_picker as usize].cards;
                    let new_offered = cards::random_cards(&mut seed, 3, held);
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
        self.elapsed_time += dt;

        // Track which players actually fire this frame (for DoppelGanger)
        let mut did_fire = vec![false; self.players.len()];

        // Check if any player has Confusion — inverts opponent controls
        let confusion_owners: Vec<usize> = self.players.iter().enumerate()
            .filter(|(_, p)| p.alive && p.stats.confusion)
            .map(|(i, _)| i)
            .collect();

        for i in 0..self.players.len() {
            if !self.players[i].alive {
                continue;
            }
            let mut inp = inputs.get(i).cloned().unwrap_or_else(PlayerInput::empty);

            // Confusion: invert controls if any opponent has Confusion
            let confused = confusion_owners.iter().any(|&owner| owner != i);
            if confused {
                inp.move_dir = -inp.move_dir;
                inp.aim_dir.x = -inp.aim_dir.x;
                inp.aim_dir.y = -inp.aim_dir.y;
            }

            // Upsized stacks → bigger hitbox
            let upsized_mult = 1.0 + 0.05 * self.players[i].upsized_stacks as f32;
            self.players[i].size.x = 0.6 * self.players[i].stats.size_mult * upsized_mult;
            self.players[i].size.y = 1.6 * self.players[i].stats.size_mult * upsized_mult;
            self.players[i].size.z = 0.6 * self.players[i].stats.size_mult * upsized_mult;

            // Compute speed multiplier from active buffs/debuffs
            let mut speed_mult = 1.0_f32;
            if self.players[i].overclock_timer > 0.0 { speed_mult *= 2.0; }
            if self.players[i].overclock_crash_timer > 0.0 { speed_mult *= 0.6; }
            if self.players[i].adrenaline_timer > 0.0 { speed_mult *= 1.5; }
            if self.players[i].bloodthirsty_timer > 0.0 { speed_mult *= 1.5; }
            if self.players[i].slow_timer > 0.0 { speed_mult *= 0.5; }

            let was_grounded = self.players[i].grounded;
            let prev_vy = self.players[i].velocity.y;
            movement::update_with_speed(&mut self.players[i], &inp, &self.level.platforms, dt, speed_mult, self.game_settings.gravity_scale);
            self.players[i].aim_dir = inp.aim_dir;

            // Detect jump (velocity went from non-positive to jump velocity)
            if self.players[i].velocity.y > 5.0 && prev_vy <= 0.1 {
                events.push(GameEvent::Jumped { owner: i as u8 });
            }
            // Detect landing (was airborne, now grounded)
            if !was_grounded && self.players[i].grounded {
                events.push(GameEvent::Landed { owner: i as u8 });
            }

            // Reload
            let max_ammo = MAX_BULLETS + self.players[i].stats.extra_ammo;
            if self.players[i].reload_timer > 0.0 {
                let mut reload_mult = self.players[i].stats.reload_time_mult;
                if self.players[i].overclock_timer > 0.0 { reload_mult *= 0.5; }
                if self.players[i].adrenaline_timer > 0.0 { reload_mult *= 0.5; }
                self.players[i].reload_timer = (self.players[i].reload_timer - dt / reload_mult.max(0.1)).max(0.0);
                if self.players[i].reload_timer <= 0.0 {
                    self.players[i].bullets_remaining = max_ammo;
                }
            }

            // Shoot
            self.players[i].shoot_cooldown = (self.players[i].shoot_cooldown - dt).max(0.0);
            if inp.shoot_pressed
                && self.players[i].shoot_cooldown <= 0.0
                && self.players[i].bullets_remaining > 0
                && self.players[i].reload_timer <= 0.0
            {
                let aim = self.players[i].aim_dir;
                let color = self.players[i].color;
                let size_scale = self.players[i].size.x / 0.6;
                let head_y = self.players[i].position.y + 1.15 * size_scale;
                let spawn = Vector3::new(
                    self.players[i].position.x + aim.x * 0.5,
                    head_y + aim.y * 0.5,
                    self.players[i].position.z,
                );
                let stats = self.players[i].stats.clone();

                if stats.shotgun {
                    // Shotgun: fire all ammo at once, each bullet gets full synergy treatment
                    let count = self.players[i].bullets_remaining;
                    let total_spread = std::f32::consts::PI / 6.0;
                    for s in 0..count {
                        let t = if count > 1 {
                            (s as f32 / (count - 1) as f32) - 0.5
                        } else {
                            0.0
                        };
                        let angle = t * total_spread;
                        let rotated = Vector2::new(
                            aim.x * angle.cos() - aim.y * angle.sin(),
                            aim.x * angle.sin() + aim.y * angle.cos(),
                        );
                        let shot_bullets = self.create_shot_bullets(i, rotated, spawn, color, &stats);
                        // Echo Shot: queue ghost copies
                        if stats.echo_shot {
                            for b in &shot_bullets {
                                let mut ghost = b.clone();
                                ghost.damage *= 0.5;
                                ghost.color = Color::new(color.r / 2 + 80, color.g / 2 + 80, color.b / 2 + 80, 200);
                                self.echo_queue.push((ECHO_DELAY, ghost));
                            }
                        }
                        self.bullets.extend(shot_bullets);
                    }
                    if stats.infinite_ammo {
                        // Infinite ammo: immediately refill after shotgun blast
                        self.players[i].bullets_remaining = MAX_BULLETS + stats.extra_ammo;
                    } else {
                        self.players[i].bullets_remaining = 0;
                    }
                } else {
                    let shot_bullets = self.create_shot_bullets(i, aim, spawn, color, &stats);
                    // Echo Shot
                    if stats.echo_shot {
                        for b in &shot_bullets {
                            let mut ghost = b.clone();
                            ghost.damage *= 0.5;
                            ghost.color = Color::new(color.r / 2 + 80, color.g / 2 + 80, color.b / 2 + 80, 200);
                            self.echo_queue.push((ECHO_DELAY, ghost));
                        }
                    }
                    self.bullets.extend(shot_bullets);

                    if stats.infinite_ammo {
                        // Don't consume ammo
                    } else {
                        self.players[i].bullets_remaining -= 1;
                    }
                }

                let mut cd = SHOOT_COOLDOWN * stats.shoot_cooldown_mult;
                if self.players[i].overclock_timer > 0.0 { cd *= 0.5; }
                if self.players[i].bloodthirsty_timer > 0.0 { cd *= 0.5; }
                self.players[i].shoot_cooldown = cd;
                did_fire[i] = true;

                // Emit BulletFired event for audio
                events.push(GameEvent::BulletFired {
                    x: spawn.x, y: spawn.y, z: spawn.z,
                    vx: aim.x, vy: aim.y,
                    owner: i as u8,
                    r: color.r, g: color.g, b: color.b,
                });

                if !stats.no_reload && !stats.infinite_ammo && self.players[i].bullets_remaining <= 0 {
                    self.players[i].bullets_remaining = 0;
                    self.players[i].reload_timer = RELOAD_TIME;
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
                    if card_id == cards::CardId::Dash {
                        events.push(GameEvent::Dashed { owner: i as u8 });
                    }
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
                        cards::AbilityEffect::Sage => {
                            self.healing_zones.push(HealingZone {
                                position: self.players[i].render_center(),
                                owner: i,
                                lifetime: HEALING_ZONE_LIFETIME,
                            });
                        }
                        cards::AbilityEffect::BulletManip => {
                            // All bullets on map become yours and are homing toward nearest enemy
                            for bullet in self.bullets.iter_mut() {
                                bullet.owner = i;
                                bullet.homing = true;
                                bullet.color = self.players[i].color;
                            }
                        }
                        cards::AbilityEffect::CaseOh => {
                            // Set shake_timer on all opponents — long violent shake
                            for k in 0..self.players.len() {
                                if k == i { continue; }
                                self.players[k].shake_timer = 3.0;
                            }
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

        // HP regen (from stats + soul siphon bonus)
        for player in self.players.iter_mut() {
            if player.alive && player.stats.hp_regen > 0.0 {
                player.hp = (player.hp + player.stats.hp_regen * dt).min(player.max_hp + player.soul_siphon_bonus_hp);
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

        // Tick bloodthirsty
        for player in self.players.iter_mut() {
            if player.bloodthirsty_timer > 0.0 {
                player.bloodthirsty_timer = (player.bloodthirsty_timer - dt).max(0.0);
            }
        }

        // Tick slow (ice shots)
        for player in self.players.iter_mut() {
            if player.slow_timer > 0.0 {
                player.slow_timer = (player.slow_timer - dt).max(0.0);
            }
        }

        // Tick shake (CaseOh)
        for player in self.players.iter_mut() {
            if player.shake_timer > 0.0 {
                player.shake_timer = (player.shake_timer - dt).max(0.0);
            }
        }

        // Tick spawn invulnerability
        for player in self.players.iter_mut() {
            if player.invuln_timer > 0.0 {
                player.invuln_timer = (player.invuln_timer - dt).max(0.0);
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

        // DoppelGanger: record history and replay 1s delayed
        let mut doppel_bullets: Vec<Bullet> = Vec::new();
        for i in 0..self.players.len() {
            if !self.players[i].alive || !self.players[i].stats.doppelganger { continue; }
            let shot = did_fire[i];
            let px = self.players[i].position.x;
            let py = self.players[i].position.y;
            let pz = self.players[i].position.z;
            let ax = self.players[i].aim_dir.x;
            let ay = self.players[i].aim_dir.y;
            self.players[i].doppel_history.push((px, py, ax, ay, shot));
            // Keep ~1s of history at ~60fps = 60 samples
            if self.players[i].doppel_history.len() > 60 {
                let old = self.players[i].doppel_history.remove(0);
                self.players[i].doppel_ghost = (old.0, old.1, old.2, old.3);
                // If the 1s-ago sample had a shot, fire from ghost position
                if old.4 {
                    let ghost_scale = self.players[i].size.x / 0.6;
                    let ghost_head_y = old.1 + 1.15 * ghost_scale;
                    let ghost_pos = Vector3::new(old.0 + old.2 * 0.5, ghost_head_y + old.3 * 0.5, pz);
                    let ghost_aim = Vector2::new(old.2, old.3);
                    let color = Color::new(
                        self.players[i].color.r / 2 + 60,
                        self.players[i].color.g / 2 + 60,
                        self.players[i].color.b / 2 + 60,
                        200,
                    );
                    let stats = self.players[i].stats.clone();
                    doppel_bullets.push(Bullet::new_with_stats(ghost_pos, ghost_aim, i, color, &stats));
                }
            } else {
                self.players[i].doppel_ghost = (px, py, ax, ay);
            }
        }
        self.bullets.extend(doppel_bullets);

        // Tick echo queue: spawn delayed ghost bullets
        let mut spawned_echoes = Vec::new();
        for (delay, _) in self.echo_queue.iter_mut() {
            *delay -= dt;
        }
        for (delay, bullet) in self.echo_queue.drain(..).collect::<Vec<_>>() {
            if delay <= 0.0 {
                spawned_echoes.push(bullet);
            } else {
                self.echo_queue.push((delay, bullet));
            }
        }
        self.bullets.extend(spawned_echoes);

        // Update bullets
        let (bullet_events, sticky_datas, hit_infos) = update_bullets(&mut self.bullets, &mut self.players, &self.level.platforms, &self.level.bounce_pads, dt);
        events.extend(bullet_events);

        // Post-hit processing from BulletHitInfo
        for hit in &hit_infos {
            // Void pull: suck ALL alive enemies toward impact point (whole map range)
            if hit.void_pull {
                for j in 0..self.players.len() {
                    if j == hit.owner || !self.players[j].alive || self.players[j].ghost_timer > 0.0 { continue; }
                    let dx = hit.bullet_x - self.players[j].position.x;
                    let dy = hit.bullet_y - (self.players[j].position.y + self.players[j].size.y / 2.0);
                    let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                    // Strong constant pull regardless of distance — void is inescapable
                    let pull_force = 25.0;
                    self.players[j].velocity.x += (dx / dist) * pull_force;
                    self.players[j].velocity.y += (dy / dist) * pull_force;
                }
            }
            // Soul Siphon: grant +5 permanent max HP on kill
            if hit.target < self.players.len() && !self.players[hit.target].alive {
                if hit.owner < self.players.len() && self.players[hit.owner].stats.soul_siphon {
                    self.players[hit.owner].soul_siphon_bonus_hp += 5.0;
                    self.players[hit.owner].max_hp += 5.0;
                }
            }
        }

        // Explosion AoE splash damage + screen shake from bullet impacts
        for hit in &hit_infos {
            let blast_radius = (hit.damage / 25.0) * 2.0; // 25 dmg = 2 units, 100 dmg = 8 units
            if blast_radius > 1.0 {
                let splash_dmg = hit.damage * 0.4;
                for j in 0..self.players.len() {
                    if j == hit.target || !self.players[j].alive || self.players[j].ghost_timer > 0.0 || self.players[j].invuln_timer > 0.0 { continue; }
                    let dx = self.players[j].position.x - hit.bullet_x;
                    let dy = (self.players[j].position.y + self.players[j].size.y / 2.0) - hit.bullet_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < blast_radius {
                        let falloff = 1.0 - (dist / blast_radius);
                        let aoe_dmg = splash_dmg * falloff * self.players[j].stats.damage_taken_mult;
                        self.players[j].hp = (self.players[j].hp - aoe_dmg).max(0.0);
                        self.players[j].hit_flash_timer = HIT_FLASH_DURATION;
                        // Knockback from explosion
                        if dist > 0.01 {
                            let kb = 10.0 * falloff;
                            self.players[j].velocity.x += (dx / dist) * kb;
                            self.players[j].velocity.y += (dy / dist) * kb;
                        }
                    }
                }
            }
        }

        // Explosion screen shake from all explosion events
        for ev in events.iter() {
            if let GameEvent::Explosion { x, y, radius, .. } = ev {
                if *radius > 1.75 {
                    let shake_intensity = ((*radius - 1.75) * 0.2).min(0.5);
                    for player in self.players.iter_mut() {
                        if !player.alive { continue; }
                        let dx = player.position.x - x;
                        let dy = (player.position.y + player.size.y / 2.0) - y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        let falloff = (1.0 - dist / 25.0).max(0.0);
                        let shake = shake_intensity * falloff;
                        if shake > player.shake_timer {
                            player.shake_timer = shake;
                        }
                    }
                }
            }
        }

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

        // Tick sticky bombs
        for bomb in self.sticky_bombs.iter_mut() {
            if let Some(pi) = bomb.stuck_to {
                if pi < self.players.len() && self.players[pi].alive {
                    bomb.position = self.players[pi].render_center();
                }
            }
            bomb.fuse -= dt;
        }
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
                if !player.alive || player.invuln_timer > 0.0 { continue; }
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
                radius: STICKY_EXPLODE_RADIUS,
            });
        }

        // Tick healing zones: heal anyone inside
        for zone in self.healing_zones.iter_mut() {
            zone.lifetime -= dt;
            for player in self.players.iter_mut() {
                if !player.alive { continue; }
                let dx = player.position.x - zone.position.x;
                let dy = (player.position.y + player.size.y / 2.0) - zone.position.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < HEALING_ZONE_RADIUS {
                    player.hp = (player.hp + HEALING_ZONE_HPS * dt).min(player.max_hp + player.soul_siphon_bonus_hp);
                }
            }
        }
        self.healing_zones.retain(|z| z.lifetime > 0.0);

        // Bounce pad collisions — launch player away from the face they hit
        // Uses expanded margin so pads embedded inside walls still trigger
        // (wall collision pushes player out before this runs, so we need slack)
        let pad_margin = 0.35;
        for pad in &self.level.bounce_pads {
            let pad_min = pad.aabb.min;
            let pad_max = pad.aabb.max;
            let pad_cx = (pad_min.x + pad_max.x) * 0.5;
            let pad_cy = (pad_min.y + pad_max.y) * 0.5;
            let pad_hw = (pad_max.x - pad_min.x) * 0.5;
            let pad_hh = (pad_max.y - pad_min.y) * 0.5;
            for i in 0..self.players.len() {
                if !self.players[i].alive { continue; }
                let p = &self.players[i];
                let pmin_x = p.position.x - p.size.x / 2.0;
                let pmax_x = p.position.x + p.size.x / 2.0;
                let pmin_y = p.position.y;
                let pmax_y = p.position.y + p.size.y;
                // Expanded AABB overlap check (pad + margin)
                if pmax_x <= pad_min.x - pad_margin || pmin_x >= pad_max.x + pad_margin
                    || pmax_y <= pad_min.y - pad_margin || pmin_y >= pad_max.y + pad_margin
                {
                    continue;
                }
                // Determine which face: compare player center to pad center
                let player_cx = p.position.x;
                let player_cy = p.position.y + p.size.y / 2.0;
                let dx = (player_cx - pad_cx) / pad_hw.max(0.01);
                let dy = (player_cy - pad_cy) / pad_hh.max(0.01);
                if dx.abs() > dy.abs() {
                    // Horizontal face — launch left or right
                    let sign = if dx > 0.0 { 1.0 } else { -1.0 };
                    self.players[i].velocity.x = sign * pad.strength;
                    self.players[i].velocity.y = pad.strength * 0.3;
                } else {
                    // Vertical face — launch up or down
                    let sign = if dy > 0.0 { 1.0 } else { -1.0 };
                    self.players[i].velocity.y = sign * pad.strength;
                    self.players[i].velocity.x = 0.0;
                }
                self.players[i].air_jumps = 0;
                events.push(GameEvent::BouncePadHit {
                    x: self.players[i].position.x,
                    y: self.players[i].position.y + self.players[i].size.y * 0.5,
                    z: 0.0,
                });
            }
        }

        // Lava pool damage — drains HP over time (with margin so flush-with-floor pools work)
        // Tick lava sizzle cooldowns
        for p in self.players.iter_mut() {
            if p.lava_sizzle_cd > 0.0 { p.lava_sizzle_cd -= dt; }
        }
        let lava_margin = 0.2;
        for pool in &self.level.lava_pools {
            for i in 0..self.players.len() {
                if !self.players[i].alive { continue; }
                let p = &self.players[i];
                let pmin_x = p.position.x - p.size.x / 2.0;
                let pmax_x = p.position.x + p.size.x / 2.0;
                let pmin_y = p.position.y;
                let pmax_y = p.position.y + p.size.y;
                if pmax_x > pool.aabb.min.x - lava_margin && pmin_x < pool.aabb.max.x + lava_margin
                    && pmax_y > pool.aabb.min.y - lava_margin && pmin_y < pool.aabb.max.y + lava_margin
                {
                    self.players[i].hp -= pool.dps * dt;
                    self.players[i].hit_flash_timer = 0.05;
                    // Emit sizzle event periodically for sound + particles
                    if self.players[i].lava_sizzle_cd <= 0.0 {
                        self.players[i].lava_sizzle_cd = 0.3;
                        let px = self.players[i].position.x;
                        let py = self.players[i].position.y;
                        events.push(GameEvent::LavaSizzle { x: px, y: py, z: 0.0 });
                    }
                }
            }
        }

        // Laser beam damage — line segment vs player AABB, toggled on/off
        for laser in &self.level.lasers {
            let cycle = laser.on_time + laser.off_time;
            if cycle <= 0.0 { continue; }
            let phase = self.elapsed_time % cycle;
            if phase >= laser.on_time { continue; } // laser is off

            let lx0 = laser.start.x; let ly0 = laser.start.y;
            let lx1 = laser.end.x; let ly1 = laser.end.y;

            for i in 0..self.players.len() {
                if !self.players[i].alive || self.players[i].ghost_timer > 0.0 || self.players[i].invuln_timer > 0.0 {
                    continue;
                }
                let p = &self.players[i];
                let pmin_x = p.position.x - p.size.x / 2.0;
                let pmax_x = p.position.x + p.size.x / 2.0;
                let pmin_y = p.position.y;
                let pmax_y = p.position.y + p.size.y;
                // Line segment vs AABB intersection test
                if line_intersects_aabb(lx0, ly0, lx1, ly1, pmin_x, pmin_y, pmax_x, pmax_y) {
                    self.players[i].hp = 0.0;
                }
            }
        }

        // Check for deaths
        for i in 0..self.players.len() {
            if !self.players[i].alive {
                continue;
            }
            if self.players[i].hp <= 0.0 || self.players[i].position.y < -10.0 {
                // Soul Siphon: check who killed them (last hit owner from hit_infos)
                // Already handled above in the hit_infos loop
                self.kill_player_server(i, events);
            }
        }

        // Check for round end
        if self.alive_count() <= 1 {
            if let Some(winner_idx) = self.last_alive() {
                let name = self.players[winner_idx].name.clone();
                let c = self.players[winner_idx].color;
                self.scores[winner_idx] += 1;
                for player in self.players.iter_mut() {
                    player.shake_timer = 0.0;
                }
                self.state = GameState::RoundEnd {
                    winner_index: winner_idx as u8,
                    winner_name: name,
                    winner_color: (c.r, c.g, c.b),
                    timer: ROUND_END_DURATION,
                };
            } else {
                // Draw — all players died simultaneously, show round end screen
                for player in self.players.iter_mut() {
                    player.shake_timer = 0.0;
                }
                self.state = GameState::RoundEnd {
                    winner_index: 0xFF,
                    winner_name: "DRAW".to_string(),
                    winner_color: (200, 200, 200),
                    timer: ROUND_END_DURATION,
                };
            }
        }
    }

    pub fn state_tag(&self) -> u8 {
        match &self.state {
            GameState::RoundStart { .. } => 0,
            GameState::Playing => 1,
            GameState::RoundEnd { .. } => 2,
            GameState::CardPick { .. } => 3,
            GameState::MatchOver { .. } => 4,
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
                poison_timer: p.poison_timer,
                ghost_timer: p.ghost_timer,
                overclock_timer: p.overclock_timer,
                overclock_crash_timer: p.overclock_crash_timer,
                adrenaline_timer: p.adrenaline_timer,
                bloodthirsty_timer: p.bloodthirsty_timer,
                slow_timer: p.slow_timer,
                shake_timer: p.shake_timer,
                soul_siphon_bonus_hp: p.soul_siphon_bonus_hp,
                doppel_ghost_x: p.doppel_ghost.0,
                doppel_ghost_y: p.doppel_ghost.1,
                doppel_ghost_aim_x: p.doppel_ghost.2,
                doppel_ghost_aim_y: p.doppel_ghost.3,
                upsized_stacks: p.upsized_stacks,
                invuln_timer: p.invuln_timer,
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
            sticky_bombs: self.sticky_bombs.iter().map(|s| StickyBombSnapshot {
                x: s.position.x, y: s.position.y,
                owner: s.owner as u8, fuse: s.fuse,
                stuck_to: s.stuck_to.map(|i| i as u8).unwrap_or(0xFF),
            }).collect(),
            healing_zones: self.healing_zones.iter().map(|h| HealingZoneSnapshot {
                x: h.position.x, y: h.position.y,
                owner: h.owner as u8, lifetime: h.lifetime,
            }).collect(),
            elapsed_time: self.elapsed_time,
        }
    }

    // ── Snapshot application (client) ────────────────────────────────────────

    pub fn apply_snapshot(&mut self, snap: &WorldSnapshot) {
        if snap.level_id != self.level.id {
            self.level = level::level_by_id(snap.level_id);
        }

        let names: Vec<String> = self.players.iter().map(|p| p.name.clone()).collect();
        let colors: Vec<Color> = self.players.iter().map(|p| p.color).collect();
        let base_hp = self.base_hp();
        self.state = snap.game_state(&names, &colors);
        self.card_hover = snap.card_hover;
        self.elapsed_time = snap.elapsed_time;

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
            p.max_hp = (base_hp + p.stats.max_hp_bonus) * p.stats.max_hp_mult;
            p.poison_timer = ps.poison_timer;
            p.ghost_timer = ps.ghost_timer;
            p.overclock_timer = ps.overclock_timer;
            p.overclock_crash_timer = ps.overclock_crash_timer;
            p.adrenaline_timer = ps.adrenaline_timer;
            p.bloodthirsty_timer = ps.bloodthirsty_timer;
            p.slow_timer = ps.slow_timer;
            p.shake_timer = ps.shake_timer;
            p.soul_siphon_bonus_hp = ps.soul_siphon_bonus_hp;
            p.doppel_ghost = (ps.doppel_ghost_x, ps.doppel_ghost_y, ps.doppel_ghost_aim_x, ps.doppel_ghost_aim_y);
            p.upsized_stacks = ps.upsized_stacks;
            p.invuln_timer = ps.invuln_timer;
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
                poison: false,
                gravity_mult: 1.0,
                sticky: false,
                ice: false,
                void_pull: false,
                hit_players: Vec::new(),
            });
        }

        // Reconstruct entities from snapshot
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
        self.healing_zones.clear();
        for h in &snap.healing_zones {
            self.healing_zones.push(HealingZone {
                position: Vector3::new(h.x, h.y, 0.0),
                owner: h.owner as usize,
                lifetime: h.lifetime,
            });
        }

        spawn_from_events(&snap.events, &mut self.particles, &mut self.rng);
        self.latest_events = snap.events.clone();
    }
}

/// Line segment vs AABB intersection (2D, Liang-Barsky algorithm).
fn line_intersects_aabb(
    x0: f32, y0: f32, x1: f32, y1: f32,
    min_x: f32, min_y: f32, max_x: f32, max_y: f32,
) -> bool {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let mut tmin = 0.0_f32;
    let mut tmax = 1.0_f32;

    let edges = [
        (-dx, x0 - min_x),
        (dx, max_x - x0),
        (-dy, y0 - min_y),
        (dy, max_y - y0),
    ];
    for (p, q) in edges {
        if p.abs() < 1e-9 {
            if q < 0.0 { return false; }
        } else {
            let t = q / p;
            if p < 0.0 {
                tmin = tmin.max(t);
            } else {
                tmax = tmax.min(t);
            }
            if tmin > tmax { return false; }
        }
    }
    true
}
