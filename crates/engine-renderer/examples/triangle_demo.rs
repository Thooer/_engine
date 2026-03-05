use std::borrow::Cow;

use toyengine_app::{App, AppConfig, Engine, EngineTrait, RunApp, RunAppTrait};
use toyengine_renderer::renderer::{FrameStartError, SurfaceContextTrait};

const TRI_WGSL: &str = r#"
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.6),
        vec2<f32>(-0.6, -0.6),
        vec2<f32>(0.6, -0.6),
    );
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.1, 0.1),
        vec3<f32>(0.1, 1.0, 0.1),
        vec3<f32>(0.1, 0.1, 1.0),
    );

    var out: VsOut;
    out.pos = vec4<f32>(positions[idx], 0.0, 1.0);
    out.color = colors[idx];
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

struct TriangleDemoApp {
    pipe: Option<wgpu::RenderPipeline>,
    frames: u32,
}

trait TriangleDemoAppTrait {
    fn build_pipe(ctx: &toyengine_renderer::renderer::SurfaceContext) -> wgpu::RenderPipeline;
    fn draw(&mut self, engine: &mut Engine);
}

impl TriangleDemoAppTrait for TriangleDemoApp {
    fn build_pipe(ctx: &toyengine_renderer::renderer::SurfaceContext) -> wgpu::RenderPipeline {
        let device = ctx.device();
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tri shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(TRI_WGSL)),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tri layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tri pipe"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.color_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }

    fn draw(&mut self, engine: &mut Engine) {
        let ctx = engine.ctx_mut();
        let Some(pipe) = self.pipe.as_ref() else {
            return;
        };

        let (frame, view) = match ctx.frame_start() {
            Ok(v) => v,
            Err(FrameStartError::NoSurfaceSize) => return,
            Err(FrameStartError::Surface(wgpu::SurfaceError::OutOfMemory)) => panic!("oom"),
            Err(FrameStartError::Surface(_)) => return,
        };

        let mut encoder =
            ctx.device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("tri encoder"),
                });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tri pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.03,
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
            rp.set_pipeline(pipe);
            rp.draw(0..3, 0..1);
        }

        engine.ctx().queue().submit(Some(encoder.finish()));
        engine.ctx_mut().frame_show(frame);

        self.frames += 1;
    }
}

impl App for TriangleDemoApp {
    fn on_start(&mut self, engine: &mut Engine) {
        let pipe = <TriangleDemoApp as TriangleDemoAppTrait>::build_pipe(engine.ctx());
        self.pipe = Some(pipe);
    }

    fn on_render(&mut self, engine: &mut Engine) {
        self.draw(engine);
    }
}

fn main() {
    RunApp::run_app(
        AppConfig {
            title: "ToyEngine Triangle Demo",
            max_frames: Some(240),
            fixed_dt_seconds: Some(1.0 / 60.0),
        },
        TriangleDemoApp {
            pipe: None,
            frames: 0,
        },
    );
}

