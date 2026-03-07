use super::{DirectLight, DirectLightTrait};
use glam::Vec3;

impl DirectLightTrait for DirectLight {
    fn new(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            direction: direction.to_array(),
            _padding: 0.0,
            color: color.to_array(),
            intensity,
        }
    }
}
