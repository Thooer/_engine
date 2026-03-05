use winit::event_loop::EventLoop;

use super::{App, AppConfig, AppRunner, AppRunnerTrait, RunApp, RunAppTrait};

impl RunAppTrait for RunApp {
    fn run_app<A: App + 'static>(config: AppConfig, app: A) {
        let event_loop = EventLoop::new().expect("event loop create failed");
        let mut runner = <AppRunner<A> as AppRunnerTrait<A>>::new(config, app);
        event_loop.run_app(&mut runner).expect("event loop run failed");
    }
}
