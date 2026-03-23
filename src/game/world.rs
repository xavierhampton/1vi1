use raylib::prelude::*;

use crate::combat::bullet::{Bullet, SHOOT_COOLDOWN};
use crate::combat::combat::update_bullets;
use crate::combat::particles::{spawn_from_events, Particle, Rng};
use crate::game::cards;
use crate::game::net::{BulletSnapshot, GameEvent, PlayerSnapshot, WorldSnapshot};
use crate::game::state::GameState;
use crate::level::level::{self, Level};
use crate::lobby::state::LobbyState;
use crate::player::input::PlayerInput;
use crate::player::movement;
use crate::player::player::Player;

pub const MAX_BULLETS: i32 = 3;
pub const RELOAD_TIME: f32 = 1.5;

pub const COUNTDOWN_DURATION: f32 = 3.0;
const ROUND_END_DURATION: f32 = 3.5;
const SLOW_MO_FACTOR: f32 = 0.25;
pub const WINS_TO_MATCH: i32 = 3;

const CARD_ENTRANCE_DURATION: f32 = 0.8;
const CARD_EXIT_DURATION: f32 = 0.8;
const MATCH_OVER_DURATION: f32 = 5.0;

pub struct World {
    pub players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    pub particles: Vec<Particle>,
    pub level: Level,
    pub state: GameState,
    pub scores: Vec<i32>,
    pub rng: Rng,
    pub cursor_positions: Vec<(f32, f32)>, // normalized cursor per player
    pub card_hover: u8, // 0xFF = none, 0-2 = picker hovering this card
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
        }
    }

    fn reset_round(&mut self) {
        // Pick a new random map
        self.level = level::random_level(self.rng.next());

        for (i, player) in self.players.iter_mut().enumerate() {
            player.position = self.level.spawn_points[i];
            player.velocity = Vector3::new(0.0, 0.0, 0.0);
            player.hp = player.max_hp;
            player.alive = true;
            player.hit_flash_timer = 0.0;
            player.bullets_remaining = MAX_BULLETS;
            player.reload_timer = 0.0;
            player.shoot_cooldown = 0.0;
            player.aim_dir = Vector2::new(if i % 2 == 0 { 1.0 } else { -1.0 }, 0.0);
            // Reset ability cooldowns but keep cards
            for (_, cd) in player.cards.iter_mut() {
                *cd = 0.0;
            }
            // Recompute stats from powerup cards and apply
            player.stats = cards::compute_stats(&player.cards);
            cards::apply_stats(player, &player.stats.clone());
        }
        self.bullets.clear();
        self.particles.clear();
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

    /// Build the pick order: all losers (non-winner, all players)
    fn build_pick_order(&self, winner_idx: u8) -> Vec<u8> {
        (0..self.players.len() as u8)
            .filter(|&i| i != winner_idx)
            .collect()
    }

    /// Enter card pick phase for the first loser
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

    /// Process a card choice from a player (called by server)
    pub fn process_card_choice(&mut self, player_index: u8, card_slot: u8) {
        if let GameState::CardPick { current_picker, chosen_card, .. } = &mut self.state {
            if player_index == *current_picker && chosen_card.is_none() && card_slot < 3 {
                *chosen_card = Some(card_slot);
            }
        }
    }

    // ── Server-authoritative update (processes ALL players) ──────────────────

    pub fn server_update(&mut self, inputs: &[PlayerInput], dt: f32) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Store cursor positions from inputs
        for (i, inp) in inputs.iter().enumerate() {
            if i < self.cursor_positions.len() {
                self.cursor_positions[i] = (inp.cursor_x, inp.cursor_y);
            }
        }

        // Read hover_card from the current picker's input
        if let GameState::CardPick { current_picker, .. } = &self.state {
            let pi = *current_picker as usize;
            self.card_hover = inputs.get(pi).map(|inp| inp.hover_card).unwrap_or(0xFF);
        } else {
            self.card_hover = 0xFF;
        }

        match &self.state {
            GameState::RoundStart { timer } => {
                let new_timer = *timer - dt;
                // Let players look around during countdown
                for (i, inp) in inputs.iter().enumerate() {
                    if i < self.players.len() {
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

                // Winner can still move during slow-mo
                if let Some(inp) = inputs.get(wi as usize) {
                    if (wi as usize) < self.players.len() && self.players[wi as usize].alive {
                        movement::update(&mut self.players[wi as usize], inp, &self.level.platforms, slow_dt);
                        self.players[wi as usize].aim_dir = inp.aim_dir;
                    }
                }

                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    // Check if anyone has won the match
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
                    // Match over — stay in this state until Escape (handled by main loop)
                    self.state = GameState::MatchOver { winner_index: wi, timer: 0.0 };
                } else {
                    self.state = GameState::MatchOver { winner_index: wi, timer: new_timer };
                }
            }
        }

        events
    }

    fn server_update_card_pick(&mut self, dt: f32) {
        // Extract state to avoid borrow issues
        let (winner_index, current_picker, offered_cards, pick_order, phase_timer, chosen_card, exit_timer) =
            if let GameState::CardPick {
                winner_index, current_picker, offered_cards, pick_order, phase_timer, chosen_card, exit_timer,
            } = &self.state {
                (*winner_index, *current_picker, *offered_cards, pick_order.clone(), *phase_timer, *chosen_card, *exit_timer)
            } else {
                return;
            };

        if chosen_card.is_some() {
            // Card was chosen — tick exit timer
            let new_exit = exit_timer + dt;
            if new_exit >= CARD_EXIT_DURATION {
                // Store the ability on the picker
                if let Some(slot) = chosen_card {
                    let card_id_u8 = offered_cards[slot as usize];
                    if let Some(card_id) = cards::CardId::from_u8(card_id_u8) {
                        let picker_idx = current_picker as usize;
                        if picker_idx < self.players.len() {
                            self.players[picker_idx].cards.push((card_id, 0.0));
                            // Recompute stats if it's a powerup
                            let p = &mut self.players[picker_idx];
                            p.stats = cards::compute_stats(&p.cards);
                            cards::apply_stats(p, &p.stats.clone());
                        }
                    }
                }

                // Advance to next picker or round
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
            // Entrance / waiting for pick
            let new_phase = (phase_timer - dt).max(0.0);
            self.state = GameState::CardPick {
                winner_index, current_picker, offered_cards, pick_order,
                phase_timer: new_phase, chosen_card, exit_timer,
            };
        }
    }

    fn server_update_playing(&mut self, inputs: &[PlayerInput], dt: f32, events: &mut Vec<GameEvent>) {
        // Process input for ALL alive players
        for i in 0..self.players.len() {
            if !self.players[i].alive {
                continue;
            }
            let inp = inputs.get(i).cloned().unwrap_or_else(PlayerInput::empty);

            movement::update(&mut self.players[i], &inp, &self.level.platforms, dt);
            self.players[i].aim_dir = inp.aim_dir;

            // Reload
            if self.players[i].reload_timer > 0.0 {
                self.players[i].reload_timer = (self.players[i].reload_timer - dt).max(0.0);
                if self.players[i].reload_timer <= 0.0 {
                    self.players[i].bullets_remaining = MAX_BULLETS;
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
                let spawn = Vector3::new(
                    self.players[i].position.x + aim.x * 0.5,
                    self.players[i].position.y + 1.1 + aim.y * 0.5,
                    self.players[i].position.z,
                );
                self.bullets.push(Bullet::new(spawn, aim, i, color));
                self.players[i].bullets_remaining -= 1;
                self.players[i].shoot_cooldown = SHOOT_COOLDOWN;
                if self.players[i].bullets_remaining == 0 {
                    self.players[i].reload_timer = RELOAD_TIME * self.players[i].stats.reload_time_mult;
                }
            }

            // Tick ability cooldowns (only for abilities, not powerups)
            for (card_id, cd) in self.players[i].cards.iter_mut() {
                if cards::CARD_CATALOG[*card_id as u8 as usize].is_ability() {
                    *cd = (*cd - dt).max(0.0);
                }
            }

            // Activate abilities on right-click (skip powerups)
            if inp.ability_pressed {
                let to_activate: Vec<(usize, cards::CardId)> = self.players[i].cards.iter()
                    .enumerate()
                    .filter(|(_, (card_id, cd))| {
                        cards::CARD_CATALOG[*card_id as u8 as usize].is_ability() && *cd <= 0.0
                    })
                    .map(|(j, (card_id, _))| (j, *card_id))
                    .collect();
                for (j, card_id) in to_activate {
                    let cd = cards::activate_ability(card_id, &mut self.players[i]);
                    self.players[i].cards[j].1 = cd;
                }
            }
        }

        // Tick hit flash timers
        for player in self.players.iter_mut() {
            player.hit_flash_timer = (player.hit_flash_timer - dt).max(0.0);
        }

        // Update bullets
        let bullet_events = update_bullets(&mut self.bullets, &mut self.players, &self.level.platforms, dt);
        events.extend(bullet_events);

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
        }).collect();

        // Card pick fields
        let (card_current_picker, card_offered, card_remaining_pickers, card_phase_timer, card_chosen, card_exit_timer, card_hover) =
            if let GameState::CardPick { current_picker, offered_cards, pick_order, phase_timer, chosen_card, exit_timer, .. } = &self.state {
                (*current_picker, *offered_cards, pick_order.len() as u8, *phase_timer,
                 chosen_card.unwrap_or(0xFF), *exit_timer, self.card_hover)
            } else {
                (0, [0, 0, 0], 0, 0.0, 0xFF, 0.0, 0xFF)
            };

        // MatchOver fields
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
        }
    }

    // ── Snapshot application (client) ────────────────────────────────────────

    pub fn apply_snapshot(&mut self, snap: &WorldSnapshot) {
        // Swap level if changed
        if snap.level_id != self.level.id {
            self.level = level::level_by_id(snap.level_id);
        }

        // Update game state
        let names: Vec<String> = self.players.iter().map(|p| p.name.clone()).collect();
        let colors: Vec<Color> = self.players.iter().map(|p| p.color).collect();
        self.state = snap.game_state(&names, &colors);
        self.card_hover = snap.card_hover;

        // Update players
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
            // Update cursor positions
            if i < self.cursor_positions.len() {
                self.cursor_positions[i] = (ps.cursor_x, ps.cursor_y);
            }
            // Update cards
            p.cards.clear();
            for (card_id_u8, cooldown) in &ps.cards {
                if let Some(card_id) = cards::CardId::from_u8(*card_id_u8) {
                    p.cards.push((card_id, *cooldown));
                }
            }
            // Recompute stats from cards
            p.stats = cards::compute_stats(&p.cards);
            p.max_hp = 100.0 + p.stats.max_hp_bonus;
        }

        // Update scores
        for (i, &s) in snap.scores.iter().enumerate() {
            if i < self.scores.len() {
                self.scores[i] = s;
            }
        }

        // Rebuild bullets from snapshot
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
            });
        }

        // Spawn particles from events
        spawn_from_events(&snap.events, &mut self.particles, &mut self.rng);
    }

}
