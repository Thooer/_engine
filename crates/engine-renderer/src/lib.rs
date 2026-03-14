//! ToyEngine Renderer - 渲染系统模块
//!
//! 包含图形渲染、材质系统、UI 渲染等功能

/// 渲染模块
pub mod renderer;

/// 渲染 Pass 模块
pub mod passes;
pub mod uniforms;

/// 材质模块
mod materials;

/// UI 模块
mod ui;

/// 通用图形基础
pub mod graphics;

/// ECS 渲染组件
pub mod ecs;

/// 相机系统
pub mod camera;

/// 网格地面系统
pub mod grid;

/// 渲染图抽象
pub mod render_graph;
