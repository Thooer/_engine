//! Platform 层 - 窗口与事件循环抽象

use raw_window_handle::RawWindowHandle;
use winit::window::WindowId;

/// 窗口 trait，提供窗口操作和句柄访问
#[allow(dead_code)]
pub trait WindowTrait {
    /// 获取窗口 ID
    fn id(&self) -> WindowId;

    /// 获取原始窗口句柄
    fn raw_window_handle(&self) -> Result<RawWindowHandle, raw_window_handle::HandleError>;

    /// 设置窗口标题
    fn set_title(&self, title: &str);

    /// 获取窗口是否应该关闭
    fn should_close(&self) -> bool;

    /// 请求关闭窗口
    fn request_close(&mut self);
}
#[cfg(test)]
mod tests;

/// 事件循环控制
#[allow(dead_code)]
pub struct PlatformEventLoop {
    // 预留：后续会扩展为更完整的 event loop 抽象
}

#[path = "WindowTrait_Window.rs"]
mod window_trait_window;

#[path = "Default_PlatformEventLoop.rs"]
mod default_platform_event_loop;
