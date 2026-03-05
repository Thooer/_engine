use toyengine_app::{App, AppConfig, Engine, EngineTrait, RunApp, RunAppTrait};
use toyengine_renderer::renderer::{FrameStartError, SurfaceContextTrait};

struct ClearDemoApp {
    frames: u32,
}

trait ClearDemoAppTrait {
    fn draw(&mut self, engine: &mut Engine);
}

impl ClearDemoAppTrait for ClearDemoApp {
    fn draw(&mut self, engine: &mut Engine) {
        let ctx = engine.ctx_mut();

        let (frame, view) = match ctx.frame_start() {
            Ok(v) => v,
            Err(FrameStartError::NoSurfaceSize) => return,
            Err(FrameStartError::Surface(wgpu::SurfaceError::OutOfMemory)) => panic!("oom"),
            Err(FrameStartError::Surface(_)) => return,
        };

        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear encoder"),
            });

        {
            let _rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.08,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        engine.ctx().queue().submit(Some(encoder.finish()));
        engine.ctx_mut().frame_show(frame);

        self.frames += 1;
    }
}

impl App for ClearDemoApp {
    fn on_render(&mut self, engine: &mut Engine) {
        self.draw(engine);
    }
}

fn main() {
    RunApp::run_app(
        AppConfig {
            title: "ToyEngine Clear Demo",
            max_frames: Some(120),
            fixed_dt_seconds: Some(1.0 / 60.0),
        },
        ClearDemoApp { frames: 0 },
    );
}

