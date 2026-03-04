use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// 3D 顶点数据：位置 + 颜色
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

/// 生成一个单位立方体（边长 1，中心在原点）的顶点和索引数据。
///
/// 约定：
/// - 顶点布局与 `Vertex3D` 一致：location(0) = position，location(1) = color
/// - 可用于任意简单 3D 网格管线（例如 cube3d_demo）
pub fn create_colored_cube_vertices_indices() -> (Vec<Vertex3D>, Vec<u16>) {
    let vertices = vec![
        // 前面 (z = 0.5)
        Vertex3D {
            position: [-0.5, -0.5, 0.5],
            color: [1.0, 0.0, 0.0], // 红
        },
        Vertex3D {
            position: [0.5, -0.5, 0.5],
            color: [0.0, 1.0, 0.0], // 绿
        },
        Vertex3D {
            position: [0.5, 0.5, 0.5],
            color: [0.0, 0.0, 1.0], // 蓝
        },
        Vertex3D {
            position: [-0.5, 0.5, 0.5],
            color: [1.0, 1.0, 0.0], // 黄
        },
        // 后面 (z = -0.5)
        Vertex3D {
            position: [-0.5, -0.5, -0.5],
            color: [1.0, 0.0, 1.0], // 洋红
        },
        Vertex3D {
            position: [0.5, -0.5, -0.5],
            color: [0.0, 1.0, 1.0], // 青
        },
        Vertex3D {
            position: [0.5, 0.5, -0.5],
            color: [1.0, 1.0, 1.0], // 白
        },
        Vertex3D {
            position: [-0.5, 0.5, -0.5],
            color: [0.5, 0.5, 0.5], // 灰
        },
    ];

    let indices = vec![
        // 前面
        0, 1, 2, 2, 3, 0,
        // 后面
        4, 7, 6, 6, 5, 4,
        // 左面
        4, 0, 3, 3, 7, 4,
        // 右面
        1, 5, 6, 6, 2, 1,
        // 上面
        3, 2, 6, 6, 7, 3,
        // 下面
        4, 5, 1, 1, 0, 4,
    ];

    (vertices, indices)
}

/// 最小 3D 立方体渲染资源：shader + pipeline + 顶点/索引 + MVP uniform。
#[derive(Debug)]
pub struct SimpleMesh3DResources {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub uniform_buf: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub index_count: u32,
}

/// 内置的最小 3D 着色器：输入位置 + 颜色，输出颜色，使用单个 MVP 矩阵。
const SIMPLE_MESH3D_WGSL: &str = r#"
struct Uniforms {
    mvp: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
) -> VsOut {
    var out: VsOut;
    out.pos = uniforms.mvp * vec4<f32>(position, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

/// 创建一套用于绘制彩色立方体的简单 3D 渲染资源。
///
/// - 约定顶点布局为 [`Vertex3D`]；
/// - 使用内置 WGSL shader，支持单一 MVP uniform（由调用方逐物体更新）。
pub fn create_simple_mesh3d_resources(
    device: &wgpu::Device,
    color_format: wgpu::TextureFormat,
) -> SimpleMesh3DResources {
    let (vertices, indices) = create_colored_cube_vertices_indices();

    // shader
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("simple_mesh3d shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SIMPLE_MESH3D_WGSL)),
    });

    // 顶点 / 索引缓冲
    let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("simple_mesh3d vertex buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("simple_mesh3d index buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    // MVP uniform 缓冲
    let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("simple_mesh3d uniform buffer"),
        size: std::mem::size_of::<[f32; 16]>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // bind group layout + bind group
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("simple_mesh3d bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("simple_mesh3d bind group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buf.as_entire_binding(),
        }],
    });

    // pipeline layout + pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("simple_mesh3d pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("simple_mesh3d pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex3D>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::size_of::<[f32; 3]>() as u64,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                ],
            }],
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
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    SimpleMesh3DResources {
        pipeline,
        vertex_buf,
        index_buf,
        uniform_buf,
        uniform_bind_group,
        index_count: indices.len() as u32,
    }
}


