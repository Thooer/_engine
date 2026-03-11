//! 物理查询模块 - Physics queries module
//!
//! 提供物理世界的高级查询功能：
//! - 射线检测 (Raycast)
//! - 碰撞事件 (Collision Events)
//! - 区域查询 (Overlap Queries)

mod RaycastModuleTrait;
mod RaycastModuleTrait_Default;

mod OverlapQueryTrait;
mod OverlapQueryTrait_Default;

mod CollisionEventCollectorTrait;
mod CollisionEventCollectorTrait_Default;

// 注意：禁止使用通配符重导出，需要使用 trait 方法时请直接引入对应的 trait
