//! 输入系统 v0：平台无关的键鼠状态查询
//!
//! 设计目标：
//! - 提供"按键是否按下 / 刚按下 / 刚抬起"的查询接口
//! - 对外暴露一个纯粹的状态对象，由外层在事件循环中驱动
//! - 平台无关抽象，支持多种后端（winit, SDL2, GLFW）
//!
//! # 使用方式
//!
//! ```rust
//! use engine_core::input::{InputState, InputStateExt, InputCode};
//! use winit::event::WindowEvent;
//!
//! // 创建输入状态
//! let mut input = InputState::new();
//!
//! // 处理事件
//! input.on_window_event(&event);
//!
//! // 查询状态
//! if input.is_pressed(InputCode::KeyW) { /* ... */ }
//! ```

// 重新导出 engine_platform 的类型
pub use engine_platform::input::{InputCode, WinitInputState, ToInputCode};

use std::collections::HashSet;

use bevy_ecs::prelude::Resource;
use winit::event::WindowEvent;

/// 键盘输入状态（单帧）- 平台无关版本
///
/// 使用 engine_platform::input::InputCode 替代 winit::keyboard::KeyCode
#[derive(Debug, Default, Resource)]
pub struct InputState {
    /// 当前按下中的按键集合
    pressed: HashSet<InputCode>,
    /// 本帧刚刚按下的按键集合
    just_pressed: HashSet<InputCode>,
    /// 本帧刚刚抬起的按键集合
    just_released: HashSet<InputCode>,
}

/// 为 `InputState` 提供事件驱动与查询接口
pub trait InputStateExt {
    /// 创建一个新的输入状态对象
    fn new() -> Self
    where
        Self: Sized;

    /// 处理来自 winit 的单个 `WindowEvent`，更新内部状态
    fn on_window_event(&mut self, event: &WindowEvent);

    /// 在一帧结束时调用，用于清空 `just_pressed` / `just_released`
    fn next_frame(&mut self);

    /// 某个按键当前是否处于按下状态
    fn is_pressed(&self, key: InputCode) -> bool;

    /// 某个按键在本帧是否刚刚被按下
    fn just_pressed(&self, key: InputCode) -> bool;

    /// 某个按键在本帧是否刚刚被抬起
    fn just_released(&self, key: InputCode) -> bool;
}

impl InputStateExt for InputState {
    fn new() -> Self {
        Self::default()
    }

    fn on_window_event(&mut self, event: &WindowEvent) {
        use engine_platform::input::extract_inputcode_from_event;
        
        if let Some((code, state)) = extract_inputcode_from_event(event) {
            match state {
                winit::event::ElementState::Pressed => {
                    if self.pressed.insert(code) {
                        self.just_pressed.insert(code);
                    }
                }
                winit::event::ElementState::Released => {
                    self.pressed.remove(&code);
                    self.just_released.insert(code);
                }
            }
        }
    }

    fn next_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    fn is_pressed(&self, key: InputCode) -> bool {
        self.pressed.contains(&key)
    }

    fn just_pressed(&self, key: InputCode) -> bool {
        self.just_pressed.contains(&key)
    }

    fn just_released(&self, key: InputCode) -> bool {
        self.just_released.contains(&key)
    }
}

/// 从 `WindowEvent::KeyboardInput` 中提取 `InputCode` 的辅助工具
pub struct KeyCodeExtractor;

pub trait KeyCodeExtractorTrait {
    /// 从 `WindowEvent::KeyboardInput` 中提取 `InputCode`
    fn extract_keycode_from_keyboard_event(event: &WindowEvent) -> Option<InputCode>;
}

#[path = "KeyCodeExtractorTrait_KeyCodeExtractor.rs"]
mod keycode_extractor_trait_keycode_extractor;
