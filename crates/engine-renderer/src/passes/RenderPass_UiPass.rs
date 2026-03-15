use super::{RenderPass, UiPass};
use crate::renderer::{MainRenderer, SurfaceContextTrait, FrameStartError};
use crate::ui::GuiSystemTrait;

impl RenderPass for UiPass {
    fn render(
        &self,
        renderer: &mut MainRenderer,
        ctx: &mut dyn SurfaceContextTrait,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) -> Result<(), FrameStartError> {
        // Prepare EGUI Frame
        renderer.gui.begin_frame(renderer.window);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [renderer.surface_size.width, renderer.surface_size.height],
            pixels_per_point: renderer.window.scale_factor() as f32,
        };

        // Render EGUI
        renderer.gui.end_frame(
            ctx.device(),
            ctx.queue(),
            encoder,
            view,
            screen_descriptor,
            renderer.window,
            &mut renderer.ui_objects,
        );

        Ok(())
    }
}
