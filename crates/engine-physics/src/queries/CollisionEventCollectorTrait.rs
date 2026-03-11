//! 碰撞事件 Trait - Collision event collector trait
//!
//! 定义碰撞事件收集器的接口

use rapier3d::prelude::*;
use crate::PhysicsContext;

/// 碰撞事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionEventType {
    Started,
    Stay,
    Ended,
}

/// 碰撞事件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollisionEvent {
    pub event_type: CollisionEventType,
    pub body_handle_a: RigidBodyHandle,
    pub body_handle_b: RigidBodyHandle,
    pub collider_handle_a: ColliderHandle,
    pub collider_handle_b: ColliderHandle,
}

/// 碰撞事件收集器 Trait
pub trait CollisionEventCollectorTrait {
    fn new() -> Self;

    fn collect_events(&mut self, context: &PhysicsContext) -> Vec<CollisionEvent>;

    fn collect_contacts(&self, context: &PhysicsContext) -> Vec<(RigidBodyHandle, RigidBodyHandle)>;
}
