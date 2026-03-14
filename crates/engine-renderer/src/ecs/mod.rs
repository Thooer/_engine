//! ECS Rendering Components
//!
//! 渲染相关的 ECS 组件，用于将渲染数据与 ECS World 集成

use bevy_ecs::prelude::{Component, ReflectComponent};
use bevy_reflect::Reflect;
use glam::Vec3;


/// 网格渲染组件 - 持有对 GPU 网格的引用
///
/// 用于标记一个实体是否可以渲染，需要配合 Transform 组件使用
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct MeshRenderable {
    /// 网格 ID (从 model_cache 获取)
    pub mesh_id: String,
    /// 材质 ID
    pub material_id: String,
}

impl Default for MeshRenderable {
    fn default() -> Self {
        Self {
            mesh_id: String::new(),
            material_id: String::new(),
        }
    }
}

/// 点光源组件
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct EcsPointLight {
    pub position: Vec3,
    pub range: f32,
    pub color: Vec3,
    pub intensity: f32,
}

impl Default for EcsPointLight {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            range: 20.0,
            color: Vec3::ONE,
            intensity: 1.0,
        }
    }
}

/// 方向光组件
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct EcsDirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

impl Default for EcsDirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 1.0,
        }
    }
}

/// 相机优先级 - 用于多相机切换，数值越大优先级越高
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct CameraPriority(pub i32);

/// 线条渲染组件
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct LineRenderable {
    pub start: Vec3,
    pub end: Vec3,
    pub color: [f32; 4],
}

impl Default for LineRenderable {
    fn default() -> Self {
        Self {
            start: Vec3::ZERO,
            end: Vec3::ONE,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// 轨道相机控制器配置
/// 
/// 附加到带有 Camera3D 的实体上，实现自动轨道运动
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct CameraController {
    /// 轨道半径
    pub orbit_radius: f32,
    /// 轨道速度 (弧度/帧)
    pub orbit_speed: f32,
    /// 相机高度
    pub height: f32,
    /// 初始相位偏移 (弧度)
    pub phase_offset: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            orbit_radius: 10.0,
            orbit_speed: 0.02,
            height: 6.0,
            phase_offset: 0.0,
        }
    }
}

impl CameraController {
    /// 创建标准演示配置
    pub fn demo() -> Self {
        Self::default()
    }

    /// 创建慢速环绕配置
    pub fn slow() -> Self {
        Self {
            orbit_radius: 8.0,
            orbit_speed: 0.01,
            height: 5.0,
            phase_offset: 0.0,
        }
    }
}

/// 网格地面配置组件
///
/// 附加到实体上用于配置生成的网格地面参数
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct GridConfig {
    /// 网格范围 (-range 到 range)
    pub range: i32,
    /// 网格高度 (Y 坐标)
    pub height: f32,
    /// 线条颜色 RGBA
    pub color: [f32; 4],
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            range: 5,
            height: 0.0,
            color: [0.4, 0.4, 0.4, 1.0],
        }
    }
}

impl GridConfig {
    /// 创建演示配置
    pub fn demo() -> Self {
        Self::default()
    }
}
