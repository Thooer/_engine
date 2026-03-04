use std::time::Instant;

use engine_core::input::{InputState, InputStateExt};
use engine_renderer::renderer::SurfaceSize;

use super::{App, AppConfig, AppRunner, Engine};

impl<A: App> AppRunner<A> {
    pub(crate) fn new(config: AppConfig, app: A) -> Self {
        Self {
            config,
            app,
            engine: Engine {
                window: None,
                ctx: None,
                input: InputState::new(),
                exit_requested: false,
                frame_index: 0,
            },
            last_frame_time: None,
        }
    }

    pub(crate) fn dt_seconds(&mut self) -> f32 {
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

