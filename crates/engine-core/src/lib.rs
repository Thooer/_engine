//! ToyEngine Core - 核心系统模块
//!
//! 包含 ECS、资源管理、事件系统等核心功能

use bevy_ecs::prelude::Resource;

/// 引擎核心结构
pub mod engine;
pub use engine::{EngineConfig, EngineCore, EngineCoreTrait};

/// 子系统抽象
pub use engine::subsystem::{EngineSubsystem, SubsystemRegistry};

/// 插件系统
pub mod plugins;
pub use plugins::{Plugin, PluginContext, PluginRegistry, ScheduleType};

/// 平台层 - 窗口与事件循环
mod platform;

// NOTE:
// `engine-core/src/app` 旧的 Application/EngineRunner 体系已被 `engine-app` 取代。
// 这里不再对外导出，避免出现两套应用接口并存导致的架构分裂。
// 如需运行应用，请使用 `engine-app` crate。

/// 核心模块
mod core;

/// 配置目录常量
pub mod config;

/// ECS 系统
pub mod ecs;

/// 文件系统与基础 IO 抽象（阶段 4 v0）
mod fs;

/// 资源管理
mod resources;

pub use resources::{AssetConfig, AssetHandle, AssetServer, AssetType, LoadState, Scene, SceneLoader, ComponentData, SceneEntity, AssetMetadata};

/// 输入系统 v0（键盘为主）
pub mod input;

/// 相机相关的通用工具与系统
mod camera;

/// 帧计数器资源 - 用于需要在每帧获取帧号的系统
#[derive(Resource, Default)]
pub struct FrameCounter(pub u32);

/// 待加载项目路径 - UI 选择新项目后通过此资源触发热重载
#[derive(Resource)]
pub struct PendingProjectReload {
    pub path: std::path::PathBuf,
}

impl PendingProjectReload {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path }
    }
}
