use crate::{PhysicsContext, PhysicsContextTrait};

impl PhysicsContextTrait for PhysicsContext {
    fn new() -> Self {
        PhysicsContext {
            gravity: glam::Vec3::new(0.0, -9.81, 0.0),
            integration_parameters: rapier3d::prelude::IntegrationParameters::default(),
            physics_pipeline: rapier3d::prelude::PhysicsPipeline::new(),
            islands: rapier3d::prelude::IslandManager::new(),
            broad_phase: rapier3d::prelude::BroadPhaseBvh::new(),
            narrow_phase: rapier3d::prelude::NarrowPhase::new(),
            rigid_body_set: rapier3d::prelude::RigidBodySet::new(),
            collider_set: rapier3d::prelude::ColliderSet::new(),
            impulse_joint_set: rapier3d::prelude::ImpulseJointSet::new(),
            multibody_joint_set: rapier3d::prelude::MultibodyJointSet::new(),
            ccd_solver: rapier3d::prelude::CCDSolver::new(),
        }
    }

    fn set_gravity(&mut self, gravity: glam::Vec3) {
        self.gravity = gravity;
    }

    fn step(&mut self, _dt: f32) {
        let gravity: glam::Vec3 = self.gravity;

        self.physics_pipeline.step(
            gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }
}
