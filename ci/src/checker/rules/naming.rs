use crate::checker::rules::r#trait::Rule;
use crate::checker::rules::RuleContext;
use crate::report::Report;

/// 命名规则实现
pub struct NamingRule;

impl NamingRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NamingRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for NamingRule {
    fn name(&self) -> &str {
        "naming"
    }

    fn description(&self) -> &str {
        "检查文件命名是否符合规范"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn check(&self, context: &RuleContext) -> Report {
        let mut report = Report::new();

        if !context.config.checks.naming.enabled {
            return report;
        }

        // 检查 impl 文件命名
        for impl_path in context.impl_files {
            let file_name = impl_path.file_name().unwrap().to_string_lossy().to_lowercase();

            if context.config.checks.naming.forbid_impl_suffix {
                if file_name.ends_with("impl.rs") || file_name.contains("_impl.rs") {
                    report.add_error(
                        impl_path.clone(),
                        format!("禁止使用 *impl.rs 命名：{}", impl_path.file_name().unwrap().to_string_lossy()),
                    );
                }
            }

            if context.config.checks.naming.forbid_tests_suffix {
                if file_name.ends_with("tests.rs") || file_name.ends_with("test.rs") {
                    report.add_error(
                        impl_path.clone(),
                        format!("禁止使用 *tests.rs 命名：{}", impl_path.file_name().unwrap().to_string_lossy()),
                    );
                }
            }
        }

        report
    }
}
