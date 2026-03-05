use winit::event::WindowEvent;
use winit::keyboard::KeyCode;

use super::{KeyCodeExtractor, KeyCodeExtractorTrait};

impl KeyCodeExtractorTrait for KeyCodeExtractor {
    fn extract_keycode_from_keyboard_event(event: &WindowEvent) -> Option<KeyCode> {
        if let WindowEvent::KeyboardInput { event: key_event, .. } = event {
            // 目前仅使用物理按键码，避免布局差异
            if let winit::keyboard::PhysicalKey::Code(code) = key_event.physical_key {
                return Some(code);
            }
        }
        None
    }
}
