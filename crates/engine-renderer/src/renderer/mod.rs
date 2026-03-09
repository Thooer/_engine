//! 渲染底层：wgpu 通道（v0）
//!
//! 注意：本模块文件禁止出现特定关键字串，所以这里只放类型与 trait 声明。

use engine_core::ecs::{Camera3D, Transform};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::collections::HashMap;
use std::sync::Arc;
use std::any::Any;
use crate::graphics::{GpuMaterial, GpuShader, GpuMesh, GpuModel, Texture, DirectLight, PointLight};
use crate::passes::RenderPass;

pub use self::simple_mesh2d_pass_draw::{draw_simple_mesh2d_pass, SimpleMesh2DPassConfig};
pub use self::simple_mesh3d_pass_draw::{draw_simple_mesh3d_pass, SimpleMesh3DPassConfig};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

pub struct SurfaceSizeHelper;

pub trait SurfaceSizeHelperTrait {
    fn surface_size_is_zero(size: SurfaceSize) -> bool;
}

pub struct WgpuConfigHelper;

pub trait WgpuConfigHelperTrait {
    fn wgpu_debug_on() -> bool;
    fn backends_from_env() -> Option<Vec<wgpu::Backends>>;
}

#[derive(Debug)]
pub struct SurfaceContext<'w> {
    pub(crate) size: SurfaceSize,
    #[allow(dead_code)]
    pub(crate) instance: wgpu::Instance,
    pub(crate) surface: wgpu::Surface<'w>,
    #[allow(dead_code)]
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

#[derive(Debug)]
pub enum FrameStartError {
    Surface(wgpu::SurfaceError),
    NoSurfaceSize,
}

pub trait SurfaceContextTrait {
    fn size(&self) -> SurfaceSize;

    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
    fn color_format(&self) -> wgpu::TextureFormat;
    fn config(&self) -> &wgpu::SurfaceConfiguration;

    fn resize(&mut self, new_size: SurfaceSize);

    fn frame_start(
        &mut self,
    ) -> Result<(wgpu::SurfaceTexture, wgpu::TextureView), FrameStartError>;

    fn frame_show(&self, frame: wgpu::SurfaceTexture);
}

/// 简单 2D 网格渲染管线 trait：固定使用位置 vec2，片元输出固定颜色。
///
/// 设计目标：
/// - 把最基础、可复用的“phase4 风格”渲染管线放到库里
/// - 让示例只关心：传入顶点缓冲 / 索引缓冲与 index_count 即可绘制
pub trait SimpleMeshPipeline2D {
    /// 创建一条简单 2D 网格管线。
    ///
    /// - `color_format` 通常来自 `SurfaceContextTrait::color_format`
    /// - `array_stride` 来自调用方的顶点类型大小：`size_of::<Vertex>() as u64`
    fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        array_stride: u64,
    ) -> Self
    where
        Self: Sized;

    /// 在给定的 render pass 中绘制一次网格。
    fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        vertex: &'a wgpu::Buffer,
        index: &'a wgpu::Buffer,
        index_count: u32,
    );
}

/// 一个最小实现：用于 phase4 示例等场景的简单 2D 网格管线。
///
/// 约定：
/// - 仅有一个顶点缓冲，location(0) = vec2<f32> 位置
/// - 片元着色器输出固定颜色
#[derive(Debug)]
pub struct SimpleMeshPipeline2DPipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
}

/// 带实例数据的二维网格渲染管线 trait。
///
/// 约定：
/// - 顶点缓冲：location(0) = vec2<f32> 位置
/// - 实例缓冲：location(1) = vec2<f32> 偏移，location(2) = vec3<f32> 颜色
pub trait InstanceColorMeshPipeline2D {
    /// 创建一条带实例数据的二维网格管线。
    ///
    /// - `vertex_stride` 来自网格顶点类型大小：`size_of::<Vertex>() as u64`
    /// - `instance_stride` 来自实例数据类型大小：`size_of::<InstanceData>() as u64`
    fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        vertex_stride: u64,
        instance_stride: u64,
    ) -> Self
    where
        Self: Sized;

    /// 在给定的 render pass 中绘制若干实例。
    fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        vertex: &'a wgpu::Buffer,
        instance: &'a wgpu::Buffer,
        instance_count: u32,
        vertex_count: u32,
    );
}

/// 一个最小实现：用于 ECS 场景示例的实例化网格管线。
#[derive(Debug)]
pub struct InstanceColorMeshPipeline2DPipeline {
    pub(crate) pipeline: wgpu::RenderPipeline,
}

/// 创建 `SurfaceContext` 的抽象 trait。
///
/// 设计目标：
/// - 把底层 wgpu 实例 / 适配器 / 设备创建逻辑从示例中抽离出来
/// - 通过 trait 形式，允许后续替换不同的创建策略（例如调试 / 性能优先）
pub trait SurfaceContextNew {
    async fn surface_context_new<'w, W>(
        window: &'w W,
        size: SurfaceSize,
    ) -> Result<SurfaceContext<'w>, wgpu::RequestDeviceError>
    where
        W: HasWindowHandle + HasDisplayHandle + Sync + ?Sized;
}

/// 默认的 `SurfaceContext` 创建实现，封装“当前 wgpu 策略”。
#[derive(Debug, Default)]
pub struct DefaultSurfaceContextNew;

pub struct MainRenderer {
    pub surface_size: SurfaceSize,
    
    // Resource caches
    pub model_cache: HashMap<String, Arc<GpuModel>>,
    pub mesh_cache: HashMap<String, Arc<GpuMesh>>,
    pub material_cache: HashMap<String, Arc<GpuMaterial>>,
    pub shader_cache: HashMap<String, Arc<GpuShader>>,
    pub texture_cache: HashMap<String, Arc<Texture>>,
    pub uniform_cache: HashMap<String, Arc<dyn Any + Send + Sync>>,
    
    // Render Targets
    pub render_targets: HashMap<String, Arc<Texture>>,

    // Bind Groups
    pub pass_bind_group: wgpu::BindGroup,
    
    // Lighting
    pub direct_lights: Vec<DirectLight>,
    pub point_lights: Vec<PointLight>,
    pub model_objects: Vec<(Arc<GpuModel>, Transform)>,
    
    // Frame (Group 0)
    pub frame_bind_group: wgpu::BindGroup,
    pub frame_bind_group_layout: wgpu::BindGroupLayout,

    // Pass (Group 1 - Empty for now)
    pub pass_bind_group_layout: wgpu::BindGroupLayout,
    
    // Render Passes
    // pub passes: Vec<Box<dyn RenderPass>>,
}

pub trait RendererTrait {
    fn new<C: SurfaceContextTrait + ?Sized>(ctx: &C) -> Self;
    fn resize<C: SurfaceContextTrait + ?Sized>(&mut self, ctx: &C);
    fn collect_render_objects(&mut self);
    fn render<C: SurfaceContextTrait>(&mut self, ctx: &mut C) -> Result<(), FrameStartError>;
}

#[path = "SurfaceContextTrait_SurfaceContext.rs"]
mod surface_context_trait_surface_context;

#[path = "SurfaceSizeHelperTrait_SurfaceSizeHelper.rs"]
mod surface_size_helper_trait_surface_size_helper;

#[path = "WgpuConfigHelperTrait_WgpuConfigHelper.rs"]
mod wgpu_config_helper_trait_wgpu_config_helper;

#[path = "SimpleMeshPipeline2D_SimpleMeshPipeline2DPipeline.rs"]
mod simple_mesh_pipeline2d_simple_mesh_pipeline2d_pipeline;

#[path = "InstanceColorMeshPipeline2D_InstanceColorMeshPipeline2DPipeline.rs"]
mod instance_color_mesh_pipeline2d_instance_color_mesh_pipeline2d_pipeline;

#[path = "SurfaceContextNew_DefaultSurfaceContextNew.rs"]
mod surface_context_new_default_surface_context_new;

#[path = "SimpleMesh2DPass_draw.rs"]
mod simple_mesh2d_pass_draw;

#[path = "SimpleMesh3DPass_draw.rs"]
mod simple_mesh3d_pass_draw;

#[path = "SimpleMesh3D_CubeMesh.rs"]
mod simple_mesh3d_cube_mesh;

#[path = "RendererTrait_MainRenderer.rs"]
mod renderer_trait_main_renderer;

pub use self::simple_mesh3d_cube_mesh::{
    create_colored_cube_vertices_indices, create_simple_mesh3d_resources, SimpleMesh3DResources,
    Vertex3D,
};
