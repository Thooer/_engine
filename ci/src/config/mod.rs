mod matcher;

pub use matcher::PathMatcher;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub global: GlobalConfig,
    #[serde(default)]
    pub checks: ChecksConfig,
    #[serde(default)]
    pub whitelist: Vec<WhitelistEntry>,
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub root: String,
    /// crate 入口文件列表（如 lib.rs），CI 会从这些入口追踪模块树
    /// 如果为空，则使用传统的目录扫描模式
    #[serde(default)]
    pub entries: Vec<String>,
    /// 排除的路径模式（仅在传统模式下使用）
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            root: String::new(),
            entries: Vec::new(),
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/thirdparty/**".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub mod_rs: ModRsCheck,
    #[serde(default)]
    pub impl_file: ImplFileCheck,
    #[serde(default)]
    pub internal: InternalCheck,
    #[serde(default)]
    pub tests: TestsCheck,
    #[serde(default)]
    pub naming: NamingCheck,
}

fn default_true() -> bool {
    true
}

impl Default for ChecksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mod_rs: ModRsCheck::default(),
            impl_file: ImplFileCheck::default(),
            internal: InternalCheck::default(),
            tests: TestsCheck::default(),
            naming: NamingCheck::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRsCheck {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub forbid_impl: bool,
    /// mod.rs 中的所有 struct 必须是 `pub struct`，否则报错提示移到实现文件中
    #[serde(default = "default_true")]
    pub struct_must_be_public: bool,
    /// mod.rs / lib.rs 中禁止出现任何顶层函数（所有方法必须通过 trait 暴露）
    #[serde(default = "default_true")]
    pub forbid_free_functions: bool,
    /// trait 定义后紧跟的 impl 模块声明，文件路径必须以 trait 名称开头
    #[serde(default = "default_true")]
    pub trait_impl_order: bool,
    /// 禁止使用 pub mod 声明
    #[serde(default = "default_true")]
    pub forbid_pub_mod: bool,
    /// 禁止使用通配符重导出 (pub use xxx::*;)
    #[serde(default = "default_true")]
    pub forbid_wildcard_re_exports: bool,
    /// 禁止内联 mod 声明 (mod xxx { ... })
    #[serde(default = "default_true")]
    pub forbid_inline_mod: bool,
    /// 目录必须有 mod.rs 或 lib.rs（只有 internal/ 和 tests/ 是例外）
    #[serde(default = "default_true")]
    pub require_dir_has_entry: bool,
}

impl Default for ModRsCheck {
    fn default() -> Self {
        Self {
            enabled: true,
            forbid_impl: true,
            struct_must_be_public: true,
            forbid_free_functions: true,
            trait_impl_order: true,
            forbid_pub_mod: true,
            forbid_wildcard_re_exports: true,
            forbid_inline_mod: true,
            require_dir_has_entry: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplFileCheck {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub single_impl_only: bool,
    #[serde(default = "default_true")]
    pub naming_must_match_trait: bool,
    #[serde(default = "default_true")]
    pub forbid_pub: bool,
    /// 禁止固有实现（inherent impl），强制所有实现都通过 trait
    #[serde(default = "default_true")]
    pub forbid_inherent_impl: bool,
}

impl Default for ImplFileCheck {
    fn default() -> Self {
        Self {
            enabled: true,
            single_impl_only: true,
            naming_must_match_trait: true,
            forbid_pub: true,
            forbid_inherent_impl: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalCheck {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub forbid_mod_rs: bool,
    #[serde(default = "default_true")]
    pub require_brace_wrap: bool,
    #[serde(default = "default_true")]
    pub only_function_body: bool,
}

impl Default for InternalCheck {
    fn default() -> Self {
        Self {
            enabled: true,
            forbid_mod_rs: true,
            require_brace_wrap: true,
            only_function_body: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestsCheck {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub forbid_mod_rs: bool,
    #[serde(default = "default_true")]
    pub require_mod_declaration_in_parent: bool,
    #[serde(default = "default_true")]
    pub test_file_must_match_trait: bool,
}

impl Default for TestsCheck {
    fn default() -> Self {
        Self {
            enabled: true,
            forbid_mod_rs: true,
            require_mod_declaration_in_parent: true,
            test_file_must_match_trait: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingCheck {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub forbid_impl_suffix: bool,
    #[serde(default = "default_true")]
    pub forbid_tests_suffix: bool,
    /// 文件名中允许的最大下划线数量（0 表示不检查）
    #[serde(default = "default_max_underscores")]
    pub max_underscores: usize,
}

fn default_max_underscores() -> usize {
    1
}

impl Default for NamingCheck {
    fn default() -> Self {
        Self {
            enabled: true,
            forbid_impl_suffix: true,
            forbid_tests_suffix: true,
            max_underscores: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub path: String,
    #[serde(default)]
    pub allowed_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_true")]
    pub color: bool,
    #[serde(default)]
    pub verbose: bool,
    #[serde(default = "default_true")]
    pub progress: bool,
}

fn default_format() -> String {
    "human".to_string()
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "human".to_string(),
            color: true,
            verbose: false,
            progress: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            checks: ChecksConfig::default(),
            whitelist: Vec::new(),
            output: OutputConfig::default(),
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
