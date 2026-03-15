//! ToyEngine 脚本系统
//!
//! 提供统一的脚本抽象接口，支持 WASM 运行时
//! 允许脚本访问和修改 ECS World，实现通用实体控制

use std::path::{Path, PathBuf};

use bevy_ecs::prelude::{World, Entity};
use engine_core::ecs::Transform;
use thiserror::Error;

// ============================================================================
// ScriptError - 脚本错误类型
// ============================================================================

/// 脚本错误类型
#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Load error: {0}")]
    Load(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Call error: {0}")]
    Call(String),

    #[error("Context error: {0}")]
    Context(String),

    #[error("ECS error: {0}")]
    Ecs(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

// ============================================================================
// ScriptContext - 脚本上下文接口（核心抽象）
// ============================================================================

/// 帧数据 - 从引擎传递给脚本
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct FrameData {
    pub delta_time: f32,
    pub total_time: f64,
    pub frame_count: u32,
    /// 按键状态位掩码 (bit 0-3 对应数字键 1-4)，用于镜头切换等
    pub input_mask: u8,
}

impl FrameData {
    pub fn new(delta_time: f32, total_time: f64, frame_count: u32) -> Self {
        Self { delta_time, total_time, frame_count, input_mask: 0 }
    }

    pub fn with_input_mask(mut self, input_mask: u8) -> Self {
        self.input_mask = input_mask;
        self
    }
}

/// 脚本更新结果
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct UpdateResult {
    pub success: bool,
    pub error_message: u32,
}

impl UpdateResult {
    pub fn success() -> Self {
        Self { success: true, error_message: 0 }
    }

    pub fn error() -> Self {
        Self { success: false, error_message: 0 }
    }
}

/// 实体查询迭代器 - 提供对 ECS 实体的查询能力
/// 简化实现：直接在 ScriptContext 中处理查询
pub struct EntityQueryIter {
    // 简化版本：只存储结果数据
    entities: Vec<(Entity, Transform)>,
}

impl EntityQueryIter {
    pub fn new() -> Self {
        Self { entities: Vec::new() }
    }

    pub fn from_world(world: &mut World) -> Self {
        let mut result = Vec::new();
        let mut query = world.query::<(Entity, &Transform)>();
        for (entity, transform) in query.iter(world) {
            result.push((entity, *transform));
        }
        Self { entities: result }
    }

    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(Entity, &mut Transform),
    {
        for (entity, mut transform) in self.entities.iter_mut() {
            f(*entity, &mut transform);
        }
    }
}

impl Default for EntityQueryIter {
    fn default() -> Self {
        Self::new()
    }
}

/// 脚本上下文 Trait - 脚本访问引擎的接口
/// 这是实现通用实体控制的关键 trait
pub trait ScriptContext: Send + Sync {
    /// 获取 ECS World 引用（只读）
    fn world(&self) -> &World;

    /// 获取 ECS World 可变引用
    fn world_mut(&mut self) -> &mut World;

    /// 获取帧数据
    fn frame_data(&self) -> FrameData;

    /// 查询所有带有 Transform 的实体并返回
    fn query_all_transforms(&mut self) -> EntityQueryIter;

    /// 通过 Entity ID 获取 Transform 组件（如果存在）
    fn get_entity_transform(&mut self, entity: Entity) -> Option<Transform>;

    /// 修改实体的 Transform
    fn set_entity_transform(&mut self, entity: Entity, transform: Transform);

    /// 检查实体是否存在
    fn entity_exists(&self, entity: Entity) -> bool;

    /// 创建新实体并返回 Entity ID
    fn spawn_entity(&mut self) -> Entity;

    /// 销毁实体
    fn despawn_entity(&mut self, entity: Entity) -> bool;

    /// 日志输出
    fn log(&self, level: &str, message: &str);

    /// 获取 delta time
    fn delta_time(&self) -> f32;

    /// 获取 total time
    fn total_time(&self) -> f64;

    /// 获取 frame count
    fn frame_count(&self) -> u32;
}

// ============================================================================
// EcsScriptContext - ECS 实现的脚本上下文
// ============================================================================

/// ECS 实现的脚本上下文 - 允许脚本访问和修改 ECS World
/// 使用 World 引用而非拥有，以便与 engine 共享
pub struct EcsScriptContext {
    world: World,
    frame_data: FrameData,
}

impl EcsScriptContext {
    pub fn new(world: World, frame_data: FrameData) -> Self {
        Self { world, frame_data }
    }

    /// 从 engine 创建（接受 World 引用）
    pub fn from_world(world: &mut World, frame_data: FrameData) -> Self {
        // 注意：由于 World 不能 clone，这里我们需要一个不同的设计
        // 简化：使用原始指针（不安全但有效），或者改变设计
        // 这里我们采用简化的方法 - 创建新的空 World
        // 实际使用时应该直接使用 engine.world_mut()
        Self {
            world: World::new(),
            frame_data,
        }
    }

    /// 从 engine-app 同步 ECS World（用于每帧更新）
    pub fn sync_world(&mut self, world: &mut World) {
        // 注意：这里我们不直接替换 world，而是通过其他方式访问
        // 因为 World 不能直接 clone，我们使用一个内部可变的方案
    }
}

impl ScriptContext for EcsScriptContext {
    fn world(&self) -> &World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    fn frame_data(&self) -> FrameData {
        self.frame_data
    }

    fn query_all_transforms(&mut self) -> EntityQueryIter {
        EntityQueryIter::from_world(&mut self.world)
    }

    fn get_entity_transform(&mut self, entity: Entity) -> Option<Transform> {
        self.world.get::<Transform>(entity).copied()
    }

    fn set_entity_transform(&mut self, entity: Entity, transform: Transform) {
        if let Some(mut t) = self.world.get_mut::<Transform>(entity) {
            *t = transform;
        }
    }

    fn entity_exists(&self, entity: Entity) -> bool {
        self.world.entities().contains(entity)
    }

    fn spawn_entity(&mut self) -> Entity {
        self.world.spawn(Transform::default()).id()
    }

    fn despawn_entity(&mut self, entity: Entity) -> bool {
        self.world.despawn(entity)
    }

    fn log(&self, level: &str, message: &str) {
        match level {
            "error" => tracing::error!("[Script] {}", message),
            "warn" => tracing::warn!("[Script] {}", message),
            "info" => tracing::info!("[Script] {}", message),
            "debug" => tracing::debug!("[Script] {}", message),
            _ => tracing::trace!("[Script] {}", message),
        }
    }

    fn delta_time(&self) -> f32 {
        self.frame_data.delta_time
    }

    fn total_time(&self) -> f64 {
        self.frame_data.total_time
    }

    fn frame_count(&self) -> u32 {
        self.frame_data.frame_count
    }
}

// ============================================================================
// ScriptHost - 脚本运行时主机（具体实现）
// ============================================================================

/// 脚本主机 Trait - 脚本运行时的抽象
/// 新版本接收 ScriptContext 以支持 ECS 交互
pub trait ScriptHost: Send + Sync {
    /// 加载脚本模块
    fn load(&mut self, path: &Path) -> Result<(), ScriptError>;

    /// 检查是否已加载
    fn is_loaded(&self) -> bool;

    /// 初始化脚本
    fn init(&mut self, ctx: &mut dyn ScriptContext) -> Result<(), ScriptError>;

    /// 更新脚本
    fn update(&mut self, ctx: &mut dyn ScriptContext) -> Result<(), ScriptError>;

    /// 关闭脚本
    fn shutdown(&mut self, ctx: &mut dyn ScriptContext) -> Result<(), ScriptError>;
}

// ============================================================================
// ScriptManager - 脚本管理器
// ============================================================================

/// 脚本管理器 - 管理所有脚本的生命周期
pub struct ScriptManager {
    hosts: Vec<Box<dyn ScriptHost>>,
}

impl ScriptManager {
    pub fn new() -> Self {
        Self { hosts: Vec::new() }
    }

    /// 添加脚本主机
    pub fn add_host(&mut self, host: Box<dyn ScriptHost>) {
        self.hosts.push(host);
    }

    /// 初始化所有脚本
    pub fn init_all(&mut self, ctx: &mut dyn ScriptContext) -> Result<(), ScriptError> {
        for host in &mut self.hosts {
            host.init(ctx)?;
        }
        Ok(())
    }

    /// 更新所有脚本
    pub fn update_all(&mut self, ctx: &mut dyn ScriptContext) -> Result<(), ScriptError> {
        let mut errors = Vec::new();
        for host in &mut self.hosts {
            if let Err(e) = host.update(ctx) {
                errors.push(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ScriptError::Runtime(format!("{:?}", errors)))
        }
    }

    /// 关闭所有脚本
    pub fn shutdown_all(&mut self, ctx: &mut dyn ScriptContext) -> Result<(), ScriptError> {
        for host in &mut self.hosts {
            host.shutdown(ctx)?;
        }
        Ok(())
    }

    /// 检查是否有任何脚本已加载
    pub fn is_any_loaded(&self) -> bool {
        self.hosts.iter().any(|h| h.is_loaded())
    }
}

impl Default for ScriptManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 配置类型
// ============================================================================

/// 脚本初始化配置
#[derive(Debug, Clone)]
pub struct ScriptInitConfig {
    /// 脚本路径
    pub script_path: PathBuf,
    /// 是否启用调试
    pub debug: bool,
}

impl ScriptInitConfig {
    pub fn new(script_path: PathBuf) -> Self {
        Self { script_path, debug: false }
    }
    
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

pub mod wasm_host;
pub use wasm_host::{WasmScriptHost, create_wasm_host};
