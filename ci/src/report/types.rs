use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub path: PathBuf,
    pub message: String,
    pub line: Option<usize>,
}

impl CheckResult {
    pub fn new(path: PathBuf, message: String) -> Self {
        Self { path, message, line: None }
    }
    
    pub fn with_line(path: PathBuf, message: String, line: usize) -> Self {
        Self { path, message, line: Some(line) }
    }
}

#[derive(Debug, Clone)]
pub struct Report {
    pub errors: Vec<CheckResult>,
    pub warnings: Vec<CheckResult>,
}

impl Report {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, path: PathBuf, message: String) {
        self.errors.push(CheckResult::new(path, message));
    }
    
    pub fn add_error_with_line(&mut self, path: PathBuf, message: String, line: Option<usize>) {
        self.errors.push(CheckResult::with_line(path, message, line.unwrap_or(0)));
    }
    
    pub fn add_warning(&mut self, path: PathBuf, message: String) {
        self.warnings.push(CheckResult::new(path, message));
    }
    
    pub fn add_warning_with_line(&mut self, path: PathBuf, message: String, line: Option<usize>) {
        self.warnings.push(CheckResult::with_line(path, message, line.unwrap_or(0)));
    }
    
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn merge(&mut self, other: Report) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

impl Default for Report {
    fn default() -> Self {
        Self::new()
    }
}
