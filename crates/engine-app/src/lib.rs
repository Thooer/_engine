//! ToyEngine 应用层：封装 winit 事件循环与基础“执行流”。
//!
//! 目标：让 examples 只关注场景与渲染逻辑（init/update/render），不再重复写窗口与事件循环样板代码。

use std::time::Instant;

use engine_core::input::InputState;
use engine_renderer::renderer::{
    DefaultSurfaceContextNew, SurfaceContext, SurfaceContextNew, SurfaceContextTrait, SurfaceSize,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

#[derive(Clone, Copy, Debug)]
pub struct AppConfig {
    pub title: &'static str,
    pub max_frames: Option<u32>,
    pub fixed_dt_seconds: Option<f32>,
}

pub struct Engine {
    window: Option<&'static Window>,
    ctx: Option<SurfaceContext<'static>>,
    input: InputState,
    exit_requested: bool,
    frame_index: u32,
}

pub trait EngineTrait {
    fn window(&self) -> &Window;
    fn ctx(&self) -> &SurfaceContext<'static>;
    fn ctx_mut(&mut self) -> &mut SurfaceContext<'static>;
    fn input(&self) -> &InputState;
    fn frame_index(&self) -> u32;
    fn request_exit(&mut self);
}

pub trait App {
    fn on_start(&mut self, _engine: &mut Engine) {}
    fn on_window_event(&mut self, _engine: &mut Engine, _event: &WindowEvent) {}
    fn on_resize(&mut self, _engine: &mut Engine, _new_size: SurfaceSize) {}
    fn on_update(&mut self, _engine: &mut Engine, _dt_seconds: f32) {}
    fn on_render(&mut self, _engine: &mut Engine) {}
}

pub struct AppRunner<A: App> {
    config: AppConfig,
    app: A,
    engine: Engine,
    last_frame_time: Option<Instant>,
}

pub trait AppRunnerTrait<A: App> {
    fn new(config: AppConfig, app: A) -> Self;
    fn dt_seconds(&mut self) -> f32;
}

pub struct RunApp;

pub trait RunAppTrait {
    fn run_app<A: App + 'static>(config: AppConfig, app: A);
}

#[path = "Default_AppConfig.rs"]
mod default_app_config;

#[path = "EngineTrait_Engine.rs"]
mod engine_trait_engine;

#[path = "AppRunnerTrait_AppRunner.rs"]
mod app_runner_trait_app_runner;

#[path = "ApplicationHandler_AppRunner.rs"]
mod application_handler_app_runner;

#[path = "RunAppTrait_RunApp.rs"]
mod run_app_trait_run_app;

