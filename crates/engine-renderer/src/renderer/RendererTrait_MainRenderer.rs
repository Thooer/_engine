use std::collections::HashMap;
use std::sync::Arc;

use crate::graphics::{Texture, TextureLoader};
use super::{MainRenderer, RendererTrait, SurfaceContextTrait, FrameStartError};

impl RendererTrait for MainRenderer {
    fn new<C: SurfaceContextTrait + ?Sized>(ctx: &C) -> Self {
        let device = ctx.device();
        let config = ctx.config();

        let screen_texture = Arc::new(Texture::create_render_target(
            device,
            config,
            config.format,
            "Screen Texture",
        ));

        let depth_texture = Arc::new(Texture::create_render_target(
            device,
            config,
            wgpu::TextureFormat::Depth32Float,
            "Depth Texture",
        ));

        Self {
            material_cache: HashMap::new(),
            shader_cache: HashMap::new(),
            texture_cache: HashMap::new(),
            mesh_cache: HashMap::new(),
            model_cache: HashMap::new(),
            screen_texture,
            depth_texture,
            model_objects: Vec::new(),
            direct_lights: Vec::new(),
            point_lights: Vec::new(),
        }
    }

    fn resize<C: SurfaceContextTrait + ?Sized>(&mut self, ctx: &C) {
        let device = ctx.device();
        let config = ctx.config();

        self.screen_texture = Arc::new(Texture::create_render_target(
            device,
            config,
            config.format,
            "Screen Texture",
        ));

        self.depth_texture = Arc::new(Texture::create_render_target(
            device,
            config,
            wgpu::TextureFormat::Depth32Float,
            "Depth Texture",
        ));
    }

    fn collect_render_objects(&mut self) {
        
    }

    fn render<C: SurfaceContextTrait + ?Sized>(&mut self, _ctx: &mut C) -> Result<(), FrameStartError> {
        Ok(())
    }
}
