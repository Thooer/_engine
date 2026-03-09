use engine_app::{App, AppConfig, Engine, EngineTrait, RunApp, RunAppTrait};
use winit::event::WindowEvent;
use engine_renderer::renderer::{
    MainRenderer, RendererTrait, SurfaceContextTrait,
};
use engine_renderer::graphics::{
    ModelLoader, ModelLoaderTrait, MaterialLoader, MaterialLoaderTrait, PipelineGenerator, PipelineGeneratorTrait
};
use engine_renderer::uniforms::{*};
// use engine_core::camera::camera3d_fly_wasd;
use engine_core::ecs::{Camera3D, Transform, World};

use glam::{Mat4, Quat, Vec3};

// const MAX_FRAMES: u32 = 600;

struct SceneDemoApp {
    #[allow(dead_code)]
    frames: u32,
    main_renderer: Option<MainRenderer>,
}

impl App for SceneDemoApp {
    fn on_start(&mut self, engine: &mut Engine) {
        let mut renderer = MainRenderer::new(engine.ctx(), engine.window());
        
        // 测试 Material System
        let device = engine.ctx().device();
        let queue = engine.ctx().queue();
        let format = engine.ctx().color_format();
        
        // 假设 assets 目录在当前工作目录下
        let pipeline_generator = PipelineGenerator::new("assets");
        
        match MaterialLoader::load_materials(
            device, 
            queue, 
            "assets/materials/default.toml", 
            &pipeline_generator,
            format,
            Some(wgpu::TextureFormat::Depth32Float) // 假设深度格式
        ) {
            Ok(resources) => {
                println!("Successfully loaded materials from config");
                renderer.material_cache.extend(resources.materials);
                renderer.shader_cache.extend(resources.shaders);
                renderer.texture_cache.extend(resources.textures);
            },
            Err(e) => eprintln!("Failed to load materials: {}", e),
        }

        // 测试 glTF 加载
        let device = engine.ctx().device();
        let queue = engine.ctx().queue();
        let model_path = "assets/models/monkey.glb";
        
        match ModelLoader::load_gltf(device, queue, model_path) {
            Ok(model) => {
                println!("Successfully loaded model: {}", model.name);
                println!("  Meshes: {}", model.meshes.len());
                println!("  Materials: {:?}", model.material_names);
                println!("  Root Nodes: {}", model.root_nodes.len());
                
                // Add to renderer
                renderer.model_cache.insert(model.name.clone(), std::sync::Arc::new(model));
                renderer.collect_render_objects();
            },
            Err(e) => {
                eprintln!("Failed to load model '{}': {}", model_path, e);
            }
        }
        
        self.main_renderer = Some(renderer);
    }

    fn on_window_event(&mut self, engine: &mut Engine, event: &WindowEvent) {
        if let Some(renderer) = self.main_renderer.as_mut() {
            renderer.handle_event(engine.window(), event);
        }
    }

    fn on_render(&mut self, engine: &mut Engine) {
        self.frames += 1;
        let theta = self.frames as f32 * 0.01;
        let camera = Camera3D {
            position: Vec3::new(theta.cos() * 5.0, 5.0, theta.sin() * 5.0),
            forward: Vec3::new(-theta.cos(), -1.0, -theta.sin()).normalize(),
        };

        let renderer = self.main_renderer.as_mut().unwrap();

        // Update Camera Uniform
        if let Some(camera_uniform) = renderer.uniform_cache.get("Camera Uniform")
            .and_then(|any| any.downcast_ref::<CameraGpuUniform>()) {
            
            camera_uniform.update(engine.ctx().queue(), &camera, engine.ctx().config());
        }
        
        renderer.collect_render_objects();
        renderer.render(engine.ctx_mut()).unwrap();
    }
}

fn main() {
    RunApp::run_app(
        AppConfig {
            title: "Scene Demo",
            max_frames: Some(24000),
            fixed_dt_seconds: Some(1.0 / 60.0),
        },
        SceneDemoApp {
            frames: 0,
            main_renderer: None,
        },
    );
}