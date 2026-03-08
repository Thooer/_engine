#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_process() {
        let config = ServiceConfig::new();
        let result = config.process();
        assert!(result.is_ok());
    }

    #[test]
    fn test_service_validate() {
        let config = ServiceConfig::new();
        assert!(config.validate());
    }
}
