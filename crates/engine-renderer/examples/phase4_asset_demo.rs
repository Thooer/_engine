use std::f32::consts::PI;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use toyengine_app::{App, AppConfig, Engine, EngineTrait, RunApp, RunAppTrait};
use toyengine_core::fs::LocalFileSystem;
use toyengine_core::resources::{AssetManager, AssetManagerExt, MeshAsset};
use toyengine_renderer::renderer::{
    SimpleMesh2DPassConfig, SimpleMeshPipeline2D, SimpleMeshPipeline2DPipeline, SurfaceContext,
    SurfaceContextTrait, draw_simple_mesh2d_pass,
};
use wgpu::util::DeviceExt;

/// Phase 4 Demo 的最小配置项
///
/// 不做外部文件加载，先把“可调整参数”集中到一处，避免散落硬编码。
struct Phase4DemoConfig {
    /// 网格资源路径
    mesh_path: &'static str,
    /// 清屏颜色
    clear_color: wgpu::Color,
    /// 可选的最大帧数（None 表示一直运行）
    max_frames: Option<u32>,
}

const PHASE4_DEMO_CONFIG: Phase4DemoConfig = Phase4DemoConfig {
    mesh_path: "assets/phase4_mesh.ron",
    clear_color: wgpu::Color {
        r: 0.02,
        g: 0.02,
        b: 0.03,
        a: 1.0,
    },
    max_frames: Some(600),
};

/// 顶点数据：从 MeshAsset 转换而来
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
}

struct Phase4AssetDemoApp {
    pipe: Option<SimpleMeshPipeline2DPipeline>,
    index_buf: Option<wgpu::Buffer>,
    index_count: u32,
    /// 原始模型顶点（用于在 CPU 上做硬编码运动）
    base_vertices: Vec<Vertex>,
    frames: u32,
}

trait Phase4AssetDemoAppTrait {
    fn new() -> Self;
    fn build_resources(ctx: &SurfaceContext) -> (SimpleMeshPipeline2DPipeline, Arc<MeshAsset>);
    fn animated_vertices(&self) -> Vec<Vertex>;
    fn draw(&mut self, engine: &mut Engine);
}

impl Phase4AssetDemoAppTrait for Phase4AssetDemoApp {
    fn new() -> Self {
        Self {
            pipe: None,
            index_buf: None,
            index_count: 0,
            // 原始模型顶点（用于在 CPU 上做硬编码运动）
            base_vertices: Vec::new(),
            frames: 0,
        }
    }

    fn build_resources(ctx: &SurfaceContext) -> (SimpleMeshPipeline2DPipeline, Arc<MeshAsset>) {
        let device = ctx.device();
        let pipe = <SimpleMeshPipeline2DPipeline as SimpleMeshPipeline2D>::new(
            device,
            ctx.color_format(),
            std::mem::size_of::<Vertex>() as u64,
        );

        // 注意：这里假设以工作目录为仓库根目录运行 demo
        // 使用核心层的 AssetManager + LocalFileSystem 来加载并缓存资源
        let fs = LocalFileSystem::default();
        let mut am = AssetManager::new(fs);
        let asset = am
            .load_mesh(std::path::Path::new(PHASE4_DEMO_CONFIG.mesh_path))
            .expect("failed to load mesh asset");

        (pipe, asset)
    }

    /// 根据当前帧数，为模型做一个简单的旋转动画（硬编码在 demo 中）
    fn animated_vertices(&self) -> Vec<Vertex> {
        if self.base_vertices.is_empty() {
            return Vec::new();
        }

        let t = self.frames as f32 / 60.0;
        let angle = t * PI * 0.5; // 每秒旋转 90°
        let (s, c) = angle.sin_cos();

        self.base_vertices
            .iter()
            .map(|v| {
                let x = v.position[0];
                let y = v.position[1];
                let xr = x * c - y * s;
                let yr = x * s + y * c;
                Vertex {
                    position: [xr, yr],
                }
            })
            .collect()
    }

    fn draw(&mut self, engine: &mut Engine) {
        if self.index_count == 0 || self.base_vertices.is_empty() {
            return;
        }

        // 先根据当前帧数在 CPU 上算出动画后的顶点，避免与下面对 ctx 的可变借用冲突。
        let anim_vertices = self.animated_vertices();

        let ctx = engine.ctx_mut();
        let (Some(pipe), Some(index_buf)) = (self.pipe.as_ref(), self.index_buf.as_ref()) else {
            return;
        };

        // 每帧上传一份新的顶点缓冲，实现“硬编码”的旋转动画。
        let vertex_buf = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("phase4 vertex buffer"),
                contents: bytemuck::cast_slice(&anim_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // 使用渲染模块提供的通用 2D 网格绘制通路。
        draw_simple_mesh2d_pass(
            ctx,
            pipe,
            &vertex_buf,
            index_buf,
            self.index_count,
            SimpleMesh2DPassConfig {
                clear_color: PHASE4_DEMO_CONFIG.clear_color,
            },
        );

        self.frames += 1;
    }
}

impl App for Phase4AssetDemoApp {
    fn on_start(&mut self, engine: &mut Engine) {
        let (pipe, asset) = <Phase4AssetDemoApp as Phase4AssetDemoAppTrait>::build_resources(engine.ctx());

        let base_vertices: Vec<Vertex> = asset
            .positions
            .iter()
            .map(|(x, y)| Vertex { position: [*x, *y] })
            .collect();
        let index_buf = engine
            .ctx_mut()
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("phase4 index buffer"),
                contents: bytemuck::cast_slice(&asset.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.pipe = Some(pipe);
        self.index_buf = Some(index_buf);
        self.index_count = asset.indices.len() as u32;
        self.base_vertices = base_vertices;
    }

    fn on_render(&mut self, engine: &mut Engine) {
        self.draw(engine);
    }
}

fn main() {
    RunApp::run_app(
        AppConfig {
            title: "ToyEngine Phase 4 Asset Demo",
            max_frames: PHASE4_DEMO_CONFIG.max_frames,
            fixed_dt_seconds: Some(1.0 / 60.0),
        },
        <Phase4AssetDemoApp as Phase4AssetDemoAppTrait>::new(),
    );
}

