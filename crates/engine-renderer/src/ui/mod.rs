use egui::Context;
use egui_wgpu::Renderer;
use egui_winit::State;
use wgpu::{Device, TextureFormat, TextureView, CommandEncoder};
use winit::window::Window;
use winit::event::WindowEvent;

pub trait UiComponent: Send + Sync {
    fn render(&mut self, ctx: &Context);
}

pub struct GuiSystem {
    pub context: Context,
    pub state: State,
    pub renderer: Renderer,
}

pub trait GuiSystemTrait {
    fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> Self;

    fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool;

    fn begin_frame(&mut self, window: &Window);
    
    fn end_frame(
        &mut self,
        device: &Device,
        queue: &wgpu::Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
        window: &Window,
        components: &mut [Box<dyn UiComponent>],
    );
}

#[path = "GuiSystemTrait_GuiSystem.rs"]
mod gui_system_trait_gui_system;

pub struct EngineStatsUi;

pub trait EngineStatsUiTrait {
    fn new() -> Self;
}

#[path = "EngineStatsUiTrait_EngineStatsUi.rs"]
mod engine_stats_ui_trait_engine_stats_ui;
