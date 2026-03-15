//! 配置系统抽象
//!
//! 提供平台无关的配置加载抽象。

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),
}

/// 配置加载器 Trait
///
/// 提供统一的配置加载接口，支持：
/// - 从 TOML 文件加载配置
/// - 配置不存在时使用默认值
/// - 运行时检查配置文件是否存在
pub trait ConfigLoader: Sized + Default + DeserializeOwned {
    /// 从指定路径加载配置
    ///
    /// # Arguments
    /// * `path` - 配置文件路径（不含文件名，文件名由具体实现决定）
    ///
    /// # Returns
    /// * `Ok(Config)` - 加载成功
    /// * `Err(ConfigError)` - 加载失败
    fn load(path: &Path) -> Result<Self, ConfigError>;

    /// 加载配置，失败时返回默认值
    ///
    /// # Arguments
    /// * `path` - 配置文件目录路径
    ///
    /// # Note
    /// 此方法会静默忽略所有错误，仅用于可选配置文件
    fn load_or_default(path: &Path) -> Self {
        Self::load(path).unwrap_or_default()
    }

    /// 检查配置文件是否存在
    ///
    /// # Arguments
    /// * `path` - 配置文件目录路径
    ///
    /// # Returns
    /// * `true` - 配置文件存在
    /// * `false` - 配置文件不存在
    fn exists(path: &Path) -> bool {
        Self::config_file_path(path).exists()
    }

    /// 获取配置文件路径（供内部使用）
    ///
    /// 默认实现假设配置文件名为 `config.toml`
    fn config_file_path(path: &Path) -> PathBuf {
        path.join("config.toml")
    }
}

/// 项目配置（来自 project.toml）
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProjectConfig {
    /// 项目名称（支持多种格式：顶层 name 或 [name.xxx]）
    #[serde(default)]
    name: Option<toml::Value>,
    /// 运行配置
    #[serde(default)]
    pub run: RunConfig,
}

/// 运行配置
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RunConfig {
    /// 入口场景路径（相对于项目根目录）
    #[serde(default = "default_scene")]
    pub scene: String,
    /// 资源目录
    #[serde(default = "default_assets_dir")]
    pub assets_dir: String,
    /// WASM 脚本路径（可选）
    #[serde(default)]
    pub script: Option<String>,
    /// 相机控制模式
    #[serde(default = "default_camera_mode")]
    pub camera_mode: String,
}

fn default_scene() -> String {
    "assets/scenes/main.ron".to_string()
}

fn default_assets_dir() -> String {
    "assets".to_string()
}

fn default_camera_mode() -> String {
    "orbit".to_string()
}

impl ProjectConfig {
    /// 获取项目名称
    /// 支持两种格式：
    /// 1. name = "项目名"（顶层字段）
    /// 2. [name.xxx] 或 [name] 下有一个 key
    pub fn project_name(&self) -> &str {
        // 首先尝试顶层 name 字段
        if let Some(v) = &self.name {
            if let Some(s) = v.as_str() {
                return s;
            }
            // 支持 name = { xxx = "项目名" } 格式
            if let Some(table) = v.as_table() {
                for (_key, value) in table {
                    if let Some(s) = value.as_str() {
                        return s;
                    }
                }
            }
        }
        "unnamed_project"
    }

    /// 获取完整的场景路径
    pub fn scene_path(&self, project_dir: &Path) -> PathBuf {
        project_dir.join(&self.run.scene)
    }

    /// 获取完整的资源目录路径
    pub fn assets_dir_path(&self, project_dir: &Path) -> PathBuf {
        project_dir.join(&self.run.assets_dir)
    }
}

impl ConfigLoader for ProjectConfig {
    fn load(path: &Path) -> Result<Self, ConfigError> {
        let config_path = Self::config_file_path(path);

        if !config_path.exists() {
            return Err(ConfigError::NotFound(config_path));
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    fn config_file_path(path: &Path) -> PathBuf {
        path.join("project.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProjectConfig::default();
        assert_eq!(config.run.scene, "assets/scenes/main.ron");
        assert_eq!(config.run.assets_dir, "assets");
        assert_eq!(config.run.camera_mode, "orbit");
    }
}
