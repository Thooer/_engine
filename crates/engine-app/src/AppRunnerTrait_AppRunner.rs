use std::time::Instant;

use bevy_ecs::prelude::World;
use engine_core::engine::EngineCore;
use engine_core::input::{InputState, InputStateExt};
use engine_renderer::renderer::SurfaceSize;

use super::{App, AppConfig, AppRunner, AppRunnerTrait, Engine, SystemSchedule};

impl<A: App> AppRunnerTrait<A> for AppRunner<A> {
    fn new(config: AppConfig, mut app: A) -> Self {
        // 创建 EngineCore（内部会创建新的 World）
        let mut core = EngineCore::new(config.clone());

        // 将 InputState 注入 ECS 作为 Resource
        core.world.insert_resource(InputState::new());

        // 让 App 配置 ECS
        app.configure_ecs(&mut core.world);

        // 获取系统调度器
        let schedule = app.systems();

        Self {
            config,
            app,
            engine: Engine {
                core,
                window: None,
                ctx: None,
                main_renderer: None,
            },
            last_frame_time: None,
            schedule,
            setup_done: false,
        }
    }

    fn dt_seconds(&mut self) -> f32 {
        if let Some(dt) = self.config.fixed_dt_seconds {
            return dt;
        }

        let now = Instant::now();
        let dt = if let Some(last) = self.last_frame_time {
            (now - last).as_secs_f32()
        } else {
            0.0
        };
        self.last_frame_time = Some(now);
        dt
    }
}

