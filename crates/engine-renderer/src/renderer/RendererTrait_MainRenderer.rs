use std::collections::HashMap;
use std::sync::Arc;
use winit::window::Window;
use winit::event::WindowEvent;
use bevy_ecs::prelude::World;

use super::{FrameStartError, MainRenderer, RendererTrait, SurfaceContextTrait};
use crate::ui::{GuiSystem, GuiSystemTrait, EngineStatsUi, EngineStatsUiTrait};
use crate::graphics::{
    Texture, TextureLoader,
    PointLight,
};
use crate::passes::{MeshForwardPass, LinePass, RenderPass};
use crate::uniforms::{CameraGpuUniform, CameraGpuUniformTrait, LightGpuUniform, LightGpuUniformTrait};

impl RendererTrait for MainRenderer {
    fn new<C: SurfaceContextTrait + ?Sized>(ctx: &C, window: &'static Window) -> Self {
        let device = ctx.device();
        let config = ctx.config();

        let mut render_targets = HashMap::new();
        let screen_texture = Arc::new(Texture::create_render_target(
            device,
            config,
            config.format,
            "Screen Texture",
        ));
        render_targets.insert("Screen Texture".to_string(), screen_texture);

        let depth_texture = Arc::new(Texture::create_render_target(
            device,
            config,
            wgpu::TextureFormat::Depth32Float,
            "Depth Texture",
        ));
        render_targets.insert("Depth Texture".to_string(), depth_texture);

        let mut uniform_cache: HashMap<String, Arc<dyn std::any::Any + Send + Sync>> = HashMap::new();

        // 1. Camera Uniform
        let camera_uniform = Arc::new(CameraGpuUniform::new(
            device,
            Some("Camera Uniform Buffer"),
        ));
        uniform_cache.insert("Camera Uniform".to_string(), camera_uniform.clone());

        // 2. Light Uniform
        let light_uniform = Arc::new(LightGpuUniform::new(
            device,
            Some("Light Uniform Buffer"),
        ));
        uniform_cache.insert("Light Uniform".to_string(), light_uniform.clone());

        // Samplers (Group 0 Binding 1 & 2)
        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Linear Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Nearest Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        // Frame Bind Group (Group 0)
        // Camera, Lights, Global Params...
        let frame_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Camera
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
                // Sampler Linear
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Sampler Nearest
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Light Uniform
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
            label: Some("Frame Bind Group Layout"),
        });

        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform.uniform.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler_linear),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler_nearest),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: light_uniform.uniform.buffer.as_entire_binding(),
                },
            ],
            label: Some("Frame Bind Group"),
        });

        // Pass Bind Group (Group 1 - Empty)
        let pass_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Pass Bind Group Layout"),
            entries: &[],
        });
        
        let pass_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pass Bind Group"),
            layout: &pass_bind_group_layout,
            entries: &[],
        });

        // Initialize Passes
        // let passes: Vec<Box<dyn RenderPass>> = vec![Box::new(MeshForwardPass)];

        // Initialize GUI
        let mut gui = GuiSystem::new(
            device,
            config.format,
            None,
            1,
            window,
        );

        Self {
            surface_size: ctx.size(),
            device: device.clone(),
            queue: ctx.queue().clone(),
            config: config.clone(),
            material_cache: HashMap::new(),
            shader_cache: HashMap::new(),
            texture_cache: HashMap::new(),
            mesh_cache: HashMap::new(),
            model_cache: HashMap::new(),
            uniform_cache,
            render_targets,
            model_objects: Vec::new(),
            direct_lights: Vec::new(),
            point_lights: Vec::new(),
            ui_objects: Vec::new(),
            lines: Vec::new(),
            instance_buffer: None,
            instance_buffer_capacity: 0,
            line_buffer: None,
            line_buffer_capacity: 0,
            frame_bind_group,
            frame_bind_group_layout,
            pass_bind_group,
            pass_bind_group_layout,
            window,
            gui,
            // passes,
        }
    }

    fn resize<C: SurfaceContextTrait + ?Sized>(&mut self, ctx: &C) {
        let device = ctx.device();
        let config = ctx.config();

        let screen_texture = Arc::new(Texture::create_render_target(
            device,
            config,
            config.format,
            "Screen Texture",
        ));
        self.render_targets.insert("Screen Texture".to_string(), screen_texture);

        let depth_texture = Arc::new(Texture::create_render_target(
            device,
            config,
            wgpu::TextureFormat::Depth32Float,
            "Depth Texture",
        ));
        self.render_targets.insert("Depth Texture".to_string(), depth_texture);
    }

    fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.gui.handle_event(window, event)
    }

    fn collect_from_world(&mut self, world: &mut World) {
        use engine_core::ecs::Transform;
        use crate::ecs::MeshRenderable;
        use crate::graphics::ModelLoaderTrait;
        
        self.ui_objects.clear();
        self.lines.clear();
        self.point_lights.clear();

        // Query all renderable entities with Transform and MeshRenderable
        let mut query = world.query::<(&Transform, &MeshRenderable)>();
        for (transform, mesh) in query.iter(world) {
            // Auto-load model if not in cache
            if !self.model_cache.contains_key(&mesh.mesh_id) {
                let model_path = format!("assets/models/{}", mesh.mesh_id);
                match crate::graphics::ModelLoader::load_gltf(&self.device, &self.queue, &model_path) {
                    Ok(gpu_model) => {
                        tracing::info!("Auto-loaded model: {}", mesh.mesh_id);
                        self.model_cache.insert(mesh.mesh_id.clone(), Arc::new(gpu_model));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to auto-load model {}: {}", model_path, e);
                        continue;
                    }
                }
            }

            if let Some(model) = self.model_cache.get(&mesh.mesh_id) {
                // material_override: if MeshRenderable.material_id is non-empty, use it; otherwise None
                let material_override = if mesh.material_id.is_empty() {
                    None
                } else {
                    Some(mesh.material_id.clone())
                };
                self.model_objects.push((model.clone(), *transform, material_override));
            }
        }

        // Query point lights
        let mut light_query = world.query::<&crate::ecs::EcsPointLight>();
        for light in light_query.iter(world) {
            self.point_lights.push(PointLight {
                position: [light.position.x, light.position.y, light.position.z],
                range: light.range,
                color: [light.color.x, light.color.y, light.color.z],
                intensity: light.intensity,
            });
        }

        // Query lines
        let mut line_query = world.query::<&crate::ecs::LineRenderable>();
        for line in line_query.iter(world) {
            let vertex = |pos: [f32; 3]| crate::graphics::Vertex {
                position: pos,
                normal: [0.0; 3],
                uv: [0.0; 2],
                color: line.color,
            };
            self.lines.push(vertex([line.start.x, line.start.y, line.start.z]));
            self.lines.push(vertex([line.end.x, line.end.y, line.end.z]));
        }

        // Always add UI
        self.ui_objects.push(Box::new(EngineStatsUi::new()));

        // Always add axis gizmo
        let mut add_line = |start: [f32; 3], end: [f32; 3], color: [f32; 4]| {
            let vertex = |pos| crate::graphics::Vertex {
                position: pos,
                normal: [0.0; 3],
                uv: [0.0; 2],
                color,
            };
            self.lines.push(vertex(start));
            self.lines.push(vertex(end));
        };

        add_line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]); // X Axis (Red)
        add_line([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0, 1.0]); // Y Axis (Green)
        add_line([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]); // Z Axis (Blue)
        
        // 更新相机 uniform
        self.update_camera_uniform(world);
    }

    fn collect_render_objects(&mut self) {
        self.model_objects.clear();
        self.ui_objects.clear();
        self.lines.clear();

        // Hardcode adding monkey at origin
        if let Some(model) = self.model_cache.get("monkey") {
            self.model_objects.push((
                model.clone(),
                engine_core::ecs::Transform {
                    translation: glam::Vec3::new(1.0, 0.0, 0.0),
                    rotation: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                },
                None, // no material override for hardcoded objects
            ));
        }

        // Hardcode adding UI components
        // In a real ECS system, this would query UI entities
        self.ui_objects.push(Box::new(EngineStatsUi::new()));

        // Hardcode adding lines (Axis Gizmo)
        let mut add_line = |start: [f32; 3], end: [f32; 3], color: [f32; 4]| {
            let vertex = |pos| crate::graphics::Vertex {
                position: pos,
                normal: [0.0; 3],
                uv: [0.0; 2],
                color,
            };
            self.lines.push(vertex(start));
            self.lines.push(vertex(end));
        };

        add_line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]); // X Axis (Red)
        add_line([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0, 1.0]); // Y Axis (Green)
        add_line([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]); // Z Axis (Blue)

        // Hardcode adding a point light
        self.point_lights.clear();
        self.point_lights.push(PointLight {
            position: [2.0, 2.0, 2.0],
            range: 10.0,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
        });
    }

    fn render<C: SurfaceContextTrait>(
        &mut self,
        ctx: &mut C,
    ) -> Result<(), FrameStartError> {

        // Update Light Uniform
        if let Some(light_uniform) = self.uniform_cache.get("Light Uniform") {
            if let Some(light_uniform) = light_uniform.downcast_ref::<LightGpuUniform>() {
                light_uniform.update(ctx.queue(), &self.point_lights);
            }
        }

        // Pre-upload dynamic buffers before passes run
        // Instance buffer: collect all instance data from model_objects
        {
            let mut all_instances: Vec<crate::graphics::InstanceRaw> = Vec::new();
            use glam::Mat4;
            
            for (model, transform, _material_override) in &self.model_objects {
                let model_matrix = Mat4::from_scale_rotation_translation(
                    transform.scale,
                    transform.rotation,
                    transform.translation,
                );
                
                let mut stack = Vec::new();
                for node in &model.root_nodes {
                    stack.push((node, model_matrix));
                }
                
                while let Some((node, parent_matrix)) = stack.pop() {
                    let node_local_matrix = Mat4::from_scale_rotation_translation(
                        node.transform.scale,
                        node.transform.rotation,
                        node.transform.translation,
                    );
                    let node_matrix = parent_matrix * node_local_matrix;
                    
                    if let Some(mesh_idx) = node.mesh_index {
                        if let Some(mesh) = model.meshes.get(mesh_idx) {
                            // One instance per primitive
                            for _ in &mesh.primitives {
                                all_instances.push(crate::graphics::InstanceRaw {
                                    model: node_matrix.to_cols_array_2d(),
                                });
                            }
                        }
                    }
                    
                    for child in &node.children {
                        stack.push((child, node_matrix));
                    }
                }
            }
            
            if !all_instances.is_empty() {
                self.update_instance_buffer(ctx.device(), ctx.queue(), &all_instances);
            }
        }
        
        // Line buffer: upload lines if any
        if !self.lines.is_empty() {
            let lines_data = self.lines.clone();
            self.update_line_buffer(ctx.device(), ctx.queue(), &lines_data);
        }

        let (output, view) = ctx.frame_start()?;

        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        // 顺序执行渲染管线
        // 背景Pass
        // BackgroundPass.render(self, ctx as &mut dyn SurfaceContextTrait, &mut encoder, &view)?;
        // 网格物体Pass
        MeshForwardPass.render(self, ctx as &mut dyn SurfaceContextTrait, &mut encoder, &view)?;
        // 线条Pass
        LinePass.render(self, ctx as &mut dyn SurfaceContextTrait, &mut encoder, &view)?;
        // EGUI Pass
        render_ui(self, ctx.device(), ctx.queue(), &mut encoder, &view);

        ctx.queue().submit(std::iter::once(encoder.finish()));
        ctx.frame_show(output);
        Ok(())
    }
}

fn render_ui(
    renderer: &mut MainRenderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) {
    renderer.gui.begin_frame(renderer.window);

    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [renderer.surface_size.width, renderer.surface_size.height],
        pixels_per_point: renderer.window.scale_factor() as f32,
    };

    renderer.gui.end_frame(
        device,
        queue,
        encoder,
        view,
        screen_descriptor,
        renderer.window,
        &mut renderer.ui_objects,
    );
}

// ============================================================================
// Dynamic GPU Buffer Management
// ============================================================================

impl MainRenderer {
    /// Update camera uniform from ECS world
    pub fn update_camera_uniform(&mut self, world: &mut World) {
        use engine_core::ecs::Camera3D;
        use crate::uniforms::CameraGpuUniformTrait;
        
        if let Some(camera_uniform_arc) = self.uniform_cache.get("Camera Uniform") {
            if let Some(camera_uniform) = camera_uniform_arc.downcast_ref::<crate::uniforms::CameraGpuUniform>() {
                let mut camera_query = world.query::<&Camera3D>();
                if let Some(camera) = camera_query.iter(world).next() {
                    camera_uniform.update(&self.queue, camera, &self.config);
                }
            }
        }
    }
    
    /// Ensure the instance buffer is large enough and return a slice to write to.
    /// If the buffer is too small, it will be recreated with sufficient capacity.
    pub fn get_instance_buffer(&mut self, device: &wgpu::Device, needed_instances: usize) -> &wgpu::Buffer {
        let instance_size = std::mem::size_of::<crate::graphics::InstanceRaw>();
        let needed_bytes = needed_instances * instance_size;
        
        // Recreate buffer if needed
        if self.instance_buffer.is_none() || self.instance_buffer_capacity < needed_instances {
            self.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dynamic Instance Buffer"),
                size: needed_bytes as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.instance_buffer_capacity = needed_instances;
        }
        
        self.instance_buffer.as_ref().unwrap()
    }
    
    /// Update the instance buffer with new data.
    pub fn update_instance_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, instances: &[crate::graphics::InstanceRaw]) {
        let buffer = self.get_instance_buffer(device, instances.len());
        if !instances.is_empty() {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
        }
    }
    
    /// Ensure the line buffer is large enough and return a slice to write to.
    pub fn get_line_buffer(&mut self, device: &wgpu::Device, needed_vertices: usize) -> &wgpu::Buffer {
        let vertex_size = std::mem::size_of::<crate::graphics::Vertex>();
        let needed_bytes = needed_vertices * vertex_size;
        
        // Recreate buffer if needed
        if self.line_buffer.is_none() || self.line_buffer_capacity < needed_vertices {
            self.line_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Dynamic Line Buffer"),
                size: needed_bytes as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.line_buffer_capacity = needed_vertices;
        }
        
        self.line_buffer.as_ref().unwrap()
    }
    
    /// Update the line buffer with new data.
    pub fn update_line_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, vertices: &[crate::graphics::Vertex]) {
        let buffer = self.get_line_buffer(device, vertices.len());
        if !vertices.is_empty() {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(vertices));
        }
    }

    /// Get the wgpu device reference
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get the wgpu queue reference
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get the surface configuration reference
    pub fn get_surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}
