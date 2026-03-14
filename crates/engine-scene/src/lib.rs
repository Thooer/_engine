//! ToyEngine Scene - 场景序列化与加载
//!
//! 提供场景的序列化/反序列化功能，支持从 RON 文件加载场景到 ECS World
//!
//! # 使用方式
//!
//! ```rust
//! use engine_scene::load_scene;
//!
//! fn on_start(&mut self, engine: &mut Engine) {
//!     load_scene("assets/scenes/main.ron", engine.world_mut());
//! }
//! ```

use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use std::path::Path;
use engine_renderer::graphics::ModelLoaderTrait;

// ============================================================================
// 场景组件定义 (用于序列化)
// ============================================================================

/// 场景中的实体数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneEntity {
    /// 实体名称（可选）
    pub name: Option<String>,
    /// 变换组件
    pub transform: Option<TransformData>,
    /// 渲染组件
    pub renderable: Option<RenderableData>,
    /// 网格模型组件
    pub mesh: Option<MeshData>,
    /// 相机组件
    pub camera: Option<CameraData>,
    /// 物理组件
    pub physics: Option<PhysicsData>,
    /// 线条渲染
    pub line: Option<LineData>,
    /// 光源
    pub light: Option<LightData>,
    /// 控制器
    pub controller: Option<ControllerData>,
    /// 网格配置
    pub grid: Option<GridData>,
}

/// 变换数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransformData {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

/// 渲染数据（简单颜色）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RenderableData {
    pub color: Vec3,
}

/// 网格模型数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeshData {
    /// 模型文件名（相对于 assets/models/）
    pub model: String,
    /// 材质名
    pub material: Option<String>,
}

/// 相机数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CameraData {
    pub camera_type: String,      // "2d" 或 "3d"
    pub position: Vec3,
    pub forward: Option<Vec3>,
    pub zoom: Option<f32>,
    pub priority: i32,
}

/// 物理数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhysicsData {
    pub body_type: String,        // "dynamic", "fixed", "kinematic"
    pub collider_shape: String,   // "ball" 或 "cuboid"
    pub mass: Option<f32>,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub half_extents: Option<Vec3>,
    pub radius: Option<f32>,
}

/// 线条数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LineData {
    pub start: Vec3,
    pub end: Vec3,
    pub color: [f32; 4],
}

/// 光源数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightData {
    pub light_type: String,        // "point" 或 "directional"
    pub position: Option<Vec3>,
    pub direction: Option<Vec3>,
    pub color: Vec3,
    pub intensity: f32,
    pub range: Option<f32>,
}

/// 控制器数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ControllerData {
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub height: f32,
    pub phase_offset: f32,
}

/// 网格配置数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GridData {
    pub range: i32,
    pub height: f32,
    pub color: [f32; 4],
}

/// 场景数据（RON 根对象）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    pub entities: Vec<SceneEntity>,
}

// ============================================================================
// 场景加载错误
// ============================================================================

#[derive(Debug)]
pub enum SceneError {
    Io(std::io::Error),
    Parse(String),
    Deserialize(String),
    Serialize(String),
    Spawn(String),
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
            Self::Deserialize(e) => write!(f, "Deserialize error: {}", e),
            Self::Serialize(e) => write!(f, "Serialize error: {}", e),
            Self::Spawn(e) => write!(f, "Spawn error: {}", e),
        }
    }
}

impl std::error::Error for SceneError {}

impl From<std::io::Error> for SceneError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

// ============================================================================
// 公共 API
// ============================================================================

/// 从 RON 文件加载场景（不含模型加载）
pub fn load_scene(path: impl AsRef<Path>, world: &mut World) -> Result<(), SceneError> {
    let path = path.as_ref();
    
    // 读取 RON 文件
    let content = std::fs::read_to_string(path)?;
    
    // 反序列化场景
    let scene: Scene = ron::from_str(&content)
        .map_err(|e| SceneError::Deserialize(e.to_string()))?;
    
    // Spawn 所有实体
    for entity_data in scene.entities {
        spawn_entity(entity_data, world, None, None, None)?;
    }
    
    tracing::info!("Scene loaded from: {:?}", path);
    
    Ok(())
}

/// 从 RON 文件加载场景（含模型加载）
/// 
/// # 参数
/// - `path`: 场景文件路径
/// - `world`: ECS World
/// - `device`: wgpu Device（用于创建 GPU 资源）
/// - `queue`: wgpu Queue（用于提交命令）
/// - `model_cache`: 模型缓存（用于存储加载的模型）
pub fn load_scene_with_renderer(
    path: impl AsRef<Path>,
    world: &mut World,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    model_cache: &mut std::collections::HashMap<String, std::sync::Arc<engine_renderer::graphics::GpuModel>>,
) -> Result<(), SceneError> {
    let path = path.as_ref();
    
    // 读取 RON 文件
    let content = std::fs::read_to_string(path)?;
    
    // 反序列化场景
    let scene: Scene = ron::from_str(&content)
        .map_err(|e| SceneError::Deserialize(e.to_string()))?;
    
    // Spawn 所有实体
    for entity_data in scene.entities {
        spawn_entity(entity_data, world, Some(device), Some(queue), Some(model_cache))?;
    }
    
    tracing::info!("Scene loaded from: {:?}", path);
    
    Ok(())
}

/// 保存场景到 RON 文件
pub fn save_scene(world: &mut World, path: impl AsRef<Path>) -> Result<(), SceneError> {
    let path = path.as_ref();
    
    // 收集所有实体数据
    let mut entities = Vec::new();
    
    // 收集 Transform
    let transforms: Vec<_> = world.query::<(Entity, &engine_core::ecs::Transform)>().iter(world).collect();
    
    for (entity, transform) in transforms {
        let mut entity_data = SceneEntity {
            name: None,
            transform: Some(TransformData {
                translation: transform.translation,
                rotation: transform.rotation,
                scale: transform.scale,
            }),
            renderable: None,
            mesh: None,
            camera: None,
            physics: None,
            line: None,
            light: None,
            controller: None,
            grid: None,
        };
        
        // 收集 Renderable
        if let Some(renderable) = world.get::<engine_core::ecs::Renderable>(entity) {
            entity_data.renderable = Some(RenderableData {
                color: renderable.color,
            });
        }
        
        // 收集 Camera2D
        if let Some(camera) = world.get::<engine_core::ecs::Camera2D>(entity) {
            entity_data.camera = Some(CameraData {
                camera_type: "2d".to_string(),
                position: camera.position,
                forward: None,
                zoom: Some(camera.zoom),
                priority: 0,
            });
        }
        
        // 收集 Camera3D
        if let Some(camera) = world.get::<engine_core::ecs::Camera3D>(entity) {
            entity_data.camera = Some(CameraData {
                camera_type: "3d".to_string(),
                position: camera.position,
                forward: Some(camera.forward),
                zoom: None,
                priority: 0,
            });
        }
        
        // 收集 LineRenderable
        if let Some(line) = world.get::<engine_renderer::ecs::LineRenderable>(entity) {
            entity_data.line = Some(LineData {
                start: line.start,
                end: line.end,
                color: line.color,
            });
        }
        
        // 收集 EcsPointLight
        if let Some(light) = world.get::<engine_renderer::ecs::EcsPointLight>(entity) {
            entity_data.light = Some(LightData {
                light_type: "point".to_string(),
                position: Some(light.position),
                direction: None,
                color: light.color,
                intensity: light.intensity,
                range: Some(light.range),
            });
        }
        
        // 收集 EcsDirectionalLight
        if let Some(light) = world.get::<engine_renderer::ecs::EcsDirectionalLight>(entity) {
            entity_data.light = Some(LightData {
                light_type: "directional".to_string(),
                position: None,
                direction: Some(light.direction),
                color: light.color,
                intensity: light.intensity,
                range: None,
            });
        }
        
        // 收集 CameraController
        if let Some(controller) = world.get::<engine_renderer::ecs::CameraController>(entity) {
            entity_data.controller = Some(ControllerData {
                orbit_radius: controller.orbit_radius,
                orbit_speed: controller.orbit_speed,
                height: controller.height,
                phase_offset: controller.phase_offset,
            });
        }
        
        // 收集 GridConfig
        if let Some(grid) = world.get::<engine_renderer::ecs::GridConfig>(entity) {
            entity_data.grid = Some(GridData {
                range: grid.range,
                height: grid.height,
                color: grid.color,
            });
        }
        
        // 收集物理组件
        let physics_data = collect_physics_data(world, entity);
        if physics_data.is_some() {
            entity_data.physics = physics_data;
        }
        
        entities.push(entity_data);
    }
    
    // 创建场景
    let scene = Scene { entities };
    
    // 序列化
    let content = ron::to_string(&scene)
        .map_err(|e| SceneError::Serialize(e.to_string()))?;
    
    // 写入文件
    std::fs::write(path, content)?;
    
    tracing::info!("Scene saved to: {:?}", path);
    
    Ok(())
}

/// 收集物理数据
fn collect_physics_data(world: &World, entity: Entity) -> Option<PhysicsData> {
    let rigid_body = world.get::<engine_physics::RigidBody>(entity)?;
    let collider = world.get::<engine_physics::Collider>(entity)?;
    
    let body_type = match rigid_body.body_type {
        engine_physics::RigidBodyType::Dynamic => "dynamic",
        engine_physics::RigidBodyType::Fixed => "fixed",
        engine_physics::RigidBodyType::KinematicPositionBased => "kinematic",
    }.to_string();
    
    let (collider_shape, half_extents, radius) = match &collider.shape {
        engine_physics::ColliderShape::Ball { radius } => ("ball".to_string(), None, Some(*radius)),
        engine_physics::ColliderShape::Cuboid { half_extents } => ("cuboid".to_string(), Some(*half_extents), None),
    };
    
    Some(PhysicsData {
        body_type,
        collider_shape,
        mass: rigid_body.additional_mass,
        friction: collider.friction,
        restitution: collider.restitution,
        density: collider.density,
        half_extents,
        radius,
    })
}

/// Spawn 单个实体 - 使用 Commands 直接 spawn
/// 
/// # 参数
/// - `data`: 场景实体数据
/// - `world`: ECS World
/// - `device`: 可选的 wgpu Device（用于加载模型）
/// - `queue`: 可选的 wgpu Queue（用于加载模型）
/// - `model_cache`: 可选的模型缓存（用于存储加载的模型）
fn spawn_entity(
    data: SceneEntity,
    world: &mut World,
    device: Option<&wgpu::Device>,
    queue: Option<&wgpu::Queue>,
    model_cache: Option<&mut std::collections::HashMap<String, std::sync::Arc<engine_renderer::graphics::GpuModel>>>,
) -> Result<(), SceneError> {
    let mut commands = world.spawn_empty();
    
    // Transform
    if let Some(t) = data.transform {
        commands.insert(engine_core::ecs::Transform {
            translation: t.translation,
            rotation: t.rotation,
            scale: t.scale,
        });
    }
    
    // Renderable
    if let Some(r) = data.renderable {
        commands.insert(engine_core::ecs::Renderable {
            color: r.color,
        });
    }
    
    // Mesh (模型)
    if let Some(m) = data.mesh {
        // 确保有渲染资源可用
        if let (Some(device), Some(queue), Some(cache)) = (device, queue, model_cache) {
            let model_path = format!("assets/models/{}", m.model);
            
            // 尝试加载模型（如果尚未缓存）
            if !cache.contains_key(&m.model) {
                match engine_renderer::graphics::ModelLoader::load_gltf(device, queue, &model_path) {
                    Ok(gpu_model) => {
                        tracing::info!("Loaded model: {}", m.model);
                        cache.insert(m.model.clone(), std::sync::Arc::new(gpu_model));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load model {}: {}", model_path, e);
                    }
                }
            }
            
            // 添加 MeshRenderable 组件
            commands.insert(engine_renderer::ecs::MeshRenderable {
                mesh_id: m.model.clone(),
                material_id: m.material.unwrap_or_else(|| "default".to_string()),
            });
        } else {
            tracing::warn!("Mesh specified but no renderer context available");
        }
    }
    
    // Camera
    if let Some(c) = data.camera {
        match c.camera_type.as_str() {
            "2d" => {
                commands.insert(engine_core::ecs::Camera2D {
                    position: c.position,
                    zoom: c.zoom.unwrap_or(1.0),
                });
            }
            "3d" => {
                commands.insert(engine_core::ecs::Camera3D {
                    position: c.position,
                    forward: c.forward.unwrap_or(Vec3::new(0.0, -1.0, -1.0)),
                });
            }
            _ => {}
        }
    }
    
    // Physics
    if let Some(p) = data.physics {
        let body_type = match p.body_type.as_str() {
            "dynamic" => engine_physics::RigidBodyType::Dynamic,
            "fixed" => engine_physics::RigidBodyType::Fixed,
            "kinematic" => engine_physics::RigidBodyType::KinematicPositionBased,
            _ => engine_physics::RigidBodyType::Dynamic,
        };
        
        commands.insert(engine_physics::RigidBody {
            body_type,
            additional_mass: p.mass,
            linear_damping: 0.0,
            angular_damping: 0.0,
            can_sleep: true,
            ccd_enabled: false,
        });
        
        let shape = match p.collider_shape.as_str() {
            "ball" => engine_physics::ColliderShape::Ball { 
                radius: p.radius.unwrap_or(0.5) 
            },
            "cuboid" => engine_physics::ColliderShape::Cuboid { 
                half_extents: p.half_extents.unwrap_or(Vec3::new(0.5, 0.5, 0.5)) 
            },
            _ => engine_physics::ColliderShape::Ball { radius: 0.5 },
        };
        
        commands.insert(engine_physics::Collider {
            shape,
            friction: p.friction,
            restitution: p.restitution,
            density: p.density,
            sensor: false,
        });
    }
    
    // LineRenderable
    if let Some(l) = data.line {
        commands.insert(engine_renderer::ecs::LineRenderable {
            start: l.start,
            end: l.end,
            color: l.color,
        });
    }
    
    // Light
    if let Some(l) = data.light {
        match l.light_type.as_str() {
            "point" => {
                commands.insert(engine_renderer::ecs::EcsPointLight {
                    position: l.position.unwrap_or(Vec3::ZERO),
                    range: l.range.unwrap_or(20.0),
                    color: l.color,
                    intensity: l.intensity,
                });
            }
            "directional" => {
                commands.insert(engine_renderer::ecs::EcsDirectionalLight {
                    direction: l.direction.unwrap_or(Vec3::new(0.0, -1.0, 0.0)),
                    color: l.color,
                    intensity: l.intensity,
                });
            }
            _ => {}
        }
    }
    
    // Controller
    if let Some(c) = data.controller {
        commands.insert(engine_renderer::ecs::CameraController {
            orbit_radius: c.orbit_radius,
            orbit_speed: c.orbit_speed,
            height: c.height,
            phase_offset: c.phase_offset,
        });
    }
    
    // Grid
    if let Some(g) = data.grid {
        commands.insert(engine_renderer::ecs::GridConfig {
            range: g.range,
            height: g.height,
            color: g.color,
        });
    }
    
    tracing::debug!("Spawned entity from scene data");
    
    Ok(())
}
