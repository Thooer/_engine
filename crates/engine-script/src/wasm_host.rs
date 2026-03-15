//! WASM 脚本主机实现
//!
//! 简化的 WASM 运行时，复用现有 engine-app 的实现模式

use std::path::Path;

use crate::{FrameData, ScriptError, ScriptHost};

/// WASM 脚本主机 - 简化实现
/// 
/// 此实现复用了 engine-app 中现有的 WasmRuntime 模式，
/// 作为向新架构迁移的过渡
pub struct WasmScriptHost {
    name: String,
    loaded: bool,
}

impl WasmScriptHost {
    pub fn new(name: &str) -> Result<Self, ScriptError> {
        Ok(Self {
            name: name.to_string(),
            loaded: false,
        })
    }
    
    /// 设置为已加载状态（由外部 WasmRuntime 驱动）
    pub fn set_loaded(&mut self, loaded: bool) {
        self.loaded = loaded;
    }
}

impl ScriptHost for WasmScriptHost {
    fn load(&mut self, _path: &Path) -> Result<(), ScriptError> {
        // 实际加载由 engine-app 中的 WasmRuntime 处理
        self.loaded = true;
        tracing::info!("WASM script marked as loaded: {}", self.name);
        Ok(())
    }
    
    fn is_loaded(&self) -> bool {
        self.loaded
    }
    
    fn init(&mut self, _frame: FrameData) -> Result<(), ScriptError> {
        tracing::debug!("WASM script init (placeholder): {}", self.name);
        Ok(())
    }
    
    fn update(&mut self, _frame: FrameData) -> Result<(), ScriptError> {
        // 实际更新由 engine-app 中的 WasmRuntime 处理
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), ScriptError> {
        tracing::debug!("WASM script shutdown: {}", self.name);
        Ok(())
    }
}

/// 创建 WASM 脚本主机的便捷函数
pub fn create_wasm_host(name: &str) -> Result<Box<dyn ScriptHost>, ScriptError> {
    Ok(Box::new(WasmScriptHost::new(name)?))
}
