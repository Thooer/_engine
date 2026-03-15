use bevy_ecs::prelude::World;
use crate::renderer::MainRenderer;
use std::sync::Arc;
use engine_core::ecs::{Transform, MeshRenderable, PointLight, LineRenderable};
use crate::loaders::ModelLoaderTrait;
use crate::graphics::PointLight as GpuPointLight;
use crate::ui::{EngineStatsUi, EngineStatsUiTrait};

pub fn collect_from_world(world: &mut World, renderer: &mut MainRenderer) {
    // 不清空 ui_objects，保留应用在 on_start 中注册的 UI（如 ProjectOpener）
    renderer.lines.clear();
    renderer.point_lights.clear();

    // Query all renderable entities with Transform and MeshRenderable
    let mut query = world.query::<(&Transform, &MeshRenderable)>();
    for (transform, mesh) in query.iter(world) {
        // Auto-load model if not in cache
        if !renderer.model_cache.contains_key(&mesh.mesh_id) {
            let model_path = format!("assets/models/{}", mesh.mesh_id);
            // Note: renderer.device and renderer.queue are pub(crate), so we can access them here
            // as long as this module is part of the engine-renderer crate.
            match crate::loaders::ModelLoader::load_gltf(&renderer.device, &renderer.queue, &model_path) {
                Ok(gpu_model) => {
                    tracing::info!("Auto-loaded model: {}", mesh.mesh_id);
                    renderer.model_cache.insert(mesh.mesh_id.clone(), Arc::new(gpu_model));
                }
                Err(e) => {
                    tracing::warn!("Failed to auto-load model {}: {}", model_path, e);
                    continue;
                }
            }
        }

        if let Some(model) = renderer.model_cache.get(&mesh.mesh_id) {
            // material_override: if MeshRenderable.material_id is non-empty, use it; otherwise None
            let material_override = if mesh.material_id.is_empty() {
                None
            } else {
                Some(mesh.material_id.clone())
            };
            renderer.model_objects.push((model.clone(), *transform, material_override));
        }
    }

    // Query point lights
    let mut light_query = world.query::<&PointLight>();
    for light in light_query.iter(world) {
        renderer.point_lights.push(GpuPointLight {
            position: [light.position.x, light.position.y, light.position.z],
            range: light.range,
            color: [light.color.x, light.color.y, light.color.z],
            intensity: light.intensity,
        });
    }

    // Query lines
    let mut line_query = world.query::<&LineRenderable>();
    for line in line_query.iter(world) {
        let vertex = |pos: [f32; 3]| crate::graphics::Vertex {
            position: pos,
            normal: [0.0; 3],
            uv: [0.0; 2],
            color: line.color,
        };
        renderer.lines.push(vertex([line.start.x, line.start.y, line.start.z]));
        renderer.lines.push(vertex([line.end.x, line.end.y, line.end.z]));
    }

    // 确保默认 EngineStatsUi 存在，且不覆盖应用注册的 UI（如 ProjectOpener）
    let has_engine_stats = renderer.ui_objects.iter().any(|c| c.id() == "engine_stats");
    if !has_engine_stats {
        renderer.ui_objects.insert(0, Box::new(EngineStatsUi::new()));
    }

    // Always add axis gizmo
    let mut add_line = |start: [f32; 3], end: [f32; 3], color: [f32; 4]| {
        let vertex = |pos| crate::graphics::Vertex {
            position: pos,
            normal: [0.0; 3],
            uv: [0.0; 2],
            color,
        };
        renderer.lines.push(vertex(start));
        renderer.lines.push(vertex(end));
    };

    add_line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]); // X Axis (Red)
    add_line([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0, 1.0]); // Y Axis (Green)
    add_line([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]); // Z Axis (Blue)
    
    // 更新相机 uniform
    renderer.update_camera_uniform(world);
}
