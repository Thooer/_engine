//! ToyEngine 主入口
//!
//! 运行方式:
//!   cargo run -p engine-app                      # 运行默认项目
//!   cargo run -p engine-app -- --project ./demos/simple_demo

use std::path::PathBuf;
use engine_app::{App, AppConfig, Engine, RunApp, RunAppTrait, WasmRuntime};
use engine_app::plugins::{render_plugin, physics_plugin};
use engine_app::SystemSchedule;
use engine_core::input::InputStateExt;
use winit::keyboard::KeyCode;

/// 项目配置
#[derive(Debug, Clone)]
struct ProjectConfig {
    /// 项目名称
    name: String,
    /// 入口场景路径（相对于项目根目录）
    scene: String,
    /// 资源目录
    assets_dir: String,
    /// WASM 脚本路径（可选）
    script: Option<String>,
    /// 相机控制模式 (orbit, orbital, figure8, spiral)
    camera_mode: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "simple_demo".to_string(),
            scene: "assets/scenes/main.ron".to_string(),
            assets_dir: "assets".to_string(),
            script: None,
            camera_mode: "orbit".to_string(),
        }
    }
}

impl ProjectConfig {
    /// 从 project.toml 加载
    fn load_from_path(path: &PathBuf) -> Self {
        let config_path = path.join("project.toml");

        if !config_path.exists() {
            tracing::warn!("project.toml not found at {:?}, using defaults", config_path);
            return Self::default();
        }

        // 读取 toml 文件
        let content = match std::fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to read project.toml: {}, using defaults", e);
                return Self::default();
            }
        };

        // 解析 toml
        let config: toml::Value = match content.parse() {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Failed to parse project.toml: {}, using defaults", e);
                return Self::default();
            }
        };

        let name = config
            .get("name")
            .and_then(|v| v.get("simple_demo"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let scene = config
            .get("run")
            .and_then(|v| v.get("scene"))
            .and_then(|v| v.as_str())
            .unwrap_or("assets/scenes/main.ron")
            .to_string();

        let assets_dir = config
            .get("run")
            .and_then(|v| v.get("assets_dir"))
            .and_then(|v| v.as_str())
            .unwrap_or("assets")
            .to_string();

        let script = config
            .get("run")
            .and_then(|v| v.get("script"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let camera_mode = config
            .get("run")
            .and_then(|v| v.get("camera_mode"))
            .and_then(|v| v.as_str())
            .unwrap_or("orbit")
            .to_string();

        Self {
            name,
            scene,
            assets_dir,
            script,
            camera_mode,
        }
    }
}

/// ToyEngine 应用程序
struct ToyEngineApp {
    project_path: PathBuf,
    project_config: ProjectConfig,
    wasm_runtime: Option<WasmRuntime>,
}

impl ToyEngineApp {
    fn new(project_path: PathBuf) -> Self {
        let project_config = ProjectConfig::load_from_path(&project_path);
        tracing::info!("Loaded project: {}", project_config.name);
        tracing::info!("  scene: {}", project_config.scene);
        tracing::info!("  assets: {}", project_config.assets_dir);
        if let Some(ref script) = project_config.script {
            tracing::info!("  script: {}", script);
        }

        // 尝试加载 WASM 脚本
        let mut wasm_runtime: Option<WasmRuntime> = None;
        if let Some(ref script_path) = project_config.script {
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
        }
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
        let project_assets = resolve_project_assets(&self.project_path, &self.project_config.assets_dir);

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
        let project_assets = resolve_project_assets(&self.project_path, &self.project_config.assets_dir);

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
    }

    fn on_update(&mut self, engine: &mut Engine, dt_seconds: f32) {
        // 从 ECS 获取输入状态并构建 mask
        let mut input_mask: u8 = 0;
        if let Some(ecs_input) = engine.world.get_resource::<engine_core::input::InputState>() {
            for digit in 1..=4 {
                let key = match digit {
                    1 => KeyCode::Digit1,
                    2 => KeyCode::Digit2,
                    3 => KeyCode::Digit3,
                    4 => KeyCode::Digit4,
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
                        let mut query = engine.world.query::<&mut engine_core::ecs::Camera3D>();
                        if let Some(mut camera) = query.iter_mut(&mut engine.world).next() {
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
fn parse_args() -> PathBuf {
    let args: Vec<String> = std::env::args().collect();

    // 查找 --project 参数
    for i in 0..args.len() {
        if args[i] == "--project" && i + 1 < args.len() {
            return PathBuf::from(&args[i + 1]);
        }
    }

    // 默认项目
    PathBuf::from("./demos/simple_demo")
}

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let project_path = parse_args();

    tracing::info!("ToyEngine starting...");
    tracing::info!("Project path: {:?}", project_path);

    if !project_path.exists() {
        tracing::error!("Project path does not exist: {:?}", project_path);
        std::process::exit(1);
    }

    // 创建 App 并运行
    let app = ToyEngineApp::new(project_path);

    let config = AppConfig {
        title: "ToyEngine",
        max_frames: None,
        fixed_dt_seconds: None,
    };

    RunApp::run_app(config, app);
}
