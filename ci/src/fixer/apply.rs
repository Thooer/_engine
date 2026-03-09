use std::fs;
use std::path::PathBuf;
use crate::config::Config;
use crate::parser::{Parser, SynParser};
use crate::report::{CheckResult, Report};
use super::types::{Fix, FixResult};

/// Fixer - 自动修复器
#[allow(dead_code)]
pub struct Fixer {
    config: Config,
}

impl Fixer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 根据报告自动修复问题
    pub fn fix(&self, report: &Report) -> anyhow::Result<FixResult> {
        let mut fix_result = FixResult::new();

        for error in &report.errors {
            if let Some(fix) = self.suggest_fix(error) {
                fix_result.add_fix(error.clone(), fix);
            }
        }

        Ok(fix_result)
    }

    /// 实际执行修复
    pub fn apply_fixes(&self, report: &Report) -> anyhow::Result<FixResult> {
        let mut fix_result = FixResult::new();

        for error in &report.errors {
            if let Some(fix) = self.suggest_fix(error) {
                match self.apply_fix(error, &fix) {
                    Ok(_) => fix_result.add_fix(error.clone(), fix),
                    Err(e) => {
                        fix_result.add_error(error.clone(), format!("修复失败: {}", e));
                    }
                }
            }
        }

        Ok(fix_result)
    }

    fn suggest_fix(&self, error: &CheckResult) -> Option<Fix> {
        let message = &error.message;

        // 文件命名问题 - 建议重命名
        if message.contains("文件命名必须和 trait 对应") {
            if let Some(expected) = self.extract_expected_name(message) {
                return Some(Fix::RenameFile {
                    from: error.path.clone(),
                    to: error.path.parent().unwrap().join(format!("{}.rs", expected)),
                });
            }
        }

        // 多个 impl 块 - 建议拆分文件
        if message.contains("impl 文件只能包含一个 impl 块") {
            // 解析消息，提取期望的文件名列表
            let impl_codes = self.prepare_split_impl_files(&error.path);
            return Some(Fix::SplitImplFile {
                path: error.path.clone(),
                impl_codes,
            });
        }

        // impl 文件禁止使用 pub - 建议移除 pub
        if message.contains("impl 文件禁止使用 pub") {
            return Some(Fix::RemovePub {
                path: error.path.clone(),
            });
        }

        // mod.rs 禁止包含 impl
        if message.contains("mod.rs 禁止包含任何形式的 impl") {
            // 准备移动 impl 到单独文件
            let impl_codes = self.prepare_move_impl_to_file(&error.path);
            return Some(Fix::MoveImplToFile {
                path: error.path.clone(),
                impl_codes,
            });
        }

        // tests 目录必须在 mod.rs 中声明
        if message.contains("tests/ 目录必须在 mod.rs 中声明模块") {
            return Some(Fix::AddModuleDeclaration {
                path: error.path.clone(),
                module_name: "tests".to_string(),
            });
        }

        None
    }

    fn apply_fix(&self, _error: &CheckResult, fix: &Fix) -> anyhow::Result<()> {
        match fix {
            Fix::RenameFile { from, to } => {
                println!("重命名文件: {} -> {}", from.display(), to.display());
                fs::rename(from, to)?;
            }
            Fix::SplitImplFile { path, impl_codes } => {
                println!("拆分文件: {} 到 {} 个文件", path.display(), impl_codes.len());

                // 获取父目录
                let parent = path.parent().unwrap_or(std::path::Path::new("."));

                // 为每个 impl 创建新文件
                for (filename, impl_code) in impl_codes {
                    let new_file_path = parent.join(&filename);
                    println!("  创建文件: {}", new_file_path.display());

                    // 写入 impl 代码（简化版本：直接写入 impl 块）
                    fs::write(&new_file_path, impl_code)?;
                }

                // 删除原文件
                println!("  删除原文件: {}", path.display());
                fs::remove_file(path)?;
            }
            Fix::RemovePub { path } => {
                println!("移除 pub 关键字: {}", path.display());
                let content = fs::read_to_string(path)?;
                let new_content = content.replace("pub ", "");
                fs::write(path, new_content)?;
            }
            Fix::MoveImplToFile { path, impl_codes } => {
                println!("移动 impl 到单独文件: {} 到 {} 个文件", path.display(), impl_codes.len());

                // 获取父目录
                let parent = path.parent().unwrap_or(std::path::Path::new("."));

                // 为每个 impl 创建新文件
                for (filename, impl_code) in impl_codes {
                    let new_file_path = parent.join(&filename);
                    println!("  创建文件: {}", new_file_path.display());

                    // 写入 impl 代码
                    fs::write(&new_file_path, impl_code)?;
                }

                // 修改 mod.rs，删除 impl 块并添加模块声明
                let content = fs::read_to_string(path)?;
                let new_content = self.remove_impl_blocks_and_add_declarations(&content, impl_codes.len());
                fs::write(path, new_content)?;
            }
            Fix::AddModuleDeclaration { path, module_name } => {
                println!("添加模块声明: mod {} in {}", module_name, path.display());
                let content = fs::read_to_string(path)?;
                let new_content = format!("mod {};\n\n{}", module_name, content);
                fs::write(path, new_content)?;
            }
        }
        Ok(())
    }

    /// 从 mod.rs 中删除 impl 块并添加模块声明
    fn remove_impl_blocks_and_add_declarations(&self, content: &str, _impl_count: usize) -> String {
        let mut result = String::new();
        let mut in_impl_block = false;
        let mut brace_count = 0;
        let mut impl_block_count = 0;
        let mut module_declarations = Vec::new();

        for line in content.lines() {
            if line.trim().starts_with("impl ") && !in_impl_block {
                // 开始一个新 impl 块
                in_impl_block = true;
                brace_count = 0;
                impl_block_count += 1;
                // 生成对应的模块声明
                module_declarations.push(format!("mod impl_{};", impl_block_count));
            }

            if in_impl_block {
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;

                if brace_count == 0 {
                    in_impl_block = false;
                }
                // 跳过 impl 块内的所有内容
                continue;
            }

            // 保留非 impl 块的内容
            result.push_str(line);
            result.push('\n');
        }

        // 在文件开头添加模块声明
        let mut final_result = String::new();
        for decl in &module_declarations {
            final_result.push_str(decl);
            final_result.push('\n');
        }
        final_result.push('\n');
        final_result.push_str(&result);

        final_result.trim().to_string()
    }

    fn extract_expected_name(&self, message: &str) -> Option<String> {
        // 从消息中提取期望的文件名
        // 格式: "文件命名必须和 trait 对应：期望 Foo_Bar，实际 Foo"

        // 用 "期望 " 分割，然后取第二部分，再用 "，实际" 分割
        if let Some(after_expected) = message.split("期望 ").nth(1) {
            if let Some(before_actual) = after_expected.split("，实际").next() {
                let result = before_actual.trim().to_string();
                if !result.is_empty() {
                    return Some(result);
                }
            }
        }

        None
    }

    /// 准备拆分 impl 文件所需的信息
    fn prepare_split_impl_files(&self, path: &PathBuf) -> Vec<(String, String)> {
        let parser = SynParser::new();
        if let Ok(parsed) = parser.parse(path) {
            let all_impl_info = parsed.get_all_impl_info();
            let impl_codes = parsed.extract_all_impl_codes();

            all_impl_info.iter()
                .enumerate()
                .filter_map(|(i, (trait_name, type_name))| {
                    let code = impl_codes.get(i)?.clone();
                    let filename = if let (Some(tn), Some(ty)) = (trait_name, type_name) {
                        format!("{}_{}.rs", tn, ty)
                    } else if let Some(tn) = trait_name {
                        format!("{}_Unknown.rs", tn)
                    } else if let Some(ty) = type_name {
                        format!("Inherent_{}.rs", ty)
                    } else {
                        "Impl_Unknown.rs".to_string()
                    };
                    Some((filename, code))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 准备移动 impl 到单独文件所需的信息
    fn prepare_move_impl_to_file(&self, path: &PathBuf) -> Vec<(String, String)> {
        let parser = SynParser::new();
        if let Ok(parsed) = parser.parse(path) {
            let all_impl_info = parsed.get_all_impl_info();
            let impl_codes = parsed.extract_all_impl_codes();

            all_impl_info.iter()
                .enumerate()
                .filter_map(|(i, (trait_name, type_name))| {
                    let code = impl_codes.get(i)?.clone();
                    let filename = if let (Some(tn), Some(ty)) = (trait_name, type_name) {
                        format!("{}_{}.rs", tn, ty)
                    } else if let Some(tn) = trait_name {
                        format!("{}.rs", tn)
                    } else if let Some(ty) = type_name {
                        format!("{}.rs", ty)
                    } else {
                        "impl.rs".to_string()
                    };
                    Some((filename, code))
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
