//! 相机系统模块
//!
//! 提供相机相关的 ECS 系统

use bevy_ecs::prelude::*;
use glam::Vec3;

use engine_core::ecs::Camera3D;
use engine_core::FrameCounter;
use engine_core::ecs::CameraController;

/// 轨道相机系统
/// 
/// 自动更新带有 CameraController 组件的相机位置
/// 从 FrameCounter 资源获取帧号
pub fn orbit_camera_system(world: &mut World) {
    let frame_count = {
        let counter = world.get_resource::<FrameCounter>();
        match counter {
            Some(c) => c.0,
            None => return,
        }
    };
    
    let mut query = world.query::<(&CameraController, &mut Camera3D)>();
    
    for (controller, mut camera) in query.iter_mut(world) {
        let theta = (frame_count as f32 * controller.orbit_speed) + controller.phase_offset;
        
        camera.position = Vec3::new(
            theta.cos() * controller.orbit_radius,
            controller.height,
            theta.sin() * controller.orbit_radius,
        );
        
        // 朝向原点方向
        let target = Vec3::ZERO;
        camera.forward = (target - camera.position).normalize();
    }
}
