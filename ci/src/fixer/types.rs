use std::path::PathBuf;

/// 修复建议
#[derive(Debug, Clone)]
pub enum Fix {
    /// 重命名文件
    RenameFile {
        from: PathBuf,
        to: PathBuf,
    },
    /// 拆分 impl 文件（多个 impl 块拆分成多个文件）
    SplitImplFile {
        path: PathBuf,
        impl_codes: Vec<(String, String)>, // (文件名, impl代码)
    },
    /// 移除 pub 关键字
    RemovePub {
        path: PathBuf,
    },
    /// 将 impl 移动到单独文件（从 mod.rs 移动到单独文件）
    MoveImplToFile {
        path: PathBuf,
        impl_codes: Vec<(String, String)>, // (文件名, impl代码)
    },
    /// 添加模块声明
    AddModuleDeclaration {
        path: PathBuf,
        module_name: String,
    },
}

/// 修复结果
#[derive(Debug)]
pub struct FixResult {
    pub fixes: Vec<(crate::report::CheckResult, Fix)>,
    pub errors: Vec<(crate::report::CheckResult, String)>,
}

impl FixResult {
    pub fn new() -> Self {
        Self {
            fixes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn add_fix(&mut self, error: crate::report::CheckResult, fix: Fix) {
        self.fixes.push((error, fix));
    }

    pub fn add_error(&mut self, error: crate::report::CheckResult, message: String) {
        self.errors.push((error, message));
    }

    pub fn has_fixes(&self) -> bool {
        !self.fixes.is_empty()
    }

    pub fn fix_count(&self) -> usize {
        self.fixes.len()
    }
}

impl Default for FixResult {
    fn default() -> Self {
        Self::new()
    }
}
