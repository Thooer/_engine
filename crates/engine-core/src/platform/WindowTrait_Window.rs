//! WindowTrait trait 实现

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Window, WindowId};

use crate::platform::WindowTrait;

impl WindowTrait for Window {
    fn id(&self) -> WindowId {
        Window::id(self)
    }

    fn raw_window_handle(&self) -> Result<RawWindowHandle, raw_window_handle::HandleError> {
        HasWindowHandle::window_handle(self).map(|h| h.as_raw())
    }

    fn set_title(&self, title: &str) {
        Window::set_title(self, title);
    }

    fn should_close(&self) -> bool {
        false // winit 通过事件处理关闭
    }

    fn request_close(&mut self) {
        // winit 通过事件处理关闭
    }
}
