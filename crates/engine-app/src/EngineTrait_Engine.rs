use super::{Engine, EngineTrait};
use engine_core::input::InputState;
use engine_renderer::renderer::SurfaceContext;
use winit::window::Window;

impl EngineTrait for Engine {
    fn window(&self) -> &'static Window {
        self.window.expect("Engine window not initialized")
    }

    fn ctx(&self) -> &SurfaceContext<'static> {
        self.ctx
            .as_ref()
            .expect("Engine SurfaceContext not initialized")
    }

    fn ctx_mut(&mut self) -> &mut SurfaceContext<'static> {
        self.ctx
            .as_mut()
            .expect("Engine SurfaceContext not initialized")
    }

    fn input(&self) -> &InputState {
        &self.input
    }

    fn frame_index(&self) -> u32 {
        self.frame_index
    }

    fn request_exit(&mut self) {
        self.exit_requested = true;
    }
}

