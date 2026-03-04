use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::KeyCode;

use super::{extract_keycode_from_keyboard_event, InputState, InputStateExt};

impl InputStateExt for InputState {
    fn new() -> Self {
        Self::default()
    }

    fn on_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: _,
                        physical_key: _,
                        text: _,
                        location: _,
                        repeat: _,
                        state,
                        ..
                    },
                is_synthetic: _,
                ..
            } => {
                // winit 0.30 把 KeyCode 放在 physical_key 里，这里仅使用物理按键码。
                if let Some(keycode) = extract_keycode_from_keyboard_event(event) {
                    match state {
                        ElementState::Pressed => {
                            if self.pressed.insert(keycode) {
                                self.just_pressed.insert(keycode);
                            }
                        }
                        ElementState::Released => {
                            self.pressed.remove(&keycode);
                            self.just_released.insert(keycode);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn next_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }
}

