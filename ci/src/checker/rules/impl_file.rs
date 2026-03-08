use crate::checker::rules::{Rule, RuleContext};
use crate::parser::{Parser, SynParser};
use crate::report::Report;

/// impl 文件规则实现
pub struct ImplFileRule;

impl ImplFileRule {
    pub fn new() -> Self { Self }
}

impl Default for ImplFileRule {
    fn default() -> Self { Self::new() }
}

impl Rule for ImplFileRule {
    fn name(&self) -> &str { "impl_file" }
    fn description(&self) -> &str { "检查 impl 文件是否符合规范" }
    fn enabled(&self) -> bool { true }

    fn check(&self, context: &RuleContext) -> Report {
        let mut report = Report::new();

        if !context.config.checks.impl_file.enabled {
            return report;
        }

        let parser = SynParser::new();

        for impl_path in context.impl_files {
            let parsed = match parser.parse(impl_path) {
                Ok(p) => p,
                Err(e) => {
                    report.add_error(impl_path.clone(), format!("解析文件失败: {}", e));
                    continue;
                }
            };

            // 检查：include! 只能出现在包含 impl 块的文件中
            if parsed.contains_include_macro() && parsed.impl_count() == 0 {
                let line = parsed.find_include_macro_line_number();
                report.add_error_with_line(impl_path.clone(), "include! 只能放在实现文件中（必须包含 impl 块）".to_string(), line);
                continue;
            }

            // 如果文件不包含 impl 块，跳过
            if parsed.impl_count() == 0 {
                continue;
            }

            // 检查：禁止固有实现
            if context.config.checks.impl_file.forbid_inherent_impl && parsed.has_inherent_impl() {
                let inherent_types = parsed.get_inherent_impl_types();
                let line = parsed.find_impl_line_number();

                if inherent_types.is_empty() {
                    report.add_error_with_line(impl_path.clone(), "禁止固有实现（inherent impl）。所有方法必须通过 trait 暴露，然后在实现文件中 impl trait".to_string(), line);
                } else {
                    let type_list = inherent_types.join("、");
                    report.add_error_with_line(impl_path.clone(), format!("禁止固有实现（inherent impl）：{}。所有方法必须通过 trait 暴露，然后在实现文件中 impl trait", type_list), line);
                }
                continue;
            }

            // 检查：必须只有一个 impl 块
            let has_multiple_impls = parsed.impl_count() > 1;
            let is_empty_impls_exception = has_multiple_impls && parsed.are_all_impls_empty() && parsed.are_all_impls_same_trait();

            if context.config.checks.impl_file.single_impl_only && has_multiple_impls && !is_empty_impls_exception {
                let line = parsed.find_impl_line_number();
                let all_impl_info = parsed.get_all_impl_info();
                let mut expected_files = Vec::new();

                for (trait_name, type_name) in &all_impl_info {
                    if let (Some(trait_n), Some(ty)) = (trait_name, type_name) {
                        expected_files.push(format!("{}_{}.rs", trait_n, ty));
                    } else if let Some(trait_n) = trait_name {
                        expected_files.push(format!("{}_Unknown.rs", trait_n));
                    } else if let Some(ty) = type_name {
                        expected_files.push(format!("Inherent_{}.rs", ty));
                    } else {
                        expected_files.push("Inherent_Unknown.rs".to_string());
                    }
                }

                let error_msg = if expected_files.is_empty() {
                    format!("impl 文件只能包含一个 impl 块，但发现了 {} 个", parsed.impl_count())
                } else {
                    format!("impl 文件只能包含一个 impl 块，但发现了 {} 个，期望拆分成 {} 个文件：{}", parsed.impl_count(), expected_files.len(), expected_files.join("、"))
                };

                report.add_error_with_line(impl_path.clone(), error_msg, line);
            }

            // 检查：文件命名必须和 trait 对应
            if (!has_multiple_impls || !is_empty_impls_exception) && context.config.checks.impl_file.naming_must_match_trait {
                if let Some(trait_name) = parsed.get_impl_trait_name() {
                    let actual_name = impl_path.file_stem().unwrap().to_string_lossy();

                    if let Some(type_name) = parsed.get_impl_type_name() {
                        let expected_name = format!("{}_{}", trait_name, type_name);

                        if actual_name != expected_name {
                            let line = parsed.find_impl_line_number();
                            report.add_error_with_line(impl_path.clone(), format!("文件命名必须和 trait 对应：期望 {}，实际 {}", expected_name, actual_name), line);
                        }
                    } else {
                        let line = parsed.find_impl_line_number();
                        report.add_error_with_line(impl_path.clone(), format!("文件命名必须使用 {{TraitName}}_{{TypeName}}.rs 格式，实际 {}", actual_name), line);
                    }
                }
            }

            // 检查：禁止 pub 关键字
            if context.config.checks.impl_file.forbid_pub && parsed.contains_pub() {
                let line = parsed.find_keyword_line_number("pub ");
                report.add_error_with_line(impl_path.clone(), "impl 文件禁止使用 pub 关键字".to_string(), line);
            }
        }

        report
    }
}
