//! 物理查询模块 - Physics queries module
//!
//! 提供物理世界的高级查询功能：
//! - 射线检测 (Raycast)
//! - 碰撞事件 (Collision Events)
//! - 区域查询 (Overlap Queries)

#[allow(non_snake_case)]
mod RaycastModuleTrait;
#[allow(non_snake_case)]
mod RaycastModuleTrait_Default;

#[allow(non_snake_case)]
mod OverlapQueryTrait;
#[allow(non_snake_case)]
mod OverlapQueryTrait_Default;

#[allow(non_snake_case)]
mod CollisionEventCollectorTrait;
#[allow(non_snake_case)]
mod CollisionEventCollectorTrait_Default;

// 注意：禁止使用通配符重导出，需要使用 trait 方法时请直接引入对应的 trait
