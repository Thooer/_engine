//! SimpleMeshPipeline2D for SimpleMeshPipeline2DPipeline

use crate::renderer::{SimpleMeshPipeline2D, SimpleMeshPipeline2DPipeline};

use std::borrow::Cow;

/// 内部使用的 WGSL：position(vec2) → 固定颜色输出。
const SIMPLE_MESH_PIPELINE_2D_WGSL: &str = r#"
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(position, 0.0, 1.0);
    out.color = vec3<f32>(0.9, 0.7, 0.2);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

impl SimpleMeshPipeline2D for SimpleMeshPipeline2DPipeline {
    fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        array_stride: u64,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("simple_mesh_2d shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SIMPLE_MESH_PIPELINE_2D_WGSL)),
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        };

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("simple_mesh_2d layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("simple_mesh_2d pipe"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
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

        SimpleMeshPipeline2DPipeline { pipeline }
    }

    fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        vertex: &'a wgpu::Buffer,
        index: &'a wgpu::Buffer,
        index_count: u32,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, vertex.slice(..));
        pass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..index_count, 0, 0..1);
    }
}

