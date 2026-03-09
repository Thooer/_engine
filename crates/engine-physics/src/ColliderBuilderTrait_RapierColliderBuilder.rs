use super::ColliderBuilderTrait;
use glam::Vec3;
use rapier3d::prelude::{ColliderBuilder, ColliderHandle, ColliderSet, RigidBodyHandle, RigidBodySet};

struct RapierColliderBuilder {
    position: Vec3,
    shape_type: ColliderShapeType,
    ball_radius: f32,
    cuboid_half_extents: Vec3,
}

enum ColliderShapeType {
    Ball,
    Cuboid,
}

impl ColliderBuilderTrait for RapierColliderBuilder {
    fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            shape_type: ColliderShapeType::Ball,
            ball_radius: 0.5,
            cuboid_half_extents: Vec3::new(0.5, 0.5, 0.5),
        }
    }

    fn position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;
        self
    }

    fn rotation(&mut self, _rotation: glam::Quat) -> &mut Self {
        // TODO: 实现 rotation 支持
        self
    }

    fn ball(&mut self, radius: f32) -> &mut Self {
        self.shape_type = ColliderShapeType::Ball;
        self.ball_radius = radius;
        self
    }

    fn cuboid(&mut self, half_extents: Vec3) -> &mut Self {
        self.shape_type = ColliderShapeType::Cuboid;
        self.cuboid_half_extents = half_extents;
        self
    }

    fn build(self, collider_set: &mut ColliderSet, parent: RigidBodyHandle, rigid_body_set: &mut RigidBodySet) -> ColliderHandle {
        let collider = match self.shape_type {
            ColliderShapeType::Ball => {
                ColliderBuilder::ball(self.ball_radius)
            }
            ColliderShapeType::Cuboid => {
                ColliderBuilder::cuboid(
                    self.cuboid_half_extents.x,
                    self.cuboid_half_extents.y,
                    self.cuboid_half_extents.z,
                )
            }
        }
        .translation(self.position.into())
        .build();
        collider_set.insert_with_parent(collider, parent, rigid_body_set)
    }
}
