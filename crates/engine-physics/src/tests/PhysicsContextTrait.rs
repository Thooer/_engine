use crate::{PhysicsContext, PhysicsContextTrait};

#[cfg(test)]
mod tests {
    use super::*;
    use rapier3d::prelude::*;

    #[test]
    fn test_physics_context_new() {
        let ctx = PhysicsContext::new();
        assert_eq!(ctx.gravity, glam::Vec3::new(0.0, -9.81, 0.0));
        assert_eq!(ctx.rigid_body_set.len(), 0);
        assert_eq!(ctx.collider_set.len(), 0);
    }

    #[test]
    fn test_physics_context_set_gravity() {
        let mut ctx = PhysicsContext::new();
        ctx.set_gravity(glam::Vec3::new(0.0, -1.0, 0.0));
        assert_eq!(ctx.gravity, glam::Vec3::new(0.0, -1.0, 0.0));
    }

    #[test]
    fn test_physics_context_step_no_panic() {
        let mut ctx = PhysicsContext::new();
        ctx.step(1.0 / 60.0);
    }

    #[test]
    fn test_physics_context_rigid_body_creation() {
        let mut ctx = PhysicsContext::new();
        
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(Isometry::translation(0.0, 5.0, 0.0))
            .build();
        let handle = ctx.rigid_body_set.insert(rigid_body);
        
        let collider = ColliderBuilder::ball(0.5)
            .build();
        ctx.collider_set.insert_with_parent(collider, handle, &mut ctx.rigid_body_set);
        
        assert_eq!(ctx.rigid_body_set.len(), 1);
        assert_eq!(ctx.collider_set.len(), 1);
        
        ctx.step(1.0 / 60.0);
        
        let body = ctx.rigid_body_set.get(handle).unwrap();
        assert!(body.translation().y < 5.0);
    }

    #[test]
    fn test_physics_context_multiple_bodies() {
        let mut ctx = PhysicsContext::new();
        
        for i in 0..5 {
            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(Isometry::translation(i as f32 * 1.0, 10.0, 0.0))
                .build();
            let handle = ctx.rigid_body_set.insert(rigid_body);
            
            let collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5)
                .build();
            ctx.collider_set.insert_with_parent(collider, handle, &mut ctx.rigid_body_set);
        }
        
        assert_eq!(ctx.rigid_body_set.len(), 5);
        
        ctx.step(1.0);
        
        let mut falling_count = 0;
        for (_handle, body) in ctx.rigid_body_set.iter() {
            if body.translation().y < 10.0 {
                falling_count += 1;
            }
        }
        assert_eq!(falling_count, 5);
    }

    #[test]
    fn test_physics_context_fixed_body() {
        let mut ctx = PhysicsContext::new();
        
        let ground = RigidBodyBuilder::fixed()
            .translation(Isometry::translation(0.0, 0.0, 0.0))
            .build();
        let ground_handle = ctx.rigid_body_set.insert(ground);
        
        let ground_collider = ColliderBuilder::cuboid(10.0, 0.1, 10.0)
            .build();
        ctx.collider_set.insert_with_parent(ground_collider, ground_handle, &mut ctx.rigid_body_set);
        
        let dynamic_body = RigidBodyBuilder::dynamic()
            .translation(Isometry::translation(0.0, 5.0, 0.0))
            .build();
        let handle = ctx.rigid_body_set.insert(dynamic_body);
        
        let collider = ColliderBuilder::ball(0.5)
            .restitution(0.8)
            .build();
        ctx.collider_set.insert_with_parent(collider, handle, &mut ctx.rigid_body_set);
        
        for _ in 0..120 {
            ctx.step(1.0 / 60.0);
        }
        
        let body = ctx.rigid_body_set.get(handle).unwrap();
        assert!(body.translation().y >= 0.4 && body.translation().y <= 1.0);
    }
}
