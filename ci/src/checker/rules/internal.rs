use crate::checker::rules::{Rule, RuleContext};
use crate::parser::{Parser, SynParser};
use crate::report::Report;
use std::fs;

/// internal 目录规则实现
pub struct InternalRule;

impl InternalRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InternalRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for InternalRule {
    fn name(&self) -> &str {
        "internal"
    }

    fn description(&self) -> &str {
        "检查 internal 目录是否符合规范"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn check(&self, context: &RuleContext) -> Report {
        let mut report = Report::new();

        if !context.config.checks.internal.enabled {
            return report;
        }

        // 从 module_dir 获取 internal_dir
        let internal_dir = context.module_dir.join("internal");
        if !internal_dir.exists() {
            return report;
        }

        if context.config.checks.internal.forbid_mod_rs {
            let mod_rs_path = internal_dir.join("mod.rs");
            if mod_rs_path.exists() {
                report.add_error(mod_rs_path, "internal/ 目录禁止存在 mod.rs".to_string());
            }
        }

        let entries = match fs::read_dir(&internal_dir) {
            Ok(e) => e,
            Err(_) => return report,
        };

        let parser = SynParser::new();

        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let parsed = match parser.parse(&path) {
                Ok(p) => p,
                Err(e) => {
                    report.add_error(path.clone(), format!("解析文件失败: {}", e));
                    continue;
                }
            };

            if context.config.checks.internal.require_brace_wrap && !parsed.is_brace_wrapped() {
                report.add_error(path.clone(), "internal/ 目录中的文件必须用大括号 {} 包裹".to_string());
            }

            if context.config.checks.internal.only_function_body && parsed.has_function_signatures() {
                let line = parsed.find_keyword_line_number("fn ");
                report.add_error_with_line(path.clone(), "internal/ 目录中的文件只能包含函数体，不能包含方法签名".to_string(), line);
            }
        }

        report
    }
}
