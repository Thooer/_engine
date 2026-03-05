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
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            root: String::new(),
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
}

impl Default for ModRsCheck {
    fn default() -> Self {
        Self {
            enabled: true,
            forbid_impl: true,
            struct_must_be_public: true,
            forbid_free_functions: true,
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
}

impl Default for NamingCheck {
    fn default() -> Self {
        Self {
            enabled: true,
            forbid_impl_suffix: true,
            forbid_tests_suffix: true,
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

    pub fn is_path_excluded(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.global.exclude_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }
        false
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // 简单的“包含子串”匹配（不是完整 glob）。
        //
        // 关键点：为了跨平台（尤其是 Windows 的 `\` 路径分隔符），先把路径统一成 `/` 再做匹配。
        let norm_path = path.replace('\\', "/");
        let norm_pattern = pattern.replace('\\', "/");

        let norm_pattern = norm_pattern.replace("**/", "");
        let norm_pattern = norm_pattern.replace("**", "");

        // 如果 pattern 为空，就认为不匹配（避免误伤）
        if norm_pattern.trim().is_empty() {
            return false;
        }

        norm_path.contains(&norm_pattern)
    }

    pub fn is_whitelisted(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();
        for entry in &self.whitelist {
            if path_str.contains(&entry.path) {
                return true;
            }
        }
        false
    }
}
