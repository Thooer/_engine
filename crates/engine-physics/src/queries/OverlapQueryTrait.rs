//! 物理查询 Trait - Overlap query trait
//!
//! 定义物理查询模块的接口

use rapier3d::prelude::*;
use crate::PhysicsContext;
use glam::Vec3;

/// 形状查询类型
#[derive(Debug, Clone, Copy)]
pub enum ShapeQuery {
    Sphere { radius: f32 },
    Cuboid { half_extents: Vec3 },
    Capsule { half_height: f32, radius: f32 },
}

/// 物理查询模块 Trait
pub trait OverlapQueryTrait {
    fn overlap_sphere(
        &self,
        context: &PhysicsContext,
        center: Vec3,
        radius: f32,
    ) -> Vec<ColliderHandle>;

    fn overlap_cuboid(
        &self,
        context: &PhysicsContext,
        center: Vec3,
        half_extents: Vec3,
    ) -> Vec<ColliderHandle>;

    fn overlap_capsule(
        &self,
        context: &PhysicsContext,
        point1: Vec3,
        point2: Vec3,
        radius: f32,
    ) -> Vec<ColliderHandle>;

    fn overlap_shape(
        &self,
        context: &PhysicsContext,
        center: Vec3,
        rotation: Option<Vec3>,
        query: ShapeQuery,
    ) -> Vec<ColliderHandle>;

    fn point_intersects_shape(
        &self,
        context: &PhysicsContext,
        point: Vec3,
        query: ShapeQuery,
    ) -> bool;
}
