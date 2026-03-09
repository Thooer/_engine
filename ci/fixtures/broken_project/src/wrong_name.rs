// Wrong naming - should be MyService_ServiceConfig.rs
impl MyService for ServiceConfig {
    pub fn process(&self) -> Result<String, String> {
        Ok("processed".to_string())
    }
}

pub struct ServiceConfig;
