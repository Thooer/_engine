use std::time::Instant;
use winit::event_loop::EventLoop;

use super::{Application, EngineApp, EngineRunner, EngineRunnerTrait};

impl EngineRunnerTrait for EngineRunner {
    fn run<A: Application + 'static>(app: A) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let mut engine_app = EngineApp {
            app,
            window: None,
            frame_count: 0,
            last_frame_time: Instant::now(),
            should_exit: false,
        };
        event_loop.run_app(&mut engine_app)?;
        Ok(())
    }
}
