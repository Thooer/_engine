#[path = "test_utils.rs"]
mod test_utils;

use std::fs;
use ci::checks::Checker;
use test_utils::{create_temp_dir, create_test_config};

#[test]
fn test_internal_dir_forbid_mod_rs() {
    let temp_dir = create_temp_dir();
    let internal_dir = temp_dir.path().join("internal");
    fs::create_dir_all(&internal_dir).unwrap();
    let mod_rs_path = internal_dir.join("mod.rs");
    
    // 创建 internal/mod.rs（应该失败）
    fs::write(&mod_rs_path, "").unwrap();
    
    let config = create_test_config();
    let mut checker = Checker::new(config);
    let report = checker.check(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "internal/ 目录存在 mod.rs 应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("禁止存在 mod.rs")));
}

#[test]
fn test_internal_dir_require_brace_wrap() {
    let temp_dir = create_temp_dir();
    let internal_dir = temp_dir.path().join("internal");
    fs::create_dir_all(&internal_dir).unwrap();
    let file_path = internal_dir.join("helper.rs");
    
    // 创建没有大括号包裹的文件（应该失败）
    fs::write(&file_path, r#"
fn helper() {
    println!("help");
}
"#).unwrap();
    
    let config = create_test_config();
    let mut checker = Checker::new(config);
    let report = checker.check(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "internal/ 文件没有大括号包裹应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("必须用大括号")));
}

#[test]
fn test_internal_dir_valid_brace_wrap() {
    let temp_dir = create_temp_dir();
    let internal_dir = temp_dir.path().join("internal");
    fs::create_dir_all(&internal_dir).unwrap();
    let file_path = internal_dir.join("helper.rs");
    
    // 创建有大括号包裹的文件（只包含函数体，没有函数签名）
    fs::write(&file_path, r#"
{
    fn helper() {
        println!("help");
    }
}
"#).unwrap();
    
    let config = create_test_config();
    let mut checker = Checker::new(config);
    let report = checker.check(temp_dir.path()).unwrap();
    
    assert!(report.is_success(), "有大括号包裹的 internal/ 文件应该通过检查");
}

#[test]
fn test_internal_dir_only_function_body() {
    let temp_dir = create_temp_dir();
    let internal_dir = temp_dir.path().join("internal");
    fs::create_dir_all(&internal_dir).unwrap();
    let file_path = internal_dir.join("helper.rs");
    
    // 创建包含函数签名的文件（应该失败）
    // 注意：在 Rust 中，函数签名不能单独存在，必须在 trait 中
    // 所以这里使用 trait 来测试函数签名检测
    fs::write(&file_path, r#"
{
    trait HelperTrait {
        fn helper() -> i32;
        fn another() -> String;
    }
}
"#).unwrap();
    
    let config = create_test_config();
    let mut checker = Checker::new(config);
    let report = checker.check(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "包含函数签名的 internal/ 文件应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("只能包含函数体")));
}
