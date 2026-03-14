//! ToyEngine ECS 系统调度器
//!
//! 提供轻量级的 ECS 系统调度能力，支持多阶段系统调度
//! - Startup: 应用启动时执行一次，用于初始化
//! - PreUpdate: 每帧更新前执行
//! - Update: 每帧主更新
//! - FixedUpdate: 固定时间步更新（物理）
//! - PostUpdate: 每帧更新后执行
//! - Render: 渲染阶段

use bevy_ecs::prelude::World;
use std::collections::HashMap;

/// 系统调度阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemStage {
    /// 启动阶段 - 在 on_start 时执行一次
    Startup,
    /// 预更新阶段 - 每帧更新前执行
    PreUpdate,
    /// 主更新阶段 - 每帧更新时执行
    Update,
    /// 固定更新阶段 - 物理模拟等固定时间步操作
    FixedUpdate,
    /// 后更新阶段 - 每帧更新后执行
    PostUpdate,
    /// 渲染阶段 - 渲染相关操作
    Render,
}

/// 系统条件函数类型 - 决定系统是否应该运行
pub type SystemCondition = Box<dyn Fn(&World) -> bool + Send + Sync>;

/// 系统函数类型 - 接受可变 World 引用
/// 使用 Box<dyn Fn> 以支持闭包捕获环境变量
pub type SystemFn = Box<dyn FnMut(&mut World) + Send + Sync>;

/// 带条件的系统
pub struct ConditionalSystem {
    /// 系统函数
    pub system: SystemFn,
    /// 运行条件（如果为 None，始终运行）
    pub condition: Option<SystemCondition>,
}

impl ConditionalSystem {
    /// 创建一个新的条件系统
    pub fn new(system: SystemFn) -> Self {
        Self {
            system,
            condition: None,
        }
    }

    /// 创建一个带条件运行指示器的系统
    pub fn with_condition(mut self, condition: SystemCondition) -> Self {
        self.condition = Some(condition);
        self
    }

    /// 运行系统（如果条件满足）
    pub fn run(&mut self, world: &mut World) {
        // 如果有条件，先检查条件
        if let Some(ref cond) = self.condition {
            if !cond(world) {
                return;
            }
        }
        // 运行系统
        (self.system)(world);
    }
}

/// 系统调度器 - 管理应用的系统注册和执行
/// 
/// # 设计理念
/// 
/// 与 Bevy 的复杂调度器不同，这个调度器采用最简单的设计：
/// - 按阶段组织系统：Startup、PreUpdate、Update、FixedUpdate、PostUpdate、Render
/// - 各阶段按注册顺序执行
/// - 支持条件系统（run_if）
/// - 支持向后兼容：add_setup_system / add_update_system 映射到 Startup / Update
pub struct SystemSchedule {
    /// 按阶段组织的系统（支持条件）
    stages: HashMap<SystemStage, Vec<ConditionalSystem>>,
    /// 向后兼容：启动系统
    pub setup_systems: Vec<SystemFn>,
    /// 向后兼容：更新系统
    pub update_systems: Vec<SystemFn>,
}

impl Default for SystemSchedule {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemSchedule {
    /// 创建新的空调度器
    pub fn new() -> Self {
        Self {
            stages: HashMap::new(),
            setup_systems: Vec::new(),
            update_systems: Vec::new(),
        }
    }

    /// 添加系统到指定阶段
    pub fn add_system<F>(&mut self, system: F, stage: SystemStage) -> &mut Self
    where
        F: FnMut(&mut World) + Send + Sync + 'static
    {
        self.stages
            .entry(stage)
            .or_insert_with(Vec::new)
            .push(ConditionalSystem::new(Box::new(system)));
        self
    }

    /// 添加带条件的系统
    /// 
    /// # 示例
    /// ```rust
    /// schedule.add_system_with_condition(
    ///     my_system,
    ///     SystemStage::Update,
    ///     |world| world.get_resource::<SomeFlag>().map(|f| f.enabled).unwrap_or(false)
    /// );
    /// ```
    pub fn add_system_with_condition<F, C>(&mut self, system: F, stage: SystemStage, condition: C) -> &mut Self
    where
        F: FnMut(&mut World) + Send + Sync + 'static,
        C: Fn(&World) -> bool + Send + Sync + 'static,
    {
        self.stages
            .entry(stage)
            .or_insert_with(Vec::new)
            .push(ConditionalSystem::new(Box::new(system)).with_condition(Box::new(condition)));
        self
    }

    /// 添加启动系统（使用闭包）- 向后兼容方法
    /// 映射到 Startup 阶段
    pub fn add_setup_system<F>(&mut self, system: F) -> &mut Self
    where
        F: FnMut(&mut World) + Send + Sync + 'static
    {
        self.setup_systems.push(Box::new(system));
        self
    }

    /// 添加更新系统（使用闭包）- 向后兼容方法
    /// 映射到 Update 阶段
    pub fn add_update_system<F>(&mut self, system: F) -> &mut Self
    where
        F: FnMut(&mut World) + Send + Sync + 'static
    {
        self.update_systems.push(Box::new(system));
        self
    }

    /// 添加预更新系统
    pub fn add_pre_update_system<F>(&mut self, system: F) -> &mut Self
    where
        F: FnMut(&mut World) + Send + Sync + 'static,
    {
        self.add_system(system, SystemStage::PreUpdate)
    }

    /// 添加固定更新系统
    pub fn add_fixed_update_system<F>(&mut self, system: F) -> &mut Self
    where
        F: FnMut(&mut World) + Send + Sync + 'static,
    {
        self.add_system(system, SystemStage::FixedUpdate)
    }

    /// 添加后更新系统
    pub fn add_post_update_system<F>(&mut self, system: F) -> &mut Self
    where
        F: FnMut(&mut World) + Send + Sync + 'static,
    {
        self.add_system(system, SystemStage::PostUpdate)
    }

    /// 添加渲染系统
    pub fn add_render_system<F>(&mut self, system: F) -> &mut Self
    where
        F: FnMut(&mut World) + Send + Sync + 'static,
    {
        self.add_system(system, SystemStage::Render)
    }

    /// 运行指定阶段的所有系统
    fn run_stage(&mut self, world: &mut World, stage: SystemStage) {
        if let Some(systems) = self.stages.get_mut(&stage) {
            for system in systems {
                system.run(world);
            }
        }
    }

    /// 运行所有启动系统（包括向后兼容的 setup_systems）
    pub fn run_setup(&mut self, world: &mut World) {
        // 运行 Startup 阶段
        self.run_stage(world, SystemStage::Startup);
        // 运行向后兼容的 setup_systems
        for system in &mut self.setup_systems {
            system(world);
        }
    }

    /// 运行所有更新系统（包括向后兼容的 update_systems）
    pub fn run_update(&mut self, world: &mut World) {
        // 按顺序执行所有更新阶段
        self.run_stage(world, SystemStage::PreUpdate);
        self.run_stage(world, SystemStage::Update);
        self.run_stage(world, SystemStage::FixedUpdate);
        self.run_stage(world, SystemStage::PostUpdate);
        self.run_stage(world, SystemStage::Render);
        
        // 运行向后兼容的 update_systems
        for system in &mut self.update_systems {
            system(world);
        }
    }
}
