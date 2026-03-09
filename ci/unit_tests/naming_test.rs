#[path = "test_utils.rs"]
mod test_utils;

use std::fs;
use ci::checker::Runner;
use test_utils::{create_temp_dir, create_test_config};

#[test]
fn test_naming_forbid_impl_suffix() {
    let temp_dir = create_temp_dir();
    let mod_rs_path = temp_dir.path().join("mod.rs");
    let impl_file_path = temp_dir.path().join("something_impl.rs");
    
    // 创建 mod.rs
    fs::write(&mod_rs_path, r#"
pub trait MyTrait {
    fn method(&self);
}
"#).unwrap();
    
    // 创建以 _impl 结尾的文件（应该失败）
    fs::write(&impl_file_path, r#"
impl MyTrait for i32 {
    fn method(&self) {}
}
"#).unwrap();
    
    let config = create_test_config();
    let runner = Runner::new(config);
    let report = runner.run(temp_dir.path()).unwrap();
    
    assert!(!report.is_success(), "以 _impl 结尾的文件应该失败");
    assert!(report.errors.iter().any(|e| e.message.contains("禁止使用 *impl.rs 命名")));
}
