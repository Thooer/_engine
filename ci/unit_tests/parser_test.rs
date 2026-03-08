#[path = "test_utils.rs"]
mod test_utils;

use std::fs;
use ci::parser::{Parser, SynParser};
use test_utils::create_temp_dir;

#[test]
fn test_parser_has_function_signatures() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("test.rs");
    
    // 创建包含函数签名的文件（在 trait 中）
    fs::write(&file_path, r#"
trait TestTrait {
    fn signature() -> i32;
    fn another() -> String;
}
"#).unwrap();
    
    let parser = SynParser::new();
    let parsed = parser.parse(&file_path).unwrap();
    assert!(parsed.has_function_signatures(), "应该检测到函数签名");
}

#[test]
fn test_parser_no_function_signatures() {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join("test.rs");
    
    // 创建只包含函数体的文件（没有函数签名）
    fs::write(&file_path, r#"
{
    fn helper() {
        println!("help");
    }
}
"#).unwrap();
    
    let parser = SynParser::new();
    let parsed = parser.parse(&file_path).unwrap();
    assert!(!parsed.has_function_signatures(), "不应该检测到函数签名");
}

#[test]
fn test_parser_get_modules_after_traits() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // 创建 mod.rs，包含 trait 和模块
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

mod tests;

pub trait AnotherTrait {
    fn another(&self);
}

mod other;
"#).unwrap();
    
    let parser = SynParser::new();
    let parsed = parser.parse(&mod_rs_path).unwrap();
    let modules_after_traits = parsed.get_modules_after_traits();
    
    // 检查 tests 模块是否跟在 MyTrait 后面
    let tests_module = modules_after_traits.iter()
        .find(|(name, _)| name == "tests");
    assert!(tests_module.is_some(), "应该找到 tests 模块");
    assert_eq!(tests_module.unwrap().1, Some("MyTrait".to_string()), "tests 模块应该跟在 MyTrait 后面");
    
    // 检查 other 模块是否跟在 AnotherTrait 后面
    let other_module = modules_after_traits.iter()
        .find(|(name, _)| name == "other");
    assert!(other_module.is_some(), "应该找到 other 模块");
    assert_eq!(other_module.unwrap().1, Some("AnotherTrait".to_string()), "other 模块应该跟在 AnotherTrait 后面");
}

#[test]
fn test_parser_get_modules_after_traits_with_attributes() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    
    // 创建 mod.rs，包含 trait 和带属性的模块声明
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

#[cfg(test)]
mod tests;

pub trait AnotherTrait {
    fn another(&self);
}

#[cfg(feature = "graphics")]
mod graphics;

pub trait ThirdTrait {
    fn third(&self);
}

mod internal;
"#).unwrap();
    
    let parser = SynParser::new();
    let parsed = parser.parse(&mod_rs_path).unwrap();
    let modules_after_traits = parsed.get_modules_after_traits();
    
    // 检查 #[cfg(test)] mod tests; 是否跟在 MyTrait 后面
    let tests_module = modules_after_traits.iter()
        .find(|(name, _)| name == "tests");
    assert!(tests_module.is_some(), "应该找到 tests 模块（带属性前缀）");
    assert_eq!(tests_module.unwrap().1, Some("MyTrait".to_string()), "tests 模块应该跟在 MyTrait 后面");
    
    // 检查 #[cfg(feature = "graphics")] mod graphics; 是否跟在 AnotherTrait 后面
    let graphics_module = modules_after_traits.iter()
        .find(|(name, _)| name == "graphics");
    assert!(graphics_module.is_some(), "应该找到 graphics 模块（带属性前缀）");
    assert_eq!(graphics_module.unwrap().1, Some("AnotherTrait".to_string()), "graphics 模块应该跟在 AnotherTrait 后面");
    
    // 检查无属性的 internal 模块是否跟在 ThirdTrait 后面
    let internal_module = modules_after_traits.iter()
        .find(|(name, _)| name == "internal");
    assert!(internal_module.is_some(), "应该找到 internal 模块");
    assert_eq!(internal_module.unwrap().1, Some("ThirdTrait".to_string()), "internal 模块应该跟在 ThirdTrait 后面");
}
