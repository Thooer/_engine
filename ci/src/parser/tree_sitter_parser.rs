use crate::parser::ast::{ImplBlock, ModuleBlock, ParsedSource, Parser, TraitBlock};
use std::path::{Path, PathBuf};
use tree_sitter::Parser as TsParser;

/// TreeSitterParser - 使用 tree-sitter 的解析器实现
pub struct TreeSitterParser {
    parser: TsParser,
}

impl TreeSitterParser {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_rust::language())
            .map_err(|e| anyhow::anyhow!("failed to set tree-sitter-rust language: {}", e))?;
        Ok(Self { parser })
    }
}

impl Default for TreeSitterParser {
    fn default() -> Self {
        Self::new().expect("failed to create TreeSitterParser")
    }
}

impl Parser for TreeSitterParser {
    fn parse(&self, path: &Path) -> anyhow::Result<Box<dyn ParsedSource>> {
        TreeSitterParsedSource::parse(path.to_path_buf(), &mut self.parser.clone())
    }
}

/// TreeSitterParsedSource - Tree-sitter 解析后的源代码实现
pub struct TreeSitterParsedSource {
    path: PathBuf,
    content: String,
    impls: Vec<ImplBlock>,
    traits: Vec<TraitBlock>,
    modules: Vec<ModuleBlock>,
    non_public_structs: Vec<String>,
    top_level_functions: Vec<String>,
    top_level_function_bodies: Vec<(String, Vec<String>)>,
    has_pub: bool,
    has_include_macro: bool,
    is_brace_wrapped: bool,
    has_fn_sigs: bool,
}

impl TreeSitterParsedSource {
    pub fn parse(path: PathBuf, mut parser: TsParser) -> anyhow::Result<Box<dyn ParsedSource>> {
        let content = std::fs::read_to_string(&path)?;

        let trimmed = content.trim();
        let parse_content = if trimmed.starts_with('{') && trimmed.ends_with('}') {
            let inner_content = &trimmed[1..trimmed.len() - 1];
            format!("fn __internal_wrapper__() {{ {} }}", inner_content)
        } else {
            content.clone()
        };

        let tree = parser
            .parse(&parse_content, None)
            .ok_or_else(|| anyhow::anyhow!("failed to parse file: {}", path.display()))?;

        let root_node = tree.root_node();

        let mut visitor = AstVisitor {
            content: &content,
            impls: Vec::new(),
            traits: Vec::new(),
            modules: Vec::new(),
            non_public_structs: Vec::new(),
            top_level_functions: Vec::new(),
            top_level_function_bodies: Vec::new(),
            has_pub: false,
            has_include_macro: false,
            has_fn_sigs: false,
        };
        visitor.visit_node(&root_node);

        let is_brace_wrapped = trimmed.starts_with('{') && trimmed.ends_with('}');

        // 提取顶层函数体信息
        let top_level_function_bodies = Self::extract_top_level_function_bodies(&content);

        Ok(Box::new(Self {
            path,
            content,
            impls: visitor.impls,
            traits: visitor.traits,
            modules: visitor.modules,
            non_public_structs: visitor.non_public_structs,
            top_level_functions: visitor.top_level_functions,
            top_level_function_bodies,
            has_pub: visitor.has_pub,
            has_include_macro: visitor.has_include_macro,
            is_brace_wrapped,
            has_fn_sigs: visitor.has_fn_sigs,
        }))
    }
}

impl ParsedSource for TreeSitterParsedSource {
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
        self.impls
            .iter()
            .filter(|i| i.trait_name.is_none())
            .filter_map(|i| i.type_name.clone())
            .collect()
    }

    fn get_all_impl_info(&self) -> Vec<(Option<String>, Option<String>)> {
        self.impls
            .iter()
            .map(|i| (i.trait_name.clone(), i.type_name.clone()))
            .collect()
    }

    fn are_all_impls_empty(&self) -> bool {
        self.impls.iter().all(|i| i.is_empty)
    }

    fn are_all_impls_same_trait(&self) -> bool {
        if self.impls.is_empty() {
            return false;
        }
        let first_trait = self.impls[0].trait_name.as_ref();
        if first_trait.is_none() {
            return false;
        }
        self.impls
            .iter()
            .all(|i| i.trait_name.as_ref() == first_trait)
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

        for line in self.content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("trait ") {
                if let Some(name) = trimmed.strip_prefix("trait ") {
                    if let Some(name) = name.split_whitespace().next() {
                        last_trait = Some(name.to_string());
                    }
                }
            } else if trimmed.starts_with("mod ") {
                if let Some(name) = trimmed.strip_prefix("mod ") {
                    if let Some(name) = name.split_whitespace().next() {
                        let name = name.trim_end_matches(';').to_string();
                        result.push((name, last_trait.clone()));
                    }
                }
            } else if trimmed.starts_with("struct ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("type ")
            {
                last_trait = None;
            }
        }

        result
    }
    
    fn get_trait_impl_mods(&self) -> Vec<(Option<String>, String, Option<String>, usize)> {
        let mut result = Vec::new();
        
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
        let mut path_idx = 0;
        for (mod_line, mod_name) in &mod_declarations {
            let mut found_path: Option<String> = None;
            while path_idx < path_attributes.len() && path_attributes[path_idx].0 < *mod_line {
                found_path = Some(path_attributes[path_idx].1.clone());
                path_idx += 1;
            }
            
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

    fn find_impl_line_number(&self) -> Option<usize> {
        for (line_num, line) in self.content.lines().enumerate() {
            if line.contains("impl ") {
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
        for (fn_name, statements) in &self.top_level_function_bodies {
            for stmt in statements {
                if stmt.contains("include!(") || stmt.contains("include! (") {
                    if let Some(file_name) = extract_include_file_name(stmt) {
                        return Some((fn_name.clone(), file_name));
                    }
                }
            }
        }
        None
    }
    
    fn get_top_level_function_bodies(&self) -> Vec<(String, Vec<String>)> {
        self.top_level_function_bodies.clone()
    }
}
                    let fn_name = lines[fn_start]
                        .strip_prefix("fn ")
                        .and_then(|s| s.split_whitespace().next())
                        .map(|s| s.trim_end_matches('(').to_string());
                    
                    if let Some(fn_name) = fn_name {
                        let mut brace_count = 0;
                        let mut body_start = i;
                        let mut found_start = false;
                        
                        for (j, line) in lines[i..].iter().enumerate() {
                            for ch in line.chars() {
                                if ch == '{' {
                                    brace_count += 1;
                                    found_start = true;
                                } else if ch == '}' {
                                    brace_count -= 1;
                                }
                            }
                            if found_start && brace_count == 0 {
                                let body_lines: Vec<String> = lines[body_start..i + j + 1]
                                    .iter()
                                    .map(|s| s.to_string())
                                    .collect();
                                
                                let statements = Self::parse_function_body_statements(&body_lines);
                                result.push((fn_name, statements));
                                break;
                            }
                        }
                    }
                }
            } else if line.starts_with("fn ") && line.contains('{') {
                let fn_name = line
                    .strip_prefix("fn ")
                    .and_then(|s| s.split_whitespace().next())
                    .map(|s| s.trim_end_matches('(').to_string());
                
                if let Some(fn_name) = fn_name {
                    if let Some(start) = line.find('{') {
                        if let Some(end) = line.rfind('}') {
                            let body = &line[start+1..end];
                            let statements: Vec<String> = body
                                .split(';')
                                .filter(|s| !s.trim().is_empty())
                                .map(|s| s.trim().to_string())
                                .collect();
                            result.push((fn_name, statements));
                        }
                    }
                }
            }
            i += 1;
        }
        
        result
    }
    
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
    
    fn extract_impl_code(&self, index: usize) -> Option<String> {
        if index >= self.impls.len() {
            return None;
        }
        let impl_block = &self.impls[index];
        let start = impl_block.start_line?;
        let end = impl_block.end_line?;

        let lines: Vec<&str> = self.content.lines().collect();
        if start > lines.len() {
            return None;
        }

        // 提取代码（1-indexed 转 0-indexed）
        let code_lines = lines[start - 1..end.min(lines.len())].to_vec();
        Some(code_lines.join("\n"))
    }

    fn extract_all_impl_codes(&self) -> Vec<String> {
        (0..self.impls.len())
            .filter_map(|i| self.extract_impl_code(i))
            .collect()
    }

    fn extract_function_code(&self, name: &str) -> Option<String> {
        // 简化实现：使用文本搜索
        let mut in_function = false;
        let mut function_lines = Vec::new();
        let mut brace_count = 0;

        for line in self.content.lines() {
            if line.contains(&format!("fn {}", name)) {
                in_function = true;
                function_lines.push(line.to_string());
                continue;
            }

            if in_function {
                function_lines.push(line.to_string());
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;

                if brace_count == 0 && function_lines.len() > 1 {
                    break;
                }
            }
        }

        if function_lines.is_empty() {
            None
        } else {
            Some(function_lines.join("\n"))
        }
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
            if trimmed.starts_with("use ") {
                use_statements.push(line.to_string());
            }
        }
        use_statements
    }
}

struct AstVisitor<'a> {
    content: &'a str,
    impls: Vec<ImplBlock>,
    traits: Vec<TraitBlock>,
    modules: Vec<ModuleBlock>,
    non_public_structs: Vec<String>,
    top_level_functions: Vec<String>,
    top_level_function_bodies: Vec<(String, Vec<String>)>,
    has_pub: bool,
    has_include_macro: bool,
    has_fn_sigs: bool,
}

impl<'a> AstVisitor<'a> {
    fn visit_node(&mut self, node: &tree_sitter::Node) {
        match node.kind() {
            "impl_item" => {
                if let Some(impl_block) = self.extract_impl_block(node) {
                    self.impls.push(impl_block);
                }
            }
            "trait_item" => {
                if let Some(trait_block) = self.extract_trait_block(node) {
                    self.traits.push(trait_block);
                }
            }
            "mod_item" => {
                if let Some(module_block) = self.extract_module_block(node) {
                    self.modules.push(module_block);
                }
            }
            "struct_item" => {
                self.extract_struct_info(node);
            }
            "function_item" => {
                self.extract_function_info(node);
            }
            "visibility_modifier" => {
                if node.utf8_text(self.content.as_bytes()) == Some("pub") {
                    self.has_pub = true;
                }
            }
            "macro_invocation" => {
                if let Some(text) = node.utf8_text(self.content.as_bytes()) {
                    if text.starts_with("include!") {
                        self.has_include_macro = true;
                    }
                }
            }
            "function_signature" => {
                self.has_fn_sigs = true;
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child);
        }
    }

    fn extract_impl_block(&self, node: &tree_sitter::Node) -> Option<ImplBlock> {
        let mut trait_name: Option<String> = None;
        let mut type_name: Option<String> = None;
        let mut is_empty = true;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "type_identifier" | "field_identifier" => {
                    if type_name.is_none() {
                        type_name = child.utf8_text(self.content.as_bytes()).map(|s| s.to_string());
                    }
                }
                "generic_type" => {
                    if let Some(generic_type) = self.extract_type_from_generic(&child) {
                        if trait_name.is_none() {
                            trait_name = Some(generic_type);
                        }
                    }
                }
                "scoped_identifier" | "identifier" => {
                    if trait_name.is_none() {
                        trait_name = child.utf8_text(self.content.as_bytes()).map(|s| s.to_string());
                    }
                }
                "declaration" | "block" => {
                    is_empty = false;
                }
                _ => {}
            }
        }

        Some(ImplBlock {
            trait_name,
            type_name,
            is_empty,
            line_number: Some(node.start_position().row + 1),
            start_line: Some(node.start_position().row + 1),
            end_line: Some(node.end_position().row + 1),
        })
    }

    fn extract_type_from_generic(&self, node: &tree_sitter::Node) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return child.utf8_text(self.content.as_bytes()).map(|s| s.to_string());
            }
        }
        None
    }

    fn extract_trait_block(&self, node: &tree_sitter::Node) -> Option<TraitBlock> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                return Some(TraitBlock {
                    name: child.utf8_text(self.content.as_bytes())?.to_string(),
                    line_number: Some(node.start_position().row + 1),
                });
            }
        }
        None
    }

    fn extract_module_block(&self, node: &tree_sitter::Node) -> Option<ModuleBlock> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(ModuleBlock {
                    name: child.utf8_text(self.content.as_bytes())?.to_string(),
                    line_number: Some(node.start_position().row + 1),
                });
            }
        }
        None
    }

    fn extract_struct_info(&mut self, node: &tree_sitter::Node) {
        let mut is_pub = false;
        let mut name: Option<String> = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "visibility_modifier" => {
                    if child.utf8_text(self.content.as_bytes()) == Some("pub") {
                        is_pub = true;
                    }
                }
                "type_identifier" => {
                    if name.is_none() {
                        name = child.utf8_text(self.content.as_bytes()).map(|s| s.to_string());
                    }
                }
                _ => {}
            }
        }

        if !is_pub {
            if let Some(name) = name {
                self.non_public_structs.push(name);
            }
        }
    }

    fn extract_function_info(&mut self, node: &tree_sitter::Node) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                self.top_level_functions.push(child.utf8_text(self.content.as_bytes()).unwrap().to_string());
                break;
            }
        }
    }
}
