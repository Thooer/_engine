//! 引擎配置目录常量
//!
//! 提供统一的配置目录路径获取接口

use std::path::PathBuf;

/// 引擎配置目录
pub mod dirs {
    use super::PathBuf;

    /// 获取引擎根目录
    ///
    /// 开发时: 当前工作目录
    /// 发布时: 相对于可执行文件的路径
    pub fn engine_root() -> PathBuf {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    /// 获取引擎配置目录
    ///
    /// 默认: 引擎根目录
    pub fn config_dir() -> PathBuf {
        engine_root()
    }

    /// 获取默认项目目录
    ///
    /// 默认: 引擎根目录下的 `demos` 文件夹
    pub fn projects_dir() -> PathBuf {
        engine_root().join("demos")
    }

    /// 获取全局配置文件路径
    ///
    /// 默认: 引擎根目录下的 `config.toml`
    pub fn global_config_path() -> PathBuf {
        config_dir().join("config.toml")
    }
}
