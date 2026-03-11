//! 输入事件定义

/// 按钮输入类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    Key(u32),
    Mouse(u32),
    Gamepad(u16),
}

/// 按钮按下事件
#[derive(Debug, Clone)]
pub struct ButtonPressed {
    pub button: Button,
}

/// 按钮释放事件
#[derive(Debug, Clone)]
pub struct ButtonReleased {
    pub button: Button,
}

/// 鼠标移动事件
#[derive(Debug, Clone)]
pub struct MouseMoved {
    pub position: (f32, f32),
    pub delta: (f32, f32),
}

/// 鼠标滚轮事件
#[derive(Debug, Clone)]
pub struct MouseWheel {
    pub delta: (f32, f32),
}
