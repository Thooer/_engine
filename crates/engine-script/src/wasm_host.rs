//! WASM 脚本主机实现
//!
//! 桥接 Wasmer 和 Script Trait，支持脚本访问 ECS World
//! 采用简化实现，复用已有的 wasmer 调用模式

use std::path::Path;

use wasmer::{Module, Instance, Store, Value, Memory};
use glam::Vec3;

use crate::{FrameData, ScriptError, ScriptHost, ScriptContext};

/// WASM 脚本主机 - 桥接 Wasmer 和 Script Trait
///
/// 支持两种通信模式：
/// 1. 结构化模式（新版）：通过 init/update/shutdown 传递序列化上下文
/// 2. 兼容模式（旧版）：通过 get_camera_x/y/z 获取相机位置
pub struct WasmScriptHost {
    name: String,
    loaded: bool,
    store: Option<Store>,
    instance: Option<Instance>,
    memory: Option<Memory>,
    /// 相机位置缓存（兼容模式用）
    cached_camera_pos: Vec3,
}

impl WasmScriptHost {
    pub fn new(name: &str) -> Result<Self, ScriptError> {
        Ok(Self {
            name: name.to_string(),
            loaded: false,
            store: None,
            instance: None,
            memory: None,
            cached_camera_pos: Vec3::ZERO,
        })
    }

    /// 加载 WASM 模块
    pub fn load(&mut self, path: &Path) -> Result<(), ScriptError> {
        let bytes = std::fs::read(path)
            .map_err(|e| ScriptError::Load(format!("Failed to read WASM file: {}", e)))?;

        let mut store = Store::default();
        let module = Module::new(&store, bytes)
            .map_err(|e| ScriptError::Module(format!("Failed to compile WASM: {}", e)))?;

        let import_object = wasmer::imports! {
            "env" => {}
        };

        let instance = Instance::new(&mut store, &module, &import_object)
            .map_err(|e| ScriptError::Module(format!("Failed to instantiate WASM: {}", e)))?;

        let memory = instance.exports.get_memory("memory").ok().cloned();

        self.store = Some(store);
        self.instance = Some(instance);
        self.memory = memory;
        self.loaded = true;

        tracing::info!("WASM script loaded: {} from {:?}", self.name, path);
        Ok(())
    }

    /// 获取相机位置（用于引擎查询）
    pub fn get_camera_position(&self) -> Vec3 {
        self.cached_camera_pos
    }

    /// 从 WASM 获取卫星数量（脚本导出 get_satellite_count）
    pub fn get_satellite_count(&mut self) -> i32 {
        if let (Some(instance), Some(store)) = (self.instance.as_ref(), self.store.as_mut()) {
            if let Ok(f) = instance.exports.get_function("get_satellite_count") {
                if let Ok(res) = f.call(store, &[]) {
                    return res.first().and_then(|v| v.i32()).unwrap_or(0);
                }
            }
        }
        0
    }

    /// 从 WASM 获取卫星 X 位置（脚本导出 get_satellite_x）
    pub fn get_satellite_x(&mut self, index: i32) -> f32 {
        if let (Some(instance), Some(store)) = (self.instance.as_ref(), self.store.as_mut()) {
            if let Ok(f) = instance.exports.get_function("get_satellite_x") {
                if let Ok(res) = f.call(store, &[Value::I32(index)]) {
                    if let Some(bits) = res.first().and_then(|v| v.i32()) {
                        return i32_to_f32(bits);
                    }
                }
            }
        }
        0.0
    }

    /// 从 WASM 获取卫星 Z 位置（脚本导出 get_satellite_z）
    pub fn get_satellite_z(&mut self, index: i32) -> f32 {
        if let (Some(instance), Some(store)) = (self.instance.as_ref(), self.store.as_mut()) {
            if let Ok(f) = instance.exports.get_function("get_satellite_z") {
                if let Ok(res) = f.call(store, &[Value::I32(index)]) {
                    if let Some(bits) = res.first().and_then(|v| v.i32()) {
                        return i32_to_f32(bits);
                    }
                }
            }
        }
        0.0
    }

    /// 从 WASM 获取卫星颜色 R/G/B（脚本导出 get_satellite_color_r/g/b）
    pub fn get_satellite_color(&mut self, index: i32) -> (f32, f32, f32) {
        let r = self.get_satellite_color_component("get_satellite_color_r", index);
        let g = self.get_satellite_color_component("get_satellite_color_g", index);
        let b = self.get_satellite_color_component("get_satellite_color_b", index);
        (r, g, b)
    }

    fn get_satellite_color_component(&mut self, name: &str, index: i32) -> f32 {
        if let (Some(instance), Some(store)) = (self.instance.as_ref(), self.store.as_mut()) {
            if let Ok(f) = instance.exports.get_function(name) {
                if let Ok(res) = f.call(store, &[Value::I32(index)]) {
                    if let Some(bits) = res.first().and_then(|v| v.i32()) {
                        return i32_to_f32(bits);
                    }
                }
            }
        }
        1.0
    }
}

impl ScriptHost for WasmScriptHost {
    fn load(&mut self, path: &Path) -> Result<(), ScriptError> {
        WasmScriptHost::load(self, path)
    }

    fn is_loaded(&self) -> bool {
        self.loaded
    }

    fn init(&mut self, _ctx: &mut dyn ScriptContext) -> Result<(), ScriptError> {
        let instance = self.instance.as_ref()
            .ok_or_else(|| ScriptError::Runtime("WASM instance not loaded".to_string()))?;

        // 尝试调用 init 函数
        if let Ok(init_func) = instance.exports.get_function("init") {
            if let Some(ref mut store) = self.store {
                let _ = init_func.call(store, &[]);
            }
        }

        tracing::debug!("WASM script initialized: {}", self.name);
        Ok(())
    }

    fn update(&mut self, ctx: &mut dyn ScriptContext) -> Result<(), ScriptError> {
        if !self.loaded {
            return Ok(());
        }

        let instance = self.instance.as_ref()
            .ok_or_else(|| ScriptError::Runtime("WASM instance not loaded".to_string()))?;

        let store = self.store.as_mut()
            .ok_or_else(|| ScriptError::Runtime("WASM store not initialized".to_string()))?;

        let frame = ctx.frame_data();

        // 尝试调用 update 函数（兼容模式）
        if let Ok(update_func) = instance.exports.get_function("update") {
            // 参数: dt, frame_count, radius, height, speed, input_mask (bit 0-3 对应数字键 1-4)
            let dt_bits = f32_to_i32(frame.delta_time);
            let frame_bits = frame.frame_count as i32;
            let radius_bits = f32_to_i32(10.0);
            let height_bits = f32_to_i32(5.0);
            let speed_bits = f32_to_i32(1.0);
            let input_mask = frame.input_mask;

            let _ = update_func.call(store, &[
                Value::I32(dt_bits),
                Value::I32(frame_bits),
                Value::I32(radius_bits),
                Value::I32(height_bits),
                Value::I32(speed_bits),
                Value::I32(input_mask as i32),
            ]);
        }

        // 获取返回值 - 调用 get_camera_x/y/z
        if let (Ok(x_func), Ok(y_func), Ok(z_func)) = (
            instance.exports.get_function("get_camera_x"),
            instance.exports.get_function("get_camera_y"),
            instance.exports.get_function("get_camera_z"),
        ) {
            if let (Ok(x), Ok(y), Ok(z)) = (
                x_func.call(store, &[]),
                y_func.call(store, &[]),
                z_func.call(store, &[]),
            ) {
                // wasmer 的 i32() 返回的是 i32 值本身，不是位表示
                let x_val = x.first().and_then(|v| v.i32()).unwrap_or(0);
                let y_val = y.first().and_then(|v| v.i32()).unwrap_or(0);
                let z_val = z.first().and_then(|v| v.i32()).unwrap_or(0);

                // 将 i32 位表示转换回 f32
                self.cached_camera_pos = Vec3::new(
                    i32_to_f32(x_val),
                    i32_to_f32(y_val),
                    i32_to_f32(z_val),
                );
            }
        }

        Ok(())
    }

    fn shutdown(&mut self, _ctx: &mut dyn ScriptContext) -> Result<(), ScriptError> {
        let instance = self.instance.as_ref()
            .ok_or_else(|| ScriptError::Runtime("WASM instance not loaded".to_string()))?;

        // 尝试调用 shutdown 函数
        if let Ok(shutdown_func) = instance.exports.get_function("shutdown") {
            if let Some(ref mut store) = self.store {
                let _ = shutdown_func.call(store, &[]);
            }
        }

        tracing::debug!("WASM script shutdown: {}", self.name);
        Ok(())
    }
}

/// 创建 WASM 脚本主机的便捷函数
pub fn create_wasm_host(name: &str) -> Result<Box<dyn ScriptHost>, ScriptError> {
    Ok(Box::new(WasmScriptHost::new(name)?))
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 将 f32 转换为 i32 位表示
#[inline]
fn f32_to_i32(f: f32) -> i32 {
    i32::from_le_bytes(f.to_le_bytes())
}

/// 将 i32 位表示转换为 f32
#[inline]
fn i32_to_f32(i: i32) -> f32 {
    f32::from_le_bytes(i.to_le_bytes())
}
