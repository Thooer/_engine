use super::RigidBodyBuilderTrait;
use glam::Vec3;
use rapier3d::prelude::{RigidBodyBuilder, RigidBodySet, RigidBodyType};

struct RapierRigidBodyBuilder {
    position: Vec3,
    velocity: Vec3,
    body_type: RigidBodyType,
}

impl RigidBodyBuilderTrait for RapierRigidBodyBuilder {
    fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            body_type: RigidBodyType::Dynamic,
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

    fn velocity(&mut self, velocity: Vec3) -> &mut Self {
        self.velocity = velocity;
        self
    }

    fn dynamic(&mut self) -> &mut Self {
        self.body_type = RigidBodyType::Dynamic;
        self
    }

    fn fixed(&mut self) -> &mut Self {
        self.body_type = RigidBodyType::Fixed;
        self
    }

    fn build(self, rigid_body_set: &mut RigidBodySet) -> rapier3d::prelude::RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::new(self.body_type)
            .translation(self.position.into())
            .linvel(self.velocity.into())
            .build();
        rigid_body_set.insert(rigid_body)
    }
}
