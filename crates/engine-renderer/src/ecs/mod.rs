//! ECS Rendering Components
//!
//! 注意：此模块中的组件已移至 engine-core。
//! 请直接使用 engine_core::ecs::* 中的组件。

pub use engine_core::ecs::{
    MeshRenderable,
    PointLight,
    DirectionalLight,
    CameraPriority,
    LineRenderable,
    CameraController,
    GridConfig,
};
