use crate::graphics::PointLight;
use crate::uniforms::{GpuUniform, LightUniform};
use crate::uniforms::{LightGpuUniform, LightGpuUniformTrait};

impl LightGpuUniformTrait for LightGpuUniform {
    fn new(device: &wgpu::Device, label: Option<&str>) -> Self {
        let size = std::mem::size_of::<LightUniform>() as u64;
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

    fn update(&self, queue: &wgpu::Queue, point_lights: &[PointLight]) {
        let mut uniform_data = LightUniform {
            point_lights: [PointLight {
                position: [0.0; 3],
                range: 0.0,
                color: [0.0; 3],
                intensity: 0.0,
            }; 16],
            point_light_info: [0; 4],
        };

        let count = point_lights.len().min(16);
        uniform_data.point_lights[..count].copy_from_slice(&point_lights[..count]);
        uniform_data.point_light_info[0] = count as u32;

        queue.write_buffer(&self.uniform.buffer, 0, bytemuck::cast_slice(&[uniform_data]));
    }
}
