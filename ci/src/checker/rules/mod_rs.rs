use crate::checker::rules::{Rule, RuleContext};
use crate::report::Report;
use crate::verbose::debug_log;

/// mod.rs 规则实现
pub struct ModRsRule;

impl ModRsRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ModRsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ModRsRule {
    fn name(&self) -> &str {
        "mod_rs"
    }

    fn description(&self) -> &str {
        "检查 mod.rs 是否符合规范"
    }

    fn enabled(&self) -> bool {
        true
    }

    fn check(&self, context: &RuleContext) -> Report {
        let mut report = Report::new();

        if !context.config.checks.mod_rs.enabled {
            debug_log("mod_rs check is disabled");
            return report;
        }
        
        debug_log(format!("mod_rs check is enabled, trait_impl_order = {}", context.config.checks.mod_rs.trait_impl_order));

        let mod_rs_path = match context.mod_rs_path {
            Some(p) => p,
            None => return report,
        };

        let parsed = match context.parsed_mod_rs {
            Some(p) => p,
            None => return report,
        };

        // 检查：禁止包含任何形式的 impl
        if context.config.checks.mod_rs.forbid_impl && parsed.contains_impl() {
            let line = parsed.find_keyword_line_number("impl");
            report.add_error_with_line(mod_rs_path.clone(), "mod.rs 禁止包含任何形式的 impl".to_string(), line);
        }

        // 检查：禁止使用 include! 宏
        if parsed.contains_include_macro() {
            let line = parsed.find_include_macro_line_number();
            report.add_error_with_line(mod_rs_path.clone(), "mod.rs 禁止使用 include! 宏（include! 只能放在实现文件中）".to_string(), line);
        }

        // 检查：mod.rs 中的 struct 必须是公开的
        if context.config.checks.mod_rs.struct_must_be_public {
            for struct_name in parsed.get_non_public_struct_names() {
                let line = parsed.find_struct_line_number(&struct_name);
                report.add_error_with_line(mod_rs_path.clone(), format!("mod.rs 中的 struct 必须是公开的（pub struct）：{}。如果它是内部结构，请移动到实现文件中", struct_name), line);
            }
        }

        // 检查：禁止顶层函数
        if context.config.checks.mod_rs.forbid_free_functions {
            let top_level_fns = parsed.get_top_level_function_names();
            for fn_name in top_level_fns {
                let line = parsed.find_function_line_number(&fn_name);
                report.add_error_with_line(mod_rs_path.clone(), format!("mod.rs / lib.rs 中禁止存在顶层函数：fn {}。所有方法必须通过 trait 暴露，并由实现文件中的 impl 实现", fn_name), line);
            }
        }

        // 检查：trait 定义后的 impl 模块声明必须以 trait 名称开头
        if context.config.checks.mod_rs.trait_impl_order {
            // 收集文件中定义的所有 trait
            let defined_traits: std::collections::HashSet<String> = parsed.get_trait_names().into_iter().collect();
            debug_log(format!("defined_traits = {:?}", defined_traits));

            let trait_impl_mods = parsed.get_trait_impl_mods();
            debug_log(format!("trait_impl_mods = {:?}", trait_impl_mods));

            for (_belongs_to_trait, mod_name, path_filename, line) in trait_impl_mods {
                // 如果有 #[path = "..."]，从文件名提取 trait 名称
                if let Some(filename) = path_filename {
                    // 从文件名提取可能的 trait 名称（去掉 _ 后缀）
                    if let Some(underscore_pos) = filename.find('_') {
                        let expected_trait = &filename[..underscore_pos];

                        // 如果这个 trait 在文件中定义了，检查顺序
                        if defined_traits.contains(expected_trait) {
                            // 找到 trait 定义的位置
                            let trait_line = parsed.find_keyword_line_number(&format!("trait {}", expected_trait));

                            // 如果 trait 定义存在但 mod 不在 trait 之后紧跟，报错
                            if let Some(trait_l) = trait_line {
                                // mod 声明必须在 trait 定义之后
                                if line <= trait_l {
                                    report.add_error_with_line(
                                        mod_rs_path.clone(),
                                        format!(
                                            "impl 模块必须放在对应 trait 定义之后：{} 在 trait {} (行 {}) 之前",
                                            mod_name, expected_trait, trait_l
                                        ),
                                        Some(line)
                                    );
                                } else {
                                    // 检查是否紧跟（中间没有其他 trait 定义）
                                    let has_other_trait_between = parsed.get_trait_names().iter().any(|t| {
                                        if t == expected_trait {
                                            return false;
                                        }
                                        // 查找这个 trait 的行号
                                        if let Some(other_trait_line) = parsed.find_keyword_line_number(&format!("trait {}", t)) {
                                            other_trait_line > trait_l && other_trait_line < line
                                        } else {
                                            false
                                        }
                                    });

                                    if has_other_trait_between {
                                        report.add_error_with_line(
                                            mod_rs_path.clone(),
                                            format!(
                                                "impl 模块必须紧跟在对应 trait 定义之后，中间不能有其他 trait：{} (在行 {}) 与 trait {} (在行 {}) 之间有其他 trait",
                                                mod_name, line, expected_trait, trait_l
                                            ),
                                            Some(line)
                                        );
                                    }
                                }
                            }
                        } else {
                            // 文件名中的 trait 不在文件中定义，可能是正常的（外部 trait）
                            debug_log(format!("filename trait '{}' not found in defined_traits, skipping check", expected_trait));
                        }
                    }
                }
            }
        }

        report
    }
}
