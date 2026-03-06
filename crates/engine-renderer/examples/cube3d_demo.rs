use engine_app::{App, AppConfig, Engine, EngineTrait, RunApp, RunAppTrait};
use engine_renderer::renderer::{
    create_simple_mesh3d_resources, draw_simple_mesh3d_pass, SimpleMesh3DPassConfig,
    SimpleMesh3DResources, SurfaceContextTrait, SurfaceSize,
};
use engine_core::camera::camera3d_fly_wasd;
use engine_core::ecs::{Camera3D, Transform, World};

use glam::{Mat4, Quat, Vec3};

const MAX_FRAMES: u32 = 600;

struct Cube3DDemo {
    mesh: Option<SimpleMesh3DResources>,
    depth_texture: Option<wgpu::TextureView>,
    world: World,
}

trait Cube3DDemoTrait {
    fn new() -> Self;
    fn update_camera(&mut self, engine: &Engine, dt: f32);
    fn create_depth_texture_internal(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> wgpu::TextureView;
    fn draw(&mut self, engine: &mut Engine);
}

impl Cube3DDemoTrait for Cube3DDemo {
    fn new() -> Self {
        let mut world = World::new();

        // 初始化 3D 相机实体
        world.spawn((
            Camera3D {
                position: Vec3::new(5.0, 5.0, 5.0),
                // 固定一个初始朝向，不再每帧自动看向原点，避免“绕物体旋转”的感觉
                forward: Vec3::new(-1.0, -1.0, -1.0).normalize(),
            },
        ));

        // 在场景中放置若干立方体实体，仅使用 Transform 组件记录位置
        let cube_positions = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(-2.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, -2.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 0.0, -2.0),
        ];

        for pos in cube_positions {
            world.spawn((
                Transform {
                    translation: pos,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            ));
        }

        Self {
            mesh: None,
            depth_texture: None,
            world,
        }
    }

    /// 根据键盘输入更新 3D 相机位置（委托给核心层的通用相机控制函数）。
    fn update_camera(&mut self, engine: &Engine, dt: f32) {
        let speed = 5.0; // 世界单位 / 秒
        camera3d_fly_wasd(&mut self.world, engine.input(), dt, speed);
    }

    /// 创建一个与给定尺寸匹配的深度纹理视图。
    fn create_depth_texture_internal(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cube3d depth texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn draw(&mut self, engine: &mut Engine) {
        // 先计算MVP矩阵并更新uniforms
        let size = engine.ctx().size();
        let aspect = size.width as f32 / size.height as f32;

        // 从 ECS 世界中读取当前相机数据
        let (camera_pos, camera_forward) = if let Ok(cam) =
            self.world.query::<&Camera3D>().single(&self.world)
        {
            (cam.position, cam.forward)
        } else {
            (Vec3::new(5.0, 5.0, 5.0), Vec3::new(-1.0, -1.0, -1.0).normalize())
        };

        // 视图矩阵：使用固定的朝向向量，这样相机平移时画面主要是“平移/拉近”，而不是绕物体旋转
        let view_matrix =
            Mat4::look_to_rh(camera_pos, camera_forward, Vec3::Y);

        // 投影矩阵：透视投影
        let projection =
            Mat4::perspective_rh(std::f32::consts::PI / 4.0, aspect, 0.1, 100.0);

        let (Some(mesh), Some(depth_view)) = (self.mesh.as_ref(), self.depth_texture.as_ref())
        else {
            return;
        };

        // 提前克隆一个队列句柄，避免在可变借用 ctx 的同时，在闭包中再次以不可变方式捕获 ctx
        let queue = engine.ctx().queue().clone();

        // 使用渲染模块提供的通用 3D 网格绘制通路，集中处理 frame_start / render_pass / 提交等细节。
        draw_simple_mesh3d_pass(
            engine.ctx_mut(),
            depth_view,
            SimpleMesh3DPassConfig {
                clear_color: wgpu::Color {
                    r: 0.1,
                    g: 0.1,
                    b: 0.15,
                    a: 1.0,
                },
                depth_clear: 1.0,
            },
            |rp| {
                rp.set_pipeline(&mesh.pipeline);
                rp.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
                rp.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint16);

                // 从 ECS 世界中收集所有带 Transform 的实体，逐个绘制立方体
                let mut query = self.world.query::<&Transform>();
                for transform in query.iter(&self.world) {
                    // 计算每个立方体的模型矩阵
                    let model = Mat4::from_translation(transform.translation);
                    let mvp: Mat4 = projection * view_matrix * model;

                    // 更新 MVP uniform
                    let mvp_array: [f32; 16] = mvp.to_cols_array();
                    let mvp_bytes: &[u8] = bytemuck::cast_slice(&mvp_array);
                    queue.write_buffer(&mesh.uniform_buf, 0, mvp_bytes);

                    rp.set_bind_group(0, &mesh.uniform_bind_group, &[]);
                    rp.draw_indexed(0..mesh.index_count, 0, 0..1);
                }
            },
        );
    }
}

impl App for Cube3DDemo {
    fn on_start(&mut self, engine: &mut Engine) {
        let mesh = create_simple_mesh3d_resources(engine.ctx().device(), engine.ctx().color_format());
        let depth_view = <Cube3DDemo as Cube3DDemoTrait>::create_depth_texture_internal(
            engine.ctx().device(),
            engine.ctx().size().width,
            engine.ctx().size().height,
        );

        self.mesh = Some(mesh);
        self.depth_texture = Some(depth_view);
    }

    fn on_resize(&mut self, engine: &mut Engine, new_size: SurfaceSize) {
        let depth_view =
            <Cube3DDemo as Cube3DDemoTrait>::create_depth_texture_internal(engine.ctx().device(), new_size.width, new_size.height);
        self.depth_texture = Some(depth_view);
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
            title: "ToyEngine 3D Cube Demo",
            max_frames: Some(MAX_FRAMES),
            fixed_dt_seconds: Some(1.0 / 60.0),
        },
        <Cube3DDemo as Cube3DDemoTrait>::new(),
    );
}
