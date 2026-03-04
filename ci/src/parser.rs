use syn::{Item, ItemImpl, ItemTrait, ItemMod, visit::Visit, File};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ParsedFile {
    #[allow(dead_code)] // 保留用于调试和错误报告
    pub path: PathBuf,
    #[allow(dead_code)] // 保留用于未来可能的扩展
    pub items: Vec<Item>,
    pub impls: Vec<ItemImpl>,
    pub traits: Vec<ItemTrait>,
    pub modules: Vec<ItemMod>,
    pub ast: File,
    pub content: String,
}

impl ParsedFile {
    pub fn parse(path: PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        
        // 检查是否是大括号包裹的文件（用于 internal 目录）
        let trimmed = content.trim();
        let parse_content = if trimmed.starts_with('{') && trimmed.ends_with('}') {
            // 提取大括号内的内容，并用函数包装以便 syn 能够解析
            let inner_content = &trimmed[1..trimmed.len()-1];
            format!("fn __internal_wrapper__() {{ {} }}", inner_content)
        } else {
            content.clone()
        };
        
        // 为了调试方便，在解析失败时把文件路径也打印出来
        let ast = syn::parse_file(&parse_content).map_err(|e| {
            anyhow::anyhow!(
                "cannot parse file {} into token stream: {}",
                path.display(),
                e
            )
        })?;

        let mut impls = Vec::new();
        let mut traits = Vec::new();
        let mut modules = Vec::new();

        for item in &ast.items {
            match item {
                Item::Impl(impl_item) => impls.push(impl_item.clone()),
                Item::Trait(trait_item) => traits.push(trait_item.clone()),
                Item::Mod(mod_item) => modules.push(mod_item.clone()),
                _ => {}
            }
        }

        Ok(Self {
            path,
            items: ast.items.clone(),
            impls,
            traits,
            modules,
            ast,
            content,
        })
    }

    pub fn contains_impl(&self) -> bool {
        !self.impls.is_empty()
    }

    pub fn count_impl_blocks(&self) -> usize {
        self.impls.len()
    }

    pub fn get_impl_trait_name(&self) -> Option<String> {
        self.impls.first().and_then(|impl_block| {
            impl_block.trait_.as_ref().map(|(_, path, _)| {
                path.segments.last().map(|seg| seg.ident.to_string())
            }).flatten()
        })
    }

    /// 获取 impl 块中的 type 名（提取主要类型名，忽略泛型参数和生命周期）
    pub fn get_impl_type_name(&self) -> Option<String> {
        self.impls.first().and_then(|impl_block| {
            extract_type_name(&impl_block.self_ty)
        })
    }

    /// 获取所有 impl 块的信息（trait名和类型名）
    /// 返回 Vec<(trait_name, type_name)>，如果某个impl块没有trait（固有实现），trait_name为None
    pub fn get_all_impl_info(&self) -> Vec<(Option<String>, Option<String>)> {
        self.impls.iter().map(|impl_block| {
            let trait_name = impl_block.trait_.as_ref()
                .map(|(_, path, _)| {
                    path.segments.last().map(|seg| seg.ident.to_string())
                })
                .flatten();
            let type_name = extract_type_name(&impl_block.self_ty);
            (trait_name, type_name)
        }).collect()
    }

    pub fn contains_pub(&self) -> bool {
        struct PubVisitor {
            found: bool,
        }
        
        impl<'ast> Visit<'ast> for PubVisitor {
            fn visit_visibility(&mut self, vis: &'ast syn::Visibility) {
                if matches!(vis, syn::Visibility::Public(_)) {
                    self.found = true;
                }
                syn::visit::visit_visibility(self, vis);
            }
        }
        
        let mut visitor = PubVisitor { found: false };
        visitor.visit_file(&self.ast);
        visitor.found
    }

    pub fn contains_include_macro(&self) -> bool {
        struct IncludeVisitor {
            found: bool,
        }

        impl<'ast> Visit<'ast> for IncludeVisitor {
            fn visit_macro(&mut self, mac: &'ast syn::Macro) {
                if mac.path.is_ident("include") {
                    self.found = true;
                }
                syn::visit::visit_macro(self, mac);
            }
        }

        let mut visitor = IncludeVisitor { found: false };
        visitor.visit_file(&self.ast);
        visitor.found
    }

    pub fn find_include_macro_line_number(&self) -> Option<usize> {
        for (line_num, line) in self.content.lines().enumerate() {
            if line.contains("include!(")
                || line.contains("include! (")
                || line.contains("include !(")
                || line.contains("include ! (")
            {
                return Some(line_num + 1);
            }
        }
        None
    }

    pub fn is_brace_wrapped(&self) -> bool {
        let trimmed = self.content.trim();
        trimmed.starts_with('{') && trimmed.ends_with('}')
    }

    /// 使用 AST 检查是否包含函数签名（没有函数体的函数声明）
    pub fn has_function_signatures(&self) -> bool {
        struct FnSigVisitor {
            found: bool,
        }
        
        impl<'ast> Visit<'ast> for FnSigVisitor {
            fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
                // 如果函数体为空（只有大括号但没有语句），说明是函数签名
                if item.block.stmts.is_empty() {
                    self.found = true;
                }
                syn::visit::visit_item_fn(self, item);
            }
            
            fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
                // trait 中的函数默认是签名（可能有默认实现，但通常没有函数体）
                if item.default.is_none() {
                    self.found = true;
                }
                syn::visit::visit_trait_item_fn(self, item);
            }
            
            fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
                // impl 块中的函数签名（没有函数体）
                if item.block.stmts.is_empty() {
                    self.found = true;
                }
                syn::visit::visit_impl_item_fn(self, item);
            }
        }
        
        let mut visitor = FnSigVisitor { found: false };
        visitor.visit_file(&self.ast);
        visitor.found
    }

    /// 查找字符串在文件中的行号（返回第一个匹配的行号）
    pub fn find_line_number(&self, pattern: &str) -> Option<usize> {
        for (line_num, line) in self.content.lines().enumerate() {
            if line.contains(pattern) {
                return Some(line_num + 1); // 行号从1开始
            }
        }
        None
    }

    /// 查找 impl 块所在的行号
    pub fn find_impl_line_number(&self) -> Option<usize> {
        if let Some(impl_block) = self.impls.first() {
            // 尝试从 AST 获取行号
            if let Some(line) = self.find_item_line_number(&Item::Impl(impl_block.clone())) {
                return Some(line);
            }
            
            // 回退到字符串搜索
            let trait_name = impl_block.trait_.as_ref()
                .and_then(|(_, path, _)| path.segments.last().map(|seg| seg.ident.to_string()));
            
            if let Some(trait_name) = trait_name {
                let pattern = format!("impl {}", trait_name);
                return self.find_line_number(&pattern);
            } else {
                return self.find_line_number("impl ");
            }
        }
        None
    }

    /// 查找包含特定关键字的行号（如 "pub", "impl" 等）
    pub fn find_keyword_line_number(&self, keyword: &str) -> Option<usize> {
        self.find_line_number(keyword)
    }

    /// 尝试从 Item 获取行号（通过查找对应的代码片段）
    fn find_item_line_number(&self, item: &Item) -> Option<usize> {
        // 将 item 转换为字符串，然后在原文件中查找
        let item_str = match item {
            Item::Impl(impl_item) => {
                if let Some((_, path, _)) = &impl_item.trait_ {
                    if let Some(seg) = path.segments.last() {
                        format!("impl {}", seg.ident)
                    } else {
                        "impl ".to_string()
                    }
                } else {
                    "impl ".to_string()
                }
            }
            Item::Trait(trait_item) => format!("trait {}", trait_item.ident),
            Item::Mod(mod_item) => format!("mod {}", mod_item.ident),
            _ => return None,
        };
        self.find_line_number(&item_str)
    }

    /// 检查所有 impl 块是否都是空的（marker trait 实现）
    pub fn are_all_impls_empty(&self) -> bool {
        self.impls.iter().all(|impl_block| impl_block.items.is_empty())
    }

    /// 检查所有 impl 块是否都是同一个 trait
    pub fn are_all_impls_same_trait(&self) -> bool {
        if self.impls.is_empty() {
            return false;
        }
        
        let first_trait = self.impls[0].trait_.as_ref()
            .and_then(|(_, path, _)| path.segments.last().map(|seg| seg.ident.to_string()));
        
        if first_trait.is_none() {
            return false; // 固有实现不算
        }
        
        self.impls.iter().all(|impl_block| {
            impl_block.trait_.as_ref()
                .and_then(|(_, path, _)| path.segments.last().map(|seg| seg.ident.to_string()))
                == first_trait
        })
    }

    /// 获取所有 trait 的名称列表
    pub fn get_trait_names(&self) -> Vec<String> {
        self.traits.iter()
            .map(|t| t.ident.to_string())
            .collect()
    }

    /// 检查是否包含指定名称的 trait
    pub fn has_trait(&self, trait_name: &str) -> bool {
        self.traits.iter()
            .any(|t| t.ident.to_string() == trait_name)
    }

    /// 获取所有模块的名称列表
    #[allow(dead_code)] // 保留用于未来可能的扩展
    pub fn get_module_names(&self) -> Vec<String> {
        self.modules.iter()
            .map(|m| m.ident.to_string())
            .collect()
    }

    /// 检查是否包含指定名称的模块声明
    pub fn has_module(&self, module_name: &str) -> bool {
        self.modules.iter()
            .any(|m| m.ident.to_string() == module_name)
    }

    /// 获取所有顶层函数名称（仅限文件最外层的 `fn`，不包含 trait/impl 内的方法）
    pub fn get_top_level_function_names(&self) -> Vec<String> {
        self.ast
            .items
            .iter()
            .filter_map(|item| {
                if let Item::Fn(item_fn) = item {
                    Some(item_fn.sig.ident.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// 查找顶层函数所在的行号
    pub fn find_function_line_number(&self, fn_name: &str) -> Option<usize> {
        self.find_line_number(&format!("fn {}", fn_name))
    }

    /// 检查模块声明是否跟在某个 trait 后面
    /// 返回 (module_name, trait_name) 的列表，表示哪些模块跟在哪些 trait 后面
    pub fn get_modules_after_traits(&self) -> Vec<(String, Option<String>)> {
        let mut result = Vec::new();
        let items = &self.ast.items;
        
        // 遍历所有 items，记录 trait 的位置
        let mut last_trait: Option<String> = None;
        
        for item in items {
            match item {
                Item::Trait(trait_item) => {
                    last_trait = Some(trait_item.ident.to_string());
                }
                Item::Mod(mod_item) => {
                    result.push((mod_item.ident.to_string(), last_trait.clone()));
                    // 模块声明后，重置 last_trait（因为下一个模块不应该继承之前的 trait）
                    // 但这里我们保留，因为配置要求模块必须跟在 trait 后
                }
                // 如果遇到其他顶级 item（如 struct, enum 等），重置 last_trait
                Item::Struct(_) | Item::Enum(_) | Item::Type(_) | Item::Const(_) | Item::Static(_) => {
                    last_trait = None;
                }
                _ => {}
            }
        }
        
        result
    }

    /// 查找 trait 所在的行号
    #[allow(dead_code)] // 保留用于未来可能的扩展
    pub fn find_trait_line_number(&self, trait_name: &str) -> Option<usize> {
        for trait_item in &self.traits {
            if trait_item.ident.to_string() == trait_name {
                return self.find_line_number(&format!("trait {}", trait_name));
            }
        }
        None
    }

    /// 查找模块声明所在的行号
    pub fn find_module_line_number(&self, module_name: &str) -> Option<usize> {
        for mod_item in &self.modules {
            if mod_item.ident.to_string() == module_name {
                return self.find_line_number(&format!("mod {}", module_name));
            }
        }
        None
    }

    /// 获取所有非 `pub struct` 的 struct 名称（包括 pub(crate) / private）
    pub fn get_non_public_struct_names(&self) -> Vec<String> {
        struct StructVisitor {
            names: Vec<String>,
        }

        impl<'ast> Visit<'ast> for StructVisitor {
            fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
                if !matches!(item.vis, syn::Visibility::Public(_)) {
                    self.names.push(item.ident.to_string());
                }
                syn::visit::visit_item_struct(self, item);
            }
        }

        let mut visitor = StructVisitor { names: Vec::new() };
        visitor.visit_file(&self.ast);
        visitor.names
    }

    pub fn find_struct_line_number(&self, struct_name: &str) -> Option<usize> {
        self.find_line_number(&format!("struct {}", struct_name))
    }
}

/// 递归提取类型名，支持包装类型（&T, Box<T>, *const T 等）
fn extract_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(type_path) => {
            // 提取最后一个 segment（例如：crate::memory::Allocator -> Allocator）
            type_path.path.segments.last().map(|seg| seg.ident.to_string())
        }
        syn::Type::Reference(type_ref) => {
            extract_type_name(&type_ref.elem)
        }
        syn::Type::Ptr(type_ptr) => {
            extract_type_name(&type_ptr.elem)
        }
        syn::Type::Slice(type_slice) => {
            extract_type_name(&type_slice.elem)
        }
        syn::Type::Array(type_array) => {
            extract_type_name(&type_array.elem)
        }
        syn::Type::Tuple(_) => {
            // 对于元组类型，返回 None（不支持）
            None
        }
        syn::Type::BareFn(_) => {
            // 对于函数类型，返回 None（不支持）
            None
        }
        _ => None,
    }
}
