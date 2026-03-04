//! Camera helpers (v0)
//!
//! 提供一些与相机相关的通用系统 / 工具函数，避免在示例中重复实现。

use glam::Vec3;
use winit::keyboard::KeyCode;

use crate::ecs::{Camera3D, World};
use crate::input::{InputState, InputStateExt};

/// 简单 3D 自由飞行相机控制（WASD + Space/Ctrl）。
///
/// - W / S: 沿 -Z / +Z 方向移动
/// - A / D: 沿 -X / +X 方向移动
/// - Space / Ctrl: 沿 +Y / -Y 方向移动
///
/// 该函数假定世界中最多只有一个 `Camera3D`，并将移动应用到该相机。
pub fn camera3d_fly_wasd(world: &mut World, input: &InputState, dt: f32, speed: f32) {
    let mut dir = Vec3::ZERO;

    if input.is_pressed(KeyCode::KeyW) {
        dir.z -= 1.0;
    }
    if input.is_pressed(KeyCode::KeyS) {
        dir.z += 1.0;
    }
    if input.is_pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if input.is_pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    if input.is_pressed(KeyCode::Space) {
        dir.y += 1.0;
    }
    if input.is_pressed(KeyCode::ControlLeft) || input.is_pressed(KeyCode::ControlRight) {
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

