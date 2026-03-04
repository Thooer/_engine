use super::AppConfig;

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "ToyEngine App",
            max_frames: None,
            fixed_dt_seconds: Some(1.0 / 60.0),
        }
    }
}

