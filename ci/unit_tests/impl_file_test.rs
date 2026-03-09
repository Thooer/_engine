#[path = "test_utils.rs"]
mod test_utils;

use std::fs;
use ci::checker::Runner;
use test_utils::{create_temp_dir, create_test_config};

#[test]
fn test_impl_file_single_impl_only() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("MyTrait_i32.rs");
    
    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    // 创建包含多个 impl 块的文件（应该失败）
    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    fn method(&self) {}
}

impl MyTrait for u32 {
    fn method(&self) {}
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "包含多个 impl 块的文件应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("只能包含一个 impl 块")));
}

#[test]
fn test_impl_file_naming_must_match_trait() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("WrongName.rs");
    
    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    // 创建命名不匹配的 impl 文件（应该失败）
    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    fn method(&self) {}
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "命名不匹配的 impl 文件应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("文件命名必须和 trait 对应")));
}

#[test]
fn test_impl_file_valid_naming() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("MyTrait_i32.rs");
    
    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    // 创建正确命名的 impl 文件
    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    fn method(&self) {}
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(report.is_success(), "正确命名的 impl 文件应该通过检查");
}

#[test]
fn test_impl_file_forbid_pub() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("MyTrait_i32.rs");
    
    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    // 创建包含 pub 关键字的 impl 文件（应该失败）
    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    pub fn method(&self) {}
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "包含 pub 关键字的 impl 文件应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("禁止使用 pub 关键字")));
}

#[test]
fn test_empty_impl_exception() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("MyTrait_i32.rs");
    
    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    // 创建包含多个空 impl 块的文件（同一个 trait，应该允许）
    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {}
impl MyTrait for u32 {}
impl MyTrait for f64 {}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    // 多个空 impl 块且是同一个 trait，应该允许
    assert!(report.is_success(), "多个空 impl 块且是同一个 trait 应该允许");
}

#[test]
fn test_include_macro_must_be_in_impl_file() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let helper_path = temp_dir.path().join("helper.rs");

    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();

    // helper.rs 里使用 include!，但不包含 impl 块（应该失败）
    fs::write(&helper_path, r#"
include!("internal/some.rs");
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success(), "非实现文件使用 include! 应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("include! 只能放在实现文件中")));
}

#[test]
fn test_include_macro_allowed_in_impl_file() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("MyTrait_i32.rs");

    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();

    // 实现文件中允许 include!
    fs::write(&impl_file_path, r#"
include!("internal/some.rs");
impl MyTrait for i32 {
    fn method(&self) {}
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(report.is_success(), "实现文件中使用 include! 应该允许");
}