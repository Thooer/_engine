//! 输入系统 v0：基于 winit 的最小键鼠状态查询
//!
//! 设计目标：
//! - 提供“按键是否按下 / 刚按下 / 刚抬起”的查询接口
//! - 对外暴露一个纯粹的状态对象，由外层在事件循环中驱动
//! - 目前仅支持键盘，后续可扩展鼠标 / 手柄（gilrs）

use std::collections::HashSet;

use winit::event::WindowEvent;
use winit::keyboard::KeyCode;

/// 键盘输入状态（单帧）
#[derive(Debug, Default)]
pub struct InputState {
    /// 当前按下中的按键集合
    pub(crate) pressed: HashSet<KeyCode>,
    /// 本帧刚刚按下的按键集合
    pub(crate) just_pressed: HashSet<KeyCode>,
    /// 本帧刚刚抬起的按键集合
    pub(crate) just_released: HashSet<KeyCode>,
}

/// 为 `InputState` 提供事件驱动与查询接口的扩展 trait。
pub trait InputStateExt {
    /// 创建一个新的输入状态对象。
    fn new() -> Self
    where
        Self: Sized;

    /// 处理来自 winit 的单个 `WindowEvent`，更新内部状态。
    ///
    /// 调用方应在事件循环中对每个事件调用一次。
    fn on_window_event(&mut self, event: &WindowEvent);

    /// 在一帧结束时调用，用于清空 `just_pressed` / `just_released`。
    fn next_frame(&mut self);

    /// 某个按键当前是否处于按下状态。
    fn is_pressed(&self, key: KeyCode) -> bool;

    /// 某个按键在本帧是否刚刚被按下。
    fn just_pressed(&self, key: KeyCode) -> bool;

    /// 某个按键在本帧是否刚刚被抬起。
    fn just_released(&self, key: KeyCode) -> bool;
}

/// 从 `WindowEvent::KeyboardInput` 中提取 `KeyCode`。
///
/// 这里单独封装一层，方便后续适配 winit API 变更。
fn extract_keycode_from_keyboard_event(event: &WindowEvent) -> Option<KeyCode> {
    if let WindowEvent::KeyboardInput { event: key_event, .. } = event {
        // 目前仅使用物理按键码，避免布局差异
        if let winit::keyboard::PhysicalKey::Code(code) = key_event.physical_key {
            return Some(code);
        }
    }
    None
}

#[path = "InputStateExt_InputState.rs"]
mod inputstateext_inputstate;

