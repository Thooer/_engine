//! Winit 输入实现

use std::collections::HashSet;

use bevy_ecs::prelude::Resource;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::KeyCode;

/// 平台无关的按键码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputCode {
    // 键盘按键
    KeySpace,
    KeyBackquote,
    Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
    KeyMinus, KeyEqual,
    KeyTab, KeyQ, KeyW, KeyE, KeyR, KeyT, KeyY, KeyU, KeyI, KeyO, KeyP,
    KeyBracketLeft, KeyBracketRight, KeyBackslash,
    KeyA, KeyS, KeyD, KeyF, KeyG, KeyH, KeyJ, KeyK, KeyL,
    KeySemicolon, KeyQuote,
    KeyZ, KeyX, KeyC, KeyV, KeyB, KeyN, KeyM,
    KeyComma, KeyPeriod, KeySlash,
    // 功能键
    Escape,
    Enter, Backspace, Insert, Delete,
    ArrowLeft, ArrowRight, ArrowUp, ArrowDown,
    Home, End, PageUp, PageDown,
    // 修饰键
    ShiftLeft, ShiftRight,
    ControlLeft, ControlRight,
    AltLeft, AltRight,
    SuperLeft, SuperRight,
    // F1-F12
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // 数字小键盘
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadDecimal, NumpadDivide, NumpadMultiply,
    NumpadSubtract, NumpadAdd, NumpadEnter,
    // 鼠标
    MouseLeft,
    MouseRight,
    MouseMiddle,
    MouseX1,
    MouseX2,
    /// 未知按键
    Unknown,
}

impl From<winit::keyboard::KeyCode> for InputCode {
    fn from(code: KeyCode) -> Self {
        use KeyCode::*;
        match code {
            // 键盘
            Space => Self::KeySpace,
            Backquote => Self::KeyBackquote,
            Digit1 => Self::Key1, Digit2 => Self::Key2, Digit3 => Self::Key3,
            Digit4 => Self::Key4, Digit5 => Self::Key5, Digit6 => Self::Key6,
            Digit7 => Self::Key7, Digit8 => Self::Key8, Digit9 => Self::Key9,
            Digit0 => Self::Key0,
            Minus => Self::KeyMinus, Equal => Self::KeyEqual,
            Tab => Self::KeyTab, KeyQ => Self::KeyQ, KeyW => Self::KeyW,
            KeyE => Self::KeyE, KeyR => Self::KeyR, KeyT => Self::KeyT,
            KeyY => Self::KeyY, KeyU => Self::KeyU, KeyI => Self::KeyI,
            KeyO => Self::KeyO, KeyP => Self::KeyP,
            BracketLeft => Self::KeyBracketLeft,
            BracketRight => Self::KeyBracketRight,
            Backslash => Self::KeyBackslash,
            KeyA => Self::KeyA, KeyS => Self::KeyS, KeyD => Self::KeyD,
            KeyF => Self::KeyF, KeyG => Self::KeyG, KeyH => Self::KeyH,
            KeyJ => Self::KeyJ, KeyK => Self::KeyK, KeyL => Self::KeyL,
            Semicolon => Self::KeySemicolon, Quote => Self::KeyQuote,
            KeyZ => Self::KeyZ, KeyX => Self::KeyX, KeyC => Self::KeyC,
            KeyV => Self::KeyV, KeyB => Self::KeyB, KeyN => Self::KeyN,
            KeyM => Self::KeyM,
            Comma => Self::KeyComma, Period => Self::KeyPeriod, Slash => Self::KeySlash,
            // 功能键
            Escape => Self::Escape,
            Enter => Self::Enter, Backspace => Self::Backspace,
            Insert => Self::Insert, Delete => Self::Delete,
            ArrowLeft => Self::ArrowLeft, ArrowRight => Self::ArrowRight,
            ArrowUp => Self::ArrowUp, ArrowDown => Self::ArrowDown,
            Home => Self::Home, End => Self::End,
            PageUp => Self::PageUp, PageDown => Self::PageDown,
            // 修饰键
            ShiftLeft => Self::ShiftLeft, ShiftRight => Self::ShiftRight,
            ControlLeft => Self::ControlLeft, ControlRight => Self::ControlRight,
            AltLeft => Self::AltLeft, AltRight => Self::AltRight,
            SuperLeft => Self::SuperLeft, SuperRight => Self::SuperRight,
            // F1-F12
            F1 => Self::F1, F2 => Self::F2, F3 => Self::F3,
            F4 => Self::F4, F5 => Self::F5, F6 => Self::F6,
            F7 => Self::F7, F8 => Self::F8, F9 => Self::F9,
            F10 => Self::F10, F11 => Self::F11, F12 => Self::F12,
            // 数字小键盘
            Numpad0 => Self::Numpad0, Numpad1 => Self::Numpad1,
            Numpad2 => Self::Numpad2, Numpad3 => Self::Numpad3,
            Numpad4 => Self::Numpad4, Numpad5 => Self::Numpad5,
            Numpad6 => Self::Numpad6, Numpad7 => Self::Numpad7,
            Numpad8 => Self::Numpad8, Numpad9 => Self::Numpad9,
            NumpadDecimal => Self::NumpadDecimal,
            NumpadDivide => Self::NumpadDivide,
            NumpadMultiply => Self::NumpadMultiply,
            NumpadSubtract => Self::NumpadSubtract,
            NumpadAdd => Self::NumpadAdd,
            NumpadEnter => Self::NumpadEnter,
            // 鼠标（winit 映射）
            _ => Self::Unknown,
        }
    }
}

/// 从 WindowEvent 中提取 InputCode 的辅助函数
pub fn extract_inputcode_from_event(event: &WindowEvent) -> Option<(InputCode, ElementState)> {
    match event {
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(code),
                    state,
                    ..
                },
            ..
        } => Some((InputCode::from(*code), *state)),
        _ => None,
    }
}

/// Winit 输入状态实现
#[derive(Debug, Default, Resource)]
pub struct WinitInputState {
    /// 当前按下中的按键集合
    pressed: HashSet<InputCode>,
    /// 本帧刚刚按下的按键集合
    just_pressed: HashSet<InputCode>,
    /// 本帧刚刚抬起的按键集合
    just_released: HashSet<InputCode>,
}

impl WinitInputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 处理来自 winit 的单个 WindowEvent，更新内部状态
    pub fn on_window_event(&mut self, event: &WindowEvent) {
        if let Some((code, state)) = extract_inputcode_from_event(event) {
            match state {
                ElementState::Pressed => {
                    if self.pressed.insert(code) {
                        self.just_pressed.insert(code);
                    }
                }
                ElementState::Released => {
                    self.pressed.remove(&code);
                    self.just_released.insert(code);
                }
            }
        }
    }

    /// 在一帧结束时调用，用于清空 just_pressed / just_released
    pub fn next_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// 某个按键当前是否处于按下状态
    pub fn is_pressed(&self, code: InputCode) -> bool {
        self.pressed.contains(&code)
    }

    /// 某个按键在本帧是否刚刚被按下
    pub fn just_pressed(&self, code: InputCode) -> bool {
        self.just_pressed.contains(&code)
    }

    /// 某个按键在本帧是否刚刚被抬起
    pub fn just_released(&self, code: InputCode) -> bool {
        self.just_released.contains(&code)
    }
}

/// 从 winit KeyCode 转换为 InputCode 的 trait（别名）
pub trait ToInputCode {
    fn to_input_code(&self) -> InputCode;
}

impl ToInputCode for KeyCode {
    fn to_input_code(&self) -> InputCode {
        InputCode::from(*self)
    }
}
