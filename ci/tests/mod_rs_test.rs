#[path = "test_utils.rs"]
mod test_utils;

use std::fs;
use ci::checks::Checker;
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
    let mut checker = Checker::new(config);
    let report = checker.check(temp_dir.path()).unwrap();
    
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
    let mut checker = Checker::new(config);
    let report = checker.check(temp_dir.path()).unwrap();

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
    let mut checker = Checker::new(config);
    let report = checker.check(temp_dir.path()).unwrap();
    
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
    let mut checker = Checker::new(config);
    let report = checker.check(temp_dir.path()).unwrap();

    assert!(!report.is_success(), "mod.rs 包含 include! 应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("mod.rs 禁止使用 include! 宏")));
}