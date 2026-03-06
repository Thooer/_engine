use std::collections::HashMap;
use std::path::{Path, PathBuf};
use wgpu;
use crate::graphics::gpu_shader_loader::ShaderLoader;
use crate::graphics::Vertex;

/// Pipeline 生成器
///
/// 负责扫描指定目录下的着色器文件，并自动生成 wgpu::RenderPipeline。
pub struct PipelineGenerator {
    loader: ShaderLoader,
    root_dir: PathBuf,
}

impl PipelineGenerator {
    pub fn new(assets_dir: impl AsRef<Path>) -> Self {
        let root_dir = assets_dir.as_ref().join("shaders");
        Self {
            loader: ShaderLoader::new(assets_dir),
            root_dir,
        }
    }

    /// 扫描 `assets/shaders/custom` 目录并为每个 `.wgsl` 文件生成 Pipeline
    ///
    /// 返回 Map: 相对路径 (如 "custom/basic_diffuse.wgsl") -> RenderPipeline
    pub fn scan_and_generate_pipelines(
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

    /// 为单个着色器创建 Pipeline
    ///
    /// 默认假设：
    /// - 顶点入口: `vs_main`
    /// - 片元入口: `fs_main`
    /// - 拓扑: `TriangleList`
    /// - 剔除: `Back`
    pub fn create_pipeline(
        &self,
        device: &wgpu::Device,
        shader_path: &str,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<wgpu::RenderPipeline, String> {
        // 1. 编译 ShaderModule
        let module = self.loader.create_shader_module(device, shader_path, Some(shader_path))?;


        // 2. Create Pipeline Layout
        // 我们不创建显式的 Pipeline Layout，而是让 wgpu 根据 Shader 自动推导。
        // 这样可以最大化 Shader 编写的自由度（例如随意添加 Texture/Sampler）。
        // let layout = ... (removed)

        // 3. 配置 Pipeline Descriptor
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(shader_path),
            layout: None, // Enable implicit layout derivation (关键修改！)
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()], // 使用 mod.rs 中定义的标准 Vertex Layout
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Ok(pipeline)
    }
}
