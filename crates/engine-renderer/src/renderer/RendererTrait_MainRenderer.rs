use std::collections::HashMap;
use std::sync::Arc;

use super::{FrameStartError, MainRenderer, RendererTrait, SurfaceContextTrait};
use crate::graphics::{
    PipelineGeneratorTrait,
    Texture, TextureLoader,
    PointLight,
};
use crate::passes::{MeshForwardPass, RenderPass};
use crate::uniforms::{CameraGpuUniform, CameraGpuUniformTrait, LightGpuUniform, LightGpuUniformTrait};

impl RendererTrait for MainRenderer {
    fn new<C: SurfaceContextTrait + ?Sized>(ctx: &C) -> Self {
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
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Nearest Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::Repeat,
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

        Self {
            surface_size: ctx.size(),
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
            frame_bind_group,
            frame_bind_group_layout,
            pass_bind_group,
            pass_bind_group_layout,
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

    fn collect_render_objects(&mut self) {
        self.model_objects.clear();
        // Hardcode adding monkey at origin
        if let Some(model) = self.model_cache.get("monkey") {
            self.model_objects.push((
                model.clone(),
                engine_core::ecs::Transform {
                    translation: glam::Vec3::ZERO,
                    rotation: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                },
            ));
        }

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

        let (output, view) = ctx.frame_start()?;

        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        // 顺序执行渲染管线
        // BackgroundPass.render(self, ctx as &mut dyn SurfaceContextTrait, &mut encoder, &view)?;
        MeshForwardPass.render(self, ctx as &mut dyn SurfaceContextTrait, &mut encoder, &view)?;

        ctx.queue().submit(std::iter::once(encoder.finish()));
        ctx.frame_show(output);
        Ok(())
    }
}
