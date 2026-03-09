#[path = "test_utils.rs"]
mod test_utils;

use std::fs;
use ci::checker::Runner;
use ci::fixer::{Fixer, Fix};
use test_utils::{create_temp_dir, create_test_config};

#[test]
fn test_fix_rename_file() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("WrongName.rs");

    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();

    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    fn method(&self) {}
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success());
    assert!(report.errors.iter().any(|e| e.message.contains("文件命名必须和 trait 对应")));

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.apply_fixes(&report).unwrap();

    assert!(fix_result.has_fixes());
    assert_eq!(fix_result.fix_count(), 1);

    let new_path = temp_dir.path().join("MyTrait_i32.rs");
    assert!(new_path.exists(), "文件应该被重命名为 MyTrait_i32.rs");
    assert!(!impl_file_path.exists(), "原文件应该被删除");
}

#[test]
fn test_fix_rename_file_with_chinese_error_message() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("错误名称.rs");

    fs::write(&mod_rs_path, r#"
pub trait TestTrait {
    fn do_something(&self);
}
"#).unwrap();

    fs::write(&impl_file_path, r#"
impl TestTrait for String {
    fn do_something(&self) {}
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success());

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.apply_fixes(&report).unwrap();

    assert!(fix_result.has_fixes());
    let new_path = temp_dir.path().join("TestTrait_String.rs");
    assert!(new_path.exists());
}

#[test]
fn test_fix_remove_pub() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("MyTrait_i32.rs");

    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();

    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    pub fn method(&self) {}
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success());
    assert!(report.errors.iter().any(|e| e.message.contains("禁止使用 pub")));

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.apply_fixes(&report).unwrap();

    assert!(fix_result.has_fixes());

    let content = fs::read_to_string(&impl_file_path).unwrap();
    assert!(!content.contains("pub fn"), "pub 关键字应该被移除");
}

#[test]
fn test_fix_add_module_declaration() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir(&tests_dir).unwrap();
    let test_file = tests_dir.join("example_test.rs");

    fs::write(&mod_rs_path, r#"
pub fn hello() {}
"#).unwrap();

    fs::write(&test_file, r#"
#[test]
fn test_example() {
    assert!(true);
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success());
    assert!(report.errors.iter().any(|e| e.message.contains("tests/ 目录必须在 mod.rs 中声明")));

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.apply_fixes(&report).unwrap();

    assert!(fix_result.has_fixes());

    let content = fs::read_to_string(&mod_rs_path).unwrap();
    assert!(content.contains("mod tests;"), "应该添加 mod tests; 声明");
}

#[test]
fn test_fix_suggest_rename_file() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("BadName.rs");

    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();

    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    fn method(&self) {}
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.fix(&report).unwrap();

    assert!(fix_result.has_fixes());
    assert_eq!(fix_result.fix_count(), 1);

    let (_, fix) = &fix_result.fixes[0];
    match fix {
        Fix::RenameFile { from, to } => {
            assert_eq!(from.file_name().unwrap(), "BadName.rs");
            assert_eq!(to.file_name().unwrap(), "MyTrait_i32.rs");
        }
        _ => panic!("期望 RenameFile 修复建议"),
    }
}

#[test]
fn test_fix_suggest_remove_pub() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("MyTrait_i32.rs");

    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();

    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    pub fn method(&self) {}
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.fix(&report).unwrap();

    assert!(fix_result.has_fixes());

    let (_, fix) = &fix_result.fixes[0];
    match fix {
        Fix::RemovePub { .. } => {}
        _ => panic!("期望 RemovePub 修复建议"),
    }
}

#[test]
fn test_fix_suggest_add_module_declaration() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir(&tests_dir).unwrap();
    let test_file = tests_dir.join("example_test.rs");

    fs::write(&mod_rs_path, r#"
pub fn hello() {}
"#).unwrap();

    fs::write(&test_file, r#"
#[test]
fn test_example() {
    assert!(true);
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.fix(&report).unwrap();

    assert!(fix_result.has_fixes());

    let (_, fix) = &fix_result.fixes[0];
    match fix {
        Fix::AddModuleDeclaration { module_name, .. } => {
            assert_eq!(module_name, "tests");
        }
        _ => panic!("期望 AddModuleDeclaration 修复建议"),
    }
}

#[test]
fn test_fix_multiple_errors() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file1 = temp_dir.path().join("WrongName1.rs");
    let impl_file2 = temp_dir.path().join("WrongName2.rs");

    fs::write(&mod_rs_path, r#"
pub trait TraitA {
    fn method_a(&self);
}
pub trait TraitB {
    fn method_b(&self);
}
"#).unwrap();

    fs::write(&impl_file1, r#"
impl TraitA for i32 {
    fn method_a(&self) {}
}
"#).unwrap();

    fs::write(&impl_file2, r#"
impl TraitB for String {
    fn method_b(&self) {}
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success());
    assert_eq!(report.errors.len(), 2);

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.apply_fixes(&report).unwrap();

    assert_eq!(fix_result.fix_count(), 2);
    assert!(temp_dir.path().join("TraitA_i32.rs").exists());
    assert!(temp_dir.path().join("TraitB_String.rs").exists());
}

#[test]
fn test_fix_no_errors() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("MyTrait_i32.rs");

    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();

    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    fn method(&self) {}
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(report.is_success());

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.fix(&report).unwrap();

    assert!(!fix_result.has_fixes());
    assert_eq!(fix_result.fix_count(), 0);
}

#[test]
fn test_fix_split_impl_file() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let multi_impl_path = temp_dir.path().join("MultiImpl.rs");

    fs::write(&mod_rs_path, r#"
pub trait TraitA {
    fn method_a(&self);
}
pub trait TraitB {
    fn method_b(&self);
}
"#).unwrap();

    fs::write(&multi_impl_path, r#"
impl TraitA for i32 {
    fn method_a(&self) {
        println!("i32");
    }
}

impl TraitB for String {
    fn method_b(&self) {
        println!("String");
    }
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success());
    assert!(report.errors.iter().any(|e| e.message.contains("impl 文件只能包含一个 impl 块")));

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.apply_fixes(&report).unwrap();

    // 应该有 1 个修复（拆分文件）
    assert!(fix_result.has_fixes());

    // 检查新文件是否被创建
    let trait_a_i32_path = temp_dir.path().join("TraitA_i32.rs");
    let trait_b_string_path = temp_dir.path().join("TraitB_String.rs");

    assert!(trait_a_i32_path.exists(), "TraitA_i32.rs 应该被创建");
    assert!(trait_b_string_path.exists(), "TraitB_String.rs 应该被创建");

    // 检查原文件是否被删除
    assert!(!multi_impl_path.exists(), "原 MultiImpl.rs 应该被删除");
}

#[test]
fn test_fix_move_impl_to_file() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");

    // mod.rs 包含 impl 块
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

pub struct MyType;

impl MyTrait for MyType {
    fn method(&self) {
        println!("MyType");
    }
}
"#).unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success());
    assert!(report.errors.iter().any(|e| e.message.contains("mod.rs 禁止包含任何形式的 impl")));

    let fixer = Fixer::new(create_test_config());
    let fix_result = fixer.apply_fixes(&report).unwrap();

    assert!(fix_result.has_fixes());

    // 检查新文件是否被创建（文件名是 Trait_Type 格式）
    let impl_file_path = temp_dir.path().join("MyTrait_MyType.rs");
    assert!(impl_file_path.exists(), "MyTrait_MyType.rs 应该被创建");

    // 检查 mod.rs 是否被修改（删除了 impl 块）
    let mod_content = fs::read_to_string(&mod_rs_path).unwrap();
    assert!(!mod_content.contains("impl MyTrait"), "mod.rs 应该删除 impl 块");
}
