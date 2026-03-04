//! ToyEngine Core - 核心系统模块
//!
//! 包含 ECS、资源管理、事件系统等核心功能

/// 平台层 - 窗口与事件循环
pub mod platform;

/// 应用程序层 - 生命周期与主循环
pub mod app;

/// 核心模块
pub mod core {
    // 未来实现核心功能
}

/// ECS 系统
pub mod ecs;

/// 文件系统与基础 IO 抽象（阶段 4 v0）
pub mod fs;

/// 资源管理
pub mod resources;

/// 输入系统 v0（键盘为主）
pub mod input;

/// 相机相关的通用工具与系统
pub mod camera;
