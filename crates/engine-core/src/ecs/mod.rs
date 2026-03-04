//! ECS 模块 v0：提供基础组件与 bevy_ecs 预导出
//!
//! 注意：此文件不包含任何 `impl` 相关实现，只定义类型与别名。

// 对外暴露 bevy_ecs 常用类型，方便上层直接使用
pub use bevy_ecs::prelude::*;

use glam::{Quat, Vec3};

/// 基础变换组件
#[derive(Component, Clone, Copy, Debug)]
pub struct Transform {
    /// 位置
    pub translation: Vec3,
    /// 旋转（四元数）
    pub rotation: Quat,
    /// 缩放
    pub scale: Vec3,
}

/// 可渲染标记与颜色数据
#[derive(Component, Clone, Copy, Debug)]
pub struct Renderable {
    /// 线性颜色
    pub color: Vec3,
}

/// 简单二维相机组件（阶段 5 v0）
///
/// 设计目标：
/// - 仅用于 2D / 平面场景的平移缩放
/// - 暂不处理旋转，避免过早复杂化
#[derive(Component, Clone, Copy, Debug)]
pub struct Camera2D {
    /// 相机在世界坐标中的位置
    pub position: Vec3,
    /// 缩放因子：>1 放大，<1 缩小，目前示例仅使用 position
    pub zoom: f32,
}

/// 简单三维相机组件（阶段 5 v0 / cube3d_demo 使用）
///
/// 设计目标：
/// - 提供最小化的三维相机数据：位置 + 朝向
/// - 视图矩阵通过 `Mat4::look_to_rh(position, forward, Vec3::Y)` 计算
#[derive(Component, Clone, Copy, Debug)]
pub struct Camera3D {
    /// 相机在世界坐标中的位置
    pub position: Vec3,
    /// 观察方向（必须为单位向量）
    pub forward: Vec3,
}

#[path = "Default_Camera2D.rs"]
mod default_camera2d;

