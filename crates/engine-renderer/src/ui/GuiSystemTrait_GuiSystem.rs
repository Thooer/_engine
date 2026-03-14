use super::{GuiSystem, GuiSystemTrait, UiComponent};
use egui::Context;
use egui_wgpu::{Renderer, RendererOptions};
use egui_winit::State;
use wgpu::{Device, TextureFormat, TextureView, CommandEncoder};
use winit::window::Window;
use winit::event::WindowEvent;

impl GuiSystemTrait for GuiSystem {
    fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> Self {
        // 创建支持中文的 Context
        let mut context = Context::default();

        // 尝试加载中文字体
        let mut fonts = egui::FontDefinitions::default();

        // 尝试多个可能的中文字体
        let font_paths = [
            "C:\\Windows\\Fonts\\msyh.ttc",   // 微软雅黑
            "C:\\Windows\\Fonts\\simsun.ttc", // 宋体
            "C:\\Windows\\Fonts\\simhei.ttf", // 黑体
        ];

        for font_path in font_paths {
            if std::path::Path::new(font_path).exists() {
                if let Ok(font_data) = std::fs::read(font_path) {
                    let font_name = std::path::Path::new(font_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("chinese")
                        .to_string();

                    fonts.font_data.insert(
                        font_name.clone(),
                        std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                    );

                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, font_name.clone());

                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .insert(0, font_name.clone());

                    tracing::info!("Loaded Chinese font: {}", font_path);
                    break;
                }
            }
        }

        context.set_fonts(fonts);
        let viewport_id = context.viewport_id();
        
        let state = State::new(
            context.clone(),
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
            Some(device.limits().max_texture_dimension_2d as usize),
        );

        let renderer_options = RendererOptions {
            depth_stencil_format: output_depth_format,
            msaa_samples,
            ..Default::default()
        };

        let renderer = Renderer::new(
            device,
            output_color_format,
            renderer_options,
        );
        
        Self {
            context,
            state,
            renderer,
        }
    }

    fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_pass(raw_input);
    }
    
    fn end_frame(
        &mut self,
        device: &Device,
        queue: &wgpu::Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
        window: &Window,
        components: &mut [Box<dyn UiComponent>],
    ) {
        // Run all registered UI components
        for component in components {
            component.render(&self.context);
        }

        let full_output = self.context.end_pass();
        
        self.state.handle_platform_output(
            window,
            full_output.platform_output
        );

        let clipped_primitives = self.context.tessellate(
            full_output.shapes,
            full_output.pixels_per_point,
        );

        for (id, delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, delta);
        }

        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Egui Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        }).forget_lifetime();

        self.renderer.render(&mut rpass, &clipped_primitives, &screen_descriptor);

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
