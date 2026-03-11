use crate::parser::ast::{ImplBlock, ModuleBlock, ParsedSource, Parser, TraitBlock};
use std::path::{Path, PathBuf};
use syn::{visit::Visit, Item};

/// SynParser - 使用 syn 的解析器实现
pub struct SynParser;

impl SynParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SynParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for SynParser {
    fn parse(&self, path: &Path) -> anyhow::Result<Box<dyn ParsedSource>> {
        SynParsedSource::parse(path.to_path_buf())
    }
}

/// SynParsedSource - Syn 解析后的源代码实现
pub struct SynParsedSource {
    path: PathBuf,
    content: String,
    impls: Vec<ImplBlock>,
    traits: Vec<TraitBlock>,
    modules: Vec<ModuleBlock>,
    non_public_structs: Vec<String>,
    top_level_functions: Vec<String>,
    top_level_function_bodies: Vec<(String, Vec<String>)>,  // (函数名, 函数体内的语句)
    has_pub: bool,
    has_include_macro: bool,
    is_brace_wrapped: bool,
    has_fn_sigs: bool,
    has_pub_mod: bool,
    wildcard_re_exports: Vec<(String, usize)>,  // (导出路径, 行号)
    inline_mods: Vec<(String, usize)>,  // (模块名, 行号)
}

impl SynParsedSource {
    pub fn parse(path: PathBuf) -> anyhow::Result<Box<dyn ParsedSource>> {
        let content = std::fs::read_to_string(&path)?;
        
        let trimmed = content.trim();
        let parse_content = if trimmed.starts_with('{') && trimmed.ends_with('}') {
            let inner_content = &trimmed[1..trimmed.len()-1];
            format!("fn __internal_wrapper__() {{ {} }}", inner_content)
        } else {
            content.clone()
        };
        
        let ast = syn::parse_file(&parse_content).map_err(|e| {
            anyhow::anyhow!("cannot parse file {} into token stream: {}", path.display(), e)
        })?;

        let mut impls = Vec::new();
        
        for item in &ast.items {
            match item {
                Item::Impl(impl_item) => {
                    let trait_name = impl_item.trait_.as_ref()
                        .and_then(|(_, path, _)| path.segments.last().map(|seg| seg.ident.to_string()));
                    let type_name = extract_type_name(&impl_item.self_ty);
                    let is_empty = impl_item.items.is_empty();

                    // 行号信息暂时使用 None，通过文本搜索获取
                    let start_line = None;
                    let end_line = None;
                    
                    impls.push(ImplBlock {
                        trait_name,
                        type_name,
                        is_empty,
                        line_number: None,
                        start_line,
                        end_line,
                    });
                }
                _ => {}
            }
        }
        
        let traits: Vec<TraitBlock> = ast.items.iter()
            .filter_map(|item| {
                if let Item::Trait(t) = item {
                    Some(TraitBlock { name: t.ident.to_string(), line_number: None })
                } else {
                    None
                }
            })
            .collect();
        
        let modules: Vec<ModuleBlock> = ast.items.iter()
            .filter_map(|item| {
                if let Item::Mod(m) = item {
                    // 检查是 mod 还是 pub mod
                    let is_pub = matches!(m.vis, syn::Visibility::Public(_));
                    if !is_pub {
                        Some(ModuleBlock { name: m.ident.to_string(), line_number: None })
                    } else {
                        // pub mod 也需要被识别为模块
                        Some(ModuleBlock { name: m.ident.to_string(), line_number: None })
                    }
                } else {
                    None
                }
            })
            .collect();
        
        let mut pub_visitor = PubVisitor { found: false };
        pub_visitor.visit_file(&ast);
        let has_pub = pub_visitor.found;
        
        let mut include_visitor = IncludeVisitor { found: false };
        include_visitor.visit_file(&ast);
        let has_include_macro = include_visitor.found;
        
        let is_brace_wrapped = trimmed.starts_with('{') && trimmed.ends_with('}');
        
        let mut fn_sig_visitor = FnSigVisitor { found: false };
        fn_sig_visitor.visit_file(&ast);
        let has_fn_sigs = fn_sig_visitor.found;

        let mut pub_mod_visitor = PubModVisitor { found: false };
        pub_mod_visitor.visit_file(&ast);
        let has_pub_mod = pub_mod_visitor.found;

        let mut wildcard_visitor = WildcardReExportVisitor { exports: Vec::new(), content: content.clone() };
        wildcard_visitor.visit_file(&ast);
        let wildcard_re_exports = wildcard_visitor.exports;

        let mut inline_mod_visitor = InlineModVisitor { mods: Vec::new(), content: content.clone() };
        inline_mod_visitor.visit_file(&ast);
        let inline_mods = inline_mod_visitor.mods;
        
        let mut struct_visitor = StructVisitor { names: Vec::new() };
        struct_visitor.visit_file(&ast);
        let non_public_structs = struct_visitor.names;
        
        let top_level_functions: Vec<String> = ast.items.iter()
            .filter_map(|item| {
                if let Item::Fn(item_fn) = item {
                    Some(item_fn.sig.ident.to_string())
                } else {
                    None
                }
            })
            .collect();
        
        // 收集顶层函数体信息：提取每个函数的语句
        let top_level_function_bodies = extract_top_level_function_bodies(&content);
        
        Ok(Box::new(Self {
            path,
            content,
            impls,
            traits,
            modules,
            non_public_structs,
            top_level_functions,
            top_level_function_bodies,
            has_pub,
            has_include_macro,
            is_brace_wrapped,
            has_fn_sigs,
            has_pub_mod,
            wildcard_re_exports,
            inline_mods,
        }))
    }

    // 辅助函数：从行中提取模块名称
    fn extract_module_name(line: &str) -> Option<String> {
        if let Some(name) = line.strip_prefix("mod ") {
            if let Some(name) = name.split_whitespace().next() {
                return Some(name.trim_end_matches(';').to_string());
            }
        } else if let Some(name) = line.strip_prefix("pub mod ") {
            if let Some(name) = name.split_whitespace().next() {
                return Some(name.trim_end_matches(';').to_string());
            }
        }
        None
    }

    // 辅助函数：从属性行中提取模块名称（处理 #[mod] 或 #[cfg(...)] mod 等情况）
    // 返回 (模块名, 是否完整（即下一行不需要组合）)
    fn extract_module_name_from_attribute(line: &str) -> Option<(String, bool)> {
        // 检查是否有内联的 "mod " 或 "pub mod " 在属性后面
        // 例如: #[cfg(test)] mod tests;
        if let Some(rest) = line.strip_prefix("#[") {
            // 找到属性结束的位置
            if let Some(after_attr) = rest.find("] ") {
                let after = &rest[after_attr + 2..];
                if let Some(name) = Self::extract_module_name(after) {
                    return Some((name, true));
                }
            } else if let Some(after_attr) = rest.find("]") {
                let after = &rest[after_attr + 1..];
                if let Some(name) = Self::extract_module_name(after) {
                    return Some((name, true));
                }
            }
            // 如果没有找到内联的 mod，说明属性和 mod 声明是分开的
            // 返回 None，表示需要与下一行组合
            return None;
        }
        None
    }
}

impl ParsedSource for SynParsedSource {
    fn path(&self) -> &Path {
        &self.path
    }
    
    fn content(&self) -> &str {
        &self.content
    }
    
    fn impls(&self) -> &[ImplBlock] {
        &self.impls
    }
    
    fn impl_count(&self) -> usize {
        self.impls.len()
    }
    
    fn get_impl_trait_name(&self) -> Option<String> {
        self.impls.first().and_then(|i| i.trait_name.clone())
    }
    
    fn get_impl_type_name(&self) -> Option<String> {
        self.impls.first().and_then(|i| i.type_name.clone())
    }
    
    fn has_inherent_impl(&self) -> bool {
        self.impls.iter().any(|i| i.trait_name.is_none())
    }
    
    fn get_inherent_impl_types(&self) -> Vec<String> {
        self.impls.iter().filter(|i| i.trait_name.is_none()).filter_map(|i| i.type_name.clone()).collect()
    }
    
    fn get_all_impl_info(&self) -> Vec<(Option<String>, Option<String>)> {
        self.impls.iter().map(|i| (i.trait_name.clone(), i.type_name.clone())).collect()
    }
    
    fn are_all_impls_empty(&self) -> bool {
        self.impls.iter().all(|i| i.is_empty)
    }
    
    fn are_all_impls_same_trait(&self) -> bool {
        if self.impls.is_empty() { return false; }
        let first_trait = self.impls[0].trait_name.as_ref();
        if first_trait.is_none() { return false; }
        self.impls.iter().all(|i| i.trait_name.as_ref() == first_trait)
    }
    
    fn traits(&self) -> &[TraitBlock] {
        &self.traits
    }
    
    fn get_trait_names(&self) -> Vec<String> {
        self.traits.iter().map(|t| t.name.clone()).collect()
    }
    
    fn has_trait(&self, name: &str) -> bool {
        self.traits.iter().any(|t| t.name == name)
    }
    
    fn get_non_public_struct_names(&self) -> Vec<String> {
        self.non_public_structs.clone()
    }
    
    fn modules(&self) -> &[ModuleBlock] {
        &self.modules
    }
    
    fn has_module(&self, name: &str) -> bool {
        self.modules.iter().any(|m| m.name == name)
    }
    
    fn get_module_names(&self) -> Vec<String> {
        self.modules.iter().map(|m| m.name.clone()).collect()
    }
    
    fn get_modules_after_traits(&self) -> Vec<(String, Option<String>)> {
        let mut result = Vec::new();
        let mut last_trait: Option<String> = None;

        // 用于跨多行属性和模块声明的累积内容
        let mut pending_mod_line: Option<String> = None;

        for line in self.content.lines() {
            let trimmed = line.trim();

            // 处理跨多行的情况：如果有累积的待处理行，先检查它
            if let Some(pending) = pending_mod_line.take() {
                let combined = format!("{}\n{}", pending, trimmed);
                // 检查组合后的行是否包含模块声明
                if let Some(name) = Self::extract_module_name(&combined) {
                    result.push((name, last_trait.clone()));
                    continue;
                }
                // 如果不是模块声明，继续作为普通行处理
            }

            // 匹配 trait 或 pub trait
            if trimmed.starts_with("trait ") {
                if let Some(name) = trimmed.strip_prefix("trait ") {
                    if let Some(name) = name.split_whitespace().next() {
                        last_trait = Some(name.to_string());
                    }
                }
            } else if trimmed.starts_with("pub trait ") {
                if let Some(name) = trimmed.strip_prefix("pub trait ") {
                    if let Some(name) = name.split_whitespace().next() {
                        last_trait = Some(name.to_string());
                    }
                }
            } else if trimmed.starts_with("mod ") {
                if let Some(name) = Self::extract_module_name(trimmed) {
                    result.push((name, last_trait.clone()));
                }
            } else if trimmed.starts_with("pub mod ") {
                if let Some(name) = Self::extract_module_name(trimmed) {
                    result.push((name, last_trait.clone()));
                }
            } else if trimmed.starts_with("#[") {
                // 属性行：检查是否有内联的模块声明，或者标记为待处理
                if let Some((name, is_complete)) = Self::extract_module_name_from_attribute(trimmed) {
                    if is_complete {
                        result.push((name, last_trait.clone()));
                    } else {
                        // 需要与下一行组合
                        pending_mod_line = Some(trimmed.to_string());
                    }
                }
            } else if trimmed.starts_with("struct ") || trimmed.starts_with("enum ") || trimmed.starts_with("type ") || trimmed.starts_with("pub struct ") || trimmed.starts_with("pub enum ") || trimmed.starts_with("pub type ") {
                last_trait = None;
            }
        }

        result
    }
    
    fn get_trait_impl_mods(&self) -> Vec<(Option<String>, String, Option<String>, usize)> {
        let mut result = Vec::new();
        let _last_trait: Option<String> = None;
        let _current_path_filename: Option<String> = None;
        
        // 收集所有 trait 名称和行号
        let trait_definitions: Vec<(String, usize)> = self.content.lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let trimmed = line.trim();
                if trimmed.starts_with("trait ") {
                    if let Some(name) = trimmed.strip_prefix("trait ") {
                        if let Some(name) = name.split_whitespace().next() {
                            return Some((name.to_string(), idx + 1));
                        }
                    }
                } else if trimmed.starts_with("pub trait ") {
                    if let Some(name) = trimmed.strip_prefix("pub trait ") {
                        if let Some(name) = name.split_whitespace().next() {
                            return Some((name.to_string(), idx + 1));
                        }
                    }
                }
                None
            })
            .collect();
        
        // 收集所有 #[path = "..."] 属性的行号和对应的文件名
        let path_attributes: Vec<(usize, String)> = self.content.lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let trimmed = line.trim();
                if trimmed.starts_with("#[path") {
                    if let Some(path) = trimmed.strip_prefix("#[path = \"") {
                        if let Some(end) = path.find("\"]") {
                            let filename = path[..end].to_string();
                            return Some((idx + 1, filename));
                        }
                    }
                }
                None
            })
            .collect();
        
        // 收集所有 mod 声明
        let mod_declarations: Vec<(usize, String)> = self.content.lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let trimmed = line.trim();
                if trimmed.starts_with("mod ") {
                    if let Some(name) = trimmed.strip_prefix("mod ") {
                        let name = name.split_whitespace().next()
                            .map(|n| n.trim_end_matches(';').to_string());
                        return name.map(|n| (idx + 1, n));
                    }
                } else if trimmed.starts_with("pub mod ") {
                    if let Some(name) = trimmed.strip_prefix("pub mod ") {
                        let name = name.split_whitespace().next()
                            .map(|n| n.trim_end_matches(';').to_string());
                        return name.map(|n| (idx + 1, n));
                    }
                }
                None
            })
            .collect();
        
        // 匹配 path 属性和 mod 声明
        // path 属性应该在 mod 声明之前
        let mut path_idx = 0;
        for (mod_line, mod_name) in &mod_declarations {
            // 查找最近的 path 属性（在这个 mod 声明之前）
            let mut found_path: Option<String> = None;
            while path_idx < path_attributes.len() && path_attributes[path_idx].0 < *mod_line {
                found_path = Some(path_attributes[path_idx].1.clone());
                path_idx += 1;
            }
            
            // 找到这个 mod 所属的 trait（在 mod 声明之前的最后一个 trait）
            let mut belongs_to_trait: Option<String> = None;
            for (trait_name, trait_line) in &trait_definitions {
                if *trait_line < *mod_line {
                    belongs_to_trait = Some(trait_name.clone());
                } else {
                    break;
                }
            }
            
            result.push((belongs_to_trait, mod_name.clone(), found_path, *mod_line));
        }
        
        result
    }
    
    fn get_top_level_function_names(&self) -> Vec<String> {
        self.top_level_functions.clone()
    }

    fn extract_impl_code(&self, index: usize) -> Option<String> {
        if index >= self.impls.len() {
            return None;
        }

        // 使用文本搜索定位 impl 块
        let impl_info = &self.impls[index];
        let trait_name = impl_info.trait_name.as_deref().unwrap_or("");
        let type_name = impl_info.type_name.as_deref().unwrap_or("");

        // 构建搜索模式
        let search_pattern = if !trait_name.is_empty() && !type_name.is_empty() {
            format!("impl {} for {}", trait_name, type_name)
        } else if !type_name.is_empty() {
            format!("impl {}", type_name)
        } else if !trait_name.is_empty() {
            format!("impl {}", trait_name)
        } else {
            return None;
        };

        // 找到 impl 块的起始位置
        let mut start_idx: Option<usize> = None;
        for (idx, line) in self.content.lines().enumerate() {
            if line.contains(&search_pattern) {
                start_idx = Some(idx);
                break;
            }
        }

        let start_idx = start_idx?;

        // 找到 impl 块的结束位置（配对的大括号）
        let mut brace_count = 0;
        let mut found_start = false;
        let mut end_idx = start_idx;

        for (idx, line) in self.content.lines().enumerate().skip(start_idx) {
            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    found_start = true;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }
            end_idx = idx;
            if found_start && brace_count == 0 {
                break;
            }
        }

        let lines: Vec<&str> = self.content.lines().collect();
        let code_lines = lines[start_idx..=end_idx].to_vec();
        Some(code_lines.join("\n"))
    }

    fn extract_all_impl_codes(&self) -> Vec<String> {
        (0..self.impls.len())
            .filter_map(|i| self.extract_impl_code(i))
            .collect()
    }

    fn extract_function_code(&self, name: &str) -> Option<String> {
        // 使用 AST 解析来精确定位函数
        let parse_content = if self.content.trim().starts_with('{') && self.content.trim().ends_with('}') {
            let trimmed = self.content.trim();
            let inner = &trimmed[1..trimmed.len() - 1];
            format!("fn __wrapper__() {{ {} }}", inner)
        } else {
            self.content.clone()
        };

        if let Ok(ast) = syn::parse_file(&parse_content) {
            for item in &ast.items {
                if let Item::Fn(item_fn) = item {
                    if item_fn.sig.ident.to_string() == name {
                        // 使用文本搜索找到函数
                        let fn_signature = format!("fn {}", name);
                        let mut start_idx: Option<usize> = None;

                        for (idx, line) in self.content.lines().enumerate() {
                            if line.contains(&fn_signature) {
                                start_idx = Some(idx);
                                break;
                            }
                        }

                        let start_idx = match start_idx {
                            Some(idx) => idx,
                            None => return None,
                        };

                        // 找到函数结束位置
                        let mut brace_count = 0;
                        let mut found_start = false;
                        let mut end_idx = start_idx;

                        for (idx, line) in self.content.lines().enumerate().skip(start_idx) {
                            for ch in line.chars() {
                                if ch == '{' {
                                    brace_count += 1;
                                    found_start = true;
                                } else if ch == '}' {
                                    brace_count -= 1;
                                }
                            }
                            end_idx = idx;
                            if found_start && brace_count == 0 {
                                break;
                            }
                        }

                        let lines: Vec<&str> = self.content.lines().collect();
                        if end_idx >= lines.len() {
                            return None;
                        }
                        let code_lines = lines[start_idx..=end_idx].to_vec();
                        return Some(code_lines.join("\n"));
                    }
                }
            }
        }
        None
    }

    fn extract_all_top_level_function_codes(&self) -> Vec<(String, String)> {
        self.top_level_functions.iter()
            .filter_map(|name| {
                self.extract_function_code(name).map(|code| (name.clone(), code))
            })
            .collect()
    }

    fn get_use_statements(&self) -> Vec<String> {
        let mut use_statements = Vec::new();
        for line in self.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") && !trimmed.starts_with("use crate") {
                // 包含外部 use 语句
                if trimmed.contains("::") {
                    use_statements.push(line.to_string());
                }
            } else if trimmed.starts_with("use crate") {
                // 内部 use 语句
                use_statements.push(line.to_string());
            }
        }
        use_statements
    }
    
    fn contains_impl(&self) -> bool {
        !self.impls.is_empty()
    }
    
    fn contains_pub(&self) -> bool {
        self.has_pub
    }
    
    fn contains_include_macro(&self) -> bool {
        self.has_include_macro
    }
    
    fn is_brace_wrapped(&self) -> bool {
        self.is_brace_wrapped
    }
    
    fn has_function_signatures(&self) -> bool {
        self.has_fn_sigs
    }

    fn contains_pub_mod(&self) -> bool {
        self.has_pub_mod
    }

    fn get_wildcard_re_exports(&self) -> Vec<(String, usize)> {
        self.wildcard_re_exports.clone()
    }

    fn contains_inline_mod(&self) -> bool {
        !self.inline_mods.is_empty()
    }

    fn get_inline_mods(&self) -> Vec<(String, usize)> {
        self.inline_mods.clone()
    }
    
    fn find_impl_line_number(&self) -> Option<usize> {
        for (line_num, line) in self.content.lines().enumerate() {
            let trimmed = line.trim();
            // 匹配行首的 impl 关键字，支持泛型 impl<A>
            if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
                return Some(line_num + 1);
            }
        }
        None
    }
    
    fn find_keyword_line_number(&self, keyword: &str) -> Option<usize> {
        for (line_num, line) in self.content.lines().enumerate() {
            if line.contains(keyword) {
                return Some(line_num + 1);
            }
        }
        None
    }
    
    fn find_struct_line_number(&self, name: &str) -> Option<usize> {
        for (line_num, line) in self.content.lines().enumerate() {
            if line.contains(&format!("struct {}", name)) {
                return Some(line_num + 1);
            }
        }
        None
    }
    
    fn find_function_line_number(&self, name: &str) -> Option<usize> {
        for (line_num, line) in self.content.lines().enumerate() {
            if line.contains(&format!("fn {}", name)) {
                return Some(line_num + 1);
            }
        }
        None
    }
    
    fn find_module_line_number(&self, name: &str) -> Option<usize> {
        for (line_num, line) in self.content.lines().enumerate() {
            if line.contains(&format!("mod {}", name)) {
                return Some(line_num + 1);
            }
        }
        None
    }
    
    fn find_include_macro_line_number(&self) -> Option<usize> {
        for (line_num, line) in self.content.lines().enumerate() {
            if line.contains("include!(") || line.contains("include! (") {
                return Some(line_num + 1);
            }
        }
        None
    }
    
    fn get_top_level_function_bodies(&self) -> Vec<(String, Vec<String>)> {
        self.top_level_function_bodies.clone()
    }
    
    fn is_include_macro_in_function_body(&self) -> bool {
        self.get_include_macro_function_info().is_some()
    }
    
    fn get_include_macro_function_info(&self) -> Option<(String, String)> {
        // 遍历每个顶层函数
        for (fn_name, statements) in &self.top_level_function_bodies {
            // 检查函数体中是否包含 include! 宏
            for stmt in statements {
                if stmt.contains("include!(") || stmt.contains("include! (") {
                    // 提取 include! 中的文件名
                    if let Some(file_name) = extract_include_file_name(stmt) {
                        return Some((fn_name.clone(), file_name));
                    }
                }
            }
        }
        None
    }
    
    fn get_all_include_macro_functions(&self) -> Vec<String> {
        let mut result = Vec::new();
        // 遍历每个顶层函数
        for (fn_name, statements) in &self.top_level_function_bodies {
            // 检查函数体中是否包含 include! 宏
            for stmt in statements {
                if stmt.contains("include!(") || stmt.contains("include! (") {
                    result.push(fn_name.clone());
                    break;  // 每个函数只需要记录一次
                }
            }
        }
        result
    }
}

/// 辅助函数：从 include! 语句中提取文件名
fn extract_include_file_name(stmt: &str) -> Option<String> {
    let pattern = r#"include!\s*\(\s*["'](?:internal/)?([^"']+)["']\s*\)"#;
    if let Ok(re) = regex::Regex::new(pattern) {
        if let Some(caps) = re.captures(stmt) {
            return Some(caps.get(1)?.as_str().to_string());
        }
    }
    None
}

/// 辅助函数：从源代码中提取顶层函数体
fn extract_top_level_function_bodies(content: &str) -> Vec<(String, Vec<String>)> {
    let mut result = Vec::new();
    
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        // 检查是否是函数开始（fn 开头）
        // 跳过以 /// 或 // 开头的注释行
        let is_fn_line = line.starts_with("fn ") || (line.starts_with("pub ") && !line.starts_with("pub struct") && !line.starts_with("pub trait") && !line.starts_with("pub enum"));
        
        if is_fn_line {
            let fn_start = i;
            
            // 找到函数体的开始（{）
            // 需要处理多行函数签名的情况
            let mut j = fn_start;
            while j < lines.len() && !lines[j].contains('{') {
                j += 1;
            }
            
            if j < lines.len() && lines[j].contains('{') {
                // 提取函数名
                let fn_signature_line = lines[fn_start];
                let fn_name = fn_signature_line
                    .strip_prefix("fn ")
                    .or_else(|| fn_signature_line.strip_prefix("pub fn "))
                    .and_then(|s| s.split_whitespace().next())
                    .map(|s| s.trim_end_matches('(').to_string());
                
                if let Some(fn_name) = fn_name {
                    let mut brace_count = 0;
                    let mut found_start = false;
                    
                    // 从 { 所在行开始提取函数体内容
                    let body_start = j;
                    
                    for (k, line) in lines[j..].iter().enumerate() {
                        for ch in line.chars() {
                            if ch == '{' {
                                brace_count += 1;
                                found_start = true;
                            } else if ch == '}' {
                                brace_count -= 1;
                            }
                        }
                        // 函数体结束条件：找到过 { 且 brace_count 回到 0
                        if found_start && brace_count == 0 {
                            // 处理函数体在同一行的情况（如 fn foo() { }）
                            // body_start + 1 > j + k 表示 { 和 } 在同一行，没有内容
                            if body_start < j + k {
                                // 函数体结束，只提取 { 和 } 之间的内容（不包括 { 和 }）
                                let body_lines: Vec<String> = lines[body_start + 1..j + k]
                                    .iter()
                                    .map(|s| s.to_string())
                                    .collect();
                            
                                let statements = parse_function_body_statements(&body_lines);
                                result.push((fn_name, statements));
                            } else {
                                // { 和 } 在同一行，没有函数体内容
                                result.push((fn_name, Vec::new()));
                            }
                            
                            // 设置 i 为函数体结束的位置，以便继续寻找下一个函数
                            i = j + k;
                            break;
                        }
                    }
                }
            }
        }
        i += 1;
    }
    
    result
}

/// 辅助函数：解析函数体中的语句
fn parse_function_body_statements(body_lines: &[String]) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_stmt = String::new();
    let mut brace_depth = 0;
    
    for line in body_lines {
        let trimmed = line.trim();
        
        if trimmed == "{" {
            brace_depth += 1;
            if brace_depth > 1 {
                current_stmt.push_str(trimmed);
            }
            continue;
        }
        if trimmed == "}" {
            brace_depth -= 1;
            if brace_depth > 0 {
                current_stmt.push_str(trimmed);
            }
            continue;
        }
        
        for ch in trimmed.chars() {
            if ch == '{' {
                brace_depth += 1;
            } else if ch == '}' {
                brace_depth -= 1;
            }
        }
        
        if !current_stmt.is_empty() {
            current_stmt.push_str(" ");
        }
        current_stmt.push_str(trimmed);
        
        if brace_depth == 0 && !current_stmt.is_empty() {
            statements.push(current_stmt.clone());
            current_stmt.clear();
        }
    }
    
    if !current_stmt.trim().is_empty() {
        statements.push(current_stmt);
    }
    
    statements
}

fn extract_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(type_path) => type_path.path.segments.last().map(|seg| seg.ident.to_string()),
        syn::Type::Reference(type_ref) => extract_type_name(&type_ref.elem),
        syn::Type::Ptr(type_ptr) => extract_type_name(&type_ptr.elem),
        syn::Type::Slice(type_slice) => extract_type_name(&type_slice.elem),
        syn::Type::Array(type_array) => extract_type_name(&type_array.elem),
        _ => None,
    }
}

struct PubVisitor { found: bool }

impl<'ast> Visit<'ast> for PubVisitor {
    fn visit_visibility(&mut self, vis: &'ast syn::Visibility) {
        if matches!(vis, syn::Visibility::Public(_)) {
            self.found = true;
        }
        syn::visit::visit_visibility(self, vis);
    }
}

struct IncludeVisitor { found: bool }

impl<'ast> Visit<'ast> for IncludeVisitor {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac.path.is_ident("include") {
            self.found = true;
        }
        syn::visit::visit_macro(self, mac);
    }
}

struct FnSigVisitor { found: bool }

impl<'ast> Visit<'ast> for FnSigVisitor {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if item.block.stmts.is_empty() {
            self.found = true;
        }
        syn::visit::visit_item_fn(self, item);
    }
    
    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        if item.default.is_none() {
            self.found = true;
        }
        syn::visit::visit_trait_item_fn(self, item);
    }
    
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if item.block.stmts.is_empty() {
            self.found = true;
        }
        syn::visit::visit_impl_item_fn(self, item);
    }
}

struct StructVisitor { names: Vec<String> }

impl<'ast> Visit<'ast> for StructVisitor {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if !matches!(item.vis, syn::Visibility::Public(_)) {
            self.names.push(item.ident.to_string());
        }
        syn::visit::visit_item_struct(self, item);
    }
}

/// 检测 pub mod 声明的 Visitor
struct PubModVisitor { found: bool }

impl<'ast> Visit<'ast> for PubModVisitor {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if matches!(item.vis, syn::Visibility::Public(_)) {
            self.found = true;
        }
        syn::visit::visit_item_mod(self, item);
    }
}

/// 检测通配符重导出的 Visitor
struct WildcardReExportVisitor { exports: Vec<(String, usize)>, content: String }

impl<'ast> Visit<'ast> for WildcardReExportVisitor {
    fn visit_file(&mut self, file: &'ast syn::File) {
        // 使用文本搜索更简单直接
        for (line_num, line) in self.content.lines().enumerate() {
            let trimmed = line.trim();
            // 匹配 pub use xxx::*;
            if trimmed.starts_with("pub use ") && trimmed.contains("::*") {
                // 提取路径：pub use xxx::* -> xxx
                if let Some(path) = trimmed.strip_prefix("pub use ") {
                    let path = path.trim_end_matches("::*").trim_end_matches("::*;").to_string();
                    self.exports.push((path, line_num));
                }
            }
        }
        syn::visit::visit_file(self, file);
    }
}

/// 检测内联 mod 的 Visitor
struct InlineModVisitor { mods: Vec<(String, usize)>, content: String }

impl<'ast> Visit<'ast> for InlineModVisitor {
    fn visit_file(&mut self, file: &'ast syn::File) {
        // 使用文本搜索更简单直接
        for (line_num, line) in self.content.lines().enumerate() {
            let trimmed = line.trim();
            // 匹配 mod xxx { 形式的内联模块
            if trimmed.starts_with("mod ") && trimmed.contains('{') {
                if let Some(name) = trimmed.strip_prefix("mod ") {
                    let name = name.split('{').next().unwrap_or("").split_whitespace().next();
                    if let Some(name) = name {
                        self.mods.push((name.to_string(), line_num));
                    }
                }
            }
        }
        syn::visit::visit_file(self, file);
    }
}
