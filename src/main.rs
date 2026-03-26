mod combat;
mod game;
mod level;
mod lobby;
mod menu;
mod physics;
mod player;
mod render;

use game::client::GameClient;
use game::server::GameServer;
use game::state::GameState;
use game::world::World;
use lobby::client::LobbyClient;
use lobby::screen::{draw_lobby, lobby_input, LobbyInput, LobbySettingsState};
use lobby::server::LobbyServer;
use lobby::state::{LobbyColor, PlayerSlot};
use menu::menu::{Menu, MenuAction};
use raylib::prelude::*;
use render::cards::CardPickAnim;
use render::crt::CrtFilter;

const SCREEN_WIDTH: i32 = 960;
const SCREEN_HEIGHT: i32 = 540;
const DEFAULT_PORT: u16 = 7878;

enum LobbyRole {
    Host(LobbyServer),
    Client(LobbyClient),
}

enum AppState {
    Menu,
    Lobby(LobbyRole),
    InGameHost(GameServer),
    InGameClient(GameClient),
    ReturnToLobby { name: String, dev_mode: bool, accessories: Vec<(u8, u8, u8, u8)> },
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("1VI1")
        .resizable()
        .build();

    rl.set_target_fps(144);
    rl.set_exit_key(None);

    let mut crt = CrtFilter::new(&mut rl, &thread, SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut render_w = SCREEN_WIDTH;
    let mut render_h = SCREEN_HEIGHT;
    let mut menu = Menu::new();
    let mut app_state = AppState::Menu;
    let mut game_time: f32 = 0.0;
    let mut lobby_time: f32 = 0.0;
    let mut card_anim = CardPickAnim::new();
    let mut dev_overlay_open = false;
    let mut lobby_settings = LobbySettingsState::new();

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_F11) {
            rl.toggle_fullscreen();
        }

        let w = rl.get_screen_width();
        let h = rl.get_screen_height();
        if w != render_w || h != render_h {
            render_w = w;
            render_h = h;
            crt = CrtFilter::new(&mut rl, &thread, render_w, render_h);
        }

        let mut next_state = None;

        match &mut app_state {
            AppState::Menu => {
                match menu.update(&mut rl, dt) {
                    MenuAction::Host => {
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
                        let name = if menu.player_name.is_empty() { "Player" } else { &menu.player_name };
                        let acc = menu.accessories.iter().filter(|a| a.0 != 0xFF).cloned().collect();
                        match LobbyClient::connect(&addr, name, acc) {
                            Ok(client) => {
                                lobby_time = 0.0;
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
                    menu.draw(&mut d);
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
                                server.host_toggle_ready();
                            }
                            LobbyInput::Leave => {
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
                            d.clear_background(theme.bg);
                            draw_lobby(
                                &mut d,
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
                                client.toggle_ready();
                            }
                            LobbyInput::Leave => {
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
                            d.clear_background(theme.bg);
                            draw_lobby(
                                &mut d,
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
                    }
                }
            }
            AppState::InGameHost(game_server) => {
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    if dev_overlay_open {
                        dev_overlay_open = false;
                    } else if matches!(game_server.world.state, GameState::MatchOver { .. }) {
                        let name = if menu.player_name.is_empty() { "Player".to_string() } else { menu.player_name.clone() };
                        let acc: Vec<_> = menu.accessories.iter().filter(|a| a.0 != 0xFF).cloned().collect();
                        next_state = Some(AppState::ReturnToLobby { name, dev_mode: menu.dev_mode, accessories: acc });
                    } else {
                        let name = game_server.world.players[0].name.clone();
                        game_server.notify_leaving(&name);
                        next_state = Some(AppState::Menu);
                    }
                } else {
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

                    if let Some(ref left_name) = game_server.player_left {
                        menu.show_error(&format!("{} Left", left_name));
                        next_state = Some(AppState::Menu);
                    }

                    card_anim.update(&game_server.world, dt);
                    let theme = menu.theme();
                    render::game::draw_world(
                        &mut rl, &thread, &mut crt, &game_server.world, camera,
                        render_w, render_h, theme, game_time, &card_anim, 0,
                        dev_overlay_open,
                    );
                }
            }
            AppState::InGameClient(game_client) => {
                if let Some(ref msg) = game_client.disconnect_message {
                    menu.show_error(msg);
                    next_state = Some(AppState::Menu);
                } else if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    next_state = Some(AppState::Menu);
                } else {
                    game_time += dt;
                    let camera = render::camera::game_camera(&game_client.world);
                    game_client.update(&rl, &camera, dt);
                    card_anim.update(&game_client.world, dt);
                    let theme = menu.theme();
                    render::game::draw_world(
                        &mut rl, &thread, &mut crt, &game_client.world, camera,
                        render_w, render_h, theme, game_time, &card_anim,
                        game_client.my_index, false,
                    );
                }
            }
            AppState::ReturnToLobby { .. } => {} // handled below
        }

        // Handle deferred lobby creation (port is now free after old state dropped)
        if let AppState::ReturnToLobby { ref name, dev_mode, ref accessories } = app_state {
            let name = name.clone();
            let acc = accessories.clone();
            if let Ok(mut server) = LobbyServer::start(&name, DEFAULT_PORT, acc) {
                server.dev_mode = dev_mode;
                lobby_time = 0.0;
                app_state = AppState::Lobby(LobbyRole::Host(server));
            } else {
                app_state = AppState::Menu;
            }
        }

        if let Some(new_state) = next_state {
            app_state = new_state;
        }
    }
}
