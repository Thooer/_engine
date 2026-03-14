use super::{InputCode, KeyCodeExtractor, KeyCodeExtractorTrait};
use winit::event::WindowEvent;

impl KeyCodeExtractorTrait for KeyCodeExtractor {
    fn extract_keycode_from_keyboard_event(event: &WindowEvent) -> Option<InputCode> {
        use engine_platform::input::extract_inputcode_from_event;
        extract_inputcode_from_event(event).map(|(code, _)| code)
    }
}
