use super::Camera2D;
use glam::Vec3;

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            zoom: 1.0,
        }
    }
}

