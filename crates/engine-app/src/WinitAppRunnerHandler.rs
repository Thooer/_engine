use std::time::Instant;

use engine_core::input::InputStateExt;
use engine_renderer::renderer::{
    DefaultSurfaceContextNew, RendererTrait, SurfaceContextNew, SurfaceContextTrait, SurfaceSize,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::Window,
};

use super::{App, AppRunner, AppRunnerTrait, EngineTrait, FrameCounter};

impl<A: App> ApplicationHandler for AppRunner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        tracing::info!("resumed called");
        
        if self.engine.window.is_some() {
            tracing::debug!("window already exists, skipping");
            return;
        }

        let window_attributes = Window::default_attributes().with_title(self.config.title);
        let window = event_loop
            .create_window(window_attributes)
            .expect("window create failed");
        let window = Box::leak(Box::new(window));
        
        // 触发第一次渲染
        window.request_redraw();
        tracing::debug!("first request_redraw called");

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

        // 让 App 配置系统调度器（添加需要渲染器上下文的系统）
        self.app.configure_schedule(&mut self.schedule);

        // 运行 setup 系统（依赖 on_start 里 spawn 的实体/资源）
        if !self.setup_done {
            self.schedule.run_setup(&mut self.engine.core.world);
            self.setup_done = true;

            // 初始化帧计数器
            self.engine.core.world.insert_resource(FrameCounter::default());
        }

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // 只更新 ECS Resource 中的 InputState（不再同时更新 Engine.input）
        if let Some(mut input) = self.engine.core.world.get_resource_mut::<engine_core::input::InputState>() {
            input.on_window_event(&event);
        }
        
        // P4: 自动转发事件到渲染器
        let window = self.engine.window();
        if let Some(renderer) = self.engine.main_renderer.as_mut() {
            renderer.handle_event(window, &event);
        }
        
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
                
                // 递增帧计数器（让本帧系统看到最新帧号）
                if let Some(mut counter) = self.engine.core.world.get_resource_mut::<FrameCounter>() {
                    counter.0 += 1;
                }

                // 运行每帧更新系统（物理、相机等）
                self.schedule.run_update(&mut self.engine.core.world);
                
                self.app.on_update(&mut self.engine, dt);
                
                // P2: 自动调用渲染
                if let Some(mut renderer) = self.engine.main_renderer.take() {
                    engine_renderer::loaders::collect_from_world(&mut self.engine.core.world, &mut renderer);
                    
                    let ctx = self.engine.ctx_mut();
                    let _ = renderer.render(ctx);
                    
                    self.engine.main_renderer = Some(renderer);
                    
                    // 请求下一帧渲染
                    if let Some(window) = &self.engine.window {
                        window.request_redraw();
                    }
                }
                
                self.app.on_render(&mut self.engine);
                self.engine.core.frame_index += 1;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // 只更新 ECS Resource 中的 InputState（不再同时更新 Engine.input）
        if let Some(mut input) = self.engine.core.world.get_resource_mut::<engine_core::input::InputState>() {
            input.next_frame();
        }

        if self.engine.core.exit_requested {
            event_loop.exit();
            return;
        }

        if let Some(window) = self.engine.window {
            window.request_redraw();
        }
    }
}
