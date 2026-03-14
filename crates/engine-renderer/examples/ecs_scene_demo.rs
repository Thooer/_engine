use std::borrow::Cow;

use toyengine_app::{App, AppConfig, Engine, EngineTrait, RunApp, RunAppTrait};
use toyengine_core::ecs::{Camera2D, Renderable, Transform, World};
use toyengine_renderer::materials::{InstanceBatch2D, Mesh2D};
use toyengine_renderer::renderer::{
    FrameStartError, InstanceColorMeshPipeline2D, InstanceColorMeshPipeline2DPipeline,
    SurfaceContext, SurfaceContextTrait,
};

use bytemuck::{Pod, Zeroable};
use glam::{Quat, Vec3};
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;

/// 顶点数据：一个单位三角形
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
}

const VERTICES: [Vertex; 3] = [
    Vertex {
        position: [0.0, 0.5],
    },
    Vertex {
        position: [-0.5, -0.5],
    },
    Vertex {
        position: [0.5, -0.5],
    },
];

/// 实例数据：每个实体的偏移与颜色
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InstanceData {
    offset: [f32; 2],
    color: [f32; 3],
}

const ECS_TRI_WGSL: &str = r#"
struct Instance {
    @location(1) offset: vec2<f32>,
    @location(2) color: vec3<f32>,
};

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    inst: Instance,
) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(position + inst.offset, 0.0, 1.0);
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

struct EcsSceneDemoApp {
    pipe: Option<InstanceColorMeshPipeline2DPipeline>,
    mesh: Option<Mesh2D>,
    world: World,
    frames: u32,
}

trait EcsSceneDemoAppTrait {
    fn new() -> Self;
    fn build_resources(ctx: &SurfaceContext) -> (InstanceColorMeshPipeline2DPipeline, Mesh2D);
    fn collect_instances(&mut self) -> Vec<InstanceData>;
    fn update_camera(&mut self, engine: &Engine, dt: f32);
    fn draw(&mut self, engine: &mut Engine);
}

impl EcsSceneDemoAppTrait for EcsSceneDemoApp {
    fn new() -> Self {
        let mut world = World::new();

        // 左侧红色三角形
        world.spawn((
            Transform {
                translation: Vec3::new(-0.5, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Renderable {
                color: Vec3::new(1.0, 0.2, 0.2),
            },
        ));

        // 右侧绿色三角形
        world.spawn((
            Transform {
                translation: Vec3::new(0.5, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            Renderable {
                color: Vec3::new(0.2, 1.0, 0.2),
            },
        ));

        // 简单 2D 相机，默认位于世界原点
        world.spawn(Camera2D::default());

        Self {
            pipe: None,
            mesh: None,
            world,
            frames: 0,
        }
    }

    fn build_resources(ctx: &SurfaceContext) -> (InstanceColorMeshPipeline2DPipeline, Mesh2D) {
        let device = ctx.device();

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ecs tri vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let pipe = <InstanceColorMeshPipeline2DPipeline as InstanceColorMeshPipeline2D>::new(
            device,
            ctx.color_format(),
            std::mem::size_of::<Vertex>() as u64,
            std::mem::size_of::<InstanceData>() as u64,
        );

        let mesh = Mesh2D {
            vertex: vertex_buf,
            vertex_count: VERTICES.len() as u32,
        };

        (pipe, mesh)
    }

    fn collect_instances(&mut self) -> Vec<InstanceData> {
        let mut result = Vec::new();
        let mut query = self.world.query::<(&Transform, &Renderable)>();

        // 目前仅支持单一 2D 相机，若未找到则视为原点相机。
        let mut camera_pos = Vec3::ZERO;
        if let Ok(mut q_cam) = self.world.query::<&Camera2D>().single(&self.world) {
            camera_pos = q_cam.position;
        }

        for (transform, renderable) in query.iter(&self.world) {
            // 在 CPU 侧做一个最简单的 2D 相机平移：实体位置减去相机位置。
            let world_pos = transform.translation - camera_pos;
            result.push(InstanceData {
                offset: [world_pos.x, world_pos.y],
                color: [renderable.color.x, renderable.color.y, renderable.color.z],
            });
        }

        result
    }

    /// 简单 2D 相机控制：WASD 平移
    fn update_camera(&mut self, engine: &Engine, dt: f32) {
        use winit::keyboard::KeyCode;
        use toyengine_core::input::InputState;

        let speed = 1.0; // 世界单位 / 秒

        // 从 ECS World 获取 InputState
        let input = match engine.world().get_resource::<InputState>() {
            Some(i) => i,
            None => return,
        };

        let mut dir = Vec3::ZERO;
        if input.is_pressed(KeyCode::KeyW) {
            dir.y += 1.0;
        }
        if input.is_pressed(KeyCode::KeyS) {
            dir.y -= 1.0;
        }
        if input.is_pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if input.is_pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }

        if dir.length_squared() == 0.0 {
            return;
        }

        let dir = dir.normalize() * speed * dt;

        if let Ok(mut cam) = self
            .world
            .query::<&mut Camera2D>()
            .single_mut(&mut self.world)
        {
            cam.position += dir;
        }
    }

    fn draw(&mut self, engine: &mut Engine) {
        let instances = self.collect_instances();
        if instances.is_empty() {
            return;
        }

        let ctx = engine.ctx_mut();
        let (Some(pipe), Some(mesh)) = (self.pipe.as_ref(), self.mesh.as_ref()) else {
            return;
        };

        let instance_buf = ctx.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ecs tri instance buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let batch = InstanceBatch2D {
            buffer: instance_buf,
            count: instances.len() as u32,
        };

        let (frame, view) = match ctx.frame_start() {
            Ok(v) => v,
            Err(FrameStartError::NoSurfaceSize) => return,
            Err(FrameStartError::Surface(wgpu::SurfaceError::OutOfMemory)) => panic!("oom"),
            Err(FrameStartError::Surface(_)) => return,
        };

        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ecs tri encoder"),
            });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ecs tri pass"),
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

            pipe.draw(
                &mut rp,
                &mesh.vertex,
                &batch.buffer,
                batch.count,
                mesh.vertex_count,
            );
        }

        engine.ctx().queue().submit(Some(encoder.finish()));
        engine.ctx_mut().frame_show(frame);

        self.frames += 1;
    }
}

impl App for EcsSceneDemoApp {
    fn on_start(&mut self, engine: &mut Engine) {
        let (pipe, mesh) = <EcsSceneDemoApp as EcsSceneDemoAppTrait>::build_resources(engine.ctx());
        self.pipe = Some(pipe);
        self.mesh = Some(mesh);
    }

    fn on_update(&mut self, engine: &mut Engine, dt_seconds: f32) {
        self.update_camera(engine, dt_seconds);
    }

    fn on_render(&mut self, engine: &mut Engine) {
        self.draw(engine);
    }
}

fn main() {
    RunApp::run_app(
        AppConfig {
            title: "ToyEngine ECS Scene Demo",
            max_frames: Some(240),
            fixed_dt_seconds: Some(1.0 / 60.0),
        },
        <EcsSceneDemoApp as EcsSceneDemoAppTrait>::new(),
    );
}

