//! `separate_inherent` 过程宏
//!
//! 允许固有实现（inherent impl）分离到独立文件，同时保持前后端分离。
//!
//! # 使用方法
//!
//! 在 `mod.rs` 中声明接口：
//!
//! ```ignore
//! use separate_inherent::separate_inherent;
//!
//! pub struct User {
//!     name: String,
//! }
//!
//! separate_inherent!("user/User.rs", {
//!     impl User {
//!         fn new(name: String) -> Self;
//!         fn name(&self) -> &str;
//!     }
//! });
//! ```
//!
//! 在 `user/User.rs` 中实现：
//!
//! ```ignore
//! impl User {
//!     fn new(name: String) -> Self {
//!         Self { name }
//!     }
//!
//!     fn name(&self) -> &str {
//!         &self.name
//!     }
//! }
//! ```
//!
//! 宏展开后会生成完整的 `impl User { ... }` 块。

use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    Ident, ImplItem, Signature,
};

/// 宏入口
///
/// 使用方式：
/// ```ignore
/// // 相对路径从调用点文件位置计算
/// separate_inherent!("user/User.rs", {
///     impl User {
///         fn new(name: String) -> Self;
///         fn name(&self) -> &str;
///     }
/// });
/// ```
///
/// 参数：
/// - 第一个参数：实现文件的相对路径（相对于调用点文件）
/// - 第二个参数：`impl Type { ... }` 块，只包含方法签名
#[proc_macro]
pub fn separate_inherent(input: TokenStream) -> TokenStream {
    // 1. 解析宏输入
    let input = parse_macro_input!(input as MacroInput);

    // 2. 读取实现文件
    let impl_content = match fs::read_to_string(&input.impl_file) {
        Ok(content) => content,
        Err(e) => {
            return syn::Error::new(
                input.self_ty.span(),
                format!(
                    "separate_inherent: 无法读取实现文件 {:?}: {}\n\
                     请确保实现文件存在",
                    input.impl_file,
                    e,
                ),
            )
            .to_compile_error()
            .into();
        }
    };

    // 3. 解析实现文件
    let impl_block = match parse_impl_block(&impl_content, &input.self_ty) {
        Ok(block) => block,
        Err(e) => return e.to_compile_error().into(),
    };

    // 4. 校验签名一致性
    if let Err(e) = validate_signatures(&input.methods, &impl_block.methods, &input.self_ty) {
        return e.to_compile_error().into();
    }

    // 5. 生成最终的 impl 块
    generate_impl_block(&impl_block)
}

/// 宏输入解析
struct MacroInput {
    impl_file: syn::LitStr,
    self_ty: Ident,
    methods: Vec<MethodDecl>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // 解析路径字符串
        let impl_file: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        
        // 解析 `impl TypeName { ... }`
        input.parse::<syn::Token![impl]>()?;
        let self_ty: Ident = input.parse()?;
        let content;
        syn::braced!(content in input);

        // 解析方法声明列表
        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(content.parse()?);
        }

        Ok(Self {
            impl_file,
            self_ty,
            methods,
        })
    }
}

/// 方法声明（以分号结尾）
struct MethodDecl {
    sig: Signature,
}

impl Parse for MethodDecl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let sig: Signature = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        Ok(Self { sig })
    }
}

/// 实现块解析
struct ImplBlock {
    self_ty: Ident,
    methods: Vec<MethodImpl>,
}

/// 方法实现
struct MethodImpl {
    sig: Signature,
    block: syn::Block,
}

/// 解析实现文件
fn parse_impl_block(content: &str, expected_ty: &Ident) -> syn::Result<ImplBlock> {
    let file: syn::File = syn::parse_str(content)
        .map_err(|e| syn::Error::new(e.span(), format!("实现文件解析失败: {}", e)))?;

    // 查找 impl TypeName 块
    for item in &file.items {
        if let syn::Item::Impl(item_impl) = item {
            // 检查 self_ty
            let self_ty = match &*item_impl.self_ty {
                syn::Type::Path(ty) => ty.path.get_ident(),
                _ => continue,
            };

            if self_ty.map(|i| i != expected_ty).unwrap_or(true) {
                continue;
            }

            // 提取方法
            let mut methods = Vec::new();
            for item in &item_impl.items {
                if let ImplItem::Fn(item_fn) = item {
                    // 只处理有 body 的方法（非 trait 方法声明）
                    // 通过检查 defaultness 来区分：有 body 的方法 defaultness 为 None
                    if item_fn.defaultness.is_none() && !item_fn.block.stmts.is_empty() {
                        methods.push(MethodImpl {
                            sig: item_fn.sig.clone(),
                            block: item_fn.block.clone(),
                        });
                    }
                }
            }

            return Ok(ImplBlock {
                self_ty: expected_ty.clone(),
                methods,
            });
        }
    }

    Err(syn::Error::new(
        expected_ty.span(),
        format!(
            "separate_inherent: 在实现文件中未找到 `impl {}` 块",
            expected_ty
        ),
    ))
}

/// 签名校验
fn validate_signatures(
    decls: &[MethodDecl],
    impls: &[MethodImpl],
    self_ty: &Ident,
) -> syn::Result<()> {
    // 1. 检查方法数量
    if decls.len() != impls.len() {
        return Err(syn::Error::new(
            self_ty.span(),
            format!(
                "separate_inherent: 方法数量不匹配\n\
                 声明了 {} 个方法，但实现文件中有 {} 个方法",
                decls.len(),
                impls.len()
            ),
        ));
    }

    // 2. 逐个校验签名
    for (decl, impl_) in decls.iter().zip(impls.iter()) {
        validate_single_signature(&decl.sig, &impl_.sig)?;
    }

    Ok(())
}

/// 校验单个方法签名
fn validate_single_signature(decl: &Signature, impl_: &Signature) -> syn::Result<()> {
    // 方法名
    if decl.ident != impl_.ident {
        return Err(syn::Error::new(
            decl.ident.span(),
            format!(
                "separate_inherent: 方法名不匹配\n\
                 声明: {}\n\
                 实现: {}",
                decl.ident, impl_.ident
            ),
        ));
    }

    // receiver (self, &self, &mut self)
    let decl_receiver = extract_receiver(decl);
    let impl_receiver = extract_receiver(impl_);

    if decl_receiver != impl_receiver {
        return Err(syn::Error::new(
            decl.ident.span(),
            format!(
                "separate_inherent: 方法 {} 的 receiver 不匹配\n\
                 声明: {:?}\n\
                 实现: {:?}",
                decl.ident, decl_receiver, impl_receiver
            ),
        ));
    }

    // 参数
    if decl.inputs.len() != impl_.inputs.len() {
        return Err(syn::Error::new(
            decl.ident.span(),
            format!(
                "separate_inherent: 方法 {} 的参数数量不匹配\n\
                 声明: {} 个参数\n\
                 实现: {} 个参数",
                decl.ident,
                decl.inputs.len(),
                impl_.inputs.len()
            ),
        ));
    }

    // 返回类型
    match (&decl.output, &impl_.output) {
        (syn::ReturnType::Default, syn::ReturnType::Default) => {}
        (syn::ReturnType::Type(_, decl_ty), syn::ReturnType::Type(_, impl_ty)) => {
            if !type_equal(&*decl_ty, &*impl_ty) {
                return Err(syn::Error::new(
                    decl.ident.span(),
                    format!(
                        "separate_inherent: 方法 {} 的返回类型不匹配\n\
                         声明: {}\n\
                         实现: {}",
                        decl.ident,
                        quote! { #decl_ty },
                        quote! { #impl_ty }
                    ),
                ));
            }
        }
        _ => {
            return Err(syn::Error::new(
                decl.ident.span(),
                format!(
                    "separate_inherent: 方法 {} 的返回类型不匹配\n\
                     声明: {}\n\
                     实现: {}",
                    decl.ident,
                    quote! { #decl.output },
                    quote! { #impl_.output }
                ),
            ));
        }
    }

    // async
    if decl.asyncness.is_some() != impl_.asyncness.is_some() {
        return Err(syn::Error::new(
            decl.ident.span(),
            format!(
                "separate_inherent: 方法 {} 的 async 修饰不匹配",
                decl.ident
            ),
        ));
    }

    // const
    if decl.constness.is_some() != impl_.constness.is_some() {
        return Err(syn::Error::new(
            decl.ident.span(),
            format!(
                "separate_inherent: 方法 {} 的 const 修饰不匹配",
                decl.ident
            ),
        ));
    }

    // unsafe
    if decl.unsafety.is_some() != impl_.unsafety.is_some() {
        return Err(syn::Error::new(
            decl.ident.span(),
            format!(
                "separate_inherent: 方法 {} 的 unsafe 修饰不匹配",
                decl.ident
            ),
        ));
    }

    // 泛型参数
    let decl_generics = decl.generics.params.len();
    let impl_generics = impl_.generics.params.len();
    if decl_generics != impl_generics {
        return Err(syn::Error::new(
            decl.ident.span(),
            format!(
                "separate_inherent: 方法 {} 的泛型参数数量不匹配\n\
                 声明: {} 个\n\
                 实现: {} 个",
                decl.ident, decl_generics, impl_generics
            ),
        ));
    }

    // where 子句
    let decl_where = decl.generics.where_clause.is_some();
    let impl_where = impl_.generics.where_clause.is_some();
    if decl_where != impl_where {
        return Err(syn::Error::new(
            decl.ident.span(),
            format!(
                "separate_inherent: 方法 {} 的 where 子句不匹配",
                decl.ident
            ),
        ));
    }

    Ok(())
}

/// 提取 receiver 类型
fn extract_receiver(sig: &Signature) -> Option<String> {
    if let Some(receiver) = sig.receiver() {
        let mut result = String::new();
        if receiver.reference.is_some() {
            result.push('&');
            if receiver.mutability.is_some() {
                result.push_str("mut ");
            }
        }
        result.push_str("self");
        Some(result)
    } else {
        None
    }
}

/// 简单的类型比较（简化版）
fn type_equal(a: &syn::Type, b: &syn::Type) -> bool {
    quote! { #a }.to_string() == quote! { #b }.to_string()
}

/// 生成最终的 impl 块
fn generate_impl_block(block: &ImplBlock) -> TokenStream {
    let methods = block.methods.iter().map(|m| {
        let sig = &m.sig;
        let block = &m.block;
        // 自动添加 pub（因为父 struct 是 pub 的）
        quote! {
            pub #sig #block
        }
    });

    let self_ty = &block.self_ty;

    quote! {
        impl #self_ty {
            #(#methods)*
        }
    }
    .into()
}
