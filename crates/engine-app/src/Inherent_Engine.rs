use super::Engine;
use engine_core::input::InputState;
use engine_renderer::renderer::SurfaceContext;
use winit::window::Window;

impl Engine {
    pub(crate) fn window(&self) -> &Window {
        self.window.expect("Engine window not initialized")
    }

    pub(crate) fn ctx(&self) -> &SurfaceContext<'static> {
        self.ctx
            .as_ref()
            .expect("Engine SurfaceContext not initialized")
    }

    pub(crate) fn ctx_mut(&mut self) -> &mut SurfaceContext<'static> {
        self.ctx
            .as_mut()
            .expect("Engine SurfaceContext not initialized")
    }

    pub(crate) fn input(&self) -> &InputState {
        &self.input
    }

    pub(crate) fn frame_index(&self) -> u32 {
        self.frame_index
    }

    pub(crate) fn request_exit(&mut self) {
        self.exit_requested = true;
    }
}

