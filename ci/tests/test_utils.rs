use tempfile::TempDir;
use ci::config::Config;

/// 创建临时目录并返回路径
pub fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// 创建测试用的配置文件
pub fn create_test_config() -> Config {
    let mut config = Config::default();
    config.checks.enabled = true;
    config.checks.mod_rs.enabled = true;
    config.checks.mod_rs.forbid_impl = true;
    config.checks.mod_rs.struct_must_be_public = true;
    config.checks.impl_file.enabled = true;
    config.checks.impl_file.single_impl_only = true;
    config.checks.impl_file.naming_must_match_trait = true;
    config.checks.impl_file.forbid_pub = true;
    config.checks.internal.enabled = true;
    config.checks.internal.forbid_mod_rs = true;
    config.checks.internal.require_brace_wrap = true;
    config.checks.internal.only_function_body = true;
    config.checks.tests.enabled = true;
    config.checks.tests.forbid_mod_rs = true;
    config.checks.tests.require_mod_declaration_in_parent = true;
    config.checks.tests.test_file_must_match_trait = true;
    config.checks.naming.enabled = true;
    config.checks.naming.forbid_impl_suffix = true;
    config.checks.naming.forbid_tests_suffix = true;
    config
}
