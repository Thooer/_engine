//! ToyEngine 事件系统
//!
//! 提供统一的事件发布-订阅机制

use std::any::{Any, TypeId};
use std::collections::HashMap;

// 使用 Bevy 的 Event trait 作为基础
pub use bevy_ecs::event::Event;

/// 事件存储容器
pub trait EventContainer: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[path = "EventContainer_TypedEventContainer.rs"]
mod event_container_typed_event_container;

/// 泛型事件容器
pub struct TypedEventContainer<T: Event> {
    pub events: Vec<T>,
}

/// 泛型事件容器 Trait
pub trait TypedEventContainerTrait<T: Event> {
    fn new() -> Self;
}

#[path = "TypedEventContainerTrait_TypedEventContainer.rs"]
mod typed_event_container_trait_typed_event_container;

/// 事件世界 trait
pub trait EventWorld {
    fn send<T: Event + Clone>(&mut self, event: T);
    fn read<T: Event + Clone>(&self) -> Vec<T>;
    fn clear<T: Event>(&mut self);
}

#[path = "EventWorld_DefaultEventWorld.rs"]
mod event_world_default_event_world;

/// DefaultEventWorld Trait
pub trait DefaultEventWorldTrait {
    fn new() -> Self;
}

#[path = "DefaultEventWorldTrait_DefaultEventWorld.rs"]
mod default_event_world_trait_default_event_world;

/// Internal helper trait for event sending
pub trait EventSender {
    fn send_event<T: Event + Clone>(&mut self, event: T);
}

#[path = "EventSender_DefaultEventWorld.rs"]
mod event_sender_default_event_world;

/// 默认事件世界实现
pub struct DefaultEventWorld {
    pub containers: HashMap<TypeId, Box<dyn EventContainer>>,
}

#[path = "Default_DefaultEventWorld.rs"]
mod default_default_event_world;

/// 预定义事件
mod events;

// 显式重导出事件类型
pub use events::{
    Collision, CollisionStatus,
    Button, ButtonPressed, ButtonReleased, MouseMoved, MouseWheel,
    Spawned, Despawned,
};
