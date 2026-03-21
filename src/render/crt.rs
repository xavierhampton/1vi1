use raylib::prelude::*;

pub struct CrtFilter {
    pub env_target: RenderTexture2D,
    pub player_target: RenderTexture2D,
    pub shader: Shader,
    pub shader_no_aberration: Shader,
}

impl CrtFilter {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Self {
        let env_target = rl
            .load_render_texture(thread, width as u32, height as u32)
            .expect("Failed to create render texture");

        let player_target = rl
            .load_render_texture(thread, width as u32, height as u32)
            .expect("Failed to create player render texture");

        let shader = rl.load_shader(thread, None, Some("assets/shaders/crt.fs"));
        let shader_no_aberration =
            rl.load_shader(thread, None, Some("assets/shaders/crt_no_aberration.fs"));

        Self {
            env_target,
            player_target,
            shader,
            shader_no_aberration,
        }
    }
}
