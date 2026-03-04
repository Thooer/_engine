use std::path::PathBuf;
use colored::*;

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub path: PathBuf,
    pub message: String,
    pub line: Option<usize>,
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
        self.add_error_with_line(path, message, None);
    }

    pub fn add_error_with_line(&mut self, path: PathBuf, message: String, line: Option<usize>) {
        self.errors.push(CheckResult { path, message, line });
    }

    #[allow(dead_code)]
    pub fn add_warning(&mut self, path: PathBuf, message: String) {
        self.add_warning_with_line(path, message, None);
    }

    #[allow(dead_code)]
    pub fn add_warning_with_line(&mut self, path: PathBuf, message: String, line: Option<usize>) {
        self.warnings.push(CheckResult { path, message, line });
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn print(&self, color: bool) {
        if self.errors.is_empty() && self.warnings.is_empty() {
            if color {
                println!("{}", "✓ 所有检查通过".green().bold());
            } else {
                println!("✓ 所有检查通过");
            }
            return;
        }

        if !self.errors.is_empty() {
            if color {
                println!("{}", format!("✗ 发现 {} 个错误", self.errors.len()).red().bold());
            } else {
                println!("✗ 发现 {} 个错误", self.errors.len());
            }
            
            for error in &self.errors {
                let path_str = error.path.to_string_lossy();
                let line_info = if let Some(line) = error.line {
                    format!(":{}", line)
                } else {
                    String::new()
                };
                if color {
                    println!("  {} {}{}", "ERROR:".red().bold(), path_str, line_info);
                    println!("    {}", error.message.red());
                } else {
                    println!("  ERROR: {}{}", path_str, line_info);
                    println!("    {}", error.message);
                }
            }
        }

        if !self.warnings.is_empty() {
            if color {
                println!("{}", format!("⚠ 发现 {} 个警告", self.warnings.len()).yellow().bold());
            } else {
                println!("⚠ 发现 {} 个警告", self.warnings.len());
            }
            
            for warning in &self.warnings {
                let path_str = warning.path.to_string_lossy();
                let line_info = if let Some(line) = warning.line {
                    format!(":{}", line)
                } else {
                    String::new()
                };
                if color {
                    println!("  {} {}{}", "WARNING:".yellow().bold(), path_str, line_info);
                    println!("    {}", warning.message.yellow());
                } else {
                    println!("  WARNING: {}{}", path_str, line_info);
                    println!("    {}", warning.message);
                }
            }
        }
    }
}

impl Default for Report {
    fn default() -> Self {
        Self::new()
    }
}
