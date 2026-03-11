//! ToyEngine Physics - 物理系统模块
//!
//! ECS 风格的 Rapier 封装，提供高性能物理模拟

use bevy_ecs::prelude::*;
use engine_core::ecs::*;
use rapier3d::prelude::*;
use glam::Vec3;

// ============================================================================
// 物理组件 (Physics Components)
// ============================================================================

/// 刚体类型
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RigidBodyType {
    Dynamic,
    Fixed,
    KinematicPositionBased,
}

/// 刚体组件 - 与 Rapier 物理引擎集成
#[derive(Component, Clone, Debug)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub additional_mass: Option<f32>,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub can_sleep: bool,
    pub ccd_enabled: bool,
}

/// 碰撞体类型
#[derive(Component, Clone, Debug)]
pub enum ColliderShape {
    Ball { radius: f32 },
    Cuboid { half_extents: Vec3 },
}

/// 碰撞体组件
#[derive(Component, Clone, Debug)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub sensor: bool,
}

/// 速度组件
#[derive(Component, Clone, Debug, Default)]
pub struct Velocity {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

/// 外力组件
#[derive(Component, Clone, Debug, Default)]
pub struct ExternalForce {
    pub force: Vec3,
    pub torque: Vec3,
}

/// 内部组件 - 存储 Rapier 句柄
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
// 物理扩展功能 - 射线检测、碰撞事件、区域查询
// ============================================================================

mod queries;

/// 测试模块
#[cfg(test)]
mod tests;
