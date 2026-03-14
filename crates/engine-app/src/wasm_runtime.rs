//! WASM 脚本运行时模块
//!
//! 提供 WASM 脚本加载和执行能力，用于实现运行时逻辑热更

use std::path::Path;
use wasmer::{Module, Instance, Store, imports, Value};
use glam::Vec3;

/// 将 f32 转换为 i32 位表示
fn f32_to_i32_bits(f: f32) -> i32 {
    i32::from_le_bytes(f.to_le_bytes())
}

/// 将 i32 位表示转换为 f32
fn i32_to_f32_bits(i: i32) -> f32 {
    f32::from_le_bytes(i.to_le_bytes())
}

/// 输入状态（简化版，用于 WASM）
#[derive(Default, Clone)]
pub struct InputState {
    /// 按下的键集合（使用 u8 位掩码，只支持数字键 1-4）
    pub pressed_mask: u8,
}

impl InputState {
    /// 检查指定数字键是否按下 (1-4 -> bit 0-3)
    pub fn is_digit_pressed(&self, digit: u8) -> bool {
        if digit >= 1 && digit <= 4 {
            self.pressed_mask & (1 << (digit - 1)) != 0
        } else {
            false
        }
    }
}

/// WASM 运行时
pub struct WasmRuntime {
    store: Store,
    instance: Option<Instance>,
}

impl WasmRuntime {
    /// 创建新的 WASM 运行时
    pub fn new() -> Result<Self, String> {
        let store = Store::default();
        Ok(Self {
            store,
            instance: None,
        })
    }

    /// 加载并实例化 WASM 模块
    pub fn load(&mut self, wasm_path: &Path) -> Result<(), String> {
        // 读取 WASM 文件
        let bytes = std::fs::read(wasm_path)
            .map_err(|e| format!("Failed to read WASM file: {}", e))?;

        // 编译模块
        let module = Module::new(&self.store, bytes)
            .map_err(|e| format!("Failed to compile WASM: {}", e))?;

        // 创建导入对象
        let import_object = imports! {
            "env" => {}
        };

        // 实例化
        let instance = Instance::new(&mut self.store, &module, &import_object)
            .map_err(|e| format!("Failed to instantiate WASM: {}", e))?;

        tracing::info!("Loaded WASM script: {:?}", wasm_path);

        self.instance = Some(instance);
        Ok(())
    }

    /// 调用指定名称的相机更新函数
    /// 参数新增 input_mask: 按键状态位掩码 (bit 0-3 对应数字键 1-4)
    pub fn call_camera_func(
        &mut self,
        func_name: &str,
        dt: f32,
        frame_count: u32,
        radius: f32,
        height: f32,
        speed: f32,
        input_mask: u8,
    ) -> Result<Vec3, String> {
        // 从 WASM 模块获取相机位置
        if let Some(ref instance) = self.instance {
            // 获取 update 函数并调用
            if let Ok(update_func) = instance.exports.get_function(func_name) {
                // 准备参数: dt, frame_count, radius, height, speed, input_mask (全部转为 i32 位表示)
                let dt_bits = f32_to_i32_bits(dt);
                let frame_bits = frame_count as i32;
                let radius_bits = f32_to_i32_bits(radius);
                let height_bits = f32_to_i32_bits(height);
                let speed_bits = f32_to_i32_bits(speed);

                // 调用 WASM update 函数（新增 input_mask 参数）
                let _ = update_func.call(&mut self.store, &[
                    Value::I32(dt_bits),
                    Value::I32(frame_bits),
                    Value::I32(radius_bits),
                    Value::I32(height_bits),
                    Value::I32(speed_bits),
                    Value::I32(input_mask as i32),
                ]);
            } else {
                tracing::warn!("WASM function '{}' not found, using fallback", func_name);
            }

            // 获取返回值 - 调用 get_camera_x/y/z
            if let (Ok(x_func), Ok(y_func), Ok(z_func)) = (
                instance.exports.get_function("get_camera_x"),
                instance.exports.get_function("get_camera_y"),
                instance.exports.get_function("get_camera_z"),
            ) {
                let x_result = x_func.call(&mut self.store, &[]);
                let y_result = y_func.call(&mut self.store, &[]);
                let z_result = z_func.call(&mut self.store, &[]);

                let x_bits = x_result.ok().and_then(|v| v.first().and_then(|x| x.i32()));
                let y_bits = y_result.ok().and_then(|v| v.first().and_then(|y| y.i32()));
                let z_bits = z_result.ok().and_then(|v| v.first().and_then(|z| z.i32()));

                if let (Some(x), Some(y), Some(z)) = (x_bits, y_bits, z_bits) {
                    tracing::debug!("WASM {} returned: ({}, {}, {})", func_name, x, y, z);
                    return Ok(Vec3::new(
                        i32_to_f32_bits(x),
                        i32_to_f32_bits(y),
                        i32_to_f32_bits(z),
                    ));
                }
            }
        }

        // fallback: 轨道运动
        let theta = frame_count as f32 * speed;

        Ok(Vec3::new(
            theta.cos() * radius,
            height,
            theta.sin() * radius,
        ))
    }

    /// 检查是否已加载
    pub fn is_loaded(&self) -> bool {
        self.instance.is_some()
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create WasmRuntime")
    }
}