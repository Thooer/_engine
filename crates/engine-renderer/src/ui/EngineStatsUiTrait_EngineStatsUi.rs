use super::{EngineStatsUi, EngineStatsUiTrait, UiComponent};
use egui::Context;

impl EngineStatsUiTrait for EngineStatsUi {
    fn new() -> Self {
        Self
    }
}

impl UiComponent for EngineStatsUi {
    fn id(&self) -> &'static str {
        "engine_stats"
    }

    fn render(&mut self, ctx: &Context) {
        egui::Window::new("Engine Stats")
            .resizable(true)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.label("Hello from ToyEngine UI!");
                if ui.button("Click me").clicked() {
                    println!("Clicked!");
                }
            });
    }
}
