use std::time::Instant;

use engine_renderer::renderer::{
    DefaultSurfaceContextNew, SurfaceContextNew, SurfaceContextTrait, SurfaceSize,
};
use engine_core::input::InputStateExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::Window,
};

use super::{App, AppRunner, AppRunnerTrait, EngineTrait};

impl<A: App> ApplicationHandler for AppRunner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes().with_title(self.config.title);
        let window = event_loop
            .create_window(window_attributes)
            .expect("window create failed");
        let window = Box::leak(Box::new(window));

        let s = window.inner_size();
        let size = SurfaceSize {
            width: s.width,
            height: s.height,
        };

        let ctx = pollster::block_on(DefaultSurfaceContextNew::surface_context_new(window, size))
            .expect("surface ctx create failed");

        self.engine.ctx = Some(ctx);
        self.engine.window = Some(window);
        self.last_frame_time = Some(Instant::now());

        self.app.on_start(&mut self.engine);

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.engine.input.on_window_event(&event);
        self.app.on_window_event(&mut self.engine, &event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if self.engine.ctx.is_some() {
                    self.engine.ctx_mut().resize(SurfaceSize {
                        width: size.width,
                        height: size.height,
                    });
                    self.app.on_resize(
                        &mut self.engine,
                        SurfaceSize {
                            width: size.width,
                            height: size.height,
                        },
                    );
                }
            }
            WindowEvent::RedrawRequested => {
                let dt = self.dt_seconds();
                self.app.on_update(&mut self.engine, dt);
                self.app.on_render(&mut self.engine);

                self.engine.frame_index += 1;
                if let Some(max_frames) = self.config.max_frames {
                    if self.engine.frame_index >= max_frames {
                        event_loop.exit();
                        return;
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.engine.input.next_frame();

        if self.engine.exit_requested {
            event_loop.exit();
            return;
        }

        if let Some(window) = self.engine.window {
            window.request_redraw();
        }
    }
}

