use raylib::prelude::*;

pub struct CrtFilter {
    pub target: RenderTexture2D,
    pub shader: Shader,
}

impl CrtFilter {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Self {
        let target = rl
            .load_render_texture(thread, width as u32, height as u32)
            .expect("Failed to create render texture");

        let shader = rl.load_shader(thread, None, Some("assets/shaders/crt.fs"));

        Self { target, shader }
    }
}
