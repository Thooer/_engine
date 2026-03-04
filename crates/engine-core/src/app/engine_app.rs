use std::time::Instant;

use crate::app::Application;

/// 引擎应用内部结构
///
/// 注意：这是内部实现细节，不应作为 public API 暴露。
pub(crate) struct EngineApp<A: Application> {
    pub(crate) app: A,
    pub(crate) window: Option<winit::window::Window>,
    pub(crate) frame_count: u64,
    pub(crate) last_frame_time: Instant,
    pub(crate) should_exit: bool,
}

