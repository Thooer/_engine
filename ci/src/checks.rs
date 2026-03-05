use crate::config::Config;
use crate::parser::ParsedFile;
use crate::report::Report;
use std::path::{Path, PathBuf};
use std::fs;

pub struct Checker {
    config: Config,
    report: Report,
}

impl Checker {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            report: Report::new(),
        }
    }

    pub fn check(&mut self, root: &Path) -> anyhow::Result<Report> {
        if !self.config.checks.enabled {
            return Ok(self.report.clone());
        }

        let root_path = root.to_path_buf();
        self.check_module(&root_path)?;
        Ok(self.report.clone())
    }

    fn check_module(&mut self, dir: &PathBuf) -> anyhow::Result<()> {
        // 检查是否在白名单中
        if self.config.is_whitelisted(dir) {
            return Ok(());
        }

        // 检查是否被排除
        if self.config.is_path_excluded(dir) {
            return Ok(());
        }

        let mod_rs_path = dir.join("mod.rs");
        let lib_rs_path = dir.join("lib.rs");

        // 1. 选择“模块入口文件”：优先使用 mod.rs，其次是 lib.rs
        let module_entry = if mod_rs_path.exists() {
            Some(mod_rs_path.clone())
        } else if lib_rs_path.exists() {
            Some(lib_rs_path.clone())
        } else {
            None
        };

        // 对模块入口文件应用 mod.rs 的规则（lib.rs 视为等价的模块入口）
        if let Some(ref module_path) = module_entry {
            self.check_mod_rs(module_path)?;
        }

        // 2. 检查同级的 impl 文件
        self.check_impl_files(dir)?;

        // 3. 检查子目录
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // 子目录级别也应用白名单 / 排除规则
            if self.config.is_whitelisted(&path) {
                continue;
            }

            if self.config.is_path_excluded(&path) {
                continue;
            }
            
            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name().unwrap().to_string_lossy();

            if dir_name == "internal" {
                self.check_internal_dir(&path)?;
            } else if dir_name == "tests" {
                // tests/ 目录的“父模块文件”：优先使用 mod.rs，其次是 lib.rs
                let parent_module = if mod_rs_path.exists() {
                    mod_rs_path.clone()
                } else if lib_rs_path.exists() {
                    lib_rs_path.clone()
                } else {
                    // 维持原有行为：如果都不存在，仍然把 mod.rs 的路径（即便文件不存在）传进去，
                    // 由 tests 检查逻辑给出“父目录没有 mod.rs”的错误提示。
                    mod_rs_path.clone()
                };

                self.check_tests_dir(&path, &parent_module)?;
            } else {
                // 递归检查其他目录
                self.check_module(&path)?;
            }
        }

        Ok(())
    }

    fn check_mod_rs(&mut self, mod_rs_path: &PathBuf) -> anyhow::Result<()> {
        if !self.config.checks.mod_rs.enabled {
            return Ok(());
        }

        let parsed = ParsedFile::parse(mod_rs_path.clone())?;

        // 检查：禁止包含任何形式的 impl
        if self.config.checks.mod_rs.forbid_impl && parsed.contains_impl() {
            let line = parsed.find_keyword_line_number("impl");
            self.report.add_error_with_line(
                mod_rs_path.clone(),
                "mod.rs 禁止包含任何形式的 impl".to_string(),
                line,
            );
        }

        // 检查：禁止使用 include!（include! 只能放在实现文件中）
        if parsed.contains_include_macro() {
            let line = parsed.find_include_macro_line_number();
            self.report.add_error_with_line(
                mod_rs_path.clone(),
                "mod.rs 禁止使用 include! 宏（include! 只能放在实现文件中）".to_string(),
                line,
            );
        }

        // 检查：mod.rs 中的 struct 必须是公开的（pub struct）
        if self.config.checks.mod_rs.struct_must_be_public {
            for struct_name in parsed.get_non_public_struct_names() {
                let line = parsed.find_struct_line_number(&struct_name);
                self.report.add_error_with_line(
                    mod_rs_path.clone(),
                    format!(
                        "mod.rs 中的 struct 必须是公开的（pub struct）：{}。如果它是内部结构，请移动到实现文件中",
                        struct_name
                    ),
                    line,
                );
            }
        }

        // 检查：mod.rs / lib.rs 禁止存在任何顶层函数
        if self.config.checks.mod_rs.forbid_free_functions {
            let top_level_fns = parsed.get_top_level_function_names();
            for fn_name in top_level_fns {
                let line = parsed.find_function_line_number(&fn_name);
                self.report.add_error_with_line(
                    mod_rs_path.clone(),
                    format!(
                        "mod.rs / lib.rs 中禁止存在顶层函数：fn {}。所有方法必须通过 trait 暴露，并由实现文件中的 impl 实现",
                        fn_name
                    ),
                    line,
                );
            }
        }

        Ok(())
    }

    fn check_impl_files(&mut self, dir: &PathBuf) -> anyhow::Result<()> {
        if !self.config.checks.impl_file.enabled {
            return Ok(());
        }

        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }

            // 检查是否被排除
            if self.config.is_path_excluded(&path) {
                continue;
            }

            // 检查是否在白名单中
            if self.config.is_whitelisted(&path) {
                continue;
            }

            let file_name = path.file_name().unwrap().to_string_lossy();
            
            // 跳过 mod.rs
            if file_name == "mod.rs" {
                continue;
            }

            // 只检查 .rs 文件
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            // 跳过明显不是 impl 文件的文件（如 main.rs, lib.rs 等）
            let file_stem = path.file_stem().unwrap().to_string_lossy();
            if file_stem == "main" || file_stem == "lib" {
                continue;
            }

            // 检查命名：禁止 xxximpl 或 xxxtests
            if self.config.checks.naming.enabled {
                let name_lower = file_name.to_lowercase();
                if self.config.checks.naming.forbid_impl_suffix 
                    && (name_lower.ends_with("impl.rs") || name_lower.contains("_impl.rs")) {
                    self.report.add_error(
                        path.clone(),
                        format!("禁止使用 *impl.rs 命名：{}", file_name),
                    );
                    continue;
                }
                if self.config.checks.naming.forbid_tests_suffix 
                    && (name_lower.ends_with("tests.rs") || name_lower.ends_with("test.rs")) {
                    self.report.add_error(
                        path.clone(),
                        format!("禁止使用 *tests.rs 命名：{}", file_name),
                    );
                    continue;
                }
            }

            // 解析文件（先解析，以便检查结构性问题）
            let parsed = match ParsedFile::parse(path.clone()) {
                Ok(p) => p,
                Err(e) => {
                    self.report.add_error(
                        path.clone(),
                        format!("解析文件失败: {}", e),
                    );
                    continue;
                }
            };

            // 检查：include! 只能出现在“实现文件”（即包含 impl 块的文件）中
            // 目的：internal/ 内容必须被实现文件 include，而不是由 mod.rs 或普通文件 include。
            if parsed.contains_include_macro() && parsed.count_impl_blocks() == 0 {
                let line = parsed.find_include_macro_line_number();
                self.report.add_error_with_line(
                    path.clone(),
                    "include! 只能放在实现文件中（必须包含 impl 块）".to_string(),
                    line,
                );
                continue;
            }

            // 如果文件不包含 impl 块，跳过（不是所有 .rs 文件都必须是 impl 文件）
            let impl_count = parsed.count_impl_blocks();
            if impl_count == 0 {
                continue;
            }

            // ========== 固有实现检查 ==========
            // 检查：禁止固有实现（inherent impl），强制所有实现都通过 trait
            if self.config.checks.impl_file.forbid_inherent_impl && parsed.has_inherent_impl() {
                let inherent_types = parsed.get_inherent_impl_types();
                let line = parsed.find_impl_line_number();
                
                if inherent_types.is_empty() {
                    self.report.add_error_with_line(
                        path.clone(),
                        "禁止固有实现（inherent impl）。所有方法必须通过 trait 暴露，然后在实现文件中 impl trait".to_string(),
                        line,
                    );
                } else {
                    let type_list = inherent_types.join("、");
                    self.report.add_error_with_line(
                        path.clone(),
                        format!(
                            "禁止固有实现（inherent impl）：{}。所有方法必须通过 trait 暴露，然后在实现文件中 impl trait",
                            type_list
                        ),
                        line,
                    );
                }
                continue;
            }

            // ========== 结构性问题检查 ==========
            // 检查：必须只有一个 impl 块（除非所有 impl 块都是空的且是同一个 trait）
            let has_multiple_impls = impl_count > 1;
            let is_empty_impls_exception = has_multiple_impls 
                && parsed.are_all_impls_empty() 
                && parsed.are_all_impls_same_trait();
            
            if self.config.checks.impl_file.single_impl_only {
                if has_multiple_impls && !is_empty_impls_exception {
                    let line = parsed.find_impl_line_number();
                    // 获取所有impl块的信息，生成期望的文件名列表
                    let all_impl_info = parsed.get_all_impl_info();
                    let mut expected_files = Vec::new();
                    
                    for (trait_name, type_name) in &all_impl_info {
                        if let (Some(trait_n), Some(ty)) = (trait_name, type_name) {
                            expected_files.push(format!("{}_{}.rs", trait_n, ty));
                        } else if let Some(trait_n) = trait_name {
                            // 只有trait名，没有类型名（不应该发生，但处理一下）
                            expected_files.push(format!("{}_Unknown.rs", trait_n));
                        } else if let Some(ty) = type_name {
                            // 固有实现，没有trait
                            expected_files.push(format!("Inherent_{}.rs", ty));
                        } else {
                            expected_files.push("Inherent_Unknown.rs".to_string());
                        }
                    }
                    
                    let error_msg = if expected_files.is_empty() {
                        format!("impl 文件只能包含一个 impl 块，但发现了 {} 个", impl_count)
                    } else {
                        format!(
                            "impl 文件只能包含一个 impl 块，但发现了 {} 个，期望拆分成 {} 个文件：{}",
                            impl_count,
                            expected_files.len(),
                            expected_files.join("、")
                        )
                    };
                    
                    self.report.add_error_with_line(
                        path.clone(),
                        error_msg,
                        line,
                    );
                }
            }

            // ========== 细节检查 ==========
            // 如果符合空 impl 块例外规则，跳过命名检查（因为多个空impl块可以放在同一个文件中）
            // 检查文件命名规范：impl 文件必须和 trait 对应，格式为 {TraitName}_{TypeName}.rs
            if (!has_multiple_impls || !is_empty_impls_exception) && self.config.checks.impl_file.naming_must_match_trait {
                // 检查：文件命名必须和 trait 对应（格式为 {TraitName}_{TypeName}.rs）
                if let Some(trait_name) = parsed.get_impl_trait_name() {
                    let actual_name = path.file_stem().unwrap().to_string_lossy();
                    
                    // 格式：{TraitName}_{TypeName}（使用单个下划线分隔符，PascalCase）
                    if let Some(type_name) = parsed.get_impl_type_name() {
                        let expected_name = format!("{}_{}", trait_name, type_name);
                        
                        if actual_name != expected_name {
                            let line = parsed.find_impl_line_number();
                            self.report.add_error_with_line(
                                path.clone(),
                                format!(
                                    "文件命名必须和 trait 对应：期望 {}，实际 {}",
                                    expected_name, actual_name
                                ),
                                line,
                            );
                        }
                    } else {
                        // 如果无法获取 type_name，仍然要求使用 {TraitName}_{TypeName} 格式
                        let line = parsed.find_impl_line_number();
                        self.report.add_error_with_line(
                            path.clone(),
                            format!(
                                "文件命名必须使用 {{TraitName}}_{{TypeName}}.rs 格式，实际 {}",
                                actual_name
                            ),
                            line,
                        );
                    }
                }
            }

            // 检查：禁止 pub 关键字
            if self.config.checks.impl_file.forbid_pub && parsed.contains_pub() {
                let line = parsed.find_keyword_line_number("pub ");
                self.report.add_error_with_line(
                    path.clone(),
                    "impl 文件禁止使用 pub 关键字".to_string(),
                    line,
                );
            }
        }

        Ok(())
    }

    fn check_internal_dir(&mut self, internal_dir: &PathBuf) -> anyhow::Result<()> {
        if !self.config.checks.internal.enabled {
            return Ok(());
        }

        // 检查：禁止存在 mod.rs
        let mod_rs_path = internal_dir.join("mod.rs");
        if self.config.checks.internal.forbid_mod_rs && mod_rs_path.exists() {
            self.report.add_error(
                mod_rs_path,
                "internal/ 目录禁止存在 mod.rs".to_string(),
            );
        }

        // 检查所有 .rs 文件
        let entries = fs::read_dir(internal_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let parsed = match ParsedFile::parse(path.clone()) {
                Ok(p) => p,
                Err(e) => {
                    self.report.add_error(
                        path.clone(),
                        format!("解析文件失败: {}", e),
                    );
                    continue;
                }
            };

            // 检查：必须用大括号包裹
            if self.config.checks.internal.require_brace_wrap && !parsed.is_brace_wrapped() {
                self.report.add_error(
                    path.clone(),
                    "internal/ 目录中的文件必须用大括号 {} 包裹".to_string(),
                );
            }

            // 检查：只包含函数体，不包含方法签名
            if self.config.checks.internal.only_function_body && parsed.has_function_signatures() {
                let line = parsed.find_keyword_line_number("fn ");
                self.report.add_error_with_line(
                    path.clone(),
                    "internal/ 目录中的文件只能包含函数体，不能包含方法签名".to_string(),
                    line,
                );
            }
        }

        Ok(())
    }

    fn check_tests_dir(&mut self, tests_dir: &PathBuf, mod_rs_path: &PathBuf) -> anyhow::Result<()> {
        if !self.config.checks.tests.enabled {
            return Ok(());
        }

        // 检查：禁止存在 mod.rs
        let tests_mod_rs = tests_dir.join("mod.rs");
        if self.config.checks.tests.forbid_mod_rs && tests_mod_rs.exists() {
            self.report.add_error(
                tests_mod_rs,
                "tests/ 目录禁止存在 mod.rs".to_string(),
            );
        }

        // 检查：必须在 mod.rs 中声明模块
        if self.config.checks.tests.require_mod_declaration_in_parent && mod_rs_path.exists() {
            let mod_rs_parsed = match ParsedFile::parse(mod_rs_path.clone()) {
                Ok(p) => p,
                Err(e) => {
                    self.report.add_error(
                        mod_rs_path.clone(),
                        format!("解析 mod.rs 失败: {}", e),
                    );
                    return Ok(());
                }
            };
            
            // 使用 AST 检查是否有 tests 模块声明
            if !mod_rs_parsed.has_module("tests") {
                self.report.add_error(
                    mod_rs_path.clone(),
                    "tests/ 目录必须在 mod.rs 中声明模块".to_string(),
                );
            } else {
                // 检查模块声明是否跟在某个 trait 后面
                let modules_after_traits = mod_rs_parsed.get_modules_after_traits();
                let tests_module = modules_after_traits.iter()
                    .find(|(name, _)| name == "tests");
                
                if let Some((_, trait_name)) = tests_module {
                    if trait_name.is_none() {
                        let line = mod_rs_parsed.find_module_line_number("tests");
                        self.report.add_error_with_line(
                            mod_rs_path.clone(),
                            "tests/ 模块声明必须跟在某个 trait 后面".to_string(),
                            line,
                        );
                    }
                }
            }
        } else if self.config.checks.tests.require_mod_declaration_in_parent && !mod_rs_path.exists() {
            self.report.add_error(
                mod_rs_path.clone(),
                "tests/ 目录存在但父目录没有 mod.rs".to_string(),
            );
        }

        // 检查所有测试文件
        let entries = fs::read_dir(tests_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            // 检查：测试文件必须和 trait 同名
            if self.config.checks.tests.test_file_must_match_trait {
                let file_stem = path.file_stem().unwrap().to_string_lossy();
                
                // 尝试在父目录中找到对应的 impl 文件
                let parent_dir = tests_dir.parent().unwrap();
                let impl_file = parent_dir.join(format!("{}.rs", file_stem));
                
                if !impl_file.exists() {
                    // 使用 AST 检查是否有对应的 trait 在 mod.rs 中
                    if mod_rs_path.exists() {
                        let mod_rs_parsed = match ParsedFile::parse(mod_rs_path.clone()) {
                            Ok(p) => p,
                            Err(e) => {
                                self.report.add_error(
                                    mod_rs_path.clone(),
                                    format!("解析 mod.rs 失败: {}", e),
                                );
                                continue;
                            }
                        };
                        
                        // 使用 AST 检查 trait 是否存在
                        if !mod_rs_parsed.has_trait(&file_stem) {
                            // 尝试大小写不敏感匹配
                            let trait_names = mod_rs_parsed.get_trait_names();
                            let found = trait_names.iter()
                                .any(|name| name.to_lowercase() == file_stem.to_lowercase());
                            
                            if !found {
                                self.report.add_error(
                                    path.clone(),
                                    format!(
                                        "测试文件 {} 必须和某个 trait 同名，但在 mod.rs 中未找到对应的 trait（找到的 trait: {}）",
                                        file_stem,
                                        if trait_names.is_empty() {
                                            "无".to_string()
                                        } else {
                                            trait_names.join(", ")
                                        }
                                    ),
                                );
                            }
                        }
                    } else {
                        self.report.add_error(
                            path.clone(),
                            format!(
                                "测试文件 {} 必须和某个 trait 同名，但父目录没有 mod.rs",
                                file_stem
                            ),
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
