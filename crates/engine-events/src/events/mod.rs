//! 预定义事件模块

mod collision;
mod input;
mod lifecycle;

// 显式重导出
pub use collision::{Collision, CollisionStatus};
pub use input::{Button, ButtonPressed, ButtonReleased, MouseMoved, MouseWheel};
pub use lifecycle::{Spawned, Despawned};
