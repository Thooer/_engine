//! 射线检测 Trait - Raycast module trait
//!
//! 定义射线检测模块的接口

use rapier3d::prelude::*;
use crate::PhysicsContext;
use glam::Vec3;

/// 射线检测结果
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RaycastHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub distance: f32,
}

/// 射线检测模块 Trait
#[allow(dead_code)]
pub trait RaycastModuleTrait {
    fn raycast(
        &self,
        context: &PhysicsContext,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<RaycastHit>;

    fn raycast_simple(
        &self,
        context: &PhysicsContext,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(ColliderHandle, f32)>;

    fn raycast_all(
        &self,
        context: &PhysicsContext,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Vec<RaycastHit>;
}
