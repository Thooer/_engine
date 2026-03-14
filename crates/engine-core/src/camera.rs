//! Camera helpers (v0)
//!
//! 提供一些与相机相关的通用系统 / 工具函数，避免在示例中重复实现。

use glam::Vec3;

use crate::ecs::{Camera3D, World};
use crate::input::{InputCode, InputState, InputStateExt};

/// 简单 3D 自由飞行相机控制（WASD + Space/Ctrl）。
///
/// - W / S: 沿 -Z / +Z 方向移动
/// - A / D: 沿 -X / +X 方向移动
/// - Space / Ctrl: 沿 +Y / -Y 方向移动
///
/// 该函数假定世界中最多只有一个 `Camera3D`，并将移动应用到该相机。
/// 
/// 注意：现在从 World 中获取 InputState Resource，_input 参数保留用于兼容但已废弃
#[allow(unused_variables)]
#[allow(dead_code)]
pub fn camera3d_fly_wasd(world: &mut World, _input: &InputState, dt: f32, speed: f32) {
    // 从 ECS World 获取 InputState
    let input = match world.get_resource::<InputState>() {
        Some(i) => i,
        None => return,
    };

    let mut dir = Vec3::ZERO;

    if input.is_pressed(InputCode::KeyW) {
        dir.z -= 1.0;
    }
    if input.is_pressed(InputCode::KeyS) {
        dir.z += 1.0;
    }
    if input.is_pressed(InputCode::KeyA) {
        dir.x -= 1.0;
    }
    if input.is_pressed(InputCode::KeyD) {
        dir.x += 1.0;
    }
    if input.is_pressed(InputCode::KeySpace) {
        dir.y += 1.0;
    }
    if input.is_pressed(InputCode::ControlLeft) || input.is_pressed(InputCode::ControlRight) {
        dir.y -= 1.0;
    }

    if dir.length_squared() == 0.0 {
        return;
    }

    let dir = dir.normalize() * speed * dt;

    if let Ok(mut cam) = world.query::<&mut Camera3D>().single_mut(world) {
        cam.position += dir;
    }
}

