//! ToyEngine 核心引擎结构
//!
//! 提供分层架构：
//! - EngineCore: 核心层（ECS World + 配置）
//! - PlatformContext: 平台层（窗口 + GPU 上下文，可选）
//! - Subsystems: 子系统层（渲染器、物理等，可选）

use bevy_ecs::prelude::World;

pub mod subsystem;
pub use subsystem::{EngineSubsystem, SubsystemRegistry};

/// 引擎核心配置
#[derive(Clone, Copy, Debug)]
pub struct EngineConfig {
    /// 窗口标题
    pub title: &'static str,
    /// 最大帧数限制（None 表示无限制）
    pub max_frames: Option<u32>,
    /// 固定物理时间步长（秒），None 表示不使用固定步长
    pub fixed_dt_seconds: Option<f32>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: "ToyEngine",
            max_frames: None,
            fixed_dt_seconds: Some(1.0 / 60.0),
        }
    }
}

/// 引擎核心 - 无状态，只包含 ECS World 和基础配置
///
/// 这是引擎的最小必需部分，支持：
/// - Headless 模式（无窗口/无渲染）
/// - 服务器端运行
/// - 单元测试
#[derive(Debug)]
pub struct EngineCore {
    /// ECS 世界
    pub world: World,
    /// 引擎配置
    pub config: EngineConfig,
    /// 当前帧索引
    pub frame_index: u32,
    /// 请求退出标志
    pub exit_requested: bool,
}

impl EngineCore {
    /// 创建新的引擎核心
    pub fn new(config: EngineConfig) -> Self {
        Self {
            world: World::new(),
            config,
            frame_index: 0,
            exit_requested: false,
        }
    }

    /// 请求退出引擎
    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    /// 推进帧索引
    pub fn advance_frame(&mut self) {
        self.frame_index += 1;
    }
}

impl Default for EngineCore {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

/// 引擎核心 Trait - 用于泛型编程
pub trait EngineCoreTrait: Send + Sync {
    /// 获取 ECS World 引用
    fn world(&self) -> &World;
    /// 获取 ECS World 可变引用
    fn world_mut(&mut self) -> &mut World;
    /// 获取当前帧索引
    fn frame_index(&self) -> u32;
    /// 请求退出
    fn request_exit(&mut self);
}

impl EngineCoreTrait for EngineCore {
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
