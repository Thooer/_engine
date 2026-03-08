use crate::checker::rules::{Rule, RuleContext};
use crate::parser::{Parser, SynParser};
use crate::report::Report;
use std::fs;

/// tests 目录规则实现
pub struct TestsRule;

impl TestsRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TestsRule {
    fn name(&self) -> &str {
        "tests"
    }

    fn description(&self) -> &str {
        "检查 tests 目录是否符合规范"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn check(&self, context: &RuleContext) -> Report {
        let mut report = Report::new();

        if !context.config.checks.tests.enabled {
            return report;
        }

        let tests_dir = context.module_dir.join("tests");
        if !tests_dir.exists() {
            return report;
        }

        let mod_rs_path = match context.mod_rs_path {
            Some(p) => p,
            None => {
                if context.config.checks.tests.require_mod_declaration_in_parent {
                    report.add_error(tests_dir.clone(), "tests/ 目录存在但父目录没有 mod.rs".to_string());
                }
                return report;
            }
        };

        if context.config.checks.tests.forbid_mod_rs {
            let tests_mod_rs = tests_dir.join("mod.rs");
            if tests_mod_rs.exists() {
                report.add_error(tests_mod_rs, "tests/ 目录禁止存在 mod.rs".to_string());
            }
        }

        if context.config.checks.tests.require_mod_declaration_in_parent {
            let parser = SynParser::new();
            let parsed = match parser.parse(mod_rs_path) {
                Ok(p) => p,
                Err(_) => return report,
            };

            if !parsed.has_module("tests") {
                report.add_error(mod_rs_path.clone(), "tests/ 目录必须在 mod.rs 中声明模块".to_string());
            } else {
                let modules_after_traits = parsed.get_modules_after_traits();
                let tests_module = modules_after_traits.iter().find(|(name, _)| name == "tests");

                if let Some((_, trait_name)) = tests_module {
                    if trait_name.is_none() {
                        let line = parsed.find_module_line_number("tests");
                        report.add_error_with_line(mod_rs_path.clone(), "tests/ 模块声明必须跟在某个 trait 后面".to_string(), line);
                    }
                }
            }
        }

        if context.config.checks.tests.test_file_must_match_trait {
            let entries = match fs::read_dir(&tests_dir) {
                Ok(e) => e,
                Err(_) => return report,
            };

            for entry in entries.flatten() {
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                    continue;
                }

                let file_stem = path.file_stem().unwrap().to_string_lossy();
                let parent_dir = tests_dir.parent().unwrap();
                let impl_file = parent_dir.join(format!("{}.rs", file_stem));

                if !impl_file.exists() {
                    let parser = SynParser::new();
                    let parsed = match parser.parse(mod_rs_path) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    if !parsed.has_trait(&file_stem) {
                        let trait_names = parsed.get_trait_names();
                        let found = trait_names.iter().any(|name| name.to_lowercase() == file_stem.to_lowercase());

                        if !found {
                            report.add_error(path.clone(), format!("测试文件 {} 必须和某个 trait 同名，但在 mod.rs 中未找到对应的 trait（找到的 trait: {}）", file_stem, if trait_names.is_empty() { "无".to_string() } else { trait_names.join(", ") }));
                        }
                    }
                }
            }
        }

        report
    }
}
