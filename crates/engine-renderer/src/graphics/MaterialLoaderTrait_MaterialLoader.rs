use crate::graphics::{
    GpuMaterial, LoadedMaterialResources, MaterialConfig, MaterialData, MaterialInput,
    MaterialLoader, MaterialLoaderTrait, PipelineGenerator, PipelineGeneratorTrait, Texture,
    TextureLoader,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

fn calculate_uniform_size(fields: &[crate::graphics::UniformField]) -> u64 {
    let mut offset = 0;
    for field in fields {
        let (size, align) = match field.r#type.as_str() {
            "f32" | "i32" | "u32" => (4, 4),
            "vec2" => (8, 8),
            "vec3" => (12, 16),
            "vec4" => (16, 16),
            "mat4" => (64, 16),
            _ => (4, 4),
        };

        let padding = (align - (offset % align)) % align;
        offset += padding;
        offset += size;
    }
    // Align to 16 bytes
    let padding = (16 - (offset % 16)) % 16;
    offset + padding
}

impl MaterialLoaderTrait for MaterialLoader {
    fn load_materials(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
        pipeline_generator: &PipelineGenerator,
        format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Result<LoadedMaterialResources, String> {
        let path = path.as_ref();
        let parent_dir = path.parent().unwrap_or(Path::new("."));

        // 1. 解析配置文件
        let mut configs = Self::load_material_config(path)?;

        // 自动分配 Binding
        for config in &mut configs {
            if let Some(inputs) = &mut config.inputs {
                for (i, input) in inputs.iter_mut().enumerate() {
                    let binding = i as u32;
                    match input {
                        MaterialInput::Texture { binding: b, .. } => *b = binding,
                        MaterialInput::Uniform { binding: b, .. } => *b = binding,
                        MaterialInput::Buffer { binding: b, .. } => *b = binding,
                    }
                }
            }
        }

        let mut resources = LoadedMaterialResources {
            materials: HashMap::new(),
            shaders: HashMap::new(),
            textures: HashMap::new(),
        };

        // Pre-load textures to avoid borrow checker issues
        for config in &configs {
            if let Some(inputs) = &config.inputs {
                for input in inputs {
                    if let MaterialInput::Texture {
                        path: tex_path_str, ..
                    } = input
                    {
                        let tex_path = if Path::new(tex_path_str).is_absolute() {
                            tex_path_str.clone()
                        } else {
                            let p_relative = parent_dir.join(tex_path_str);
                            let p_cwd = Path::new(tex_path_str);
                            if p_relative.exists() {
                                p_relative.to_string_lossy().to_string()
                            } else if p_cwd.exists() {
                                p_cwd.to_string_lossy().to_string()
                            } else {
                                format!("assets/textures/{}", tex_path_str)
                            }
                        };

                        if !resources.textures.contains_key(&tex_path) {
                            let texture = if Path::new(&tex_path).exists() {
                                let bytes = std::fs::read(&tex_path).map_err(|e| {
                                    format!("Failed to read texture {}: {}", tex_path, e)
                                })?;
                                Texture::from_bytes(device, queue, &bytes, &tex_path)?
                            } else {
                                return Err(format!("Texture file not found: {}", tex_path));
                            };
                            resources
                                .textures
                                .insert(tex_path.clone(), Arc::new(texture));
                        }
                    }
                }
            }
        }

        for config in configs {
            // A. 准备 Layouts & Shader
            // Standard Layouts
            let frame_layout = PipelineGenerator::create_frame_bind_group_layout(device);
            let pass_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Empty Layout"),
                entries: &[],
            });

            // Material Layout (Group 2)
            let inputs = config.inputs.as_deref().unwrap_or(&[]);
            let material_layout = Self::create_bind_group_layout(device, inputs);

            // Generate a unique key for the shader based on path and pipeline state
            let shader_key = format!("{}?{:?}", config.shader, config.pipeline_state);

            if !resources.shaders.contains_key(&shader_key) {
                // 使用 PipelineGenerator 创建 GpuShader (手动配置 Layout)
                let gpu_shader = pipeline_generator.create_gpu_shader(
                    device,
                    &config.shader,
                    format,
                    depth_format,
                    &[&frame_layout, &pass_layout, &material_layout],
                    &config.pipeline_state,
                )?;
                resources
                    .shaders
                    .insert(shader_key.clone(), Arc::new(gpu_shader));
            }

            // B. 创建 BindGroup (Group 2)
            let mut entries = Vec::new();
            let mut buffers = Vec::new(); // Keep buffers alive
            
            for input in inputs {
                match input {
                    MaterialInput::Texture { path, binding } => {
                        let tex_path = if Path::new(path).is_absolute() {
                            path.clone()
                        } else {
                            let p_relative = parent_dir.join(path);
                            let p_cwd = Path::new(path);
                            if p_relative.exists() {
                                p_relative.to_string_lossy().to_string()
                            } else if p_cwd.exists() {
                                p_cwd.to_string_lossy().to_string()
                            } else {
                                format!("assets/textures/{}", path)
                            }
                        };
                        
                        if let Some(texture) = resources.textures.get(&tex_path) {
                            entries.push(wgpu::BindGroupEntry {
                                binding: *binding,
                                resource: wgpu::BindingResource::TextureView(&texture.view),
                            });
                        }
                    }
                    MaterialInput::Uniform { size, binding } => {
                        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some(&format!("{} Uniform Buffer", config.name)),
                            size: *size,
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        });
                        buffers.push((*binding, buffer));
                    }
                    MaterialInput::Buffer { fields, binding } => {
                        let size = calculate_uniform_size(fields);
                        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some(&format!("{} Custom Buffer", config.name)),
                            size,
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        });
                        buffers.push((*binding, buffer));
                    }
                }
            }

            for (binding, buffer) in &buffers {
                entries.push(wgpu::BindGroupEntry {
                    binding: *binding,
                    resource: buffer.as_entire_binding(),
                });
            }

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{} BindGroup", config.name)),
                layout: &material_layout,
                entries: &entries,
            });

            // C. 创建 GpuMaterial
            let material = GpuMaterial {
                bind_group,
                data: MaterialData {
                    inputs: config.inputs.clone().unwrap_or_default(),
                },
                shader_name: shader_key,
            };

            resources
                .materials
                .insert(config.name.clone(), Arc::new(material));
        }

        Ok(resources)
    }

    /// 从 TOML 文件加载材质配置
    fn load_material_config(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Vec<MaterialConfig>, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read material file: {}", e))?;

        let config: crate::graphics::MaterialFile =
            toml::from_str(&content).map_err(|e| format!("Failed to parse TOML: {}", e))?;

        Ok(config.materials)
    }

    /// 根据材质输入配置生成 BindGroupLayout (针对 Group 2)
    fn create_bind_group_layout(
        device: &wgpu::Device,
        inputs: &[MaterialInput],
    ) -> wgpu::BindGroupLayout {
        let mut entries = Vec::new();

        for input in inputs {
            let binding = match input {
                MaterialInput::Texture { binding, .. } => *binding,
                MaterialInput::Uniform { binding, .. } => *binding,
                MaterialInput::Buffer { binding, .. } => *binding,
            };

            let entry = match input {
                MaterialInput::Texture { .. } => wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                MaterialInput::Uniform { size, .. } => wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(*size),
                    },
                    count: None,
                },
                MaterialInput::Buffer { fields, .. } => {
                    let size = calculate_uniform_size(fields);
                    wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: std::num::NonZeroU64::new(size),
                        },
                        count: None,
                    }
                }
            };
            entries.push(entry);
        }

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &entries,
        })
    }
}
