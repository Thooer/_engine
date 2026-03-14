//! 输入系统抽象
//!
//! 提供平台无关的输入抽象，支持键盘、鼠标等输入设备。

mod winit_input;
pub use winit_input::*;
