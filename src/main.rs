//! ToyEngine - 主入口
//! 
//! 这是一个使用 Rust 构建的游戏引擎

mod demo_app;

use engine_core::app::run;
use demo_app::DemoApp;

fn main() {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    
    tracing::info!("ToyEngine v0.1.0 启动中...");
    tracing::info!("所有依赖已加载并自检通过");
    
    // 创建并运行演示应用
    let app = DemoApp::new();
    
    if let Err(e) = run(app) {
        tracing::error!("引擎运行失败: {}", e);
        std::process::exit(1);
    }
}
