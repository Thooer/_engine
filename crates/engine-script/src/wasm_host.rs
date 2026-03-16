//! WASM 脚本主机实现
//!
//! 桥接 Wasmer 和 Script Trait，支持脚本访问 ECS World
//! 采用简化实现，复用已有的 wasmer 调用模式

use std::path::Path;

use wasmer::{Module, Instance, Store, Value, Memory, Function};
use glam::Vec3;

use crate::{ScriptError, ScriptHost, ScriptContext};

/// 设置变换命令
#[derive(Clone, Debug)]
pub struct SetTransformCommand {
    pub entity_bits: i32,
    pub position: Vec3,
    pub scale: Vec3,
}

/// 全局命令缓冲区（用于 WASM 脚本传递命令给引擎）
static PENDING_COMMANDS: std::sync::OnceLock<std::sync::Mutex<Vec<SetTransformCommand>>> = std::sync::OnceLock::new();

fn get_commands() -> &'static std::sync::Mutex<Vec<SetTransformCommand>> {
    PENDING_COMMANDS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// 导出给 WASM 的 set_transform 函数
/// 参数: entity_bits, x, y, z, sx, sy, sz
#[no_mangle]
pub extern "C" fn set_transform(
    entity_bits: i32,
    x: f32,
    y: f32,
    z: f32,
    sx: f32,
    sy: f32,
    sz: f32,
) {
    let cmd = SetTransformCommand {
        entity_bits,
        position: Vec3::new(x, y, z),
        scale: Vec3::new(sx, sy, sz),
    };
    if let Ok(mut cmds) = get_commands().lock() {
        cmds.push(cmd);
    }
    tracing::debug!("WASM set_transform called: entity={}, pos=({},{},{}), scale=({},{},{})",
        entity_bits, x, y, z, sx, sy, sz);
}

/// 清除待处理的命令（每次 update 前调用）
pub fn clear_pending_commands() {
    if let Ok(mut cmds) = get_commands().lock() {
        cmds.clear();
    }
}

/// 获取待处理的命令
pub fn take_pending_commands() -> Vec<SetTransformCommand> {
    if let Ok(mut cmds) = get_commands().lock() {
        std::mem::take(&mut cmds)
    } else {
        Vec::new()
    }
}

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

        // 创建导入对象，提供 env.set_transform 函数
        let import_object = wasmer::imports! {
            "env" => {
                "set_transform" => Function::new_typed(&mut store, set_transform_wrapper),
            }
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

        // WASM 脚本约定: entity_id_0=相机, entity_id_1..3=卫星1..3
        let world = ctx.world_mut();

        // 查询相机实体（名称 "Camera" 或带 Camera3D）
        let mut camera_query = world.query::<(bevy_ecs::prelude::Entity, &bevy_ecs::prelude::Name)>();
        let camera_id = camera_query.iter(world)
            .find(|(_, name)| name.as_str() == "Camera")
            .map(|(e, _)| e.to_bits())
            .unwrap_or(0);

        // 查询卫星实体（名称以 "Satellite" 开头）
        let mut name_query = world.query::<(bevy_ecs::prelude::Entity, &bevy_ecs::prelude::Name)>();
        let mut satellite_ids: Vec<u64> = Vec::new();
        for (entity, name) in name_query.iter(world) {
            let name_str = name.as_str();
            if name_str.starts_with("Satellite") {
                satellite_ids.push(entity.to_bits());
            }
        }

        // 按脚本约定顺序: [camera, sat1, sat2, sat3, 0..0]
        let mut entity_ids: Vec<u64> = vec![camera_id];
        entity_ids.extend(satellite_ids.iter().take(9).copied());
        while entity_ids.len() < 10 {
            entity_ids.push(0);
        }

        tracing::info!("Entity IDs: camera={}, satellites={:?}", camera_id, &entity_ids[1..4]);

        // 尝试调用 update 函数（兼容模式）
        if let Ok(update_func) = instance.exports.get_function("update") {
            // 参数: dt, frame_count, radius, height, speed, input_mask, entity_id_0, ...
            let dt_bits = f32_to_i32(frame.delta_time);
            let frame_bits = frame.frame_count as i32;
            let radius_bits = f32_to_i32(10.0);
            let height_bits = f32_to_i32(5.0);
            let speed_bits = f32_to_i32(1.0);
            let input_mask = frame.input_mask;

            let mut args: Vec<Value> = vec![
                Value::I32(dt_bits),
                Value::I32(frame_bits),
                Value::I32(radius_bits),
                Value::I32(height_bits),
                Value::I32(speed_bits),
                Value::I32(input_mask as i32),
            ];

            for i in 0..10 {
                let id = entity_ids.get(i).copied().unwrap_or(0);
                let id_i32 = (id as u32) as i32;
                args.push(Value::I32(id_i32));
            }

            // 清除之前的命令
            clear_pending_commands();

            let _ = update_func.call(store, &args);

            // 执行 WASM 生成的命令
            let commands = take_pending_commands();
            for cmd in commands {
                // 从 entity_bits 恢复 Entity - i32 -> u32 -> u64 (只取低32位)
                let entity = bevy_ecs::entity::Entity::from_bits((cmd.entity_bits as u32) as u64);
                tracing::info!("set_transform called: entity_bits={} (i32), converted to u64={}, pos=({},{},{}), scale=({},{},{})", 
                    cmd.entity_bits, 
                    (cmd.entity_bits as u32) as u64,
                    cmd.position.x, cmd.position.y, cmd.position.z,
                    cmd.scale.x, cmd.scale.y, cmd.scale.z);
                // 使用 entity_mut 获取可变引用
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    // 更新 Transform 组件
                    if let Some(mut transform) = entity_mut.get_mut::<engine_core::ecs::Transform>() {
                        transform.translation = cmd.position;
                        transform.scale = cmd.scale;
                    }
                    // 如果实体有 Camera3D 组件，也更新其 position
                    if let Some(mut camera) = entity_mut.get_mut::<engine_core::ecs::Camera3D>() {
                        camera.position = cmd.position;
                    }
                    tracing::debug!("Applied set_transform: entity={}, pos=({},{},{})",
                        cmd.entity_bits, cmd.position.x, cmd.position.y, cmd.position.z);
                } else {
                    tracing::warn!("set_transform: entity not found: {}", cmd.entity_bits);
                }
            }
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

/// WASM set_transform 的包装器（用于导入对象）
/// 参数: i32, f32, f32, f32, f32, f32, f32
fn set_transform_wrapper(
    entity_bits: i32,
    x: f32,
    y: f32,
    z: f32,
    sx: f32,
    sy: f32,
    sz: f32,
) {
    // 直接调用全局函数
    set_transform(entity_bits, x, y, z, sx, sy, sz);
}

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
