mod audio;
mod combat;
mod editor;
mod game;
mod level;
mod lobby;
mod menu;
mod physics;
mod player;
mod render;

use audio::AudioManager;
use game::client::GameClient;
use game::server::GameServer;
use game::state::GameState;
use game::world::World;
use lobby::client::LobbyClient;
use lobby::screen::{draw_lobby, lobby_input, LobbyInput, LobbySettingsState};
use lobby::server::LobbyServer;
use lobby::state::{GameSettings, LobbyColor, PlayerSlot};
use menu::menu::{Menu, MenuAction};
use raylib::core::audio::RaylibAudio;
use raylib::prelude::*;
use render::cards::CardPickAnim;
use render::crt::CrtFilter;

const SCREEN_WIDTH: i32 = 960;
const SCREEN_HEIGHT: i32 = 540;
const DEFAULT_PORT: u16 = 7878;

fn match_over_mouse_hover(rl: &RaylibHandle, sel: &mut usize, screen_w: i32, screen_h: i32) {
    let mx = rl.get_mouse_x();
    let my = rl.get_mouse_y();
    let btn_size = 36;
    let btn_gap = 50;
    let base_y = screen_h - 200;
    let hit_w = 260;
    for i in 0..2 {
        let y = base_y + i as i32 * btn_gap;
        if mx >= screen_w / 2 - hit_w / 2 && mx <= screen_w / 2 + hit_w / 2
            && my >= y - 4 && my <= y + btn_size + 4
        {
            *sel = i;
        }
    }
}

enum LobbyRole {
    Host(LobbyServer),
    Client(LobbyClient),
}

enum AppState {
    Menu,
    Lobby(LobbyRole),
    InGameHost(GameServer),
    InGameClient(GameClient),
    ReturnToLobby { name: String, dev_mode: bool, accessories: Vec<(u8, u8, u8, u8)>, settings: Option<GameSettings>, timer: f32, retries: u8 },
    Reconnecting { addr: String, name: String, accessories: Vec<(u8, u8, u8, u8)>, timer: f32, retries: u8 },
}

fn main() {
    // If assets/ exists next to the executable (release/distribution layout),
    // switch CWD there so relative paths resolve. Otherwise keep CWD as-is
    // (works for `cargo run` from the project root).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("assets").is_dir() {
                let _ = std::env::set_current_dir(dir);
            }
        }
    }

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--level") {
        return run_level_editor();
    }

    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("1VI1")
        .resizable()
        .log_level(raylib::callbacks::TraceLogLevel::LOG_WARNING)
        .build();

    let mut menu = Menu::new();
    let state = rl.get_window_state().set_vsync_hint(false);
    rl.set_window_state(state);
    rl.set_target_fps(menu.target_fps as u32);
    rl.set_exit_key(None);

    let mut crt = CrtFilter::new(&mut rl, &thread, SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut render_w = SCREEN_WIDTH;
    let mut render_h = SCREEN_HEIGHT;
    let audio = RaylibAudio::init_audio_device().expect("audio init");
    let mut sfx = AudioManager::new(&audio, menu.master_volume, menu.sound_volume, menu.music_volume);
    sfx.start_menu_music();

    let mut app_state = AppState::Menu;
    let mut game_time: f32 = 0.0;
    let mut lobby_time: f32 = 0.0;
    let mut card_anim = CardPickAnim::new();
    let mut dev_overlay_open = false;
    let mut lobby_settings = LobbySettingsState::new();
    let mut match_over_sel: usize = 0; // 0 = Rematch, 1 = Exit to Menu
    let mut prev_match_over_sel: usize = 0;
    let mut prev_lobby_settings_sel: usize = 99;
    let mut rematch_waiting = false;
    let mut client_addr = String::new();
    let mut menu_cooldown: f32 = 0.0;
    let mut prev_game_state_tag: u8 = 0; // track state transitions for sound cues
    let mut prev_countdown: i32 = 99;
    let mut prev_card_chosen: bool = false;
    let mut prev_card_hover: u8 = 0xFF;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        sfx.update();

        if rl.is_key_pressed(KeyboardKey::KEY_F11) {
            rl.toggle_fullscreen();
        }

        let w = rl.get_screen_width().max(1);
        let h = rl.get_screen_height().max(1);
        if w != render_w || h != render_h {
            render_w = w;
            render_h = h;
            crt = CrtFilter::new(&mut rl, &thread, render_w, render_h);
        }

        let mut next_state = None;

        match &mut app_state {
            AppState::Menu => {
                // Ensure menu music is playing and volumes are synced
                if !sfx.is_music_playing() {
                    sfx.start_menu_music();
                }
                sfx.master_volume = menu.master_volume;
                sfx.sound_volume = menu.sound_volume;
                sfx.music_volume = menu.music_volume;
                sfx.apply_volumes();
                rl.set_target_fps(menu.target_fps as u32);

                if menu_cooldown > 0.0 {
                    menu_cooldown -= dt;
                    menu.render_customize_preview(&mut rl, &thread);
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    {
                        let mut t = d.begin_texture_mode(&thread, &mut crt.ui_target);
                        menu.draw(&mut *t);
                    }
                    {
                        let mut s = d.begin_shader_mode(&mut crt.shader_ui);
                        s.draw_texture_rec(
                            crt.ui_target.texture(),
                            Rectangle::new(0.0, 0.0, render_w as f32, -(render_h as f32)),
                            Vector2::new(0.0, 0.0), Color::WHITE,
                        );
                    }
                    continue;
                }
                let action = menu.update(&mut rl, dt);
                if menu.hover_changed {
                    sfx.play_menu_hover();
                    menu.hover_changed = false;
                }
                if menu.select_sound {
                    sfx.play_menu_select();
                    menu.select_sound = false;
                }
                if menu.back_sound {
                    sfx.play_menu_back();
                    menu.back_sound = false;
                }
                match action {
                    MenuAction::Host => {
                        sfx.play_menu_select();
                        let name = if menu.player_name.is_empty() { "Player" } else { &menu.player_name };
                        let acc = menu.accessories.iter().filter(|a| a.0 != 0xFF).cloned().collect();
                        match LobbyServer::start(name, DEFAULT_PORT, acc) {
                            Ok(mut server) => {
                                server.dev_mode = menu.dev_mode;
                                lobby_time = 0.0;
                                next_state = Some(AppState::Lobby(LobbyRole::Host(server)));
                            }
                            Err(e) => {
                                if e.kind() != std::io::ErrorKind::AddrInUse {
                                    menu.show_error(&format!("Failed to host: {}", e));
                                }
                            }
                        }
                    }
                    MenuAction::Join(addr) => {
                        sfx.play_menu_select();
                        let name = if menu.player_name.is_empty() { "Player" } else { &menu.player_name };
                        let acc = menu.accessories.iter().filter(|a| a.0 != 0xFF).cloned().collect();
                        match LobbyClient::connect(&addr, name, acc) {
                            Ok(client) => {
                                lobby_time = 0.0;
                                client_addr = addr.clone();
                                next_state = Some(AppState::Lobby(LobbyRole::Client(client)));
                            }
                            Err(e) => {
                                menu.show_error(&format!("Failed to connect: {}", e));
                            }
                        }
                    }
                    MenuAction::Quit => break,
                    MenuAction::None => {}
                }

                if next_state.is_none() {
                    menu.render_customize_preview(&mut rl, &thread);
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    {
                        let mut t = d.begin_texture_mode(&thread, &mut crt.ui_target);
                        menu.draw(&mut *t);
                    }
                    {
                        let mut s = d.begin_shader_mode(&mut crt.shader_ui);
                        s.draw_texture_rec(
                            crt.ui_target.texture(),
                            Rectangle::new(0.0, 0.0, render_w as f32, -(render_h as f32)),
                            Vector2::new(0.0, 0.0), Color::WHITE,
                        );
                    }
                }
            }
            AppState::Lobby(role) => {
                lobby_time += dt;
                let w_now = rl.get_screen_width();
                let h_now = rl.get_screen_height();
                let accent = menu.theme().particle_color_primary;
                menu.fx.update(dt, w_now, h_now, accent);
                lobby_settings.time += dt;
                let mut input = lobby_input(&rl, lobby_settings.open);
                // Ignore the Enter keypress that carried over from the join screen
                if lobby_time < 0.1 && matches!(input, LobbyInput::ToggleReady) {
                    input = LobbyInput::None;
                }

                match role {
                    LobbyRole::Host(server) => {
                        match input {
                            LobbyInput::ToggleSettings => {
                                lobby_settings.open = !lobby_settings.open;
                            }
                            LobbyInput::SettingsUp => {
                                if lobby_settings.selected > 0 {
                                    lobby_settings.selected -= 1;
                                } else {
                                    lobby_settings.selected = 6;
                                }
                            }
                            LobbyInput::SettingsDown => {
                                lobby_settings.selected = (lobby_settings.selected + 1) % 7;
                            }
                            LobbyInput::SettingsLeft => {
                                lobby::screen::apply_settings_change(&mut server.state.settings, lobby_settings.selected, -1);
                                server.notify_settings_changed();
                            }
                            LobbyInput::SettingsRight => {
                                lobby::screen::apply_settings_change(&mut server.state.settings, lobby_settings.selected, 1);
                                server.notify_settings_changed();
                            }
                            LobbyInput::ColorLeft => {
                                let cur = server.state.slots[0].color;
                                let next = server.state.prev_available_color(cur, 0);
                                server.host_change_color(next);
                            }
                            LobbyInput::ColorRight => {
                                let cur = server.state.slots[0].color;
                                let next = server.state.next_available_color(cur, 0);
                                server.host_change_color(next);
                            }
                            LobbyInput::ToggleReady => {
                                sfx.play_ready();
                                server.host_toggle_ready();
                            }
                            LobbyInput::Leave => {
                                sfx.play_menu_back();
                                server.disband();
                                next_state = Some(AppState::Menu);
                            }
                            LobbyInput::CopyIP => {
                                let _ = rl.set_clipboard_text(&server.my_addr);
                            }
                            LobbyInput::None => {}
                        }

                        let game_start = server.update();
                        if game_start {
                            sfx.stop_menu_music();
                            sfx.start_game_music();
                            prev_game_state_tag = 0;
                            prev_countdown = 99;
                            // In dev mode solo, add a dummy player to the lobby
                            let lobby_state = if server.dev_mode && server.state.slots.len() == 1 {
                                let mut s = server.state.clone();
                                let dummy_color = s.first_available_color().unwrap_or(LobbyColor::Red);
                                s.slots.push(PlayerSlot {
                                    name: "Dummy".to_string(),
                                    color: dummy_color,
                                    ready: true,
                                    is_host: false,
                                    accessories: Vec::new(),
                                });
                                s
                            } else {
                                server.state.clone()
                            };
                            let world = World::from_lobby(&lobby_state);
                            let parts = server.into_game_parts();
                            game_time = 0.0;
                            card_anim = CardPickAnim::new();
                            let mut gs = GameServer::new(world, parts);
                            gs.dev_mode = server.dev_mode;
                            next_state = Some(AppState::InGameHost(gs));
                        }

                        if next_state.is_none() {
                            let theme = menu.theme();
                            let mut d = rl.begin_drawing(&thread);
                            d.clear_background(Color::BLACK);
                            {
                                let mut t = d.begin_texture_mode(&thread, &mut crt.ui_target);
                                t.clear_background(theme.bg);
                                draw_lobby(
                                    &mut *t,
                                    &server.state,
                                    0,
                                    true,
                                    &server.my_addr,
                                    theme,
                                    lobby_time,
                                    &menu.fx,
                                    &lobby_settings,
                                );
                            }
                            {
                                let mut s = d.begin_shader_mode(&mut crt.shader_ui);
                                s.draw_texture_rec(
                                    crt.ui_target.texture(),
                                    Rectangle::new(0.0, 0.0, render_w as f32, -(render_h as f32)),
                                    Vector2::new(0.0, 0.0), Color::WHITE,
                                );
                            }
                        }
                    }
                    LobbyRole::Client(client) => {
                        client.update();

                        match input {
                            LobbyInput::ToggleSettings => {
                                lobby_settings.open = !lobby_settings.open;
                            }
                            LobbyInput::SettingsUp => {
                                if lobby_settings.selected > 0 {
                                    lobby_settings.selected -= 1;
                                } else {
                                    lobby_settings.selected = 6;
                                }
                            }
                            LobbyInput::SettingsDown => {
                                lobby_settings.selected = (lobby_settings.selected + 1) % 7;
                            }
                            LobbyInput::SettingsLeft | LobbyInput::SettingsRight => {} // clients can't edit
                            LobbyInput::ColorLeft => {
                                let my_idx = client.my_index as usize;
                                if my_idx < client.state.slots.len() {
                                    let cur = client.state.slots[my_idx].color;
                                    let next = client.state.prev_available_color(cur, my_idx);
                                    client.change_color(next as u8);
                                }
                            }
                            LobbyInput::ColorRight => {
                                let my_idx = client.my_index as usize;
                                if my_idx < client.state.slots.len() {
                                    let cur = client.state.slots[my_idx].color;
                                    let next = client.state.next_available_color(cur, my_idx);
                                    client.change_color(next as u8);
                                }
                            }
                            LobbyInput::ToggleReady => {
                                sfx.play_ready();
                                client.toggle_ready();
                            }
                            LobbyInput::Leave => {
                                sfx.play_menu_back();
                                next_state = Some(AppState::Menu);
                            }
                            LobbyInput::CopyIP => {}
                            LobbyInput::None => {}
                        }

                        if let Some(ref host_name) = client.host_disbanded {
                            if host_name.is_empty() {
                                menu.show_error("Host disconnected");
                            } else {
                                menu.show_error(&format!("HOST {} disbanded the Lobby", host_name));
                            }
                            next_state = Some(AppState::Menu);
                        }

                        if client.rejected {
                            menu.show_error("Lobby is full");
                            next_state = Some(AppState::Menu);
                        }

                        if client.game_starting {
                            sfx.stop_menu_music();
                            sfx.start_game_music();
                            prev_game_state_tag = 0;
                            prev_countdown = 99;
                            let world = World::from_lobby(&client.state);
                            let parts = client.into_game_parts();
                            game_time = 0.0;
                            card_anim = CardPickAnim::new();
                            next_state = Some(AppState::InGameClient(
                                GameClient::new(world, parts),
                            ));
                        }

                        if next_state.is_none() {
                            let theme = menu.theme();
                            let mut d = rl.begin_drawing(&thread);
                            d.clear_background(Color::BLACK);
                            {
                                let mut t = d.begin_texture_mode(&thread, &mut crt.ui_target);
                                t.clear_background(theme.bg);
                                draw_lobby(
                                    &mut *t,
                                    &client.state,
                                    client.my_index as usize,
                                    false,
                                    "",
                                    theme,
                                    lobby_time,
                                    &menu.fx,
                                    &lobby_settings,
                                );
                            }
                            {
                                let mut s = d.begin_shader_mode(&mut crt.shader_ui);
                                s.draw_texture_rec(
                                    crt.ui_target.texture(),
                                    Rectangle::new(0.0, 0.0, render_w as f32, -(render_h as f32)),
                                    Vector2::new(0.0, 0.0), Color::WHITE,
                                );
                            }
                        }
                    }
                }
                // Lobby settings hover sound (after input processing)
                if lobby_settings.selected != prev_lobby_settings_sel {
                    sfx.play_menu_hover();
                    prev_lobby_settings_sel = lobby_settings.selected;
                }
            }
            AppState::InGameHost(game_server) => {
                let in_match_over = matches!(game_server.world.state, GameState::MatchOver { timer, .. } if timer <= 4.0);

                if in_match_over {
                    // Match over button navigation
                    match_over_mouse_hover(&rl, &mut match_over_sel, render_w, render_h);
                    if rl.is_key_pressed(KeyboardKey::KEY_W) || rl.is_key_pressed(KeyboardKey::KEY_UP) {
                        match_over_sel = 0;
                    }
                    if rl.is_key_pressed(KeyboardKey::KEY_S) || rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                        match_over_sel = 1;
                    }
                    if match_over_sel != prev_match_over_sel {
                        sfx.play_menu_hover();
                        prev_match_over_sel = match_over_sel;
                    }
                    let confirm = rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                        || rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
                    if confirm {
                        sfx.play_menu_select();
                        if match_over_sel == 0 {
                            // Rematch — notify clients, then return to lobby with settings
                            game_server.notify_rematch();
                            let name = if menu.player_name.is_empty() { "Player".to_string() } else { menu.player_name.clone() };
                            let acc: Vec<_> = menu.accessories.iter().filter(|a| a.0 != 0xFF).cloned().collect();
                            let settings = game_server.world.game_settings.clone();
                            next_state = Some(AppState::ReturnToLobby { name, dev_mode: menu.dev_mode, accessories: acc, settings: Some(settings), timer: 0.3, retries: 0 });
                            match_over_sel = 0;
                        } else {
                            // Exit to Menu
                            let name = game_server.world.players[0].name.clone();
                            game_server.notify_leaving(&name);
                            next_state = Some(AppState::Menu);
                            match_over_sel = 0;
                        }
                    }
                    if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                        let name = game_server.world.players[0].name.clone();
                        game_server.notify_leaving(&name);
                        next_state = Some(AppState::Menu);
                        match_over_sel = 0;
                    }
                } else if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    if dev_overlay_open {
                        dev_overlay_open = false;
                    } else {
                        let name = game_server.world.players[0].name.clone();
                        game_server.notify_leaving(&name);
                        next_state = Some(AppState::Menu);
                    }
                }

                if next_state.is_none() {
                    // Dev mode toggle
                    if menu.dev_mode && rl.is_key_pressed(KeyboardKey::KEY_TAB) {
                        dev_overlay_open = !dev_overlay_open;
                    }

                    // Dev mode card click — toggle on/off
                    if dev_overlay_open && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                        let mouse = rl.get_mouse_position();
                        if let Some(card_id) = render::game::dev_overlay_click(mouse, render_w, render_h) {
                            game_server.world.dev_toggle_card(0, card_id);
                        }
                    }

                    game_time += dt;
                    let camera = render::camera::game_camera(&game_server.world);
                    if !dev_overlay_open {
                        game_server.update(&rl, &camera, dt);
                    }

                    // Play SFX from game events
                    sfx.play_game_events(&game_server.local_audio_events);
                    game_server.local_audio_events.clear();

                    // State transition sounds
                    let state_tag = game_server.world.state_tag();
                    if state_tag != prev_game_state_tag {
                        match state_tag {
                            0 => {} // RoundStart — countdown sounds handled below
                            1 => sfx.play_round_start(),  // Playing
                            2 => sfx.play_round_win(),     // RoundEnd
                            3 => {}                         // CardPick
                            4 => sfx.play_match_win(),     // MatchOver
                            _ => {}
                        }
                        prev_game_state_tag = state_tag;
                    }

                    // Countdown tick sounds
                    if let GameState::RoundStart { timer } = &game_server.world.state {
                        let tick = timer.ceil() as i32;
                        if tick != prev_countdown && tick >= 1 && tick <= 3 {
                            sfx.play_countdown();
                        }
                        prev_countdown = tick;
                    }

                    // Card pick/hover sounds
                    if let GameState::CardPick { chosen_card, .. } = &game_server.world.state {
                        let hover = game_server.world.card_hover;
                        if hover != prev_card_hover && hover < 3 {
                            sfx.play_card_hover();
                        }
                        prev_card_hover = hover;
                        let is_chosen = chosen_card.is_some();
                        if is_chosen && !prev_card_chosen {
                            sfx.play_card_pick();
                        }
                        prev_card_chosen = is_chosen;
                    } else {
                        prev_card_chosen = false;
                        prev_card_hover = 0xFF;
                    }

                    if let Some(ref left_name) = game_server.player_left {
                        menu.show_error(&format!("{} Left", left_name));
                        next_state = Some(AppState::Menu);
                    }

                    card_anim.update(&game_server.world, dt);
                    let theme = menu.theme();
                    let btns = if in_match_over {
                        Some(render::game::MatchOverButtons { selected: match_over_sel, waiting: false, theme, time: game_time })
                    } else { None };
                    render::game::draw_world(
                        &mut rl, &thread, &mut crt, &game_server.world, camera,
                        render_w, render_h, theme, game_time, &card_anim, 0,
                        dev_overlay_open, btns.as_ref(),
                    );
                }
            }
            AppState::InGameClient(game_client) => {
                let in_match_over = matches!(game_client.world.state, GameState::MatchOver { timer, .. } if timer <= 4.0);

                if let Some(ref msg) = game_client.disconnect_message {
                    if rematch_waiting || game_client.rematch_signal {
                        // Host dropped connection for rematch — reconnect
                        let name = if menu.player_name.is_empty() { "Player".to_string() } else { menu.player_name.clone() };
                        let acc: Vec<_> = menu.accessories.iter().filter(|a| a.0 != 0xFF).cloned().collect();
                        next_state = Some(AppState::Reconnecting { addr: client_addr.clone(), name, accessories: acc, timer: 0.5, retries: 0 });
                        rematch_waiting = false;
                    } else {
                        menu.show_error(msg);
                        next_state = Some(AppState::Menu);
                    }
                } else if in_match_over && !rematch_waiting {
                    // Match over button navigation
                    match_over_mouse_hover(&rl, &mut match_over_sel, render_w, render_h);
                    if rl.is_key_pressed(KeyboardKey::KEY_W) || rl.is_key_pressed(KeyboardKey::KEY_UP) {
                        match_over_sel = 0;
                    }
                    if rl.is_key_pressed(KeyboardKey::KEY_S) || rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                        match_over_sel = 1;
                    }
                    if match_over_sel != prev_match_over_sel {
                        sfx.play_menu_hover();
                        prev_match_over_sel = match_over_sel;
                    }
                    let confirm = rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                        || rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
                    if confirm {
                        sfx.play_menu_select();
                        if match_over_sel == 0 {
                            // Rematch — wait for host
                            rematch_waiting = true;
                        } else {
                            next_state = Some(AppState::Menu);
                            match_over_sel = 0;
                        }
                    }
                    if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                        next_state = Some(AppState::Menu);
                        match_over_sel = 0;
                    }
                } else if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    rematch_waiting = false;
                    next_state = Some(AppState::Menu);
                    match_over_sel = 0;
                }

                if next_state.is_none() {
                    game_time += dt;
                    let camera = render::camera::game_camera(&game_client.world);
                    game_client.update(&rl, &camera, dt);

                    // Play SFX from game events (received in snapshot)
                    sfx.play_game_events(&game_client.world.latest_events);
                    game_client.world.latest_events.clear();

                    // State transition sounds
                    let state_tag = game_client.world.state_tag();
                    if state_tag != prev_game_state_tag {
                        match state_tag {
                            0 => {}
                            1 => sfx.play_round_start(),
                            2 => sfx.play_round_win(),
                            3 => {}
                            4 => sfx.play_match_win(),
                            _ => {}
                        }
                        prev_game_state_tag = state_tag;
                    }

                    // Countdown tick sounds
                    if let GameState::RoundStart { timer } = &game_client.world.state {
                        let tick = timer.ceil() as i32;
                        if tick != prev_countdown && tick >= 1 && tick <= 3 {
                            sfx.play_countdown();
                        }
                        prev_countdown = tick;
                    }

                    // Card pick/hover sounds
                    if let GameState::CardPick { chosen_card, .. } = &game_client.world.state {
                        let hover = game_client.world.card_hover;
                        if hover != prev_card_hover && hover < 3 {
                            sfx.play_card_hover();
                        }
                        prev_card_hover = hover;
                        let is_chosen = chosen_card.is_some();
                        if is_chosen && !prev_card_chosen {
                            sfx.play_card_pick();
                        }
                        prev_card_chosen = is_chosen;
                    } else {
                        prev_card_chosen = false;
                        prev_card_hover = 0xFF;
                    }

                    card_anim.update(&game_client.world, dt);
                    let theme = menu.theme();
                    let btns = if in_match_over {
                        Some(render::game::MatchOverButtons { selected: match_over_sel, waiting: rematch_waiting, theme, time: game_time })
                    } else { None };
                    render::game::draw_world(
                        &mut rl, &thread, &mut crt, &game_client.world, camera,
                        render_w, render_h, theme, game_time, &card_anim,
                        game_client.my_index, false, btns.as_ref(),
                    );
                }
            }
            AppState::ReturnToLobby { timer, .. } => {
                *timer -= dt;
                let w_now = rl.get_screen_width();
                let h_now = rl.get_screen_height();
                let accent = menu.theme().particle_color_primary;
                menu.fx.update(dt, w_now, h_now, accent);
                let mut d = rl.begin_drawing(&thread);
                d.clear_background(Color::BLACK);
                {
                    let mut t = d.begin_texture_mode(&thread, &mut crt.ui_target);
                    menu.draw_bg(&mut *t);
                }
                {
                    let mut s = d.begin_shader_mode(&mut crt.shader_ui);
                    s.draw_texture_rec(
                        crt.ui_target.texture(),
                        Rectangle::new(0.0, 0.0, render_w as f32, -(render_h as f32)),
                        Vector2::new(0.0, 0.0), Color::WHITE,
                    );
                }
            }
            AppState::Reconnecting { addr, name, accessories, timer, retries } => {
                *timer -= dt;
                if *timer <= 0.0 {
                    let addr_c = addr.clone();
                    let name_c = name.clone();
                    let acc_c = accessories.clone();
                    match LobbyClient::connect(&addr_c, &name_c, acc_c) {
                        Ok(client) => {
                            lobby_time = 0.0;
                            client_addr = addr_c;
                            next_state = Some(AppState::Lobby(LobbyRole::Client(client)));
                        }
                        Err(_) => {
                            *retries += 1;
                            if *retries >= 10 {
                                menu.show_error("Failed to reconnect");
                                next_state = Some(AppState::Menu);
                            } else {
                                *timer = 0.5;
                            }
                        }
                    }
                }
                // Draw reconnecting screen with menu background
                {
                    let w_now = rl.get_screen_width();
                    let h_now = rl.get_screen_height();
                    let accent = menu.theme().particle_color_primary;
                    menu.fx.update(dt, w_now, h_now, accent);
                    let theme = menu.theme();
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::BLACK);
                    {
                        let mut t = d.begin_texture_mode(&thread, &mut crt.ui_target);
                        menu.draw_bg(&mut *t);
                        let text = "Reconnecting...";
                        let size = 36;
                        let tw = t.measure_text(text, size);
                        t.draw_text(text, render_w / 2 - tw / 2, render_h / 2 - size / 2, size, theme.item_color);
                    }
                    {
                        let mut s = d.begin_shader_mode(&mut crt.shader_ui);
                        s.draw_texture_rec(
                            crt.ui_target.texture(),
                            Rectangle::new(0.0, 0.0, render_w as f32, -(render_h as f32)),
                            Vector2::new(0.0, 0.0), Color::WHITE,
                        );
                    }
                }
                // ESC to cancel
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    next_state = Some(AppState::Menu);
                }
            }
        }

        // Handle deferred lobby creation (wait for port to free up after GameServer drop)
        if let AppState::ReturnToLobby { ref name, dev_mode, ref accessories, ref settings, ref mut timer, ref mut retries } = app_state {
            if *timer <= 0.0 {
                let name = name.clone();
                let acc = accessories.clone();
                let saved_settings = settings.clone();
                match LobbyServer::start(&name, DEFAULT_PORT, acc) {
                    Ok(mut server) => {
                        server.dev_mode = dev_mode;
                        if let Some(s) = saved_settings {
                            server.state.settings = s;
                        }
                        lobby_time = 0.0;
                        app_state = AppState::Lobby(LobbyRole::Host(server));
                    }
                    Err(_) => {
                        *retries += 1;
                        if *retries >= 10 {
                            menu_cooldown = 0.1;
                            app_state = AppState::Menu;
                        } else {
                            *timer = 0.2;
                        }
                    }
                }
            }
        }

        if let Some(new_state) = next_state {
            if matches!(new_state, AppState::Menu) {
                sfx.stop_game_music();
                menu_cooldown = 0.1;
            }
            app_state = new_state;
        }
    }
}

fn run_level_editor() {
    let (mut rl, thread) = raylib::init()
        .size(1280, 720)
        .title("1VI1 - Level Editor")
        .resizable()
        .build();

    rl.set_target_fps(60);
    rl.set_exit_key(None);

    let mut ed = editor::Editor::new();

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        if ed.update(&mut rl, dt) {
            break;
        }
        let mut d = rl.begin_drawing(&thread);
        ed.draw(&mut d);
    }
}
