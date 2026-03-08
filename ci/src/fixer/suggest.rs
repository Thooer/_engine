use crate::config::Config;
use crate::parser::{Parser, SynParser};
use crate::report::{CheckResult, Report};
use super::types::{Fix, FixResult};

/// 修复建议器
#[allow(dead_code)]
pub struct FixSuggester {
    config: Config,
}

impl FixSuggester {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 根据报告自动生成修复建议
    pub fn suggest(&self, report: &Report) -> anyhow::Result<FixResult> {
        let mut fix_result = FixResult::new();

        for error in &report.errors {
            if let Some(fix) = self.suggest_fix(error) {
                fix_result.add_fix(error.clone(), fix);
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

    fn extract_expected_name(&self, message: &str) -> Option<String> {
        // 从消息中提取期望的文件名
        // 格式: "文件命名必须和 trait 对应：期望 Foo_Bar，实际 Foo"

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
    fn prepare_split_impl_files(&self, path: &std::path::PathBuf) -> Vec<(String, String)> {
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
    fn prepare_move_impl_to_file(&self, path: &std::path::PathBuf) -> Vec<(String, String)> {
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
