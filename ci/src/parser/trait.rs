use std::path::{Path, PathBuf};

/// Parser trait - 抽象解析器接口
pub trait Parser: Send + Sync {
    /// 解析文件，返回ParsedSource
    fn parse(&self, path: &Path) -> anyhow::Result<Box<dyn ParsedSource>>;
}

/// ParsedSource trait - 解析后的源代码抽象
pub trait ParsedSource: Send + Sync {
    // ========== 文件信息 ==========
    
    /// 获取文件路径
    fn path(&self) -> &Path;
    
    /// 获取文件内容
    fn content(&self) -> &str;
    
    // ========== Impl 查询 ==========
    
    /// 获取所有 impl 块
    fn impls(&self) -> &[ImplBlock];
    
    /// 获取 impl 块数量
    fn impl_count(&self) -> usize;
    
    /// 获取第一个 impl 块的 trait 名称
    fn get_impl_trait_name(&self) -> Option<String>;
    
    /// 获取第一个 impl 块的类型名称
    fn get_impl_type_name(&self) -> Option<String>;
    
    /// 检查是否包含固有实现（inherent impl）
    fn has_inherent_impl(&self) -> bool;
    
    /// 获取所有固有实现的类型名
    fn get_inherent_impl_types(&self) -> Vec<String>;
    
    /// 获取所有 impl 块的信息
    fn get_all_impl_info(&self) -> Vec<(Option<String>, Option<String>)>;
    
    /// 检查是否所有 impl 块都是空的（marker trait）
    fn are_all_impls_empty(&self) -> bool;
    
    /// 检查是否所有 impl 块都是同一个 trait
    fn are_all_impls_same_trait(&self) -> bool;
    
    // ========== Trait 查询 ==========
    
    /// 获取所有 trait 块
    fn traits(&self) -> &[TraitBlock];
    
    /// 获取所有 trait 名称
    fn get_trait_names(&self) -> Vec<String>;
    
    /// 检查是否包含指定名称的 trait
    fn has_trait(&self, name: &str) -> bool;
    
    // ========== Struct 查询 ==========
    
    /// 获取所有非 pub struct 名称
    fn get_non_public_struct_names(&self) -> Vec<String>;
    
    // ========== Module 查询 ==========
    
    /// 获取所有模块块
    fn modules(&self) -> &[ModuleBlock];
    
    /// 检查是否包含指定名称的模块
    fn has_module(&self, name: &str) -> bool;
    
    /// 获取所有模块名称
    fn get_module_names(&self) -> Vec<String>;
    
    /// 获取模块声明是否跟在 trait 后面
    fn get_modules_after_traits(&self) -> Vec<(String, Option<String>)>;
    
    /// 获取 trait 后的 impl 模块声明，包含 #[path = "..."] 信息
    /// 返回: (trait_name, mod_name, path_filename, mod_line_number)
    fn get_trait_impl_mods(&self) -> Vec<(Option<String>, String, Option<String>, usize)>;
    
    // ========== 函数查询 ==========
    
    /// 获取所有顶层函数名称
    fn get_top_level_function_names(&self) -> Vec<String>;

    // 代码提取（用于 fixer）
    fn extract_impl_code(&self, index: usize) -> Option<String>;
    fn extract_all_impl_codes(&self) -> Vec<String>;
    fn extract_function_code(&self, name: &str) -> Option<String>;
    fn extract_all_top_level_function_codes(&self) -> Vec<(String, String)>;
    fn get_use_statements(&self) -> Vec<String>;
    
    // ========== 宏/特殊检查 ==========
    
    /// 检查是否包含 impl
    fn contains_impl(&self) -> bool;
    
    /// 检查是否包含 pub
    fn contains_pub(&self) -> bool;
    
    /// 检查是否包含 include! 宏
    fn contains_include_macro(&self) -> bool;
    
    /// 检查是否用大括号包裹
    fn is_brace_wrapped(&self) -> bool;
    
    /// 检查是否包含函数签名
    fn has_function_signatures(&self) -> bool;
    
    // ========== 行号查找 ==========
    
    /// 查找 impl 块行号
    fn find_impl_line_number(&self) -> Option<usize>;
    
    /// 查找关键字行号
    fn find_keyword_line_number(&self, keyword: &str) -> Option<usize>;
    
    /// 查找 struct 行号
    fn find_struct_line_number(&self, name: &str) -> Option<usize>;
    
    /// 查找函数行号
    fn find_function_line_number(&self, name: &str) -> Option<usize>;
    
    /// 查找模块行号
    fn find_module_line_number(&self, name: &str) -> Option<usize>;
    
    /// 查找 include! 宏行号
    fn find_include_macro_line_number(&self) -> Option<usize>;
}

// ========== 辅助类型 ==========

/// Impl 块信息
#[derive(Debug, Clone)]
pub struct ImplBlock {
    /// trait 名称（如果有）
    pub trait_name: Option<String>,
    /// 类型名称
    pub type_name: Option<String>,
    /// 是否为空（marker trait）
    pub is_empty: bool,
    /// 行号
    pub line_number: Option<usize>,
}

/// Trait 块信息
#[derive(Debug, Clone)]
pub struct TraitBlock {
    pub name: String,
    pub line_number: Option<usize>,
}

/// Module 块信息
#[derive(Debug, Clone)]
pub struct ModuleBlock {
    pub name: String,
    pub line_number: Option<usize>,
}
