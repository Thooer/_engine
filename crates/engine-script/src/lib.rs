//! ToyEngine 脚本系统
//!
//! 提供统一的脚本抽象接口，支持 WASM 运行时

use std::path::{Path, PathBuf};

use thiserror::Error;

// ============================================================================
// ScriptError - 脚本错误类型
// ============================================================================

/// 脚本错误类型
#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Load error: {0}")]
    Load(String),
    
    #[error("Module error: {0}")]
    Module(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    
    #[error("Call error: {0}")]
    Call(String),
    
    #[error("Context error: {0}")]
    Context(String),
}

// ============================================================================
// ScriptContext - 脚本上下文接口
// ============================================================================

/// 帧数据 - 从引擎传递给脚本
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[repr(C)]
pub struct FrameData {
    pub delta_time: f32,
    pub total_time: f64,
    pub frame_count: u32,
}

impl FrameData {
    pub fn new(delta_time: f32, total_time: f64, frame_count: u32) -> Self {
        Self { delta_time, total_time, frame_count }
    }
}

/// 脚本更新结果
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct UpdateResult {
    pub success: bool,
    pub error_message: u32,
}

impl UpdateResult {
    pub fn success() -> Self {
        Self { success: true, error_message: 0 }
    }
    
    pub fn error() -> Self {
        Self { success: false, error_message: 0 }
    }
}

// ============================================================================
// ScriptHost - 脚本运行时主机（具体实现）
// ============================================================================

/// 脚本主机 Trait - 脚本运行时的抽象
pub trait ScriptHost: Send + Sync {
    /// 加载脚本模块
    fn load(&mut self, path: &Path) -> Result<(), ScriptError>;
    
    /// 检查是否已加载
    fn is_loaded(&self) -> bool;
    
    /// 初始化脚本
    fn init(&mut self, frame: FrameData) -> Result<(), ScriptError>;
    
    /// 更新脚本
    fn update(&mut self, frame: FrameData) -> Result<(), ScriptError>;
    
    /// 关闭脚本
    fn shutdown(&mut self) -> Result<(), ScriptError>;
}

// ============================================================================
// ScriptManager - 脚本管理器
// ============================================================================

/// 脚本管理器 - 管理所有脚本的生命周期
pub struct ScriptManager {
    hosts: Vec<Box<dyn ScriptHost>>,
}

impl ScriptManager {
    pub fn new() -> Self {
        Self { hosts: Vec::new() }
    }
    
    /// 添加脚本主机
    pub fn add_host(&mut self, host: Box<dyn ScriptHost>) {
        self.hosts.push(host);
    }
    
    /// 初始化所有脚本
    pub fn init_all(&mut self, frame: FrameData) -> Result<(), ScriptError> {
        for host in &mut self.hosts {
            host.init(frame)?;
        }
        Ok(())
    }
    
    /// 更新所有脚本
    pub fn update_all(&mut self, frame: FrameData) -> Result<(), ScriptError> {
        let mut errors = Vec::new();
        for host in &mut self.hosts {
            if let Err(e) = host.update(frame) {
                errors.push(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ScriptError::Runtime(format!("{:?}", errors)))
        }
    }
    
    /// 关闭所有脚本
    pub fn shutdown_all(&mut self) -> Result<(), ScriptError> {
        for host in &mut self.hosts {
            host.shutdown()?;
        }
        Ok(())
    }
    
    /// 检查是否有任何脚本已加载
    pub fn is_any_loaded(&self) -> bool {
        self.hosts.iter().any(|h| h.is_loaded())
    }
}

impl Default for ScriptManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 配置类型
// ============================================================================

/// 脚本初始化配置
#[derive(Debug, Clone)]
pub struct ScriptInitConfig {
    /// 脚本路径
    pub script_path: PathBuf,
    /// 是否启用调试
    pub debug: bool,
}

impl ScriptInitConfig {
    pub fn new(script_path: PathBuf) -> Self {
        Self { script_path, debug: false }
    }
    
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

pub mod wasm_host;
pub use wasm_host::{WasmScriptHost, create_wasm_host};
