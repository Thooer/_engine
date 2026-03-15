//! ToyEngine Physics - 统一物理更新辅助模块
//!
//! 提供简单的 API 用于在非 Bevy 调度器的应用中更新物理

use bevy_ecs::prelude::*;
use bevy_ecs::resource::Resource;
use crate::{PhysicsContext, PhysicsContextTrait, PhysicsHandle, RigidBody, Collider, Transform};

/// 物理配置 - 存储物理模拟参数
#[derive(Resource, Clone)]
pub struct PhysicsConfig {
    /// 物理步长（秒）
    pub fixed_dt: f32,
    /// 子步数
    pub substeps: u32,
    /// 是否已初始化
    pub initialized: bool,
}

impl PhysicsConfig {
    /// 创建新的物理配置
    pub fn new(fixed_dt: f32, substeps: u32) -> Self {
        Self {
            fixed_dt,
            substeps,
            initialized: false,
        }
    }

    /// 创建默认配置的物理世界（60Hz，1子步）
    pub fn default_config() -> Self {
        Self::new(1.0 / 60.0, 1)
    }

    /// 创建高频物理世界（180Hz，3子步，适合精确模拟）
    pub fn high_precision() -> Self {
        Self::new(1.0 / 180.0, 3)
    }
}

/// 初始化物理实体
/// 将 ECS 中的 RigidBody + Collider 转换为 Rapier 刚体
pub fn init_bodies(world: &mut World) {
    // 收集需要创建刚体的实体
    let rb_data: Vec<_> = {
        let mut q = world.query::<(Entity, &Transform, &RigidBody)>();
        q.iter(world).map(|(e, t, r)| (e, *t, r.clone())).collect()
    };

    let col_data: Vec<_> = {
        let mut q = world.query::<(Entity, &Collider)>();
        q.iter(world).map(|(e, c)| (e, c.clone())).collect()
    };

    let col_map: std::collections::HashMap<_, _> = col_data.iter()
        .map(|(e, c)| (*e, c))
        .collect();

    // 获取 PhysicsContext
    let mut ctx = match world.get_resource_mut::<PhysicsContext>() {
        Some(c) => c,
        None => return,
    };

    // 创建刚体
    let body_handles: Vec<_> = {
        let rbs = &mut ctx.rigid_body_set;

        rb_data.iter().map(|(entity, transform, rb)| {
            use rapier3d::prelude::*;
            use nalgebra::{UnitQuaternion, Isometry};

            let bt = match rb.body_type {
                crate::RigidBodyType::Dynamic => RigidBodyType::Dynamic,
                crate::RigidBodyType::Fixed => RigidBodyType::Fixed,
                crate::RigidBodyType::KinematicPositionBased => RigidBodyType::KinematicPositionBased,
            };

            let rot = UnitQuaternion::new_unchecked(
                nalgebra::Quaternion::new(transform.rotation.w, transform.rotation.x, transform.rotation.y, transform.rotation.z)
            );
            let pos = Isometry::from_parts(
                nalgebra::Translation3::new(transform.translation.x, transform.translation.y, transform.translation.z),
                rot,
            );

            let mut b = RigidBodyBuilder::new(bt)
                .pose(pos.into())
                .linear_damping(rb.linear_damping)
                .angular_damping(rb.angular_damping)
                .can_sleep(rb.can_sleep)
                .ccd_enabled(rb.ccd_enabled);

            if let Some(m) = rb.additional_mass {
                b = b.additional_mass(m);
            }

            let body = b.build();
            let h = rbs.insert(body);
            (*entity, h)
        }).collect()
    };

    // 需要创建碰撞体的实体（包括 scale 信息）
    let to_create: Vec<_> = {
        let mut q = world.query::<&Transform>();
        body_handles.iter()
            .filter_map(|(e, h)| {
                let transform = q.get(&*world, *e).ok()?;
                col_map.get(e).map(|c| (*e, *h, c, *transform))
            })
            .collect()
    };

    // 创建碰撞体
    let insert_data: Vec<_> = {
        use rapier3d::prelude::*;

        let mut ctx = world.get_resource_mut::<PhysicsContext>().expect("no ctx");
        let mut res = Vec::new();

        for (entity, bh, col, transform) in &to_create {
            // 应用 scale 到碰撞体尺寸
            let s = match col.shape {
                crate::ColliderShape::Ball { radius } => {
                    let scaled_radius = radius * transform.scale.x.max(transform.scale.y).max(transform.scale.z);
                    ColliderBuilder::ball(scaled_radius)
                },
                crate::ColliderShape::Cuboid { half_extents } => {
                    ColliderBuilder::cuboid(
                        half_extents.x * transform.scale.x,
                        half_extents.y * transform.scale.y,
                        half_extents.z * transform.scale.z
                    )
                }
            };

            let b = s.friction(col.friction).restitution(col.restitution).density(col.density).build();

            unsafe {
                let cs = &mut ctx.collider_set as *mut ColliderSet;
                let rbs = &mut ctx.rigid_body_set as *mut RigidBodySet;
                let ch = (*cs).insert_with_parent(b, *bh, &mut *rbs);
                res.push((*entity, *bh, Some(ch)));
            }
        }

        let with_col: Vec<_> = to_create.iter().map(|(e, _, _, _)| *e).collect();
        for (e, h) in &body_handles {
            if !with_col.contains(e) {
                res.push((*e, *h, None));
            }
        }

        res
    };

    // 插入 PhysicsHandle 组件
    for (e, bh, ch) in insert_data {
        world.entity_mut(e).insert(PhysicsHandle {
            rigid_body_handle: bh,
            collider_handle: ch,
        });
    }
}

/// 步进物理模拟
pub fn step(world: &mut World, fixed_dt: f32, substeps: u32) {
    let Some(mut ctx) = world.get_resource_mut::<PhysicsContext>() else {
        return;
    };

    for _ in 0..substeps {
        ctx.step(fixed_dt);
    }
}

/// 同步 Kinematic 刚体位置（ECS → 物理）
/// Kinematic 刚体由脚本控制位置，需要同步到物理引擎
pub fn sync_kinematic_bodies(world: &mut World) {
    // 先收集所有需要同步的数据，避免借用冲突
    let sync_data: Vec<_> = {
        let mut q = world.query::<(&PhysicsHandle, &Transform)>();
        q.iter(world)
            .filter_map(|(handle, transform)| {
                Some((handle.rigid_body_handle, *transform))
            })
            .collect()
    };

    // 然后在独立的块中处理物理同步
    if !sync_data.is_empty() {
        let mut ctx = match world.get_resource_mut::<PhysicsContext>() {
            Some(c) => c,
            None => return,
        };

        let rbs = &mut ctx.rigid_body_set;

        for (handle, transform) in sync_data {
            if let Some(body) = rbs.get_mut(handle) {
                if body.body_type() == rapier3d::prelude::RigidBodyType::KinematicPositionBased {
                    body.set_next_kinematic_translation(transform.translation);
                    body.set_next_kinematic_rotation(transform.rotation);
                }
            }
        }
    }
}

/// 同步物理位置到 ECS Transform（物理 → ECS）
/// 只同步 Dynamic/Fixed 刚体，Kinematic 由 sync_kinematic_bodies 处理
pub fn sync_transforms(world: &mut World) {
    // 这是旧版兼容函数，现在只同步 dynamic/fixed
    sync_dynamic_transforms(world);
}

/// 同步 Dynamic/Fixed 刚体位置到 ECS（物理 → ECS）
pub fn sync_dynamic_transforms(world: &mut World) {
    use glam::Quat;

    let physics_data: Vec<(rapier3d::prelude::RigidBodyHandle, f32, f32, f32, f32, f32, f32, f32)> = {
        let handles: Vec<_> = {
            let mut q = world.query::<(&PhysicsHandle, &Transform)>();
            q.iter(world)
                .map(|(h, _)| h.rigid_body_handle)
                .collect()
        };

        let ctx = match world.get_resource::<PhysicsContext>() {
            Some(c) => c,
            None => return,
        };

        let rbs = &ctx.rigid_body_set;

        let mut data = Vec::new();
        for handle in handles {
            if let Some(body) = rbs.get(handle) {
                // 只同步 Dynamic/Fixed，跳过 Kinematic
                if body.body_type() == rapier3d::prelude::RigidBodyType::KinematicPositionBased {
                    continue;
                }
                let p = body.translation();
                let r = body.rotation();
                data.push((handle, p.x, p.y, p.z, r.x, r.y, r.z, r.w));
            }
        }
        data
    };

    if !physics_data.is_empty() {
        for (handle, px, py, pz, rx, ry, rz, rw) in physics_data {
            let mut q = world.query::<(&PhysicsHandle, &mut Transform)>();
            for (h, mut t) in q.iter_mut(world) {
                if h.rigid_body_handle == handle {
                    t.translation = glam::Vec3::new(px, py, pz);
                    t.rotation = Quat::from_xyzw(rx, ry, rz, rw);
                }
            }
        }
    }
}

/// 完整的物理更新：初始化 + 步进 + 同步
///
/// 这是一个简化的更新函数，适合大多数用例
pub fn update(world: &mut World, _dt: f32) {
    let needs_init = {
        let config = match world.get_resource::<PhysicsConfig>() {
            Some(c) => !c.initialized,
            None => return,
        };
        config
    };

    // 第一次调用时初始化
    if needs_init {
        init_bodies(world);

        // 设置已初始化
        if let Some(mut cfg) = world.get_resource_mut::<PhysicsConfig>() {
            cfg.initialized = true;
        }
    }

    // 获取配置参数
    let (fixed_dt, substeps) = {
        match world.get_resource::<PhysicsConfig>() {
            Some(c) => (c.fixed_dt, c.substeps),
            None => return,
        }
    };

    // 同步 Kinematic 刚体位置（ECS → 物理）必须在物理步进之前
    sync_kinematic_bodies(world);

    // 执行物理步进
    step(world, fixed_dt, substeps);

    // 同步物理位置到 ECS（物理 → ECS）
    sync_dynamic_transforms(world);
}
