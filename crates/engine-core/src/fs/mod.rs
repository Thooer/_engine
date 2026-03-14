//! 简单的文件系统抽象（阶段 4 v0）
//!
//! 当前只提供本地文件实现，并预留将来接 VFS / 远程存储的接口。

use std::io;
use std::path::{Path, PathBuf};

/// 文件系统统一抽象
#[allow(dead_code)]
pub trait FileSystem: Send + Sync + 'static {
    /// 读取整个文本文件为 String
    fn read_to_string(&self, path: &Path) -> io::Result<String>;

    /// 读取整个二进制文件为 bytes
    fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>>;
}

/// 最简单的本地文件系统实现
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct LocalFileSystem {
    pub(crate) root: Option<PathBuf>,
}

#[path = "FileSystem_LocalFileSystem.rs"]
mod filesystem_localfilesystem;

