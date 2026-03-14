//! 默认射线检测实现 - Default raycast implementation

use rapier3d::prelude::*;
use crate::PhysicsContext;
use crate::queries::RaycastModuleTrait::RaycastModuleTrait;
use crate::queries::RaycastModuleTrait::RaycastHit;
use glam::Vec3;

#[allow(dead_code)]
struct DefaultRaycastModule;

impl RaycastModuleTrait for DefaultRaycastModule {
    fn raycast(
        &self,
        context: &PhysicsContext,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<RaycastHit> {
        let query_pipeline = context.broad_phase.as_query_pipeline(
            context.narrow_phase.query_dispatcher(),
            &context.rigid_body_set,
            &context.collider_set,
            QueryFilter::default(),
        );
        
        let dir = direction.normalize();
        let ray = Ray::new(origin.into(), dir.into());
        
        let hit = query_pipeline.cast_ray_and_get_normal(
            &ray,
            max_distance,
            true,
        );
        
        if let Some((collider_handle, intersection)) = hit {
            let collider = context.collider_set.get(collider_handle)?;
            let rigid_body_handle = collider.parent()?;
            
            let hit_point = ray.origin + ray.dir * intersection.time_of_impact;
            
            Some(RaycastHit {
                point: Vec3::from(hit_point),
                normal: Vec3::from(intersection.normal),
                rigid_body_handle,
                collider_handle,
                distance: intersection.time_of_impact,
            })
        } else {
            None
        }
    }

    fn raycast_simple(
        &self,
        context: &PhysicsContext,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(ColliderHandle, f32)> {
        let query_pipeline = context.broad_phase.as_query_pipeline(
            context.narrow_phase.query_dispatcher(),
            &context.rigid_body_set,
            &context.collider_set,
            QueryFilter::default(),
        );
        
        let dir = direction.normalize();
        let ray = Ray::new(origin.into(), dir.into());
        
        query_pipeline.cast_ray(&ray, max_distance, true)
    }

    fn raycast_all(
        &self,
        context: &PhysicsContext,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Vec<RaycastHit> {
        let query_pipeline = context.broad_phase.as_query_pipeline(
            context.narrow_phase.query_dispatcher(),
            &context.rigid_body_set,
            &context.collider_set,
            QueryFilter::default(),
        );
        
        let dir = direction.normalize();
        let ray = Ray::new(origin.into(), dir.into());
        
        let mut hits = Vec::new();
        
        for (collider_handle, _collider, intersection) in query_pipeline.intersect_ray(ray, max_distance, true) {
            if let Some(collider) = context.collider_set.get(collider_handle) {
                if let Some(rigid_body_handle) = collider.parent() {
                    let hit_point = origin + direction * intersection.time_of_impact;
                    hits.push(RaycastHit {
                        point: hit_point,
                        normal: Vec3::from(intersection.normal),
                        rigid_body_handle,
                        collider_handle,
                        distance: intersection.time_of_impact,
                    });
                }
            }
        }
        
        hits.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        hits
    }
}
