//! Graphics 模块：定义网格、材质、着色器等基础数据结构。
//!
//! 遵循“类型与实现分离”原则，本文件只包含 Struct/Enum/Trait 定义。

use std::sync::Arc;
use glam::{Vec2, Vec3, Vec4};

// ============================================================================
//  Mesh (网格) & Primitive (图元)
// ============================================================================

/// 顶点数据结构
/// 
/// 遵循 `assets/shaders/着色器设计规范.md` 中的 Layout 定义。
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    /// 获取顶点缓冲布局描述
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
            3 => Float32x4,
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

/// 网格图元 (Submesh)
///
/// 一个网格可能包含多个图元，每个图元对应一次 DrawCall，并使用一个材质。
#[derive(Clone, Debug)]
pub struct MeshPrimitive {
    /// 索引缓冲起始偏移（单位：元素个数，不是字节）
    pub index_start: u32,
    /// 索引数量
    pub index_count: u32,
    /// 材质索引 (对应 Material 列表)
    pub material_index: usize,
}

/// GPU 网格资源
///
/// 包含顶点/索引缓冲以及图元列表。
#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub index_count: u32,
    pub primitives: Vec<MeshPrimitive>,
}

pub trait MeshTrait {
    fn vertex_buffer(&self) -> &wgpu::Buffer;
    fn index_buffer(&self) -> &wgpu::Buffer;
    fn primitives(&self) -> &[MeshPrimitive];
}

// ============================================================================
//  Material (材质)
// ============================================================================

/// 材质参数数据 (CPU 侧)
///
/// 对应 Shader 中的 @group(2)
#[derive(Clone, Debug)]
pub struct MaterialData {
    pub base_color_factor: Vec4,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    // 纹理句柄等后续添加...
}

/// GPU 材质资源
///
/// 包含 BindGroup 等 GPU 对象。
#[derive(Debug)]
pub struct GpuMaterial {
    pub bind_group: wgpu::BindGroup,
    pub data: MaterialData,
}

pub trait MaterialTrait {
    fn bind_group(&self) -> &wgpu::BindGroup;
}

// ============================================================================
//  Shader (着色器)
// ============================================================================

/// 着色器资源
///
/// 包含编译后的 ShaderModule 和 PipelineLayout。
#[derive(Debug)]
pub struct GpuShader {
    pub module: wgpu::ShaderModule,
    pub layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
}

pub trait ShaderTrait {
    fn pipeline(&self) -> &wgpu::RenderPipeline;
}

// 导出加载器
#[path = "GpuShader_Loader.rs"]
pub mod gpu_shader_loader;

#[cfg(test)]
#[path = "GpuShader_Loader_Test.rs"]
mod gpu_shader_loader_tests;

#[path = "GpuPipeline_Generator.rs"]
pub mod gpu_pipeline_generator;

#[cfg(test)]
#[path = "GpuPipeline_Generator_Test.rs"]
mod gpu_pipeline_generator_tests;

// ============================================================================
//  Model (模型)
// ============================================================================

/// 模型资源
///
/// 聚合了 Mesh 和 Material，通常对应一个 glTF 文件。
#[derive(Debug)]
pub struct GpuModel {
    pub meshes: Vec<GpuMesh>,
    pub materials: Vec<GpuMaterial>,
    pub name: String,
}
