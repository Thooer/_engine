use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::graphics::{
    GpuMaterial, GpuShader, GpuModel, Texture,
    MaterialConfig, MaterialInput, PipelineState, GlobalLayouts, MaterialLayoutCache
};

// ============================================================================
//  ShaderLoader
// ============================================================================

/// 着色器加载器
pub struct ShaderLoader {
    pub root_dir: PathBuf,
    /// 内置 shader (key: shader identifier like "builtin/basic_diffuse")
    pub builtin_shaders: std::sync::Arc<std::sync::RwLock<HashMap<String, String>>>,
}

pub trait ShaderLoaderTrait {
    fn new(assets_dir: impl AsRef<Path>) -> Self;
    fn register_builtin(&self, identifier: &str, source: &str);
    fn load_shader_source(&self, shader_path: &str) -> Result<String, String>;
    fn create_shader_module(
        &self, 
        device: &wgpu::Device, 
        shader_path: &str, 
        label: Option<&str>
    ) -> Result<wgpu::ShaderModule, String>;
}

#[path = "ShaderLoaderTrait_ShaderLoader.rs"]
mod gpu_shader_loader;

// ============================================================================
//  PipelineGenerator
// ============================================================================

/// Pipeline 生成器
pub trait PipelineGeneratorTrait {
    fn new(assets_dir: impl AsRef<Path>) -> Self;
    fn register_builtin_shaders(&mut self);

    fn scan_and_generate_pipelines(
        &self,
        device: &wgpu::Device,
        global_layouts: &GlobalLayouts,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<HashMap<String, wgpu::RenderPipeline>, String>;
    
    fn create_pipeline(
        &self,
        device: &wgpu::Device,
        global_layouts: &GlobalLayouts,
        shader_path: &str,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<wgpu::RenderPipeline, String>;

    fn create_gpu_shader(
        &self,
        device: &wgpu::Device,
        shader_path: &str,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        pipeline_state: &PipelineState,
    ) -> Result<GpuShader, String>;
}

pub struct PipelineGenerator {
    pub loader: ShaderLoader,
    pub root_dir: PathBuf,
}

#[path = "PipelineGeneratorTrait_PipelineGenerator.rs"]
mod gpu_pipeline_generator;

// ============================================================================
//  MaterialLoader
// ============================================================================

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
        global_layouts: &GlobalLayouts,
        layout_cache: &mut MaterialLayoutCache,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<LoadedMaterialResources, String>;
}

#[path = "MaterialLoaderTrait_MaterialLoader.rs"]
mod gpu_material_loader;

// ============================================================================
//  TextureLoader
// ============================================================================

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
mod gpu_texture_loader;

// ============================================================================
//  ModelLoader
// ============================================================================

pub struct ModelLoader;

pub trait ModelLoaderTrait {
    fn load_gltf(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
    ) -> Result<GpuModel, String>;
}

#[path = "ModelLoaderTrait_ModelLoader.rs"]
mod gpu_model_loader;
