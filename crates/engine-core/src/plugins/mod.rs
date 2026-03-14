//! ToyEngine 插件系统 - 核心插件抽象
//!
//! 插件是扩展引擎功能的标准化方式

use bevy_ecs::prelude::{Resource, World};
use std::fmt::Debug;

/// 调度器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleType {
    Startup,
    PreUpdate,
    Update,
    PostUpdate,
    FixedUpdate,
    Render,
}

/// 插件 Trait - 引擎核心能力
///
/// 插件是扩展引擎功能的标准化方式：
/// - 注册系统到调度器
/// - 注入资源到 ECS World
/// - 初始化子系统
pub trait Plugin: Send + Sync + Debug {
    /// 插件名称
    fn name(&self) -> &str;

    /// 插件构建回调
    ///
    /// 在此处注册系统、资源、组件
    fn build(&self, context: &mut PluginContext);

    /// 初始化完成回调（可选）
    ///
    /// 在所有插件 build 完成后调用
    fn finish(&self, _context: &mut PluginContext) {}
}

/// 插件构建上下文
///
/// 提供插件与引擎交互的接口
pub struct PluginContext<'a> {
    /// ECS World 引用
    pub world: &'a mut World,
}

impl<'a> PluginContext<'a> {
    /// 创建新的插件上下文
    pub fn new(world: &'a mut World) -> Self {
        Self { world }
    }

    /// 注册资源
    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.world.insert_resource(resource);
    }

    /// 检查资源是否存在
    pub fn contains_resource<R: Resource>(&self) -> bool {
        self.world.contains_resource::<R>()
    }

    /// 获取资源引用
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.world.get_resource::<R>()
    }

    /// 获取资源可变引用
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<bevy_ecs::change_detection::Mut<R>> {
        self.world.get_resource_mut::<R>()
    }
}

/// 插件注册表 - 管理所有插件
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// 添加插件
    pub fn add<P: Plugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    /// 构建所有插件
    ///
    /// 按添加顺序调用每个插件的 build 方法
    pub fn build(&mut self, ctx: &mut PluginContext) {
        for plugin in &self.plugins {
            plugin.build(ctx);
        }
    }

    /// 完成所有插件
    ///
    /// 在所有插件 build 完成后调用
    pub fn finish(&mut self, ctx: &mut PluginContext) {
        for plugin in &self.plugins {
            plugin.finish(ctx);
        }
    }

    /// 获取插件数量
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}
