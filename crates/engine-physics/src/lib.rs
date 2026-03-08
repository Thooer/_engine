//! ToyEngine Physics - 物理系统模块 v0.1
//!
//! 简单的 Rapier 物理引擎封装

use rapier3d::prelude::*;

/// 物理世界 trait - 封装 Rapier 物理引擎的核心功能
pub trait PhysicsWorldTrait {
    fn new() -> Self;
    fn gravity(&self) -> glam::Vec3;
    fn set_gravity(&mut self, gravity: glam::Vec3);
    fn step(&mut self, dt: f32);
}

#[path = "PhysicsWorldTrait_PhysicsWorld.rs"]
mod physics_world_trait_physics_world;

/// 刚体构建器 trait
pub trait RigidBodyBuilderTrait: Sized {
    fn new() -> Self;
    fn position(&mut self, position: glam::Vec3) -> &mut Self;
    fn rotation(&mut self, rotation: glam::Quat) -> &mut Self;
    fn velocity(&mut self, velocity: glam::Vec3) -> &mut Self;
    fn dynamic(&mut self) -> &mut Self;
    fn fixed(&mut self) -> &mut Self;
    fn build(self, rigid_body_set: &mut RigidBodySet) -> RigidBodyHandle;
}

#[path = "RigidBodyBuilderTrait_RapierRigidBodyBuilder.rs"]
mod rigid_body_builder_trait_rapier_rigid_body_builder;

/// 碰撞器构建器 trait
pub trait ColliderBuilderTrait: Sized {
    fn new() -> Self;
    fn position(&mut self, position: glam::Vec3) -> &mut Self;
    fn rotation(&mut self, rotation: glam::Quat) -> &mut Self;
    fn ball(&mut self, radius: f32) -> &mut Self;
    fn cuboid(&mut self, half_extents: glam::Vec3) -> &mut Self;
    fn build(self, collider_set: &mut ColliderSet, parent: RigidBodyHandle, rigid_body_set: &mut RigidBodySet) -> ColliderHandle;
}

#[path = "ColliderBuilderTrait_RapierColliderBuilder.rs"]
mod collider_builder_trait_rapier_collider_builder;
