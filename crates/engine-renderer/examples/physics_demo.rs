use engine_app::{App, AppConfig, Engine, EngineTrait, RunApp, RunAppTrait};
use winit::event::WindowEvent;
use engine_renderer::renderer::{
    MainRenderer, RendererTrait, SurfaceContextTrait,
};
use engine_renderer::loaders::{
    ModelLoader, ModelLoaderTrait, MaterialLoader, MaterialLoaderTrait, PipelineGenerator, PipelineGeneratorTrait
};
use engine_renderer::uniforms::*;
use engine_core::ecs::{Camera3D, Transform};
use engine_renderer::ecs::{MeshRenderable, EcsPointLight, CameraController, CameraPriority};
use engine_physics::{
    PhysicsContext, PhysicsContextTrait, 
    RigidBody, RigidBodyType, 
    Collider, ColliderShape, 
    PhysicsHandle,
    physics_world::{PhysicsConfig, update as physics_update},
};

use glam::{Quat, Vec3};

struct PhysicsDemoApp {
    frames: u32,
    main_renderer: Option<MainRenderer>,
}

impl App for PhysicsDemoApp {
    fn systems(&mut self) -> engine_app::SystemSchedule {
        let mut schedule = engine_app::SystemSchedule::new();
        engine_app::plugins::physics_plugin().build(&mut schedule);
        engine_app::plugins::render_plugin().build(&mut schedule);
        schedule
    }

    fn on_start(&mut self, engine: &mut Engine) {
        // Create renderer
        let mut renderer = MainRenderer::new(engine.ctx(), engine.window());

        // Load materials
        let device = engine.ctx().device();
        let queue = engine.ctx().queue();
        let format = engine.ctx().color_format();

        let pipeline_generator = PipelineGenerator::new("assets");

        match MaterialLoader::load_materials(
            device,
            queue,
            "assets/materials",
            &pipeline_generator,
            &renderer.global_layouts,
            &mut renderer.layout_cache,
            format,
            Some(wgpu::TextureFormat::Depth32Float)
        ) {
            Ok(resources) => {
                println!("Successfully loaded materials from config");
                renderer.material_cache.extend(resources.materials);
                renderer.shader_cache.extend(resources.shaders);
                renderer.texture_cache.extend(resources.textures);
            },
            Err(e) => eprintln!("Failed to load materials: {}", e),
        }

        // 创建相机实体（带轨道控制器）
        let world = engine.world_mut();
        world.spawn((
            Camera3D {
                position: Vec3::new(8.0, 6.0, 0.0),
                forward: Vec3::new(-1.0, -0.5, 0.0).normalize(),
            },
            CameraPriority(0),
            CameraController::default(),
        ));

        // Load model
        let device = engine.ctx().device();
        let queue = engine.ctx().queue();
        let model_path = "assets/models/monkey.glb";

        match ModelLoader::load_gltf(device, queue, model_path) {
            Ok(model) => {
                println!("Successfully loaded model: {}", model.name);
                let model_name = model.name.clone();
                renderer.model_cache.insert(model_name.clone(), std::sync::Arc::new(model));

                // Spawn ground (static physics body)
                let world = engine.world_mut();
                world.spawn((
                    Transform {
                        translation: Vec3::new(0.0, -1.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::new(10.0, 0.5, 10.0),
                    },
                    RigidBody {
                        body_type: RigidBodyType::Fixed,
                        additional_mass: None,
                        linear_damping: 0.0,
                        angular_damping: 0.0,
                        can_sleep: true,
                        ccd_enabled: false,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid { half_extents: Vec3::new(5.0, 0.25, 5.0) },
                        friction: 0.5,
                        restitution: 0.3,
                        density: 1.0,
                        sensor: false,
                    },
                    MeshRenderable {
                        mesh_id: model_name.clone(),
                        material_id: String::new(),
                    },
                ));

                // Spawn falling cube (dynamic physics body)
                world.spawn((
                    Transform {
                        translation: Vec3::new(0.0, 5.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::ONE,
                    },
                    RigidBody {
                        body_type: RigidBodyType::Dynamic,
                        additional_mass: Some(1.0),
                        linear_damping: 0.01,
                        angular_damping: 0.01,
                        can_sleep: false,
                        ccd_enabled: false,
                    },
                    Collider {
                        shape: ColliderShape::Ball { radius: 0.5 },
                        friction: 0.5,
                        restitution: 0.5,
                        density: 1.0,
                        sensor: false,
                    },
                    MeshRenderable {
                        mesh_id: model_name.clone(),
                        material_id: String::new(),
                    },
                ));

                // Spawn a second falling cube
                world.spawn((
                    Transform {
                        translation: Vec3::new(0.5, 7.0, 0.0),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::splat(0.5),
                    },
                    RigidBody {
                        body_type: RigidBodyType::Dynamic,
                        additional_mass: Some(0.5),
                        linear_damping: 0.01,
                        angular_damping: 0.01,
                        can_sleep: false,
                        ccd_enabled: false,
                    },
                    Collider {
                        shape: ColliderShape::Ball { radius: 0.25 },
                        friction: 0.5,
                        restitution: 0.7,
                        density: 1.0,
                        sensor: false,
                    },
                    MeshRenderable {
                        mesh_id: model_name.clone(),
                        material_id: String::new(),
                    },
                ));

                // Spawn point light
                world.spawn((
                    EcsPointLight {
                        position: Vec3::new(2.0, 5.0, 2.0),
                        range: 15.0,
                        color: Vec3::new(1.0, 1.0, 1.0),
                        intensity: 1.0,
                    },
                ));
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

    fn on_update(&mut self, engine: &mut Engine, dt_seconds: f32) {
        // 系统调度器会自动运行：物理/相机/网格等系统
        let _ = (engine, dt_seconds);
    }

    fn on_render(&mut self, engine: &mut Engine) {
        // 从 ECS 获取相机
        let camera = {
            let world = engine.world_mut();
            let mut q = world.query::<&Camera3D>();
            q.iter(world).next().copied()
        };

        let renderer = self.main_renderer.as_mut().unwrap();

        // Update Camera Uniform
        if let Some(camera) = camera {
            if let Some(camera_uniform) = renderer.uniform_cache.get("Camera Uniform")
                .and_then(|any| any.downcast_ref::<CameraGpuUniform>()) {

                camera_uniform.update(engine.ctx().queue(), camera, engine.ctx().config());
        }

        renderer.collect_from_world(engine.world_mut());
        renderer.render(engine.ctx_mut()).unwrap();
    }
}

fn main() {
    RunApp::run_app(
        AppConfig {
            title: "Physics Demo",
            max_frames: Some(24000),
            fixed_dt_seconds: Some(1.0 / 60.0),
        },
        PhysicsDemoApp {
            frames: 0,
            main_renderer: None,
        },
    );
}
