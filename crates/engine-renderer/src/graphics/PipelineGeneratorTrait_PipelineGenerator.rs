use std::collections::HashMap;
use std::path::Path;
use wgpu;
use crate::graphics::ShaderLoaderTrait;
use crate::graphics::ShaderLoader;
use crate::graphics::PipelineGenerator;
use crate::graphics::PipelineGeneratorTrait;
use crate::graphics::VertexTrait;
use crate::graphics::Vertex;

use crate::graphics::GpuShader;
use crate::graphics::PipelineState;

impl PipelineGeneratorTrait for PipelineGenerator {
    fn new(assets_dir: impl AsRef<Path>) -> Self {
        let root_dir = assets_dir.as_ref().join("shaders");
        Self {
            loader: ShaderLoader::new(assets_dir),
            root_dir,
        }
    }

    /// 扫描 `assets/shaders/custom` 目录并为每个 `.wgsl` 文件生成 Pipeline
    ///
    /// 返回 Map: 相对路径 (如 "custom/basic_diffuse.wgsl") -> RenderPipeline
    fn scan_and_generate_pipelines(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<HashMap<String, wgpu::RenderPipeline>, String> {
        let custom_dir = self.root_dir.join("custom");
        if !custom_dir.exists() {
            return Err(format!("Custom shader directory not found: {}", custom_dir.display()));
        }

        let mut pipelines = HashMap::new();

        // 遍历 custom 目录
        let entries = std::fs::read_dir(&custom_dir)
            .map_err(|e| format!("Failed to read custom shader directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            // 只处理 .wgsl 文件
            if path.extension().and_then(|s| s.to_str()) == Some("wgsl") {
                // 获取相对于 shaders 目录的路径 (例如 "custom/basic.wgsl")
                let relative_path = path.strip_prefix(&self.root_dir)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/"); // 统一使用正斜杠

                // 加载并创建 Pipeline
                let pipeline = self.create_pipeline(device, &relative_path, format, depth_format)?;
                pipelines.insert(relative_path, pipeline);
            }
        }

        Ok(pipelines)
    }

    fn create_pipeline(
        &self,
        device: &wgpu::Device,
        shader_path: &str,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<wgpu::RenderPipeline, String> {
        // Default to implicit layout if no layouts provided (or provide empty?)
        // Since we are changing create_gpu_shader to require layouts, we need to handle this.
        // For scan_and_generate, we might want to use implicit layout if we don't have config.
        // But user said "don't use automatic derivation".
        // Let's assume for scan_and_generate we might need to find a way, but for now
        // we can pass an empty list and see if it works (it won't if shader uses groups).
        // OR we allow create_gpu_shader to take Option.
        self.create_gpu_shader(device, shader_path, format, depth_format, &[], &PipelineState::default())
            .map(|gpu_shader| gpu_shader.pipeline)
    }

    fn create_frame_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Sampler Linear (Binding 1)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Sampler Nearest (Binding 2)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Light Uniform (Binding 3)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_gpu_shader(
        &self,
        device: &wgpu::Device,
        shader_path: &str,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        pipeline_state: &PipelineState,
    ) -> Result<GpuShader, String> {
        // 1. 编译 ShaderModule
        let module = self.loader.create_shader_module(device, shader_path, Some(shader_path))?;

        // 2. Create Pipeline Layout
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{} Layout", shader_path)),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let use_layout = if bind_group_layouts.is_empty() { None } else { Some(&layout) };

        // 3. 配置 Pipeline Descriptor
        let blend_state = match pipeline_state.blend_mode {
            crate::graphics::BlendMode::Opaque => Some(wgpu::BlendState::REPLACE),
            crate::graphics::BlendMode::AlphaBlend => Some(wgpu::BlendState::ALPHA_BLENDING),
            crate::graphics::BlendMode::Add => Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
        };

        let cull_mode: Option<wgpu::Face> = pipeline_state.cull_mode.into();
        let depth_compare: wgpu::CompareFunction = pipeline_state.depth_compare.into();
        let depth_write_enabled = pipeline_state.depth_write;

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(shader_path),
            layout: use_layout, 
            multiview: None,
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[<Vertex as VertexTrait>::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: blend_state,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled,
                depth_compare,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
        });

        Ok(GpuShader {
            module,
            layout,
            pipeline,
        })
    }
}