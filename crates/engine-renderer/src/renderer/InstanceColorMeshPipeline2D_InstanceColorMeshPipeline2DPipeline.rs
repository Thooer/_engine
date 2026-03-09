//! InstanceColorMeshPipeline2D for InstanceColorMeshPipeline2DPipeline
//!
//! 实现一个带实例数据的二维网格渲染管线：
//! - 顶点缓冲：location(0) = vec2<f32> 位置
//! - 实例缓冲：location(1) = vec2<f32> 偏移，location(2) = vec3<f32> 颜色

use crate::renderer::{InstanceColorMeshPipeline2D, InstanceColorMeshPipeline2DPipeline};

use std::borrow::Cow;

/// 内部使用的 WGSL：顶点位置 + 实例偏移与颜色。
const INSTANCE_COLOR_MESH_PIPELINE_2D_WGSL: &str = r#"
struct Instance {
    @location(1) offset: vec2<f32>,
    @location(2) color: vec3<f32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    inst: Instance,
) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(position + inst.offset, 0.0, 1.0);
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

impl InstanceColorMeshPipeline2D for InstanceColorMeshPipeline2DPipeline {
    fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        vertex_stride: u64,
        instance_stride: u64,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("instance_color_mesh_2d shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                INSTANCE_COLOR_MESH_PIPELINE_2D_WGSL,
            )),
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: vertex_stride,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        };

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: instance_stride,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                1 => Float32x2, // offset
                2 => Float32x3, // color
            ],
        };

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("instance_color_mesh_2d layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("instance_color_mesh_2d pipe"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout, instance_layout],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        InstanceColorMeshPipeline2DPipeline { pipeline }
    }

    fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        vertex: &'a wgpu::Buffer,
        instance: &'a wgpu::Buffer,
        instance_count: u32,
        vertex_count: u32,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, vertex.slice(..));
        pass.set_vertex_buffer(1, instance.slice(..));
        pass.draw(0..vertex_count, 0..instance_count);
    }
}

