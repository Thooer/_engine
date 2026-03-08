pub trait MyService {
    fn process(&self) -> Result<String, String>;
    fn validate(&self) -> bool;
}

pub struct ServiceConfig {
    pub timeout: u64,
    pub retries: u32,
}

impl ServiceConfig {
    pub fn new() -> Self {
        Self {
            timeout: 30,
            retries: 3,
        }
    }
}
