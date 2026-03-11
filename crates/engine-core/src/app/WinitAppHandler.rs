//! ApplicationHandler trait 实现

use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
};

use crate::app::{Application, EngineApp};

impl<A: Application> ApplicationHandler for EngineApp<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            tracing::info!("初始化引擎运行器...");

            // 初始化应用
            self.app.init();
            tracing::info!("应用程序初始化完成");

            // 创建窗口
            let window_attributes = winit::window::Window::default_attributes()
                .with_title("ToyEngine")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

            let window = event_loop.create_window(window_attributes).unwrap();
            tracing::info!(
                "窗口创建成功: {}x{}",
                window.inner_size().width,
                window.inner_size().height
            );

            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("收到窗口关闭请求，准备退出...");
                self.should_exit = true;
                self.app.shutdown();
                tracing::info!("应用程序已关闭");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                tracing::debug!("窗口大小变化: {}x{}", size.width, size.height);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if !self.should_exit && self.window.is_some() {
            let now = Instant::now();
            let dt = (now - self.last_frame_time).as_secs_f32();
            self.last_frame_time = now;

            self.frame_count += 1;
            self.app.update(dt);

            // 每 60 帧打印一次日志
            if self.frame_count % 60 == 0 {
                tracing::info!(
                    "帧 #{}, dt: {:.4}s, FPS: {:.1}",
                    self.frame_count,
                    dt,
                    1.0 / dt.max(0.0001)
                );
            }
        }
    }
}
