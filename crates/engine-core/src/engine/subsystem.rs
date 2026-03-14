//! 子系统抽象 - Engine Subsystems
//!
//! 提供子系统 Trait，用于扩展引擎功能（渲染器、物理、网络等）

use crate::engine::EngineCore;
use std::any::Any;
use std::fmt::Debug;

/// 子系统 Trait - 引擎扩展的基础
///
/// 子系统是可选的组件，用于添加引擎功能：
/// - 渲染子系统：负责渲染
/// - 物理子系统：负责物理模拟
/// - 音频子系统：负责音频播放
/// - 网络子系统：负责网络通信
pub trait EngineSubsystem: Send + Sync + Debug {
    /// 子系统名称
    fn name(&self) -> &str;

    /// 初始化子系统
    ///
    /// 在引擎初始化阶段调用
    fn setup(&self, _core: &mut EngineCore) {}

    /// 更新子系统
    ///
    /// 在每帧更新阶段调用
    fn update(&self, _core: &mut EngineCore, _dt: f32) {}

    /// 关闭子系统
    ///
    /// 在引擎关闭前调用
    fn shutdown(&self, _core: &mut EngineCore) {}
}

/// 子系统注册表 - 管理所有子系统
#[derive(Default)]
pub struct SubsystemRegistry {
    subsystems: Vec<Box<dyn EngineSubsystem>>,
}

impl SubsystemRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            subsystems: Vec::new(),
        }
    }

    /// 添加子系统
    pub fn add<S: EngineSubsystem + 'static>(&mut self, subsystem: S) {
        self.subsystems.push(Box::new(subsystem));
    }

    /// 初始化所有子系统
    pub fn setup_all(&self, core: &mut EngineCore) {
        for subsystem in &self.subsystems {
            subsystem.setup(core);
        }
    }

    /// 更新所有子系统
    pub fn update_all(&self, core: &mut EngineCore, dt: f32) {
        for subsystem in &self.subsystems {
            subsystem.update(core, dt);
        }
    }

    /// 关闭所有子系统
    pub fn shutdown_all(&self, core: &mut EngineCore) {
        for subsystem in &self.subsystems {
            subsystem.shutdown(core);
        }
    }

    /// 获取子系统数量
    pub fn len(&self) -> usize {
        self.subsystems.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.subsystems.is_empty()
    }
}
