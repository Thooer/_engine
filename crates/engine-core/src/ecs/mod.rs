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

// ============================================================================
// 渲染组件 - 从 engine-renderer 移入
// ============================================================================

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
pub struct PointLight {
    pub position: Vec3,
    pub range: f32,
    pub color: Vec3,
    pub intensity: f32,
}

impl Default for PointLight {
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
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

impl Default for DirectionalLight {
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

