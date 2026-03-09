//! Uniforms
//!
//! 注意：本模块文件禁止出现特定关键字串，所以这里只放类型与 trait 声明。
use engine_core::ecs::Camera3D;

#[derive(Debug)]
pub struct GpuUniform {
    pub buffer: wgpu::Buffer,
    pub size: u64,
}

pub trait GpuUniformTrait {
    fn new(device: &wgpu::Device, size: u64, label: Option<&str>) -> Self;
}

#[derive(Debug)]
pub struct CameraGpuUniform {
    pub uniform: GpuUniform,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view_pos: [f32; 3],
    pub _padding: f32,
}

pub trait CameraGpuUniformTrait {
    fn new(device: &wgpu::Device, label: Option<&str>) -> Self;
    fn update(&self, queue: &wgpu::Queue, camera: &Camera3D, config: &wgpu::SurfaceConfiguration);
}

#[path = "CameraGpuUniformTrait_CameraGpuUniform.rs"]
mod camera_gpu_uniform_trait_camera_gpu_uniform;

use crate::graphics::PointLight;

#[derive(Debug)]
pub struct LightGpuUniform {
    pub uniform: GpuUniform,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub point_lights: [PointLight; 16],
    pub point_light_info: [u32; 4], // x: count, yzw: padding
}

pub trait LightGpuUniformTrait {
    fn new(device: &wgpu::Device, label: Option<&str>) -> Self;
    fn update(&self, queue: &wgpu::Queue, point_lights: &[PointLight]);
}

#[path = "LightGpuUniformTrait_LightGpuUniform.rs"]
mod light_gpu_uniform_trait_light_gpu_uniform;
