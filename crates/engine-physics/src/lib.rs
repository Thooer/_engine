//! ToyEngine Physics - 物理系统模块
//!
//! ECS 风格的 Rapier 封装，提供高性能物理模拟

use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use engine_core::ecs::Transform;
use engine_core::input::{InputState, InputStateExt};
use rapier3d::prelude::*;
use glam::Vec3;

/// 统一物理更新辅助模块
pub mod physics_world;

// ============================================================================
// 物理组件 (Physics Components)
// ============================================================================

/// 刚体类型
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum RigidBodyType {
    Dynamic,
    Fixed,
    KinematicPositionBased,
}

impl Default for RigidBodyType {
    fn default() -> Self {
        Self::Dynamic
    }
}

/// 刚体组件 - 与 Rapier 物理引擎集成
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub additional_mass: Option<f32>,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub can_sleep: bool,
    pub ccd_enabled: bool,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            additional_mass: None,
            linear_damping: 0.0,
            angular_damping: 0.0,
            can_sleep: true,
            ccd_enabled: false,
        }
    }
}

/// 碰撞体类型
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub enum ColliderShape {
    Ball { radius: f32 },
    Cuboid { half_extents: Vec3 },
}

impl Default for ColliderShape {
    fn default() -> Self {
        Self::Ball { radius: 0.5 }
    }
}

/// 碰撞体组件
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub sensor: bool,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::default(),
            friction: 0.5,
            restitution: 0.3,
            density: 1.0,
            sensor: false,
        }
    }
}

/// 速度组件
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Velocity {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

/// 外力组件
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ExternalForce {
    pub force: Vec3,
    pub torque: Vec3,
}

/// 内部组件 - 存储 Rapier 句柄（不需要反射）
#[derive(Component, Clone, Copy, Debug)]
pub struct PhysicsHandle {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: Option<ColliderHandle>,
}

// ============================================================================
// 物理资源 (Physics Resources)
// ============================================================================

/// 物理上下文 - 封装 Rapier 核心数据结构
#[derive(Resource)]
pub struct PhysicsContext {
    pub gravity: Vec3,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub islands: IslandManager,
    pub broad_phase: BroadPhaseBvh,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
}

pub trait PhysicsContextTrait {
    fn new() -> Self;
    fn set_gravity(&mut self, gravity: Vec3);
    fn step(&mut self, dt: f32);
}

#[path = "PhysicsContextTrait_PhysicsContext.rs"]
mod physics_context_trait;

#[path = "Default_PhysicsContext.rs"]
mod default_physics_context;

// ============================================================================
// 物理系统 (Physics Systems)
// ============================================================================

/// 应用外力系统 - 将 ECS 中的外力同步到 Rapier 刚体
pub fn apply_external_forces_system(
    mut physics_context: ResMut<PhysicsContext>,
    force_query: Query<(&PhysicsHandle, &ExternalForce)>,
) {
    include!("internal/apply_external_forces_system.rs");
}

/// 初始化物理刚体系统 - 将 ECS 实体转换为 Rapier 刚体
pub fn init_physics_bodies_system(
    mut commands: Commands,
    mut physics_context: ResMut<PhysicsContext>,
    rigid_body_query: Query<(Entity, &Transform, &RigidBody), Without<PhysicsHandle>>,
    collider_query: Query<&Collider, With<RigidBody>>,
) {
    include!("internal/init_physics_bodies_system.rs");
}

/// 同步变换到物理系统 - 将 ECS Transform 变化同步到 Rapier
pub fn sync_transform_to_physics_system(
    mut physics_context: ResMut<PhysicsContext>,
    mut transform_query: Query<(&Transform, &PhysicsHandle), Changed<Transform>>,
) {
    include!("internal/sync_transform_to_physics_system.rs");
}

/// 步进物理系统 - 执行一次物理模拟
pub fn step_physics_system(
    mut physics_context: ResMut<PhysicsContext>,
) {
    include!("internal/step_physics_system.rs");
}

/// 同步物理到变换系统 - 将 Rapier 刚体位置同步到 ECS Transform
pub fn sync_physics_to_transform_system(
    physics_context: Res<PhysicsContext>,
    mut transform_query: Query<(&PhysicsHandle, &mut Transform)>,
) {
    include!("internal/sync_physics_to_transform_system.rs");
}

// ============================================================================
// 输入响应组件 - 当物体掉落时自动重置
// ============================================================================

use winit::keyboard::KeyCode;

/// 标记实体在按键触发时重置
/// 
/// 配合 `reset_on_keypress_system` 使用
/// 当指定按键被按下时，重置所有标记的实体
#[derive(Debug, Clone, Copy, Component)]
pub struct ResetOnKeyPress {
    /// 触发重置的按键
    pub key: KeyCode,
    /// 重置目标位置
    pub reset_position: glam::Vec3,
}

impl ResetOnKeyPress {
    pub fn new(key: KeyCode, reset_position: glam::Vec3) -> Self {
        Self { key, reset_position }
    }
    
    pub fn space(reset_position: glam::Vec3) -> Self {
        Self { 
            key: KeyCode::Space, 
            reset_position 
        }
    }
}

impl Default for ResetOnKeyPress {
    fn default() -> Self {
        Self {
            key: KeyCode::Space,
            reset_position: glam::Vec3::new(0.0, 5.0, 0.0),
        }
    }
}

/// 按键触发重置系统
/// 
/// # 使用方式
/// ```rust
/// // 1. 给需要重置的实体添加 ResetOnKeyPress 组件
/// commands.entity(entity).insert(ResetOnKeyPress::default());
/// 
/// // 2. 在系统中注册
/// schedule.add_system(reset_on_keypress_system, SystemStage::Update);
/// ```
pub fn reset_on_keypress_system(world: &mut World) {
    // 获取输入状态
    let Some(input) = world.get_resource::<InputState>() else {
        return;
    };
    
    // 检查是否按下了空格键
    if !input.just_pressed(KeyCode::Space) {
        return;
    }
    
    // 收集需要重置的 handle
    let handles: Vec<_> = {
        let mut query = world.query::<&PhysicsHandle>();
        query.iter(world).map(|h| h.rigid_body_handle).collect()
    };
    
    if handles.is_empty() {
        return;
    }
    
    // 重置 Rapier 物理状态
    if let Some(mut ctx) = world.get_resource_mut::<PhysicsContext>() {
        for &h in &handles {
            if let Some(body) = ctx.rigid_body_set.get_mut(h) {
                body.set_translation(glam::Vec3::new(0.0, 5.0, 0.0), true);
                body.set_linvel(glam::Vec3::ZERO, true);
                body.set_angvel(glam::Vec3::ZERO, true);
            }
        }
    }
    
    // 重置 ECS Transform (只重置掉落到 threshold 以下的)
    let threshold = -1.0;
    let mut query = world.query::<(&PhysicsHandle, &mut Transform)>();
    for (handle, mut transform) in query.iter_mut(world) {
        if handles.contains(&handle.rigid_body_handle) && transform.translation.y < threshold {
            transform.translation = glam::Vec3::new(0.0, 5.0, 0.0);
        }
    }
}

// ============================================================================
// 物理扩展功能 - 射线检测、碰撞事件、区域查询
// ============================================================================

mod queries;

/// 测试模块
#[cfg(test)]
mod tests;
