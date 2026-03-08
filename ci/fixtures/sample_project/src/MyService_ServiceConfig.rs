impl MyService for ServiceConfig {
    fn process(&self) -> Result<String, String> {
        if self.timeout > 0 {
            Ok(format!("Processed with timeout {}", self.timeout))
        } else {
            Err("Timeout must be greater than 0".to_string())
        }
    }

    fn validate(&self) -> bool {
        self.timeout > 0 && self.retries > 0
    }
}
