pub trait MyService {
    fn process(&self) -> Result<String, String>;
}

pub struct ServiceConfig;
