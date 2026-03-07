//! Graphics 模块：定义网格、材质、着色器等基础数据结构。
//!
//! 遵循“类型与实现分离”原则，本文件只包含 Struct/Enum/Trait 定义。

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::Deserialize;
use glam::Vec3;

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

pub trait VertexTrait {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[path = "VertexTrait_Vertex.rs"]
mod gpu_vertex;

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

// ============================================================================
//  Loaders & Generators (加载器与生成器)
// ============================================================================

/// 着色器加载器
pub struct ShaderLoader {
    pub root_dir: PathBuf,
}

pub trait ShaderLoaderTrait {
    fn new(assets_dir: impl AsRef<Path>) -> Self;
    fn load_shader_source(&self, shader_path: &str) -> Result<String, String>;
    fn create_shader_module(
        &self, 
        device: &wgpu::Device, 
        shader_path: &str, 
        label: Option<&str>
    ) -> Result<wgpu::ShaderModule, String>;
}

#[path = "ShaderLoaderTrait_ShaderLoader.rs"]
pub mod gpu_shader_loader;

#[derive(Debug, Deserialize)]
pub struct MaterialFile {
    pub materials: Vec<MaterialConfig>,
}

#[derive(Debug, Deserialize)]
pub struct MaterialConfig {
    pub name: String,
    pub shader: String,
    pub inputs: Option<Vec<MaterialInput>>,
}

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

pub struct MaterialLoader;

/// 材质加载结果资源包
pub struct LoadedMaterialResources {
    pub materials: HashMap<String, std::sync::Arc<GpuMaterial>>,
    pub shaders: HashMap<String, std::sync::Arc<GpuShader>>,
    pub textures: HashMap<String, std::sync::Arc<Texture>>,
}

/// 材质加载器接口
pub trait MaterialLoaderTrait {
    fn load_material_config(path: impl AsRef<Path>) -> Result<Vec<MaterialConfig>, String>;
    fn create_bind_group_layout(device: &wgpu::Device, inputs: &[MaterialInput]) -> wgpu::BindGroupLayout;
    
    fn load_materials(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
        pipeline_generator: &PipelineGenerator,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<LoadedMaterialResources, String>;
}

#[path = "MaterialLoaderTrait_MaterialLoader.rs"]
pub mod gpu_material_loader;

/// Texture 生成器
pub trait TextureLoader {
    fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, String>
    where
        Self: Sized;

    fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self, String>
    where
        Self: Sized;

    fn create_render_target(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self;
}

#[path = "TextureLoader_Texture.rs"]
pub mod gpu_texture_loader;

/// Pipeline 生成器
pub trait PipelineGeneratorTrait {
    fn new(assets_dir: impl AsRef<Path>) -> Self;
    
    fn scan_and_generate_pipelines(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<HashMap<String, wgpu::RenderPipeline>, String>;
    
    fn create_pipeline(
        &self,
        device: &wgpu::Device,
        shader_path: &str,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<wgpu::RenderPipeline, String>;

    fn create_frame_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;

    fn create_gpu_shader(
        &self,
        device: &wgpu::Device,
        shader_path: &str,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> Result<GpuShader, String>;
}

pub struct PipelineGenerator {
    pub loader: ShaderLoader,
    pub root_dir: PathBuf,
}

#[path = "PipelineGeneratorTrait_PipelineGenerator.rs"]
pub mod gpu_pipeline_generator;

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

pub struct ModelLoader;

pub trait ModelLoaderTrait {
    fn load_gltf(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
    ) -> Result<GpuModel, String>;
}

#[path = "ModelLoaderTrait_ModelLoader.rs"]
pub mod gpu_model_loader;

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

pub trait PointLightTrait {
    fn new(position: Vec3, color: Vec3, intensity: f32, range: f32) -> Self;
}

#[path = "DirectLightTrait_DirectLight.rs"]
pub mod gpu_direct_light;

#[path = "PointLightTrait_PointLight.rs"]
pub mod gpu_point_light;
