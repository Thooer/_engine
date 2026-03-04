//! Application 层 - 引擎生命周期与主循环

/// 应用程序生命周期接口
pub trait Application {
    /// 初始化应用
    fn init(&mut self);

    /// 更新应用状态
    /// 
    /// # Arguments
    /// 
    /// * `dt` - 距离上一帧的时间（秒）
    fn update(&mut self, dt: f32);

    /// 关闭应用，清理资源
    fn shutdown(&mut self);
}
#[cfg(test)]
mod tests;

use std::time::Instant;
use winit::event_loop::EventLoop;

mod engine_app;
use engine_app::EngineApp;

/// 引擎运行器
pub struct EngineRunner;

#[path = "ApplicationHandler_EngineApp.rs"]
mod application_handler_engine_app;

/// 运行应用程序
pub fn run<A: Application + 'static>(app: A) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut engine_app = EngineApp {
        app,
        window: None,
        frame_count: 0,
        last_frame_time: Instant::now(),
        should_exit: false,
    };
    event_loop.run_app(&mut engine_app)?;
    Ok(())
}
