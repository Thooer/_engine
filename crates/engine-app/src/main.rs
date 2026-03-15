//! ToyEngine 主入口
//!
//! 运行方式:
//!   cargo run -p engine-app                      # 运行默认项目（从配置读取）
//!   cargo run -p engine-app -- --project ./demos/simple_demo
//!   cargo run -p engine-app -- --set-default ./demos/mc

use std::path::PathBuf;
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use engine_app::{App, AppConfig, Engine, EngineTrait, RunApp, RunAppTrait, WasmRuntime};
use engine_app::plugins::{render_plugin, physics_plugin};
use engine_app::SystemSchedule;
use engine_core::input::{InputCode, InputStateExt};
use engine_platform::config::{ProjectConfig, ConfigLoader};
use engine_renderer::grid::spawn_grid_system;
use rfd::FileDialog;
use egui::Context;
use serde::Deserialize;
use toml;

/// UI 与 App 之间的共享状态，用于触发项目热重载
pub struct SharedState {
    pub pending_project: Arc<Mutex<Option<PathBuf>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            pending_project: Arc::new(Mutex::new(None)),
        }
    }
}

/// 获取引擎配置目录（引擎根目录）
fn get_engine_config_dir() -> PathBuf {
    // 使用当前工作目录（开发时为引擎根目录）
    // 发布时可改为相对于可执行文件的路径
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 引擎全局配置
#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
struct GlobalConfig {
    /// 默认项目路径（相对于引擎根目录）
    #[serde(default)]
    default: Option<GlobalDefaultConfig>,
}

/// 全局配置中的默认项目子配置
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
struct GlobalDefaultConfig {
    /// 默认项目路径
    project: Option<String>,
}

impl GlobalConfig {
    const CONFIG_FILE: &'static str = "config.toml";

    /// 从配置文件加载
    fn load() -> Self {
        let config_path = get_engine_config_dir().join(Self::CONFIG_FILE);
        if config_path.exists() {
            if let Ok(contents) = fs::read_to_string(&config_path) {
                // 使用 serde 反序列化
                if let Ok(config) = toml::from_str::<GlobalConfig>(&contents) {
                    return config;
                }
            }
        }
        Self::default()
    }

    /// 保存到配置文件
    fn save(&self) -> io::Result<()> {
        let config_dir = get_engine_config_dir();
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join(Self::CONFIG_FILE);

        let contents = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        fs::write(config_path, contents)?;
        Ok(())
    }

    /// 设置默认项目
    fn set_default_project(&mut self, project_path: &str) {
        self.default = Some(GlobalDefaultConfig {
            project: Some(project_path.to_string()),
        });
    }

    /// 获取默认项目路径
    fn default_project(&self) -> Option<String> {
        self.default.as_ref().and_then(|d| d.project.clone())
    }
}

/// ToyEngine 应用程序
struct ToyEngineApp {
    project_path: PathBuf,
    project_config: ProjectConfig,
    wasm_runtime: Option<WasmRuntime>,
    shared_state: SharedState,
}

impl ToyEngineApp {
    fn new(project_path: PathBuf, shared_state: SharedState) -> Self {
        let project_config = ProjectConfig::load_or_default(&project_path);
        tracing::info!("Loaded project: {}", project_config.project_name());
        tracing::info!("  scene: {}", project_config.run.scene);
        tracing::info!("  assets: {}", project_config.run.assets_dir);
        if let Some(ref script) = project_config.run.script {
            tracing::info!("  script: {}", script);
        }

        // 尝试加载 WASM 脚本
        let mut wasm_runtime: Option<WasmRuntime> = None;
        if let Some(ref script_path) = project_config.run.script {
            let script_abs = project_path.join(script_path);
            if script_abs.exists() {
                match WasmRuntime::new() {
                    Ok(mut runtime) => {
                        match runtime.load(&script_abs) {
                            Ok(_) => {
                                tracing::info!("WASM script loaded successfully");
                                wasm_runtime = Some(runtime);
                            }
                            Err(e) => {
                                tracing::error!("Failed to load WASM script: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create WASM runtime: {}", e);
                    }
                }
            } else {
                tracing::warn!("WASM script not found: {:?}", script_abs);
            }
        }

        Self {
            project_path,
            project_config,
            wasm_runtime,
            shared_state,
        }
    }

    /// 重新加载项目（热重载）
    fn reload_project(&mut self, new_project_path: PathBuf, engine: &mut Engine) {
        tracing::info!("Reloading project to: {:?}", new_project_path);

        // 1. 清空 ECS 世界中的实体
        {
            let mut world = engine.world_mut();
            let entities: Vec<bevy_ecs::entity::Entity> = world.query::<bevy_ecs::entity::Entity>()
                .iter(&world)
                .collect();
            let count = entities.len();
            for entity in &entities {
                world.despawn(*entity);
            }
            tracing::info!("Cleared {} entities from ECS world", count);
        }

        // 2. 重新加载项目配置
        self.project_path = new_project_path.clone();
        self.project_config = ProjectConfig::load_or_default(&new_project_path);

        // 3. 重新加载 WASM 脚本
        self.wasm_runtime = None;
        if let Some(ref script_path) = self.project_config.run.script {
            let script_abs = new_project_path.join(script_path);
            if script_abs.exists() {
                match WasmRuntime::new() {
                    Ok(mut runtime) => {
                        if let Ok(_) = runtime.load(&script_abs) {
                            tracing::info!("WASM script reloaded successfully");
                            self.wasm_runtime = Some(runtime);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create WASM runtime: {}", e);
                    }
                }
            }
        }

        // 4. 重新加载场景和材质
        let project_assets = resolve_project_assets(&new_project_path, &self.project_config.run.assets_dir);
        let scene_path = project_assets.join("scenes/main.ron");
        let materials_path = project_assets.join("materials");

        // 重新配置渲染插件
        let mut schedule = SystemSchedule::new();
        physics_plugin().build(&mut schedule);

        let render_pl = render_plugin()
            .with_materials(materials_path.to_string_lossy().as_ref())
            .with_scene(scene_path.to_string_lossy().as_ref());

        render_pl.setup(engine);

        // 5. 生成网格地面（场景加载后需要手动调用）
        spawn_grid_system(engine.world_mut());

        // 重新添加项目选择器 UI
        if let Some(ref mut renderer) = engine.main_renderer {
            let opener = ProjectOpener::new(self.project_path.clone(), self.shared_state.pending_project.clone());
            renderer.ui_objects.push(Box::new(opener));
        }

        tracing::info!("Project reloaded successfully");
    }
}

/// 解析项目 assets 目录的绝对路径
fn resolve_project_assets(project_path: &PathBuf, assets_dir: &str) -> PathBuf {
    // 从 project.toml 读取 assets_dir（相对于项目目录）
    // 如果是相对路径，先获取项目目录的绝对路径
    let project_abs = if project_path.is_absolute() {
        project_path.clone()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(project_path)
    };

    project_abs.join(assets_dir)
}

impl App for ToyEngineApp {
    fn systems(&mut self) -> SystemSchedule {
        let mut schedule = SystemSchedule::new();
        physics_plugin().build(&mut schedule);

        // 计算项目 assets 目录的绝对路径
        let project_assets = resolve_project_assets(&self.project_path, &self.project_config.run.assets_dir);

        let scene_path = project_assets.join("scenes/main.ron");
        let materials_path = project_assets.join("materials");

        tracing::info!("Project assets: {:?}", project_assets);
        tracing::info!("Loading materials from: {:?}", materials_path);
        tracing::info!("Loading scene from: {:?}", scene_path);

        render_plugin()
            .with_materials(materials_path.to_string_lossy().as_ref())
            .with_scene(scene_path.to_string_lossy().as_ref())
            .build(&mut schedule);

        schedule
    }

    fn on_start(&mut self, engine: &mut Engine) {
        // 计算项目 assets 目录的绝对路径
        let project_assets = resolve_project_assets(&self.project_path, &self.project_config.run.assets_dir);

        let scene_path = project_assets.join("scenes/main.ron");
        let materials_path = project_assets.join("materials");

        tracing::info!("Project assets: {:?}", project_assets);
        tracing::info!("Loading materials from: {:?}", materials_path);
        tracing::info!("Loading scene from: {:?}", scene_path);

        let render_plugin = render_plugin()
            .with_materials(materials_path.to_string_lossy().as_ref())
            .with_scene(scene_path.to_string_lossy().as_ref());

        render_plugin.setup(engine);
        tracing::info!(
            "Renderer initialized, main_renderer: {:?}",
            engine.main_renderer.is_some()
        );

        // 添加项目选择器 UI
        if let Some(ref mut renderer) = engine.main_renderer {
            let opener = ProjectOpener::new(self.project_path.clone(), self.shared_state.pending_project.clone());
            renderer.ui_objects.push(Box::new(opener));
        }
    }

    fn on_update(&mut self, engine: &mut Engine, dt_seconds: f32) {
        // 检查是否需要热重载项目
        let pending = self.shared_state.pending_project.lock().unwrap().take();
        if let Some(new_project) = pending {
            self.reload_project(new_project, engine);
        }

        // 从 ECS 获取输入状态并构建 mask
        let mut input_mask: u8 = 0;
        if let Some(ecs_input) = engine.core.world.get_resource::<engine_core::input::InputState>() {
            for digit in 1..=4 {
                let key = match digit {
                    1 => InputCode::Key1,
                    2 => InputCode::Key2,
                    3 => InputCode::Key3,
                    4 => InputCode::Key4,
                    _ => continue,
                };
                if ecs_input.is_pressed(key) {
                    input_mask |= 1 << (digit - 1);
                }
            }
        }

        // 运行 WASM 脚本更新
        if let Some(ref mut runtime) = self.wasm_runtime {
            if runtime.is_loaded() {
                // 轨道相机参数
                let radius = 8.0;
                let height = 3.0;
                let speed = 0.5;  // 弧度/秒

                // 调用 WASM 函数（传入 input_mask）
                match runtime.call_camera_func("update", dt_seconds, 0, radius, height, speed, input_mask) {
                    Ok(pos) => {
                        // 更新相机位置
                        let mut query = engine.core.world.query::<&mut engine_core::ecs::Camera3D>();
                        if let Some(mut camera) = query.iter_mut(&mut engine.core.world).next() {
                            camera.position = pos;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("WASM update failed: {}", e);
                    }
                }
            }
        }
    }
}

/// 解析命令行参数
/// 解析命令行参数
///
/// 优先级: --project > 配置文件 > 硬编码默认值
fn parse_args() -> (PathBuf, bool) {
    let args: Vec<String> = std::env::args().collect();

    // 加载引擎配置
    let mut config = GlobalConfig::load();

    // 检查是否是设置默认项目的命令
    for i in 0..args.len() {
        if args[i] == "--set-default" && i + 1 < args.len() {
            let project_path = &args[i + 1];
            tracing::info!("Setting default project to: {}", project_path);
            config.set_default_project(project_path);
            if let Err(e) = config.save() {
                tracing::error!("Failed to save config: {}", e);
            } else {
                tracing::info!("Default project saved to config");
            }
            return (PathBuf::from("./demos/simple_demo"), true);
        }
    }

    // 查找 --project 参数
    for i in 0..args.len() {
        if args[i] == "--project" && i + 1 < args.len() {
            return (PathBuf::from(&args[i + 1]), false);
        }
    }

    // 使用配置文件中的默认项目
    if let Some(ref project) = config.default_project() {
        tracing::info!("Using default project from config: {}", project);
        return (PathBuf::from(project), false);
    }

    // 硬编码默认
    (PathBuf::from("./demos/simple_demo"), false)
}

/// 项目选择器 UI 组件
struct ProjectOpener {
    current_project: PathBuf,
    pending_project: Arc<Mutex<Option<PathBuf>>>,
}

impl ProjectOpener {
    fn new(project_path: PathBuf, pending_project: Arc<Mutex<Option<PathBuf>>>) -> Self {
        Self {
            current_project: project_path,
            pending_project,
        }
    }
}

impl engine_renderer::ui::UiComponent for ProjectOpener {
    fn id(&self) -> &'static str {
        "project_opener"
    }

    fn render(&mut self, ctx: &Context) {
        egui::Window::new("Project")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
            .show(ctx, |ui| {
                ui.label(format!("Current: {}", self.current_project.file_name().unwrap_or_default().to_string_lossy()));

                if ui.button("Open Project...").clicked() {
                    // 弹出文件夹选择对话框
                    if let Some(path) = FileDialog::new()
                        .set_directory(".")
                        .pick_folder()
                    {
                        tracing::info!("Selected project for hot-reload: {:?}", path);

                        // 设置待重载的项目路径，触发热重载
                        *self.pending_project.lock().unwrap() = Some(path.clone());

                        // 更新显示的项目名
                        self.current_project = path;
                    }
                }
            });
    }
}

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let (project_path, is_set_default) = parse_args();

    // 如果是设置默认项目模式，打印信息后退出
    if is_set_default {
        println!("默认项目已设置为: {:?}", project_path);
        return;
    }

    tracing::info!("ToyEngine starting...");
    tracing::info!("Project path: {:?}", project_path);

    if !project_path.exists() {
        tracing::error!("Project path does not exist: {:?}", project_path);
        std::process::exit(1);
    }

    // 创建共享状态（用于 UI 与 App 之间的通信）
    let shared_state = SharedState::new();

    // 创建 App 并运行
    let app = ToyEngineApp::new(project_path, shared_state);

    let config = AppConfig {
        title: "ToyEngine",
        max_frames: None,
        fixed_dt_seconds: None,
    };

    RunApp::run_app(config, app);
}
