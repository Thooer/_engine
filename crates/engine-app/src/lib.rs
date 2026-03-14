//! ToyEngine 应用层：封装 winit 事件循环与基础“执行流”。
//!
//! 目标：让 examples 只关注场景与渲染逻辑（init/update/render），不再重复写窗口与事件循环样板代码。

use std::time::Instant;

use bevy_ecs::prelude::World;
use engine_core::engine::{EngineCore, EngineCoreTrait, EngineConfig};
use engine_renderer::renderer::{
    DefaultSurfaceContextNew, MainRenderer, SurfaceContext, SurfaceContextNew, SurfaceContextTrait, SurfaceSize,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

mod schedule;
pub use schedule::{SystemSchedule, SystemFn, SystemStage};
pub use engine_core::FrameCounter;

/// 应用配置（向后兼容别名）
pub type AppConfig = EngineConfig;

pub struct Engine {
    /// 引擎核心（ECS World + 配置）
    pub core: EngineCore,
    /// 平台相关：窗口句柄
    pub window: Option<&'static Window>,
    /// 平台相关：GPU 上下文
    pub ctx: Option<SurfaceContext<'static>>,
    /// 主渲染器实例 (由 RenderPlugin 或 App 初始化)
    pub main_renderer: Option<MainRenderer>,
}

pub trait EngineTrait {
    fn window(&self) -> &'static Window;
    fn ctx(&self) -> &SurfaceContext<'static>;
    fn ctx_mut(&mut self) -> &mut SurfaceContext<'static>;
    fn core(&self) -> &EngineCore;
    fn core_mut(&mut self) -> &mut EngineCore;
    
    /// 向后兼容：获取 ECS World
    fn world(&self) -> &World {
        self.core().world()
    }
    
    /// 向后兼容：获取 ECS World 可变引用
    fn world_mut(&mut self) -> &mut World {
        self.core_mut().world_mut()
    }
    
    /// 向后兼容：获取帧索引
    fn frame_index(&self) -> u32 {
        self.core().frame_index
    }
    
    /// 向后兼容：请求退出
    fn request_exit(&mut self) {
        self.core_mut().request_exit();
    }
}

impl EngineTrait for Engine {
    fn window(&self) -> &'static Window {
        self.window.unwrap()
    }

    fn ctx(&self) -> &SurfaceContext<'static> {
        self.ctx.as_ref().unwrap()
    }

    fn ctx_mut(&mut self) -> &mut SurfaceContext<'static> {
        self.ctx.as_mut().unwrap()
    }

    fn core(&self) -> &EngineCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut EngineCore {
        &mut self.core
    }
}

pub trait App {
    /// 返回系统调度器 - 注册需要在引擎中运行的 ECS 系统
    /// 
    /// 默认返回空调度器，子类可以 override 添加系统
    fn systems(&mut self) -> SystemSchedule {
        SystemSchedule::new()
    }

    /// 配置系统调度器（在 on_start 之后调用）
    /// 
    /// 子类可以 override 此方法添加需要渲染器上下文的系统
    fn configure_schedule(&mut self, _schedule: &mut SystemSchedule) {}

    fn on_start(&mut self, _engine: &mut Engine) {}
    
    /// 窗口事件回调
    fn on_window_event(&mut self, _engine: &mut Engine, _event: &WindowEvent) {}
    
    fn on_resize(&mut self, _engine: &mut Engine, _new_size: SurfaceSize) {}
    
    fn on_update(&mut self, _engine: &mut Engine, _dt_seconds: f32) {}
    
    /// 渲染回调
    /// 
    /// 默认实现：自动调用渲染（需要 App 自行存储渲染器）
    fn on_render(&mut self, _engine: &mut Engine) {
        // 默认空实现，由子类实现
    }

    #[allow(unused_variables)]
    fn configure_ecs(&mut self, world: &mut World) {}
}

pub struct AppRunner<A: App> {
    config: AppConfig,
    app: A,
    engine: Engine,
    last_frame_time: Option<Instant>,
    /// 系统调度器 - 从 app 中获取
    schedule: SystemSchedule,
    /// 标记 setup 系统是否已运行
    setup_done: bool,
}

pub trait AppRunnerTrait<A: App> {
    fn new(config: AppConfig, app: A) -> Self;
    fn dt_seconds(&mut self) -> f32;
}

#[path = "AppRunnerTrait_AppRunner.rs"]
mod app_runner_trait_app_runner;

pub struct RunApp;

pub trait RunAppTrait {
    fn run_app<A: App + 'static>(config: AppConfig, app: A);
}

#[path = "Default_AppConfig.rs"]
mod default_app_config;

#[path = "WinitAppRunnerHandler.rs"]
mod application_handler_app_runner;

#[path = "RunAppTrait_RunApp.rs"]
mod run_app_trait_run_app;

/// 插件系统
pub mod plugins;

mod wasm_runtime;
pub use wasm_runtime::WasmRuntime;

