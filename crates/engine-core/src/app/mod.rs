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

mod engine_app;
use engine_app::EngineApp;

/// 引擎运行器
pub struct EngineRunner;

pub trait EngineRunnerTrait {
    fn run<A: Application + 'static>(app: A) -> Result<(), Box<dyn std::error::Error>>;
}

#[path = "ApplicationHandler_EngineApp.rs"]
mod application_handler_engine_app;

#[path = "EngineRunnerTrait_EngineRunner.rs"]
mod engine_runner_trait_engine_runner;
