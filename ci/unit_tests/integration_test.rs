#[path = "test_utils.rs"]
mod test_utils;

use std::fs;
use ci::checker::Runner;
use test_utils::{create_temp_dir, create_test_config};

#[test]
fn test_complete_valid_structure() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("MyTrait_i32.rs");
    let internal_dir = temp_dir.path().join("internal");
    let tests_dir = temp_dir.path().join("tests");
    
    fs::create_dir_all(&internal_dir).unwrap();
    fs::create_dir_all(&tests_dir).unwrap();
    
    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}

mod tests;
"#).unwrap();
    
    // 创建 impl 文件
    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    fn method(&self) {}
}
"#).unwrap();
    
    // 创建 internal 文件（只包含函数体，没有函数签名）
    let internal_file = internal_dir.join("helper.rs");
    fs::write(&internal_file, r#"
{
    fn helper() {
        println!("help");
    }
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

    // Debug: print all errors
    if !report.is_success() {
        for error in &report.errors {
            println!("Error: {:?}", error);
        }
    }

    assert!(report.is_success(), "完整的有效结构应该通过所有检查");
}
