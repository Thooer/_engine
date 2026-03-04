//! 资源系统 v0：最小 AssetManager 与缓存
//!
//! 当前只支持从本地文件加载网格（RON），后续可扩展到纹理 / glTF 等。

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;

use crate::fs::FileSystem;

/// 简单的 2D 网格资源表示（阶段 4 Demo 使用）
#[derive(Debug, Clone, Deserialize)]
pub struct MeshAsset {
    pub positions: Vec<(f32, f32)>,
    pub indices: Vec<u32>,
}

/// 最小资源管理器：
///
/// - 通过 FileSystem 统一读盘
/// - 针对同一路径做缓存，避免重复加载
pub struct AssetManager<F: FileSystem> {
    pub(crate) fs: F,
    pub(crate) meshes: HashMap<PathBuf, Arc<MeshAsset>>,
}

/// 为 `AssetManager` 提供构造与加载接口的扩展 trait。
pub trait AssetManagerExt<F: FileSystem> {
    fn new(fs: F) -> Self
    where
        Self: Sized;

    /// 加载（或复用缓存中的）网格资源
    fn load_mesh<P: AsRef<Path>>(&mut self, path: P) -> Result<Arc<MeshAsset>, AssetError>;
}

/// 资源相关错误类型（v0）
#[derive(Debug)]
pub enum AssetError {
    Io(io::Error),
    ParseMeshRon(ron::Error),
}

#[path = "AssetManagerExt_AssetManager.rs"]
mod assetmanagerext_assetmanager;

