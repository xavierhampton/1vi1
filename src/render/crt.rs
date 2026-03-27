use raylib::prelude::*;

pub struct CrtFilter {
    pub env_target: RenderTexture2D,
    pub player_target: RenderTexture2D,
    pub ui_target: RenderTexture2D,
    pub shader: Shader,
    pub shader_no_aberration: Shader,
    pub shader_ui: Shader,
}

impl CrtFilter {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Self {
        let env_target = rl
            .load_render_texture(thread, width as u32, height as u32)
            .expect("Failed to create render texture");

        let player_target = rl
            .load_render_texture(thread, width as u32, height as u32)
            .expect("Failed to create player render texture");

        let ui_target = rl
            .load_render_texture(thread, width as u32, height as u32)
            .expect("Failed to create UI render texture");

        let shader = rl.load_shader(thread, None, Some("assets/shaders/crt.fs"));
        let shader_no_aberration =
            rl.load_shader(thread, None, Some("assets/shaders/crt_no_aberration.fs"));
        let shader_ui =
            rl.load_shader(thread, None, Some("assets/shaders/crt_ui.fs"));

        Self {
            env_target,
            player_target,
            ui_target,
            shader,
            shader_no_aberration,
            shader_ui,
        }
    }
}

/// Given a texture position, find where it appears on screen after barrel distortion.
/// The shader barrel contracts UVs: `barrel(screen) = texture`, so we need the inverse.
/// Iteratively solves `t = s / (1 + |s|^2 * k)` for `s`. Must match k=0.2 in shaders.
pub fn barrel_screen_pos(tx: f32, ty: f32, w: f32, h: f32) -> (f32, f32) {
    let tc_x = tx / w - 0.5;
    let tc_y = ty / h - 0.5;
    let k = 0.2;
    let mut sx = tc_x;
    let mut sy = tc_y;
    for _ in 0..5 {
        let r2 = sx * sx + sy * sy;
        sx = tc_x * (1.0 + r2 * k);
        sy = tc_y * (1.0 + r2 * k);
    }
    ((sx + 0.5) * w, (sy + 0.5) * h)
}
