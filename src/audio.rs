// ── Audio Manager ────────────────────────────────────────────────────────────
//
// All game audio in one place. Every sound loads from assets/sounds/ and music
// from assets/music/. Missing files are silently ignored (Option<Sound>).
//
// To swap a sound: just replace the .wav/.ogg file on disk with the same name.
// To add a new sound: add a PATH constant, an Option<Sound> field, load it in
// new(), and add a play_xxx() method.

use raylib::core::audio::{Music, RaylibAudio, Sound};

use crate::game::net::GameEvent;

// ── File paths (change these to swap sounds) ────────────────────────────────

// Music
const MUSIC_MENU: &str = "assets/sounds/theme.wav";
const MUSIC_GAME: &str = "assets/sounds/game_music.wav";

// SFX — combat
const SFX_SHOOT: &str = "assets/sounds/shoot.wav";
const SFX_HIT: &str = "assets/sounds/hit.wav";
const SFX_DEATH: &str = "assets/sounds/death.wav";
const SFX_TERRAIN_HIT: &str = "assets/sounds/terrain_hit.wav";
const SFX_EXPLOSION: &str = "assets/sounds/explosion.wav";

// SFX — movement
const SFX_JUMP: &str = "assets/sounds/jump.wav";
const SFX_LAND: &str = "assets/sounds/land.wav";
const SFX_DASH: &str = "assets/sounds/dash.wav";

// SFX — hazards
const SFX_LAVA_SIZZLE: &str = "assets/sounds/lava_sizzle.wav";

// SFX — weapons
const SFX_RELOAD: &str = "assets/sounds/reload.wav";

// SFX — game flow
const SFX_COUNTDOWN: &str = "assets/sounds/countdown.wav";
const SFX_ROUND_START: &str = "assets/sounds/round_start.wav";
const SFX_ROUND_WIN: &str = "assets/sounds/round_win.wav";
const SFX_MATCH_WIN: &str = "assets/sounds/match_win.wav";

// SFX — card pick
const SFX_CARD_HOVER: &str = "assets/sounds/card_hover.wav";
const SFX_CARD_PICK: &str = "assets/sounds/card_pick.wav";

// SFX — UI
const SFX_MENU_HOVER: &str = "assets/sounds/menu_hover.wav";
const SFX_MENU_SELECT: &str = "assets/sounds/menu_select.wav";
const SFX_MENU_BACK: &str = "assets/sounds/menu_back.wav";
const SFX_READY: &str = "assets/sounds/ready.wav";

// ── Manager ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct AudioManager<'aud> {
    // Music
    music_menu: Option<Music<'aud>>,
    music_game: Option<Music<'aud>>,
    music_playing: bool,
    game_music_playing: bool,

    // SFX — combat
    sfx_shoot: Option<Sound<'aud>>,
    sfx_hit: Option<Sound<'aud>>,
    sfx_death: Option<Sound<'aud>>,
    sfx_terrain_hit: Option<Sound<'aud>>,
    sfx_explosion: Option<Sound<'aud>>,

    // SFX — hazards
    sfx_lava_sizzle: Option<Sound<'aud>>,

    // SFX — movement
    sfx_jump: Option<Sound<'aud>>,
    sfx_land: Option<Sound<'aud>>,
    sfx_dash: Option<Sound<'aud>>,

    // SFX — weapons
    sfx_reload: Option<Sound<'aud>>,

    // SFX — game flow
    sfx_countdown: Option<Sound<'aud>>,
    sfx_round_start: Option<Sound<'aud>>,
    sfx_round_win: Option<Sound<'aud>>,
    sfx_match_win: Option<Sound<'aud>>,

    // SFX — card pick
    sfx_card_hover: Option<Sound<'aud>>,
    sfx_card_pick: Option<Sound<'aud>>,

    // SFX — UI
    sfx_menu_hover: Option<Sound<'aud>>,
    sfx_menu_select: Option<Sound<'aud>>,
    sfx_menu_back: Option<Sound<'aud>>,
    sfx_ready: Option<Sound<'aud>>,

    // Volume (0.0 – 1.0 each)
    pub master_volume: f32,
    pub sound_volume: f32,
    pub music_volume: f32,
}

fn try_sound<'a>(audio: &'a RaylibAudio, path: &str) -> Option<Sound<'a>> {
    audio.new_sound(path).ok()
}

fn try_music<'a>(audio: &'a RaylibAudio, path: &str) -> Option<Music<'a>> {
    audio.new_music(path).ok()
}

#[allow(dead_code)]
impl<'aud> AudioManager<'aud> {
    pub fn new(audio: &'aud RaylibAudio, master: f32, sound: f32, music: f32) -> Self {
        Self {
            music_menu: try_music(audio, MUSIC_MENU),
            music_game: try_music(audio, MUSIC_GAME),
            music_playing: false,
            game_music_playing: false,

            sfx_shoot: try_sound(audio, SFX_SHOOT),
            sfx_hit: try_sound(audio, SFX_HIT),
            sfx_death: try_sound(audio, SFX_DEATH),
            sfx_terrain_hit: try_sound(audio, SFX_TERRAIN_HIT),
            sfx_explosion: try_sound(audio, SFX_EXPLOSION),

            sfx_lava_sizzle: try_sound(audio, SFX_LAVA_SIZZLE),

            sfx_jump: try_sound(audio, SFX_JUMP),
            sfx_land: try_sound(audio, SFX_LAND),
            sfx_dash: try_sound(audio, SFX_DASH),

            sfx_reload: try_sound(audio, SFX_RELOAD),

            sfx_countdown: try_sound(audio, SFX_COUNTDOWN),
            sfx_round_start: try_sound(audio, SFX_ROUND_START),
            sfx_round_win: try_sound(audio, SFX_ROUND_WIN),
            sfx_match_win: try_sound(audio, SFX_MATCH_WIN),

            sfx_card_hover: try_sound(audio, SFX_CARD_HOVER),
            sfx_card_pick: try_sound(audio, SFX_CARD_PICK),

            sfx_menu_hover: try_sound(audio, SFX_MENU_HOVER),
            sfx_menu_select: try_sound(audio, SFX_MENU_SELECT),
            sfx_menu_back: try_sound(audio, SFX_MENU_BACK),
            sfx_ready: try_sound(audio, SFX_READY),

            master_volume: master,
            sound_volume: sound,
            music_volume: music,
        }
    }

    // ── Volume helpers ───────────────────────────────────────────────────

    fn sfx_vol(&self) -> f32 {
        self.master_volume * self.sound_volume * 0.5
    }

    fn mus_vol(&self) -> f32 {
        self.master_volume * self.music_volume * 0.10
    }

    fn play_sfx(&self, sfx: &Option<Sound>) {
        if let Some(s) = sfx {
            s.set_volume(self.sfx_vol());
            s.play();
        }
    }

    /// Call once per frame to keep music streaming.
    pub fn update(&self) {
        if let Some(ref m) = self.music_menu {
            if self.music_playing {
                m.update_stream();
            }
        }
        if let Some(ref m) = self.music_game {
            if self.game_music_playing {
                m.update_stream();
            }
        }
    }

    pub fn apply_volumes(&self) {
        if let Some(ref m) = self.music_menu {
            m.set_volume(self.mus_vol());
        }
        if let Some(ref m) = self.music_game {
            m.set_volume(self.mus_vol());
        }
    }

    // ── Music controls ───────────────────────────────────────────────────

    pub fn start_menu_music(&mut self) {
        if let Some(ref m) = self.music_menu {
            m.set_volume(self.mus_vol() * 0.3);
            m.play_stream();
            self.music_playing = true;
        }
    }

    pub fn stop_menu_music(&mut self) {
        if let Some(ref m) = self.music_menu {
            m.stop_stream();
        }
        self.music_playing = false;
    }

    pub fn is_music_playing(&self) -> bool {
        self.music_playing
    }

    pub fn start_game_music(&mut self) {
        if let Some(ref m) = self.music_game {
            m.set_volume(self.mus_vol());
            m.play_stream();
            self.game_music_playing = true;
        }
    }

    pub fn stop_game_music(&mut self) {
        if let Some(ref m) = self.music_game {
            m.stop_stream();
        }
        self.game_music_playing = false;
    }

    // ── SFX: combat ─────────────────────────────────────────────────────

    pub fn play_shoot(&self) {
        self.play_sfx(&self.sfx_shoot);
    }

    pub fn play_hit(&self) {
        self.play_sfx(&self.sfx_hit);
    }

    pub fn play_death(&self) {
        self.play_sfx(&self.sfx_death);
    }

    pub fn play_terrain_hit(&self) {
        self.play_sfx(&self.sfx_terrain_hit);
    }

    pub fn play_explosion(&self) {
        self.play_sfx(&self.sfx_explosion);
    }

    // ── SFX: hazards ────────────────────────────────────────────────

    pub fn play_lava_sizzle(&self) {
        self.play_sfx(&self.sfx_lava_sizzle);
    }

    // ── SFX: movement ───────────────────────────────────────────────────

    pub fn play_jump(&self) {
        self.play_sfx(&self.sfx_jump);
    }

    pub fn play_land(&self) {
        self.play_sfx(&self.sfx_land);
    }

    pub fn play_dash(&self) {
        self.play_sfx(&self.sfx_dash);
    }

    // ── SFX: weapons ────────────────────────────────────────────────────

    pub fn play_reload(&self) {
        self.play_sfx(&self.sfx_reload);
    }

    // ── SFX: game flow ──────────────────────────────────────────────────

    pub fn play_countdown(&self) {
        self.play_sfx(&self.sfx_countdown);
    }

    pub fn play_round_start(&self) {
        self.play_sfx(&self.sfx_round_start);
    }

    pub fn play_round_win(&self) {
        self.play_sfx(&self.sfx_round_win);
    }

    pub fn play_match_win(&self) {
        self.play_sfx(&self.sfx_match_win);
    }

    // ── SFX: card pick ──────────────────────────────────────────────────

    pub fn play_card_hover(&self) {
        self.play_sfx(&self.sfx_card_hover);
    }

    pub fn play_card_pick(&self) {
        self.play_sfx(&self.sfx_card_pick);
    }

    // ── SFX: UI ─────────────────────────────────────────────────────────

    pub fn play_menu_hover(&self) {
        self.play_sfx(&self.sfx_menu_hover);
    }

    pub fn play_menu_select(&self) {
        self.play_sfx(&self.sfx_menu_select);
    }

    pub fn play_menu_back(&self) {
        self.play_sfx(&self.sfx_menu_back);
    }

    pub fn play_ready(&self) {
        self.play_sfx(&self.sfx_ready);
    }

    // ── Batch: play sounds from game events ─────────────────────────────

    pub fn play_game_events(&self, events: &[GameEvent]) {
        for ev in events {
            match ev {
                GameEvent::BulletFired { .. } => self.play_shoot(),
                GameEvent::PlayerHit { .. } => self.play_hit(),
                GameEvent::PlayerDied { .. } => self.play_death(),
                GameEvent::TerrainHit { .. } => self.play_terrain_hit(),
                GameEvent::Explosion { .. } => self.play_explosion(),
                GameEvent::Jumped { .. } => self.play_jump(),
                GameEvent::Landed { .. } => self.play_land(),
                GameEvent::Dashed { .. } => self.play_dash(),
                GameEvent::LavaSizzle { .. } => self.play_lava_sizzle(),
            }
        }
    }
}
