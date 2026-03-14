use std::time::Instant;

use bevy_ecs::prelude::World;
use engine_core::engine::EngineCore;
use engine_core::input::{InputState, InputStateExt};
use engine_renderer::renderer::SurfaceSize;

use super::{App, AppConfig, AppRunner, AppRunnerTrait, Engine, SystemSchedule};

impl<A: App> AppRunnerTrait<A> for AppRunner<A> {
    fn new(config: AppConfig, mut app: A) -> Self {
        let mut world = World::new();

        // 将 InputState 注入 ECS 作为 Resource（现在只有这一处）
        world.insert_resource(InputState::new());

        app.configure_ecs(&mut world);

        // 获取系统调度器
        let schedule = app.systems();

        // 创建 EngineCore（需要克隆 config）
        let core = EngineCore::new(config.clone());

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

