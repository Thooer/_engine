use std::collections::HashMap;
use std::sync::Arc;
use crate::graphics::MaterialInput;

/// 全局统一管理的 BindGroupLayout 集合
/// 包含 Group 0 (Frame/Global) 和 Group 1 (Pass) 等固定 Layout
pub struct GlobalLayouts {
    /// Group 0: Frame Level (Camera, Lights, Global Params)
    pub frame_layout: wgpu::BindGroupLayout,
    /// Group 1: Pass Level (Shadows, G-Buffer, etc.) - Currently empty/placeholder
    pub pass_layout: wgpu::BindGroupLayout,
    /// Group 3: Object Level (Instance Data) - Reserved
    pub object_layout: wgpu::BindGroupLayout,
}

impl GlobalLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        // Group 0: Frame Layout
        // Binding 0: Camera Uniform
        // Binding 1: Linear Sampler
        // Binding 2: Nearest Sampler
        // Binding 3: Light Uniform
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Global Frame Layout (Group 0)"),
            entries: &[
                // Binding 0: Camera Uniform
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
                // Binding 1: Sampler Linear
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Binding 2: Sampler Nearest
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Binding 3: Light Uniform
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
        });

        // Group 1: Pass Layout (Placeholder for now)
        let pass_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Global Pass Layout (Group 1)"),
            entries: &[],
        });

        // Group 3: Object Layout (Placeholder for now)
        // Usually handled by Vertex Buffer for instances, but reserved here for future compute/storage usage
        let object_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Global Object Layout (Group 3)"),
            entries: &[],
        });

        Self {
            frame_layout,
            pass_layout,
            object_layout,
        }
    }
}

/// 材质 Layout 缓存管理器 (Group 2)
/// 根据材质的输入特征 (Inputs) 动态创建或复用 BindGroupLayout
pub struct MaterialLayoutCache {
    cache: HashMap<String, Arc<wgpu::BindGroupLayout>>,
}

impl MaterialLayoutCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get_or_create(
        &mut self,
        device: &wgpu::Device,
        inputs: &[MaterialInput],
    ) -> Arc<wgpu::BindGroupLayout> {
        let key = self.generate_signature(inputs);
        
        if let Some(layout) = self.cache.get(&key) {
            return layout.clone();
        }

        let layout = self.create_layout(device, inputs);
        let layout = Arc::new(layout);
        self.cache.insert(key, layout.clone());
        
        layout
    }

    fn generate_signature(&self, inputs: &[MaterialInput]) -> String {
        // Simple signature generation: concat binding type and index
        // e.g. "tex:0|uni:1:64|buf:2:128"
        let mut sig = String::new();
        for input in inputs {
            match input {
                MaterialInput::Texture { binding, path: _ } => {
                    sig.push_str(&format!("tex:{}|", binding));
                }
                MaterialInput::Uniform { binding, size } => {
                    sig.push_str(&format!("uni:{}:{}|", binding, size));
                }
                MaterialInput::Buffer { binding, fields } => {
                    // Calculate size or hash fields
                    // Here we just use binding for simplicity, assuming validation elsewhere
                    // Better to hash structure, but for now binding + type is enough for layout
                    // Note: Buffer size affects layout min_binding_size
                    let size = self.calculate_uniform_size(fields);
                    sig.push_str(&format!("buf:{}:{}|", binding, size));
                }
            }
        }
        sig
    }

    fn create_layout(&self, device: &wgpu::Device, inputs: &[MaterialInput]) -> wgpu::BindGroupLayout {
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
                    let size = self.calculate_uniform_size(fields);
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
            label: Some("Material Bind Group Layout (Group 2)"),
            entries: &entries,
        })
    }

    fn calculate_uniform_size(&self, fields: &[crate::graphics::UniformField]) -> u64 {
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
}
