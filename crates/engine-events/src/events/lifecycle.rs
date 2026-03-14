//! 生命周期事件定义

use bevy_ecs::prelude::Entity;
use bevy_ecs::prelude::Event;

/// 实体创建事件
#[derive(Debug, Clone, Event)]
pub struct Spawned {
    pub entity: Entity,
}

/// 实体销毁事件
#[derive(Debug, Clone, Event)]
pub struct Despawned {
    pub entity: Entity,
}
