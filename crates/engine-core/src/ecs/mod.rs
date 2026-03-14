//! ECS 模块 v0：提供基础组件与 bevy_ecs 预导出
//!
//! 注意：此文件不包含任何 `impl` 相关实现，只定义类型与别名。

// 注意：禁止使用通配符重导出，需要使用 bevy_ecs 的类型时请直接使用
// use bevy_ecs::prelude::*;

// 如果需要使用 bevy_ecs 的类型，在使用的地方直接引入
// 示例：use bevy_ecs::prelude::{Entity, World, SystemState};

pub use bevy_ecs::prelude::{Component, World, ReflectComponent};
use bevy_reflect::Reflect;
use glam::{Quat, Vec3};

/// 基础变换组件
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct Transform {
    /// 位置
    pub translation: Vec3,
    /// 旋转（四元数）
    pub rotation: Quat,
    /// 缩放
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// 可渲染标记与颜色数据
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct Renderable {
    /// 线性颜色
    pub color: Vec3,
}

/// 简单二维相机组件（阶段 5 v0）
///
/// 设计目标：
/// - 仅用于 2D / 平面场景的平移缩放
/// - 暂不处理旋转，避免过早复杂化
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct Camera2D {
    /// 相机在世界坐标中的位置
    pub position: Vec3,
    /// 缩放因子：>1 放大，<1 缩小，目前示例仅使用 position
    pub zoom: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 10.0),
            zoom: 1.0,
        }
    }
}

/// 简单三维相机组件（阶段 5 v0 / cube3d_demo 使用）
///
/// 设计目标：
/// - 提供最小化的三维相机数据：位置 + 朝向
/// - 视图矩阵通过 `Mat4::look_to_rh(position, forward, Vec3::Y)` 计算
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct Camera3D {
    /// 相机在世界坐标中的位置
    pub position: Vec3,
    /// 观察方向（必须为单位向量）
    pub forward: Vec3,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            forward: Vec3::new(0.0, -0.5, -1.0).normalize(),
        }
    }
}

/// 物理查询结果 - 用于射线检测等
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct PhysicsQuery;

