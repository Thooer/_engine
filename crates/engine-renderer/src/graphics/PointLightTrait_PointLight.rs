use super::{PointLight, PointLightTrait};
use glam::Vec3;

impl PointLightTrait for PointLight {
    fn new(position: Vec3, color: Vec3, intensity: f32, range: f32) -> Self {
        Self {
            position: position.to_array(),
            range,
            color: color.to_array(),
            intensity,
        }
    }
}
