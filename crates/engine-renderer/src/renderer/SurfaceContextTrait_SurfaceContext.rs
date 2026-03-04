//! SurfaceContextTrait for SurfaceContext

use crate::renderer::{
    surface_size_is_zero, FrameStartError, SurfaceContext, SurfaceContextTrait, SurfaceSize,
};

fn surface_cfg_with_size(mut cfg: wgpu::SurfaceConfiguration, size: SurfaceSize) -> wgpu::SurfaceConfiguration {
    cfg.width = size.width.max(1);
    cfg.height = size.height.max(1);
    cfg
}

fn surface_recfg<'w>(ctx: &mut SurfaceContext<'w>) {
    if surface_size_is_zero(ctx.size) {
        return;
    }
    let cfg = surface_cfg_with_size(ctx.config.clone(), ctx.size);
    ctx.surface.configure(&ctx.device, &cfg);
}

impl<'w> SurfaceContextTrait for SurfaceContext<'w> {
    fn size(&self) -> SurfaceSize {
        self.size
    }

    fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn color_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    fn resize(&mut self, new_size: SurfaceSize) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        surface_recfg(self);
    }

    fn frame_start(
        &mut self,
    ) -> Result<(wgpu::SurfaceTexture, wgpu::TextureView), FrameStartError> {
        if surface_size_is_zero(self.size) {
            return Err(FrameStartError::NoSurfaceSize);
        }

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                surface_recfg(self);
                self.surface
                    .get_current_texture()
                    .map_err(FrameStartError::Surface)?
            }
            Err(e) => return Err(FrameStartError::Surface(e)),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok((frame, view))
    }

    fn frame_show(&self, frame: wgpu::SurfaceTexture) {
        frame.present();
    }
}

