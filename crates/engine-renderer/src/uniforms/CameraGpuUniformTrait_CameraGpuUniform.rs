use crate::uniforms::{GpuUniform, CameraUniform};
use crate::uniforms::{CameraGpuUniform, CameraGpuUniformTrait};
use engine_core::ecs::Camera3D;
use glam::{Mat4, Vec3};

impl CameraGpuUniformTrait for CameraGpuUniform {
    fn new(device: &wgpu::Device, label: Option<&str>) -> Self {
        let size = std::mem::size_of::<CameraUniform>() as u64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform = GpuUniform {
            buffer,
            size,
        };
        Self { uniform }
    }

    fn update(&self, queue: &wgpu::Queue, camera: &Camera3D, config: &wgpu::SurfaceConfiguration) {
        let projection = Mat4::perspective_rh(
            45.0f32.to_radians(),
            config.width as f32 / config.height as f32,
            0.1,
            100.0,
        );
        let view = Mat4::look_at_rh(camera.position, camera.position + camera.forward, Vec3::Y);
        
        let camera_uniform_data = CameraUniform {
            view_proj: (projection * view).to_cols_array_2d(),
            view_pos: camera.position.to_array(),
            _padding: 0.0,
        };
        
        queue.write_buffer(&self.uniform.buffer, 0, bytemuck::cast_slice(&[camera_uniform_data]));
    }
}
