{
    use rapier3d::prelude::*;
    use nalgebra::{UnitQuaternion, Isometry};

    use crate::{RigidBodyType, ColliderShape, PhysicsHandle};

    // First pass: create all rigid bodies and collect collider data
    #[derive(Clone)]
    struct ColliderData {
        entity: bevy_ecs::entity::Entity,
        body_handle: RigidBodyHandle,
        shape: ColliderShape,
        friction: f32,
        restitution: f32,
        density: f32,
        sensor: bool,
    }

    let mut collider_data_list: Vec<ColliderData> = Vec::new();
    let mut body_handles: Vec<(bevy_ecs::entity::Entity, RigidBodyHandle)> = Vec::new();

    // Get rigid body set reference
    let rigid_body_set = &mut physics_context.rigid_body_set;

    for (entity, transform, rb) in rigid_body_query.iter() {
        let body_type = match rb.body_type {
            RigidBodyType::Dynamic => rapier3d::prelude::RigidBodyType::Dynamic,
            RigidBodyType::Fixed => rapier3d::prelude::RigidBodyType::Fixed,
            RigidBodyType::KinematicPositionBased => rapier3d::prelude::RigidBodyType::KinematicPositionBased,
        };

        let rotation = UnitQuaternion::new_unchecked(
            nalgebra::Quaternion::new(transform.rotation.w, transform.rotation.x, transform.rotation.y, transform.rotation.z)
        );
        let position = Isometry::from_parts(
            nalgebra::Translation3::new(transform.translation.x, transform.translation.y, transform.translation.z),
            rotation,
        );

        let mut builder = RigidBodyBuilder::new(body_type)
            .pose(position.into())
            .linear_damping(rb.linear_damping)
            .angular_damping(rb.angular_damping)
            .can_sleep(rb.can_sleep)
            .ccd_enabled(rb.ccd_enabled);

        if let Some(mass) = rb.additional_mass {
            builder = builder.additional_mass(mass);
        }

        let body = builder.build();
        let body_handle = rigid_body_set.insert(body);
        body_handles.push((entity, body_handle));

        // Collect collider data if exists
        if let Ok(collider) = collider_query.get(entity) {
            collider_data_list.push(ColliderData {
                entity,
                body_handle,
                shape: collider.shape.clone(),
                friction: collider.friction,
                restitution: collider.restitution,
                density: collider.density,
                sensor: collider.sensor,
            });
        }
    }

    // Second pass: create colliders
    // Use unsafe to work around Rust's borrow checker limitation with Rapier's API
    for data in &collider_data_list {
        let shape = match data.shape {
            ColliderShape::Ball { radius } => ColliderBuilder::ball(radius),
            ColliderShape::Cuboid { half_extents } => {
                ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            }
        };

        let builder = shape
            .friction(data.friction)
            .restitution(data.restitution)
            .density(data.density)
            .sensor(data.sensor);

        let collider = builder.build();

        // SAFETY: We ensure exclusive access by only creating colliders after all rigid bodies are created
        // and we don't access rigid_body_set again after this point
        let collider_handle = unsafe {
            let collider_set_ptr = &mut physics_context.collider_set as *mut ColliderSet;
            let rigid_body_set_ptr = &mut physics_context.rigid_body_set as *mut RigidBodySet;
            (*collider_set_ptr).insert_with_parent(collider, data.body_handle, &mut *rigid_body_set_ptr)
        };

        commands.entity(data.entity).insert(PhysicsHandle {
            rigid_body_handle: data.body_handle,
            collider_handle: Some(collider_handle),
        });
    }

    // Handle entities without colliders
    let entities_with_colliders: Vec<_> = collider_data_list.iter().map(|d| d.entity).collect();
    for (entity, body_handle) in body_handles {
        if !entities_with_colliders.contains(&entity) {
            commands.entity(entity).insert(PhysicsHandle {
                rigid_body_handle: body_handle,
                collider_handle: None,
            });
        }
    }
}
