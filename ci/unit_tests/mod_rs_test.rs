#[path = "test_utils.rs"]
mod test_utils;

use std::fs;
use ci::checker::Runner;
use test_utils::{create_temp_dir, create_test_config};

#[test]
fn test_mod_rs_forbid_impl() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // 创建包含 impl 的 mod.rs（应该失败）
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

impl MyTrait for i32 {
    fn method(&self) {}
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "mod.rs 包含 impl 应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("禁止包含任何形式的 impl")));
}

#[test]
fn test_mod_rs_struct_must_be_public() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");

    // 创建包含私有 struct 的 mod.rs（应该失败）
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

struct PrivateStruct {
    field: i32,
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success(), "mod.rs 包含私有 struct 应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("struct 必须是公开的")));
}

#[test]
fn test_mod_rs_valid() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // 创建有效的 mod.rs（只包含 trait 定义）
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

pub struct MyStruct {
    pub field: i32,
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(report.is_success(), "有效的 mod.rs 应该通过检查");
}

#[test]
fn test_mod_rs_forbid_include_macro() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");

    // 创建包含 include! 的 mod.rs（应该失败）
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

include!("some_impl.rs");
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success(), "mod.rs 包含 include! 应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("mod.rs 禁止使用 include! 宏")));
}

#[test]
fn test_mod_rs_trait_impl_order_valid() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_path = temp_dir.path().join("MyTrait_MyType.rs");
    
    // 创建有效的 mod.rs（impl 在 trait 之后）
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

#[path = "MyTrait_MyType.rs"]
mod my_trait_my_type;
"#).unwrap();
    
    // 创建 impl 文件
    fs::write(&impl_path, r#"
impl MyTrait for MyType {
    fn method(&self) {}
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(report.is_success(), "impl 在 trait 之后应该通过检查");
}

#[test]
fn test_mod_rs_trait_impl_order_invalid() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // 创建无效的 mod.rs（impl 在 trait 之前）
    fs::write(&mod_rs_path, r#"
#[path = "MyTrait_MyType.rs"]
mod my_trait_my_type;

pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "impl 在 trait 之前应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("impl 模块必须放在对应 trait 定义之后")));
}

#[test]
fn test_mod_rs_trait_impl_order_with_pub_trait() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // 测试 pub trait 场景（impl 在 trait 之后 - 应该通过）
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

#[path = "MyTrait_MyType.rs"]
mod my_trait_my_type;
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(report.is_success(), "pub trait 在 impl 模块之前应该通过检查");
}

#[test]
fn test_mod_rs_trait_impl_order_pub_trait_before() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // 测试 pub trait 场景（impl 在 trait 之前 - 应该失败）
    fs::write(&mod_rs_path, r#"
#[path = "MyTrait_MyType.rs"]
mod my_trait_my_type;

pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "pub trait 在 impl 模块之后应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("impl 模块必须放在对应 trait 定义之后")));
}

#[test]
fn test_mod_rs_trait_impl_order_multiple_traits() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // 多个 trait，impl 模块紧跟在对应的 trait 之后（中间没有其他 trait）
    fs::write(&mod_rs_path, r#"
pub trait TraitA {
    fn method_a(&self);
}

#[path = "TraitA_TypeA.rs"]
mod trait_a_type_a;

pub trait TraitB {
    fn method_b(&self);
}

#[path = "TraitB_TypeB.rs"]
mod trait_b_type_b;
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(report.is_success(), "多个 trait 时，impl 紧跟对应 trait 之后应该通过");
}

#[test]
fn test_mod_rs_trait_impl_order_undefined_trait() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // trait 未在 mod.rs 中定义（应该跳过检查）
    fs::write(&mod_rs_path, r#"
#[path = "UnknownTrait_MyType.rs"]
mod unknown_trait_my_type;
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    // 不应该报错，因为 UnknownTrait 没有在 mod.rs 中定义
    assert!(report.is_success(), "未定义的 trait 应该跳过检查");
}

#[test]
fn test_mod_rs_trait_impl_order_without_path_attribute() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // 没有 #[path = "..."] 属性（应该跳过检查）
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

mod my_impl;
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    // 没有 path 属性时不检查
    assert!(report.is_success(), "没有 #[path] 属性的 mod 声明应该跳过检查");
}