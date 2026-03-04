//! 简单的 Demo 应用程序实现

use engine_core::app::Application;

/// 演示应用程序
pub struct DemoApp {
    initialized: bool,
}

impl DemoApp {
    /// 创建新的演示应用
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }
}

impl Application for DemoApp {
    fn init(&mut self) {
        if !self.initialized {
            tracing::info!("DemoApp 初始化中...");
            self.initialized = true;
            tracing::info!("DemoApp 初始化完成");
        }
    }

    fn update(&mut self, _dt: f32) {
        // 简单的更新逻辑
        // 这里可以添加游戏逻辑
    }

    fn shutdown(&mut self) {
        tracing::info!("DemoApp 正在关闭...");
    }
}

impl Default for DemoApp {
    fn default() -> Self {
        Self::new()
    }
}
