//! 默认物理查询实现 - Default overlap query implementation

use rapier3d::prelude::*;
use crate::PhysicsContext;
use crate::queries::OverlapQueryTrait::OverlapQueryTrait;
use crate::queries::OverlapQueryTrait::ShapeQuery;
use nalgebra::Isometry3;
use glam::Vec3;

struct DefaultOverlapQuery;

#[allow(dead_code)]
impl OverlapQueryTrait for DefaultOverlapQuery {
    fn overlap_sphere(
        &self,
        context: &PhysicsContext,
        center: Vec3,
        radius: f32,
    ) -> Vec<ColliderHandle> {
        let query_pipeline = context.broad_phase.as_query_pipeline(
            context.narrow_phase.query_dispatcher(),
            &context.rigid_body_set,
            &context.collider_set,
            QueryFilter::default(),
        );
        
        let shape = Ball::new(radius);
        let pose = Isometry3::translation(center.x, center.y, center.z);
        
        let mut handles = Vec::new();
        
        for (handle, _collider) in query_pipeline.intersect_shape(pose.into(), &shape) {
            handles.push(handle);
        }
        
        handles
    }

    fn overlap_cuboid(
        &self,
        context: &PhysicsContext,
        center: Vec3,
        half_extents: Vec3,
    ) -> Vec<ColliderHandle> {
        let query_pipeline = context.broad_phase.as_query_pipeline(
            context.narrow_phase.query_dispatcher(),
            &context.rigid_body_set,
            &context.collider_set,
            QueryFilter::default(),
        );
        
        let shape = Cuboid::new(half_extents.into());
        let pose = Isometry3::translation(center.x, center.y, center.z);
        
        let mut handles = Vec::new();
        
        for (handle, _collider) in query_pipeline.intersect_shape(pose.into(), &shape) {
            handles.push(handle);
        }
        
        handles
    }

    fn overlap_capsule(
        &self,
        context: &PhysicsContext,
        point1: Vec3,
        point2: Vec3,
        radius: f32,
    ) -> Vec<ColliderHandle> {
        let query_pipeline = context.broad_phase.as_query_pipeline(
            context.narrow_phase.query_dispatcher(),
            &context.rigid_body_set,
            &context.collider_set,
            QueryFilter::default(),
        );
        
        let shape = Capsule::new(point1.into(), point2.into(), radius);
        
        let mut handles = Vec::new();
        
        for (handle, _collider) in query_pipeline.intersect_shape(Isometry3::identity().into(), &shape) {
            handles.push(handle);
        }
        
        handles
    }

    fn overlap_shape(
        &self,
        context: &PhysicsContext,
        center: Vec3,
        _rotation: Option<Vec3>,
        query: ShapeQuery,
    ) -> Vec<ColliderHandle> {
        match query {
            ShapeQuery::Sphere { radius } => self.overlap_sphere(context, center, radius),
            ShapeQuery::Cuboid { half_extents } => self.overlap_cuboid(context, center, half_extents),
            ShapeQuery::Capsule { half_height, radius } => {
                let half_vec = Vec3::new(0.0, half_height, 0.0);
                self.overlap_capsule(context, center - half_vec, center + half_vec, radius)
            }
        }
    }

    fn point_intersects_shape(
        &self,
        context: &PhysicsContext,
        point: Vec3,
        query: ShapeQuery,
    ) -> bool {
        match query {
            ShapeQuery::Sphere { radius } => {
                let query_pipeline = context.broad_phase.as_query_pipeline(
                    context.narrow_phase.query_dispatcher(),
                    &context.rigid_body_set,
                    &context.collider_set,
                    QueryFilter::default(),
                );
                
                let shape = Ball::new(radius);
                let pose = Isometry3::translation(point.x, point.y, point.z);
                
                let result = query_pipeline.intersect_shape(pose.into(), &shape).next().is_some();
                result
            }
            ShapeQuery::Cuboid { half_extents } => {
                let query_pipeline = context.broad_phase.as_query_pipeline(
                    context.narrow_phase.query_dispatcher(),
                    &context.rigid_body_set,
                    &context.collider_set,
                    QueryFilter::default(),
                );
                
                let shape = Cuboid::new(half_extents.into());
                let pose = Isometry3::translation(point.x, point.y, point.z);
                
                let result = query_pipeline.intersect_shape(pose.into(), &shape).next().is_some();
                result
            }
            ShapeQuery::Capsule { .. } => false,
        }
    }
}
