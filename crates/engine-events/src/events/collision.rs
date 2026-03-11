//! 碰撞事件定义

use bevy_ecs::prelude::Entity;

use crate::Event;

/// 碰撞状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionStatus {
    Started,
    Stay,
    Ended,
}

/// 碰撞事件
#[derive(Debug, Clone)]
pub struct Collision {
    /// 实体 A
    pub entity_a: Entity,
    /// 实体 B
    pub entity_b: Entity,
    /// 碰撞状态
    pub status: CollisionStatus,
}
