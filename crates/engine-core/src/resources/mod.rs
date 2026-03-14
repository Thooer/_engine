//! 资源系统 v1：AssetServer + 场景序列化
//!
//! - AssetHandle: 资源句柄系统
//! - AssetConfig: 资源配置
//! - Scene: 场景序列化

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fs::FileSystem;

/// 资源句柄 - 用于引用资源
///
/// 使用 UUID 生成唯一标识，支持引用计数
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHandle(pub Uuid);

impl AssetHandle {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AssetHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// 资源状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    NotLoaded,
    Loading,
    Loaded,
    Failed,
}

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Model,
    Texture,
    Material,
    Scene,
}

/// 资源元数据
#[derive(Debug, Clone)]
pub struct AssetMetadata {
    /// 资源路径
    pub path: PathBuf,
    /// 资源类型
    pub asset_type: AssetType,
    /// 加载状态
    pub load_state: LoadState,
    /// 依赖的其他资源句柄
    pub dependencies: Vec<AssetHandle>,
}

/// 加载请求
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LoadRequest {
    pub handle: AssetHandle,
    pub path: PathBuf,
    pub asset_type: AssetType,
}

/// 资源服务器 - 核心资源管理
///
/// 支持：
/// - 异步加载（通过 tokio）
/// - 资源缓存
/// - 依赖追踪
/// - 资源句柄管理
#[derive(Resource)]
#[allow(dead_code)]
pub struct AssetServer {
    /// 资源缓存: Handle -> 资源数据
    cache: HashMap<AssetHandle, Arc<dyn std::any::Any + Send + Sync>>,
    /// 元数据缓存: Handle -> 元数据
    metadata: HashMap<AssetHandle, AssetMetadata>,
    /// 路径到 Handle 的映射
    path_to_handle: HashMap<PathBuf, AssetHandle>,
    /// 待处理的加载请求
    pending_loads: Vec<LoadRequest>,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetServer {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            metadata: HashMap::new(),
            path_to_handle: HashMap::new(),
            pending_loads: Vec::new(),
        }
    }

    /// 加载资源（同步版本）
    pub fn load_sync(&mut self, path: &Path, asset_type: AssetType) -> Result<AssetHandle, AssetError> {
        // 检查是否已加载
        if let Some(handle) = self.path_to_handle.get(path) {
            if let Some(meta) = self.metadata.get(handle) {
                if meta.load_state == LoadState::Loaded {
                    return Ok(*handle);
                }
            }
        }

        // 创建新的 Handle
        let handle = AssetHandle::new();
        
        // 记录元数据
        self.metadata.insert(handle, AssetMetadata {
            path: path.to_path_buf(),
            asset_type,
            load_state: LoadState::Loading,
            dependencies: Vec::new(),
        });
        
        self.path_to_handle.insert(path.to_path_buf(), handle);
        
        // 更新状态为已加载
        if let Some(meta) = self.metadata.get_mut(&handle) {
            meta.load_state = LoadState::Loaded;
        }

        Ok(handle)
    }

    /// 检查资源是否已加载
    pub fn is_loaded(&self, handle: &AssetHandle) -> bool {
        self.metadata
            .get(handle)
            .map(|m| m.load_state == LoadState::Loaded)
            .unwrap_or(false)
    }

    /// 获取资源的加载状态
    pub fn get_load_state(&self, handle: &AssetHandle) -> Option<LoadState> {
        self.metadata.get(handle).map(|m| m.load_state)
    }

    /// 获取资源元数据
    pub fn get_metadata(&self, handle: &AssetHandle) -> Option<&AssetMetadata> {
        self.metadata.get(handle)
    }

    /// 根据路径获取 Handle
    pub fn get_handle(&self, path: &Path) -> Option<AssetHandle> {
        self.path_to_handle.get(path).copied()
    }
}

/// 资源配置 - 可作为 ECS Resource 插入 World
#[derive(Resource, Debug, Clone)]
pub struct AssetConfig {
    /// 资源根目录
    pub root_path: PathBuf,
    /// 材质目录
    pub materials_path: PathBuf,
    /// 模型目录
    pub models_path: PathBuf,
    /// 纹理目录
    pub textures_path: PathBuf,
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("assets"),
            materials_path: PathBuf::from("assets/materials"),
            models_path: PathBuf::from("assets/models"),
            textures_path: PathBuf::from("assets/textures"),
        }
    }
}

impl AssetConfig {
    /// 创建自定义资源配置
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            root_path: root.clone(),
            materials_path: root.join("materials"),
            models_path: root.join("models"),
            textures_path: root.join("textures"),
        }
    }
    
    /// 获取材质完整路径
    pub fn material_path(&self, name: &str) -> PathBuf {
        self.materials_path.join(name)
    }
    
    /// 获取模型完整路径
    pub fn model_path(&self, name: &str) -> PathBuf {
        self.models_path.join(name)
    }
    
    /// 获取纹理完整路径
    pub fn texture_path(&self, name: &str) -> PathBuf {
        self.textures_path.join(name)
    }
}

/// 简单的 2D 网格资源表示（阶段 4 Demo 使用）
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MeshAsset {
    pub positions: Vec<(f32, f32)>,
    pub indices: Vec<u32>,
}

/// 最小资源管理器：
///
/// - 通过 FileSystem 统一读盘
/// - 针对同一路径做缓存，避免重复加载
#[allow(dead_code)]
pub struct AssetManager<F: FileSystem> {
    pub(crate) fs: F,
    pub(crate) meshes: HashMap<PathBuf, Arc<MeshAsset>>,
}

/// 为 `AssetManager` 提供构造与加载接口的扩展 trait。
#[allow(dead_code)]
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

// ============================================================================
// 场景序列化
// ============================================================================

use bevy_ecs::prelude::*;

/// 场景 - 可序列化/反序列化的 ECS 世界快照
///
/// 用于保存和加载游戏关卡
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Scene {
    /// 场景名称
    pub name: String,
    /// 场景中的所有实体数据
    pub entities: Vec<SceneEntity>,
}

/// 场景中的实体数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneEntity {
    /// 实体组件列表
    pub components: Vec<ComponentData>,
}

/// 组件数据 - 使用 serde tag 来区分不同组件类型
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ComponentData {
    /// Transform 组件
    Transform {
        translation: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    },
    /// 相机组件
    Camera3D {
        position: [f32; 3],
        forward: [f32; 3],
    },
    /// 刚体组件
    RigidBody {
        body_type: String,
        mass: f32,
    },
    /// 碰撞体组件
    Collider {
        shape: String,
        half_extents: Option<[f32; 3]>,
        radius: Option<f32>,
        friction: f32,
        restitution: f32,
    },
    /// 网格渲染组件
    MeshRenderable {
        mesh_id: String,
        material_id: String,
    },
    /// 点光源组件
    EcsPointLight {
        position: [f32; 3],
        range: f32,
        color: [f32; 3],
        intensity: f32,
    },
    /// 线条渲染组件
    LineRenderable {
        start: [f32; 3],
        end: [f32; 3],
        color: [f32; 4],
    },
    /// 网格配置组件
    GridConfig {
        size: f32,
        divisions: u32,
        color: [f32; 4],
    },
    /// 相机控制器组件
    CameraController {
        controller_type: String,
    },
}

/// 场景加载器
pub struct SceneLoader;

impl SceneLoader {
    /// 从文件加载场景
    pub fn load(path: &str) -> Result<Scene, SceneError> {
        let content = std::fs::read_to_string(path).map_err(SceneError::Io)?;
        let scene = ron::from_str(&content).map_err(|e: ron::error::SpannedError| SceneError::Parse(e.to_string()))?;
        Ok(scene)
    }

    /// 保存场景到文件
    pub fn save(scene: &Scene, path: &str) -> Result<(), SceneError> {
        let content = ron::to_string(scene).map_err(|e| SceneError::Parse(e.to_string()))?;
        std::fs::write(path, content).map_err(SceneError::Io)?;
        Ok(())
    }

    /// 将场景 spawn 到 World
    /// 
    /// 注意：这个方法需要外部提供组件转换函数
    /// 因为 Transform、Collider 等组件定义在 engine-physics 和 engine-renderer 中
    pub fn spawn_scene<F>(world: &mut bevy_ecs::prelude::World, scene: &Scene, mut spawn_fn: F)
    where 
        F: FnMut(&mut bevy_ecs::prelude::World, &ComponentData),
    {
        for entity in &scene.entities {
            for component in &entity.components {
                spawn_fn(world, component);
            }
        }
    }
}

/// 场景错误类型
#[derive(Debug)]
pub enum SceneError {
    Io(io::Error),
    Parse(String),
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for SceneError {}

