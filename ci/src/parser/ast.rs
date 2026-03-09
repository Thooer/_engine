use std::path::Path;

/// Parser trait - 抽象解析器接口
pub trait Parser: Send + Sync {
    /// 解析文件，返回ParsedSource
    fn parse(&self, path: &Path) -> anyhow::Result<Box<dyn ParsedSource>>;
}

/// ParsedSource trait - 解析后的源代码抽象
pub trait ParsedSource: Send + Sync {
    // 文件信息
    fn path(&self) -> &Path;
    fn content(&self) -> &str;
    
    // Impl 查询
    fn impls(&self) -> &[ImplBlock];
    fn impl_count(&self) -> usize;
    fn get_impl_trait_name(&self) -> Option<String>;
    fn get_impl_type_name(&self) -> Option<String>;
    fn has_inherent_impl(&self) -> bool;
    fn get_inherent_impl_types(&self) -> Vec<String>;
    fn get_all_impl_info(&self) -> Vec<(Option<String>, Option<String>)>;
    fn are_all_impls_empty(&self) -> bool;
    fn are_all_impls_same_trait(&self) -> bool;
    
    // Trait 查询
    fn traits(&self) -> &[TraitBlock];
    fn get_trait_names(&self) -> Vec<String>;
    fn has_trait(&self, name: &str) -> bool;
    
    // Struct 查询
    fn get_non_public_struct_names(&self) -> Vec<String>;
    
    // Module 查询
    fn modules(&self) -> &[ModuleBlock];
    fn has_module(&self, name: &str) -> bool;
    fn get_module_names(&self) -> Vec<String>;
    fn get_modules_after_traits(&self) -> Vec<(String, Option<String>)>;
    
    fn get_trait_impl_mods(&self) -> Vec<(Option<String>, String, Option<String>, usize)>;
    
    // 函数查询
    fn get_top_level_function_names(&self) -> Vec<String>;
    
    // 代码提取（用于 fixer）
    fn extract_impl_code(&self, index: usize) -> Option<String>;
    fn extract_all_impl_codes(&self) -> Vec<String>;
    fn extract_function_code(&self, name: &str) -> Option<String>;
    fn extract_all_top_level_function_codes(&self) -> Vec<(String, String)>;
    fn get_use_statements(&self) -> Vec<String>;
    
    // 宏/特殊检查
    fn contains_impl(&self) -> bool;
    fn contains_pub(&self) -> bool;
    fn contains_include_macro(&self) -> bool;
    fn is_brace_wrapped(&self) -> bool;
    fn has_function_signatures(&self) -> bool;
    
    // 行号查找
    fn find_impl_line_number(&self) -> Option<usize>;
    fn find_keyword_line_number(&self, keyword: &str) -> Option<usize>;
    fn find_struct_line_number(&self, name: &str) -> Option<usize>;
    fn find_function_line_number(&self, name: &str) -> Option<usize>;
    fn find_module_line_number(&self, name: &str) -> Option<usize>;
    fn find_include_macro_line_number(&self) -> Option<usize>;
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub trait_name: Option<String>,
    pub type_name: Option<String>,
    pub is_empty: bool,
    pub line_number: Option<usize>,
    pub start_line: Option<usize>,  // impl 块开始行（1-indexed）
    pub end_line: Option<usize>,    // impl 块结束行（1-indexed）
}

#[derive(Debug, Clone)]
pub struct TraitBlock {
    pub name: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ModuleBlock {
    pub name: String,
    pub line_number: Option<usize>,
}
