use crate::checker::rules::{Rule, RuleContext};
use crate::report::Report;
use crate::verbose::debug_log;
use std::fs;

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

        // 检查：include! 宏必须在函数体内部，且格式正确
        if parsed.contains_include_macro() {
            // 检查 include! 是否在函数体内部
            if !parsed.is_include_macro_in_function_body() {
                // include! 不在函数体内部
                let line = parsed.find_include_macro_line_number();
                report.add_error_with_line(
                    mod_rs_path.clone(),
                    "mod.rs 禁止使用 include! 宏（include! 必须放在函数体内部）".to_string(),
                    line
                );
            } else {
                // include! 在函数体内部，验证格式和文件名匹配
                if let Some((fn_name, include_file_name)) = parsed.get_include_macro_function_info() {
                    // 去掉 .rs 后缀再比较
                    let include_file_stem = include_file_name.trim_end_matches(".rs");
                    
                    // 检查函数名和 include 文件名是否匹配
                    if fn_name != include_file_stem {
                        let line = parsed.find_include_macro_line_number();
                        report.add_error_with_line(
                            mod_rs_path.clone(),
                            format!(
                                "include! 宏中的文件名必须与函数名完全相同：函数名 '{}'，文件名 '{}'",
                                fn_name, include_file_name
                            ),
                            line
                        );
                    }
                    
                    // 检查格式是否正确（必须是 internal/xxx.rs 格式）
                    let content = parsed.content();
                    let lines = content.lines();
                    for line in lines {
                        if line.contains("include!(") {
                            if !line.contains("internal/") {
                                let line_num = parsed.find_include_macro_line_number();
                                report.add_error_with_line(
                                    mod_rs_path.clone(),
                                    "include! 宏必须使用 internal/ 路径格式（include!(\"internal/xxx.rs\")）".to_string(),
                                    line_num
                                );
                            }
                        }
                    }
                }
            }
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
            // 获取所有顶层函数的名称
            let all_top_level_fns = parsed.get_top_level_function_names();

            // 获取那些函数体中包含 include! 宏的函数（这些是允许的）
            let allowed_fns_with_include = if parsed.contains_include_macro() {
                parsed.get_all_include_macro_functions()
            } else {
                vec![]
            };

            // 只报告那些不在允许列表中的顶层函数
            for fn_name in all_top_level_fns {
                if !allowed_fns_with_include.contains(&fn_name) {
                    let line = parsed.find_function_line_number(&fn_name);
                    report.add_error_with_line(mod_rs_path.clone(), format!("mod.rs / lib.rs 中禁止存在顶层函数：fn {}。所有方法必须通过 trait 暴露，或者通过 include! 宏转移到 internal 目录", fn_name), line);
                }
            }
        }

        // 检查：trait 定义后的 impl 模块声明必须以 trait 名称开头
        if context.config.checks.mod_rs.trait_impl_order {
            // 收集文件中定义的所有 trait
            let defined_traits: Vec<String> = parsed.get_trait_names();
            debug_log(format!("defined_traits = {:?}", defined_traits));

            let trait_impl_mods = parsed.get_trait_impl_mods();
            debug_log(format!("trait_impl_mods = {:?}", trait_impl_mods));

            for (_belongs_to_trait, mod_name, path_filename, line) in trait_impl_mods {
                // 如果有 #[path = "..."]，从文件名提取 trait 名称
                if let Some(filename) = path_filename {
                    // 尝试从文件名中提取 trait 名称
                    // 策略：检查文件名是否以某个已定义的 trait 名称开头
                    let mut matched_trait: Option<String> = None;
                    let mut longest_match = 0;

                    // 去掉 .rs 后缀
                    let filename_stem = filename.trim_end_matches(".rs");

                    for trait_name in &defined_traits {
                        if filename_stem.starts_with(trait_name) {
                            // 使用最长匹配（处理类似 PhysicsContextTrait vs PhysicsContext 的情况）
                            if trait_name.len() > longest_match {
                                matched_trait = Some(trait_name.clone());
                                longest_match = trait_name.len();
                            }
                        }
                    }

                    // 如果没有匹配到，检查是否使用下划线分隔的格式（传统格式）
                    if matched_trait.is_none() {
                        if let Some(underscore_pos) = filename_stem.find('_') {
                            let expected_trait = &filename_stem[..underscore_pos];
                            if defined_traits.contains(&expected_trait.to_string()) {
                                matched_trait = Some(expected_trait.to_string());
                            }
                        }
                    }

                    if let Some(expected_trait) = matched_trait {
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
                                    if t == &expected_trait {
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
                        // 文件名中的 trait 不在文件中定义，可能是正常的（外部 trait 或 Default 等内置 trait）
                        debug_log(format!("filename '{}' does not match any defined trait, skipping check", filename));
                    }
                }
            }
        }

        // 检查：禁止使用 pub mod 声明
        if context.config.checks.mod_rs.forbid_pub_mod && parsed.contains_pub_mod() {
            let line = parsed.find_keyword_line_number("pub mod");
            report.add_error_with_line(mod_rs_path.clone(), "mod.rs / lib.rs 禁止使用 pub mod 声明".to_string(), line);
        }

        // 检查：禁止使用通配符重导出
        if context.config.checks.mod_rs.forbid_wildcard_re_exports {
            for (path, line) in parsed.get_wildcard_re_exports() {
                report.add_error_with_line(
                    mod_rs_path.clone(),
                    format!("mod.rs / lib.rs 禁止使用通配符重导出：pub use {}::*;", path),
                    Some(line + 1)  // 转换为 1-indexed
                );
            }
        }

        // 检查：禁止内联 mod 声明 (mod xxx { ... })
        if context.config.checks.mod_rs.forbid_inline_mod {
            for (name, line) in parsed.get_inline_mods() {
                report.add_error_with_line(
                    mod_rs_path.clone(),
                    format!("mod.rs / lib.rs 禁止使用内联 mod 声明：mod {} {{ ... }}", name),
                    Some(line + 1)  // 转换为 1-indexed
                );
            }
        }

        // 检查：目录结构规范
        // 1. 只有 internal/ 和 tests/ 是允许的特殊目录
        // 2. 特殊目录不能有 mod.rs
        // 3. 其他目录必须有 mod.rs 或 lib.rs
        if context.config.checks.mod_rs.require_dir_has_entry {
            let module_dir = context.module_dir;
            if let Ok(entries) = fs::read_dir(module_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let dir_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    // 跳过以 . 开头的隐藏目录
                    if dir_name.starts_with('.') {
                        continue;
                    }

                    // 特殊目录只能是 internal 或 tests
                    let is_special_dir = dir_name == "internal" || dir_name == "tests";

                    if is_special_dir {
                        // 特殊目录不能有 mod.rs
                        let special_mod_rs = path.join("mod.rs");
                        if special_mod_rs.exists() {
                            report.add_error(
                                special_mod_rs,
                                format!("{}/ 目录是特殊目录，禁止存在 mod.rs", dir_name)
                            );
                        }
                    } else {
                        // 其他目录必须有 mod.rs 或 lib.rs
                        let has_mod_rs = path.join("mod.rs").exists();
                        let has_lib_rs = path.join("lib.rs").exists();

                        if !has_mod_rs && !has_lib_rs {
                            // 检查是否有对应的单文件模块 (xxx.rs)
                            let rs_file = module_dir.join(format!("{}.rs", dir_name));

                            if !rs_file.exists() {
                                report.add_error(
                                    path.clone(),
                                    format!(
                                        "目录 {} 必须包含 mod.rs 或 lib.rs 文件（只有 internal/ 和 tests/ 是例外）",
                                        dir_name
                                    )
                                );
                            }
                        }
                    }
                }
            }
        }

        report
    }
}
