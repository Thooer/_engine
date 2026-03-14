use super::{Engine, EngineTrait};
use bevy_ecs::prelude::World;
use engine_renderer::renderer::{MainRenderer, SurfaceContext};
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

    // input 已移除，使用 world.get_resource::<InputState>() 代替

    fn world(&self) -> &World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    fn frame_index(&self) -> u32 {
        self.frame_index
    }

    fn request_exit(&mut self) {
        self.exit_requested = true;
    }
}

