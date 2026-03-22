mod combat;
mod game;
mod level;
mod lobby;
mod menu;
mod physics;
mod player;
mod render;

use game::world::World;
use lobby::client::LobbyClient;
use lobby::screen::{draw_lobby, lobby_input, LobbyInput};
use lobby::server::LobbyServer;
use menu::menu::{Menu, MenuAction};
use raylib::prelude::*;
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
    InGame,
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
    let mut world = World::new();
    let mut menu = Menu::new();
    let mut app_state = AppState::Menu;
    let mut game_time: f32 = 0.0;
    let mut lobby_time: f32 = 0.0;

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

        // We need to handle state transitions carefully due to ownership
        let mut next_state = None;

        match &mut app_state {
            AppState::Menu => {
                match menu.update(&mut rl, dt) {
                    MenuAction::Host => {
                        let name = if menu.player_name.is_empty() { "Player" } else { &menu.player_name };
                        match LobbyServer::start(name, DEFAULT_PORT) {
                            Ok(server) => {
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
                        match LobbyClient::connect(&addr, name) {
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
                    let mut d = rl.begin_drawing(&thread);
                    menu.draw(&mut d);
                }
            }
            AppState::Lobby(role) => {
                lobby_time += dt;
                let input = lobby_input(&rl);

                match role {
                    LobbyRole::Host(server) => {
                        // Handle host input
                        match input {
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
                                next_state = Some(AppState::Menu);
                            }
                            LobbyInput::CopyIP => {
                                let _ = rl.set_clipboard_text(&server.my_addr);
                            }
                            LobbyInput::None => {}
                        }

                        let game_start = server.update();
                        if game_start {
                            world = World::from_lobby(&server.state);
                            game_time = 0.0;
                            next_state = Some(AppState::InGame);
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
                            );
                        }
                    }
                    LobbyRole::Client(client) => {
                        client.update();

                        match input {
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

                        if client.rejected {
                            menu.show_error("Lobby is full");
                            next_state = Some(AppState::Menu);
                        }

                        if client.game_starting {
                            world = World::from_lobby(&client.state);
                            game_time = 0.0;
                            next_state = Some(AppState::InGame);
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
                            );
                        }
                    }
                }
            }
            AppState::InGame => {
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    next_state = Some(AppState::Menu);
                } else {
                    game_time += dt;
                    let camera = render::camera::game_camera(&world);
                    world.update(&rl, &camera, dt);
                    let theme = menu.theme();
                    render::game::draw_world(
                        &mut rl, &thread, &mut crt, &world, camera, render_w, render_h, theme, game_time,
                    );
                }
            }
        }

        if let Some(new_state) = next_state {
            app_state = new_state;
        }
    }
}
