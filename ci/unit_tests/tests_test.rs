#[path = "test_utils.rs"]
mod test_utils;

use std::fs;
use ci::checker::Runner;
use test_utils::{create_temp_dir, create_test_config};

#[test]
fn test_tests_dir_forbid_mod_rs() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    let tests_mod_rs_path = tests_dir.join("mod.rs");

    // 创建父 mod.rs
    fs::write(&mod_rs_path, "pub trait MyTrait { fn method(&self); }").unwrap();

    // 创建 tests/mod.rs（应该失败）
    fs::write(&tests_mod_rs_path, "").unwrap();

    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();

    assert!(!report.is_success(), "tests/ 目录存在 mod.rs 应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("禁止存在 mod.rs")));
}

#[test]
fn test_tests_dir_require_mod_declaration() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    
    // 创建 mod.rs（不包含 tests 模块声明）
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    // 创建测试文件
    let test_file = tests_dir.join("MyTrait.rs");
    fs::write(&test_file, r#"
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_trait() {
        assert!(true);
    }
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "tests/ 目录必须在 mod.rs 中声明模块");
    assert!(report.errors.iter().any(|e| e.message.contains("必须在 mod.rs 中声明模块")));
}

#[test]
fn test_tests_dir_mod_must_follow_trait() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    
    // 创建 mod.rs（tests 模块不在 trait 后面）
    fs::write(&mod_rs_path, r#"
mod tests;

pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    // 创建测试文件
    let test_file = tests_dir.join("MyTrait.rs");
    fs::write(&test_file, r#"
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_trait() {
        assert!(true);
    }
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "tests 模块必须跟在 trait 后面");
    assert!(report.errors.iter().any(|e| e.message.contains("必须跟在某个 trait 后面")));
}

#[test]
fn test_tests_dir_valid_mod_after_trait() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    
    // 创建 mod.rs（tests 模块跟在 trait 后面）
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

mod tests;
"#).unwrap();
    
    // 创建测试文件
    let test_file = tests_dir.join("MyTrait.rs");
    fs::write(&test_file, r#"
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_trait() {
        assert!(true);
    }
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(report.is_success(), "tests 模块跟在 trait 后面应该通过检查");
}

#[test]
fn test_tests_file_must_match_trait() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    
    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

mod tests;
"#).unwrap();
    
    // 创建不匹配 trait 名的测试文件（应该失败）
    let test_file = tests_dir.join("WrongTrait.rs");
    fs::write(&test_file, r#"
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wrong() {
        assert!(true);
    }
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "测试文件名必须和 trait 同名");
    assert!(report.errors.iter().any(|e| e.message.contains("必须和某个 trait 同名")));
}

#[test]
fn test_tests_file_valid_matching_trait() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let tests_dir = temp_dir.path().join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    
    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

mod tests;
"#).unwrap();
    
    // 创建匹配 trait 名的测试文件
    let test_file = tests_dir.join("MyTrait.rs");
    fs::write(&test_file, r#"
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_trait() {
        assert!(true);
    }
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(report.is_success(), "测试文件名匹配 trait 名应该通过检查");
}
