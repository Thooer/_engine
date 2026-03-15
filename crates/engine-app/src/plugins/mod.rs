//! ToyEngine 插件系统 - 将各模块的系统注册到调度器
//!
//! 提供统一的系统注册接口，让应用可以一站式添加所有需要的系统
//!
//! 迁移公告：
//! - 核心插件 trait 已移至 `engine_core::plugins`
//! - 此模块保留物理和渲染插件的实现

pub use engine_core::plugins::{Plugin, PluginContext, PluginRegistry, ScheduleType};

use bevy_ecs::prelude::World;

use engine_physics::physics_world::{self, PhysicsConfig as EnginePhysicsConfig};
use engine_physics::{PhysicsContext, PhysicsContextTrait, reset_on_keypress_system};

use engine_renderer::grid::spawn_grid_system;
use engine_renderer::renderer::{MainRenderer, RendererTrait, SurfaceContextTrait};
use engine_renderer::loaders::{MaterialLoader, MaterialLoaderTrait, PipelineGenerator, PipelineGeneratorTrait};
use engine_renderer::uniforms::{CameraGpuUniform, CameraGpuUniformTrait};
use engine_core::ecs::Camera3D;
use engine_scene::load_scene;

use crate::{SystemSchedule, SystemStage, Engine, EngineTrait};

// ============================================================================
// 物理插件
// ============================================================================

/// 物理系统插件 - 包含物理模拟所需的所有系统
pub struct PhysicsPlugin {
    config: EnginePhysicsConfig,
    gravity: Option<f32>,
}

impl PhysicsPlugin {
    pub fn new() -> Self {
        Self {
            config: EnginePhysicsConfig::default_config(),
            gravity: None, // 使用默认值 -9.81
        }
    }

    pub fn with_config(config: EnginePhysicsConfig) -> Self {
        Self { config, gravity: None }
    }

    /// 设置重力（Y轴分量）
    pub fn with_gravity(mut self, gravity_y: f32) -> Self {
        self.gravity = Some(gravity_y);
        self
    }

    /// 构建系统到调度器
    pub fn build(&self, schedule: &mut SystemSchedule) {
        // 克隆配置，供 setup 使用
        let config = self.config.clone();
        let gravity = self.gravity;

        // Startup: 初始化物理 Context 和实体
        schedule.add_system(
            move |world: &mut World| {
                // 插入 PhysicsContext
                let mut ctx = PhysicsContext::new();
                // 如果指定了重力，覆盖默认值
                if let Some(g) = gravity {
                    ctx.set_gravity(glam::Vec3::new(0.0, g, 0.0));
                }
                world.insert_resource(ctx);

                // 插入 PhysicsConfig
                world.insert_resource(config.clone());

                // 初始化物理实体
                physics_world::init_bodies(world);
                
                // 设置已初始化
                if let Some(mut cfg) = world.get_resource_mut::<EnginePhysicsConfig>() {
                    cfg.initialized = true;
                }
            },
            SystemStage::Startup,
        );
        
        // FixedUpdate: 物理步进 - 包装 ECS 系统
        // 注意：必须先同步 Kinematic 刚体位置（ECS → 物理），然后再步进
        schedule.add_system(
            |world: &mut World| {
                // 1. 先同步 Kinematic 刚体位置（ECS → 物理）
                physics_world::sync_kinematic_bodies(world);

                // 2. 执行物理步进
                if let Some(mut ctx) = world.get_resource_mut::<PhysicsContext>() {
                    ctx.step(1.0 / 60.0);
                }
            },
            SystemStage::FixedUpdate,
        );
        
        // PostUpdate: 同步物理位置到 ECS Transform
        // 使用 physics_world::sync_transforms 函数
        schedule.add_system(
            physics_world::sync_transforms,
            SystemStage::PostUpdate,
        );
        
        // Update: 按键触发重置系统
        schedule.add_system(
            reset_on_keypress_system,
            SystemStage::Update,
        );
    }
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// 构建物理插件的辅助函数
pub fn physics_plugin() -> PhysicsPlugin {
    PhysicsPlugin::new()
}

// ============================================================================
// 渲染插件
// ============================================================================

/// 渲染插件配置
#[derive(Debug, Clone, Default)]
pub struct RenderConfig {
    /// 材质目录路径
    pub materials_path: Option<String>,
    /// 模型目录路径
    pub models_path: Option<String>,
    /// 场景文件路径
    pub scene_path: Option<String>,
    /// 是否启用默认网格地面
    pub enable_grid: bool,
}

impl RenderConfig {
    pub fn new() -> Self {
        Self {
            materials_path: None,
            models_path: Some("assets/models".to_string()),
            scene_path: None,
            enable_grid: true,
        }
    }

    pub fn with_materials(mut self, path: impl Into<String>) -> Self {
        self.materials_path = Some(path.into());
        self
    }

    pub fn with_scene(mut self, path: impl Into<String>) -> Self {
        self.scene_path = Some(path.into());
        self
    }

    pub fn with_grid(mut self, enable: bool) -> Self {
        self.enable_grid = enable;
        self
    }
}

/// 渲染系统插件 - 包含渲染相关的系统
/// 
/// 使用方式:
/// ```rust
/// fn systems(&mut self) -> SystemSchedule {
///     let mut schedule = SystemSchedule::new();
///     engine_app::plugins::render_plugin()
///         .with_materials("assets/materials")
///         .with_scene("assets/scenes/main.ron")
///         .build(&mut schedule, self);
///     schedule
/// }
/// ```
pub struct RenderPlugin {
    config: RenderConfig,
}

impl RenderPlugin {
    pub fn new() -> Self {
        Self {
            config: RenderConfig::new(),
        }
    }

    pub fn with_materials(mut self, path: impl Into<String>) -> Self {
        self.config.materials_path = Some(path.into());
        self
    }

    pub fn with_models(mut self, path: impl Into<String>) -> Self {
        self.config.models_path = Some(path.into());
        self
    }

    pub fn with_scene(mut self, path: impl Into<String>) -> Self {
        self.config.scene_path = Some(path.into());
        self
    }

    pub fn with_grid(mut self, enable: bool) -> Self {
        self.config.enable_grid = enable;
        self
    }

    /// 构建系统到调度器
    /// 
    /// 此方法会自动:
    /// 1. 在 Startup 阶段添加网格生成系统 (可选)
    /// 2. 在 Update 阶段添加相机更新系统
    /// 3. 在 Render 阶段添加渲染收集系统
    /// 4. 返回配置供 on_start 中使用
    pub fn build(&self, schedule: &mut SystemSchedule) {
        // Startup: 生成网格地面 (可选)
        if self.config.enable_grid {
            schedule.add_system(
                spawn_grid_system,
                SystemStage::Startup,
            );
        }

        // 注意: 渲染收集系统需要在 on_start 后配置 (因为需要 renderer 实例)
    }

    /// 配置渲染收集系统 (需要在 on_start 中调用)
    /// 
    /// # 参数
    /// - renderer: 渲染器实例
    /// - schedule: 系统调度器
    pub fn setup_collect_system(mut renderer: MainRenderer, schedule: &mut SystemSchedule) {
        // Render: 渲染收集 (在实际渲染之前收集 ECS 数据)
        schedule.add_system(
            move |world: &mut World| {
                // 更新相机 uniform
                let queue = renderer.queue();
                
                if let Some(camera) = world.query::<&Camera3D>().iter(world).next() {
                    if let Some(cu) = renderer.uniform_cache.get("Camera Uniform")
                        .and_then(|a| a.downcast_ref::<CameraGpuUniform>()) 
                    {
                        cu.update(queue, camera, &renderer.get_surface_config());
                    }
                }

                // 收集 ECS 渲染对象
                engine_renderer::loaders::collect_from_world(world, &mut renderer);
            },
            SystemStage::Render,
        );
    }

    /// 完全自动初始化 (在 on_start 中调用)
    /// 
    /// 此方法会:
    /// 1. 创建渲染器
    /// 2. 加载材质 (如果配置了 materials_path)
    /// 3. 加载场景 (如果配置了 scene_path)
    /// 4. 配置渲染收集系统
    /// 初始化渲染器并存储到 Engine
    pub fn setup(&self, engine: &mut Engine) {
        let ctx = engine.ctx();
        
        // 克隆 GPU 资源以避免借用冲突
        let device = ctx.device().clone();
        let queue = ctx.queue().clone();
        let format = ctx.color_format();
        
        // 1. 创建渲染器（使用配置的 models 路径）
        let models_path = self.config.models_path.as_deref().unwrap_or("assets/models");
        let mut renderer = MainRenderer::new(ctx, engine.window(), models_path);

        // 2. 加载材质
        if let Some(materials_path) = &self.config.materials_path {
            tracing::info!("Loading materials from: {}", materials_path);
            let pipeline_generator = <PipelineGenerator as PipelineGeneratorTrait>::new("assets");
            match MaterialLoader::load_materials(
                &device, &queue, materials_path,
                &pipeline_generator,
                &renderer.global_layouts,
                &mut renderer.layout_cache,
                format,
                Some(wgpu::TextureFormat::Depth32Float)
            ) {
                Ok(resources) => {
                    tracing::info!("Loaded {} materials", resources.materials.len());
                    renderer.material_cache.extend(resources.materials);
                    renderer.shader_cache.extend(resources.shaders);
                    renderer.texture_cache.extend(resources.textures);
                }
                Err(e) => {
                    tracing::error!("Failed to load materials: {}", e);
                }
            }
        }

        // 3. 加载场景
        // 注意：不再需要传入 GPU 设备，渲染器会按需自动加载模型
        if let Some(scene_path) = &self.config.scene_path {
            let world = engine.world_mut();
            if let Err(e) = engine_scene::load_scene(scene_path, world) {
                tracing::error!("Failed to load scene: {:?}", e);
            }
        }

        // 4. 存储渲染器到 Engine
        engine.main_renderer = Some(renderer);
    }

    /// 获取配置引用
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    /// 消耗自身并返回配置
    pub fn into_config(self) -> RenderConfig {
        self.config
    }
}

impl Default for RenderPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// 构建渲染插件的辅助函数
pub fn render_plugin() -> RenderPlugin {
    RenderPlugin::new()
}
