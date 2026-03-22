mod combat;
mod game;
mod level;
mod menu;
mod physics;
mod player;
mod render;

use game::world::World;
use menu::menu::{Menu, MenuAction};
use raylib::prelude::*;
use render::crt::CrtFilter;

const SCREEN_WIDTH: i32 = 960;
const SCREEN_HEIGHT: i32 = 540;

enum AppState {
    Menu,
    InGame,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("1VI1")
        .resizable()
        .build();

    rl.set_target_fps(144);
    rl.set_exit_key(None); // We handle ESC ourselves

    let mut crt = CrtFilter::new(&mut rl, &thread, SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut render_w = SCREEN_WIDTH;
    let mut render_h = SCREEN_HEIGHT;
    let mut world = World::new();
    let mut menu = Menu::new();
    let mut app_state = AppState::Menu;
    let mut game_time: f32 = 0.0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // F11 fullscreen toggle
        if rl.is_key_pressed(KeyboardKey::KEY_F11) {
            rl.toggle_fullscreen();
        }

        // Resize render texture if window size changed
        let w = rl.get_screen_width();
        let h = rl.get_screen_height();
        if w != render_w || h != render_h {
            render_w = w;
            render_h = h;
            crt = CrtFilter::new(&mut rl, &thread, render_w, render_h);
        }

        match app_state {
            AppState::Menu => {
                match menu.update(&rl, dt) {
                    MenuAction::StartGame => {
                        world = World::new();
                        app_state = AppState::InGame;
                    }
                    MenuAction::Quit => break,
                    MenuAction::None => {}
                }

                let mut d = rl.begin_drawing(&thread);
                menu.draw(&mut d);
            }
            AppState::InGame => {
                // ESC goes back to menu
                if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    app_state = AppState::Menu;
                    continue;
                }

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
}
