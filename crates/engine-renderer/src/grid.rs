//! 网格地面系统模块
//!
//! 提供网格地面生成的 ECS 系统

use bevy_ecs::prelude::*;
use glam::Vec3;

use engine_core::ecs::{GridConfig, LineRenderable};

/// 生成网格地面系统
/// 
/// 遍历所有带有 GridConfig 组件的实体，根据配置生成网格线条
pub fn spawn_grid_system(world: &mut World) {
    // 先收集配置，避免借用冲突
    let configs: Vec<(Entity, GridConfig)> = world
        .query::<(Entity, &GridConfig)>()
        .iter(world)
        .map(|(e, c)| (e, c.clone()))
        .collect();

    for (entity, config) in configs {
        let range = config.range;
        let height = config.height;
        let color = config.color;
        
        // 生成网格线条
        for i in -range..=range {
            // Z 轴线条
            world.spawn(LineRenderable {
                start: Vec3::new(-(range as f32), height, i as f32),
                end: Vec3::new(range as f32, height, i as f32),
                color,
            });
            
            // X 轴线条
            world.spawn(LineRenderable {
                start: Vec3::new(i as f32, height, -(range as f32)),
                end: Vec3::new(i as f32, height, range as f32),
                color,
            });
        }
        
        // 消耗掉配置组件（避免每帧重复生成）
        world.entity_mut(entity).remove::<GridConfig>();
    }
}
