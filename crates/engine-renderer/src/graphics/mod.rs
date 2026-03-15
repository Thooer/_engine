//! Graphics 模块：定义网格、材质、着色器等基础数据结构。
//!
//! 遵循“类型与实现分离”原则，本文件只包含 Struct/Enum/Trait 定义。

use std::fmt::Debug;
use serde::Deserialize;

use glam::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view_pos: [f32; 3],
    pub _padding: f32,
}

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

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}

pub trait VertexTrait {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub trait InstanceTrait {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[path = "VertexTrait_Vertex.rs"]
mod gpu_vertex;
#[path = "InstanceTrait_InstanceRaw.rs"]
mod gpu_instance;

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

// texture
pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

// ============================================================================
//  Material (材质)
// ============================================================================

/// 材质参数数据 (CPU 侧)
///
/// 对应 Shader 中的 @group(2)
#[derive(Clone, Debug, Default)]
pub struct MaterialData {
    pub inputs: Vec<MaterialInput>,
}

/// GPU 材质资源
///
/// 包含 BindGroup 等 GPU 对象。
#[derive(Debug)]
pub struct GpuMaterial {
    pub bind_group: wgpu::BindGroup,
    pub data: MaterialData,
    pub shader_name: String,
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

pub mod layouts;
pub use self::layouts::{GlobalLayouts, MaterialLayoutCache};

// ============================================================================
//  Loaders & Generators (加载器与生成器)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MaterialFile {
    pub materials: Vec<MaterialConfig>,
}

#[derive(Debug, Deserialize)]
pub struct MaterialConfig {
    pub name: String,
    pub shader: String,
    pub inputs: Option<Vec<MaterialInput>>,
    #[serde(default)]
    pub pipeline_state: PipelineState,
}

pub use pipeline_state::*;

mod pipeline_state;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MaterialInput {
    Texture { 
        path: String,
        #[serde(default)]
        binding: u32,
    },
    // Sampler removed as per user request (using global sampler)
    Uniform {
        #[serde(default)]
        size: u64,
        #[serde(default)]
        binding: u32,
    },
    Buffer { 
        fields: Vec<UniformField>,
        #[serde(default)]
        binding: u32,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct UniformField {
    pub name: String,
    pub r#type: String,
}

// ============================================================================
//  Model (模型)
// ============================================================================

use engine_core::ecs::Transform;

/// 模型节点
#[derive(Debug)]
pub struct ModelNode {
    pub transform: Transform,
    pub mesh_index: Option<usize>,
    pub children: Vec<ModelNode>,
}

/// 模型资源
///
/// 聚合了 Mesh 和 Material，通常对应一个 glTF 文件。
#[derive(Debug)]
pub struct GpuModel {
    pub meshes: Vec<GpuMesh>,
    pub material_names: Vec<String>, // 存储材质名称
    pub root_nodes: Vec<ModelNode>,  // 根节点列表
    pub name: String,
}

// ============================================================================
//  Light (灯光)
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectLight {
    pub direction: [f32; 3],
    pub _padding: f32, // unused
    pub color: [f32; 3],
    pub intensity: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    pub position: [f32; 3],
    pub range: f32, // packed
    pub color: [f32; 3],
    pub intensity: f32, // packed
}

pub trait DirectLightTrait {
    fn new(direction: Vec3, color: Vec3, intensity: f32) -> Self;
}

#[path = "DirectLightTrait_DirectLight.rs"]
mod gpu_direct_light;

pub trait PointLightTrait {
    fn new(position: Vec3, color: Vec3, intensity: f32, range: f32) -> Self;
}

#[path = "PointLightTrait_PointLight.rs"]
mod gpu_point_light;
