use crate::report::types::Report;
use colored::*;

pub trait Formatter {
    fn format(&self, report: &Report, color: bool) -> String;
}

pub struct HumanFormatter;

impl Formatter for HumanFormatter {
    fn format(&self, report: &Report, color: bool) -> String {
        if report.errors.is_empty() && report.warnings.is_empty() {
            return if color {
                "✓ 所有检查通过".green().bold().to_string()
            } else {
                "✓ 所有检查通过".to_string()
            };
        }
        
        let mut output = String::new();
        
        if !report.errors.is_empty() {
            if color {
                output.push_str(&format!("{}\n", format!("✗ 发现 {} 个错误", report.errors.len()).red().bold()));
            } else {
                output.push_str(&format!("✗ 发现 {} 个错误\n", report.errors.len()));
            }
            
            for error in &report.errors {
                // 强制转换为纯 Windows 反斜杠风格
                let path_str = error.path.to_string_lossy().replace("/", "\\");
                let line_info = if let Some(line) = error.line {
                    format!(":{}", line)
                } else {
                    String::new()
                };

                if color {
                    output.push_str(&format!("  {} {}{}\n", "ERROR:".red().bold(), path_str, line_info));
                    output.push_str(&format!("    {}\n", error.message.red()));
                } else {
                    output.push_str(&format!("  ERROR: {}{}\n", path_str, line_info));
                    output.push_str(&format!("    {}\n", error.message));
                }
            }
        }
        
        if !report.warnings.is_empty() {
            if color {
                output.push_str(&format!("{}\n", format!("⚠ 发现 {} 个警告", report.warnings.len()).yellow().bold()));
            } else {
                output.push_str(&format!("⚠ 发现 {} 个警告\n", report.warnings.len()));
            }

            for warning in &report.warnings {
                // 强制转换为纯 Windows 反斜杠风格
                let path_str = warning.path.to_string_lossy().replace("/", "\\");
                let line_info = if let Some(line) = warning.line {
                    format!(":{}", line)
                } else {
                    String::new()
                };

                if color {
                    output.push_str(&format!("  {} {}{}\n", "WARNING:".yellow().bold(), path_str, line_info));
                    output.push_str(&format!("    {}\n", warning.message.yellow()));
                } else {
                    output.push_str(&format!("  WARNING: {}{}\n", path_str, line_info));
                    output.push_str(&format!("    {}\n", warning.message));
                }
            }
        }
        
        output
    }
}

pub fn print_report(report: &Report, color: bool) {
    let formatter = HumanFormatter;
    println!("{}", formatter.format(report, color));
}
