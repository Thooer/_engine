//! 游戏脚本 - 演示多实体控制
//!
//! 导出 update(dt, frame_count, radius, height, speed, input_mask, entity_id_0, ..., entity_id_9)
//! 使用 raw WASM 导出
//! input_mask: 按键状态位掩码 (bit 0-3 对应数字键 1-4)
//! entity_id_N: 引擎传入的实体 ID（用于 set_transform）
//!
//! 功能：
//! - 相机轨道控制（4种模式切换）
//! - 卫星物体环绕运动
//! - 颜色随时间变化
//! - 随机生成物体（超出边界自动删除）
//!
//! 注意：现在使用 set_transform 命令直接修改 ECS 组件，不再依赖引擎查询

/// 将 i32 位表示转换为 f32
#[inline]
fn i32_to_f32_bits(i: i32) -> f32 {
    f32::from_le_bytes(i.to_le_bytes())
}

/// 将 f32 转换为 i32 位表示
#[inline]
fn f32_to_i32_bits(f: f32) -> i32 {
    i32::from_le_bytes(f.to_le_bytes())
}

/// 检查按键是否按下
#[inline]
fn is_digit_pressed(mask: u8, digit: u8) -> bool {
    if digit >= 1 && digit <= 4 {
        mask & (1 << (digit - 1)) != 0
    } else {
        false
    }
}

// ============================================================================
// 相机状态
// ============================================================================

static mut CAMERA_POS: [f32; 3] = [0.0, 5.0, 10.0];
static mut CURRENT_ANGLE: f32 = 0.0;
/// 当前相机模式 (0=orbit, 1=orbital, 2=figure8, 3=spiral)
static mut CAMERA_MODE: u8 = 0;

/// 相机实体 ID（从引擎传入）
static mut CAMERA_ENTITY_ID: i32 = 0;

// ============================================================================
// 动态物体管理状态
// ============================================================================

/// 动态物体列表（位置X, 位置Y, 位置Z, 速度Y, 存活时间）
static mut DYNAMIC_OBJECTS: [(f32, f32, f32, f32, f32); 32] = [(0.0, 0.0, 0.0, 0.0, -1.0); 32];
static mut DYNAMIC_OBJECT_COUNT: u8 = 0;
/// 生成计时器（单位：秒）
static mut SPAWN_TIMER: f32 = 0.0;
/// 边界范围（地面网格范围）
const BOUNDARY: f32 = 5.0;
/// 物体下落速度
const FALL_SPEED: f32 = 2.0;
/// 生成间隔（秒）
const SPAWN_INTERVAL: f32 = 1.5;

// ============================================================================
// 卫星物体状态 - 控制额外的环绕物体
// ============================================================================

static mut SATELLITE_COUNT: u8 = 3;
static mut SATELLITE_ANGLES: [f32; 5] = [0.0, 1.256, 2.512, 3.768, 4.712]; // 0, 72, 144, 216, 288 度
static mut SATELLITE_RADIUS: f32 = 3.0;
static mut SATELLITE_COLORS: [[f32; 3]; 5] = [
    [1.0, 0.3, 0.3], // 红
    [0.3, 1.0, 0.3], // 绿
    [0.3, 0.3, 1.0], // 蓝
    [1.0, 1.0, 0.3], // 黄
    [1.0, 0.3, 1.0], // 紫
];
/// 卫星实体 ID 列表（从引擎传入）
static mut SATELLITE_ENTITY_IDS: [i32; 5] = [0; 5];

// ============================================================================
// 外部函数声明（由引擎提供）
// ============================================================================

extern "C" {
    /// 设置实体变换
    fn set_transform(entity_bits: i32, x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32);
}

// ============================================================================
// 主更新函数
// ============================================================================

/// 主更新函数
/// 参数: dt, frame_count, radius, height, speed, input_mask, entity_id_0, ..., entity_id_9
#[no_mangle]
pub extern "C" fn update(
    dt: i32,
    _frame_count: i32,
    radius: i32,
    height: i32,
    speed: i32,
    input_mask: i32,
    entity_id_0: i32,
    entity_id_1: i32,
    entity_id_2: i32,
    entity_id_3: i32,
    entity_id_4: i32,
    entity_id_5: i32,
    entity_id_6: i32,
    entity_id_7: i32,
    entity_id_8: i32,
    entity_id_9: i32,
) {
    let dt_sec = i32_to_f32_bits(dt);
    let r = i32_to_f32_bits(radius);
    let h = i32_to_f32_bits(height);
    let spd = i32_to_f32_bits(speed);
    let mask = input_mask as u8;

    unsafe {
        // 保存实体 ID（由脚本自己解释 - 这里假设 entity_id_0 是相机，entity_id_1-3 是卫星）
        CAMERA_ENTITY_ID = entity_id_0;
        SATELLITE_ENTITY_IDS[0] = entity_id_1;
        SATELLITE_ENTITY_IDS[1] = entity_id_2;
        SATELLITE_ENTITY_IDS[2] = entity_id_3;

        // 检查数字键 1-4 切换相机模式
        for digit in 1..=4 {
            if is_digit_pressed(mask, digit) {
                let new_mode = digit - 1;
                if new_mode != CAMERA_MODE {
                    CAMERA_MODE = new_mode;
                    CURRENT_ANGLE = 0.0;
                }
            }
        }

        // 更新相机位置
        match CAMERA_MODE {
            0 => update_orbit_impl(dt_sec, r, h, spd),
            1 => update_orbital_impl(dt_sec, r, h, spd),
            2 => update_figure8_impl(dt_sec, r, h, spd),
            3 => update_spiral_impl(dt_sec, r, h, spd),
            _ => update_orbit_impl(dt_sec, r, h, spd),
        }

        // 直接通过 set_transform 修改相机 ECS 组件
        if CAMERA_ENTITY_ID != 0 {
            set_transform(
                CAMERA_ENTITY_ID,
                CAMERA_POS[0], CAMERA_POS[1], CAMERA_POS[2],
                1.0, 1.0, 1.0,
            );
        }

        // 更新卫星物体
        update_satellites(dt_sec, spd * 2.0);

        // 直接通过 set_transform 修改卫星 ECS 组件
        for i in 0..SATELLITE_COUNT as usize {
            let entity_id = SATELLITE_ENTITY_IDS[i];
            if entity_id != 0 {
                let x = SATELLITE_ANGLES[i].cos() * SATELLITE_RADIUS;
                let z = SATELLITE_ANGLES[i].sin() * SATELLITE_RADIUS;
                set_transform(
                    entity_id,
                    x, 0.5, z,
                    1.0, 1.0, 1.0,
                );
                // 添加日志确认调用
                let _ = x.sin(); // 防止编译器优化
            }
        }

        // 更新动态物体：下落 + 边界检测 + 删除
        update_dynamic_objects(dt_sec);
    }
}

/// 更新动态物体：物理下落 + 边界检测
#[inline]
fn update_dynamic_objects(dt: f32) {
    unsafe {
        // 更新所有活跃物体
        let mut i = 0;
        while i < DYNAMIC_OBJECT_COUNT as usize {
            let (x, y, z, vy, life) = DYNAMIC_OBJECTS[i];

            // 跳过已删除的物体（life < 0）
            if life < 0.0 {
                i += 1;
                continue;
            }

            // 物理下落
            let new_y = y + vy * dt;
            let new_life = life - dt;

            // 检查是否超出边界（X 或 Z 超出范围，或 Y 太低）
            let out_of_bounds = x.abs() > BOUNDARY || z.abs() > BOUNDARY || new_y < -2.0;

            if out_of_bounds {
                // 标记为已删除
                DYNAMIC_OBJECTS[i] = (x, y, z, vy, -1.0);
            } else {
                // 更新位置和生命周期
                DYNAMIC_OBJECTS[i] = (x, new_y, z, vy, new_life);
            }

            i += 1;
        }

        // 清理已删除的物体（compact）
        let mut write_idx = 0;
        for read_idx in 0..DYNAMIC_OBJECT_COUNT as usize {
            let (_, _, _, _, life) = DYNAMIC_OBJECTS[read_idx];
            if life >= 0.0 {
                if write_idx != read_idx {
                    DYNAMIC_OBJECTS[write_idx] = DYNAMIC_OBJECTS[read_idx];
                }
                write_idx += 1;
            }
        }
        DYNAMIC_OBJECT_COUNT = write_idx as u8;

        // 生成新物体
        SPAWN_TIMER += dt;
        if SPAWN_TIMER >= SPAWN_INTERVAL {
            SPAWN_TIMER = 0.0;
            spawn_random_object();
        }
    }
}

/// 生成随机物体
#[inline]
fn spawn_random_object() {
    unsafe {
        if DYNAMIC_OBJECT_COUNT >= 32 {
            return;
        }

        // 随机位置（在边界内）
        let x = (random_f32() - 0.5) * BOUNDARY * 1.5;
        let z = (random_f32() - 0.5) * BOUNDARY * 1.5;
        let y = 8.0 + random_f32() * 4.0; // 高度 8-12
        let vy = -FALL_SPEED; // 下落速度

        let idx = DYNAMIC_OBJECT_COUNT as usize;
        DYNAMIC_OBJECTS[idx] = (x, y, z, vy, 10.0); // 10秒生命周期
        DYNAMIC_OBJECT_COUNT += 1;
    }
}

/// 简单伪随机数生成器（线性同余）
static mut RANDOM_STATE: u32 = 12345;

#[inline]
fn random_f32() -> f32 {
    unsafe {
        // 线性同余生成器
        RANDOM_STATE = RANDOM_STATE.wrapping_mul(1103515245).wrapping_add(12345);
        // 转换为 0..1
        (RANDOM_STATE >> 16) as f32 / 65536.0
    }
}

/// 更新卫星物体
#[inline]
fn update_satellites(dt: f32, speed: f32) {
    unsafe {
        let count = SATELLITE_COUNT as usize;
        for i in 0..count {
            SATELLITE_ANGLES[i] += speed * dt;
            const TWO_PI: f32 = 6.28318530718;
            while SATELLITE_ANGLES[i] >= TWO_PI {
                SATELLITE_ANGLES[i] -= TWO_PI;
            }

            // 颜色随时间变化
            let color_offset = (SATELLITE_ANGLES[i] * 0.5).sin() * 0.3;
            SATELLITE_COLORS[i][0] = (0.5 + color_offset).clamp(0.2, 1.0);
            SATELLITE_COLORS[i][1] = (0.5 - color_offset).clamp(0.2, 1.0);
            SATELLITE_COLORS[i][2] = 0.8;
        }
    }
}

/// 获取卫星位置（用于扩展）
/// 返回: x, z 位置和颜色索引
#[no_mangle]
pub extern "C" fn get_satellite_x(index: i32) -> i32 {
    unsafe {
        if index >= 0 && index < SATELLITE_COUNT as i32 {
            let x = SATELLITE_ANGLES[index as usize].cos() * SATELLITE_RADIUS;
            f32_to_i32_bits(x)
        } else {
            f32_to_i32_bits(0.0)
        }
    }
}

#[no_mangle]
pub extern "C" fn get_satellite_z(index: i32) -> i32 {
    unsafe {
        if index >= 0 && index < SATELLITE_COUNT as i32 {
            let z = SATELLITE_ANGLES[index as usize].sin() * SATELLITE_RADIUS;
            f32_to_i32_bits(z)
        } else {
            f32_to_i32_bits(0.0)
        }
    }
}

#[no_mangle]
pub extern "C" fn get_satellite_color_r(index: i32) -> i32 {
    unsafe {
        if index >= 0 && index < SATELLITE_COUNT as i32 {
            f32_to_i32_bits(SATELLITE_COLORS[index as usize][0])
        } else {
            f32_to_i32_bits(1.0)
        }
    }
}

#[no_mangle]
pub extern "C" fn get_satellite_color_g(index: i32) -> i32 {
    unsafe {
        if index >= 0 && index < SATELLITE_COUNT as i32 {
            f32_to_i32_bits(SATELLITE_COLORS[index as usize][1])
        } else {
            f32_to_i32_bits(1.0)
        }
    }
}

#[no_mangle]
pub extern "C" fn get_satellite_color_b(index: i32) -> i32 {
    unsafe {
        if index >= 0 && index < SATELLITE_COUNT as i32 {
            f32_to_i32_bits(SATELLITE_COLORS[index as usize][2])
        } else {
            f32_to_i32_bits(1.0)
        }
    }
}

/// 获取卫星数量
#[no_mangle]
pub extern "C" fn get_satellite_count() -> i32 {
    unsafe { SATELLITE_COUNT as i32 }
}

// ============================================================================
// 相机轨道模式实现
// ============================================================================

#[inline]
fn update_orbit_impl(dt: f32, r: f32, h: f32, spd: f32) {
    unsafe {
        CURRENT_ANGLE += spd * dt;
        const TWO_PI: f32 = 6.28318530718;
        while CURRENT_ANGLE >= TWO_PI {
            CURRENT_ANGLE -= TWO_PI;
        }
        let x = CURRENT_ANGLE.cos() * r;
        let z = CURRENT_ANGLE.sin() * r;
        CAMERA_POS[0] = x;
        CAMERA_POS[1] = h;
        CAMERA_POS[2] = z;
    }
}

#[inline]
fn update_orbital_impl(dt: f32, r: f32, h_base: f32, spd: f32) {
    unsafe {
        CURRENT_ANGLE += spd * dt;
        const TWO_PI: f32 = 6.28318530718;
        while CURRENT_ANGLE >= TWO_PI {
            CURRENT_ANGLE -= TWO_PI;
        }
        let x = CURRENT_ANGLE.cos() * r;
        let z = CURRENT_ANGLE.sin() * r;
        let y = h_base + (CURRENT_ANGLE * 0.5).sin() * 2.0;
        CAMERA_POS[0] = x;
        CAMERA_POS[1] = y;
        CAMERA_POS[2] = z;
    }
}

#[inline]
fn update_figure8_impl(dt: f32, r: f32, h: f32, spd: f32) {
    unsafe {
        CURRENT_ANGLE += spd * dt;
        const TWO_PI: f32 = 6.28318530718;
        while CURRENT_ANGLE >= TWO_PI {
            CURRENT_ANGLE -= TWO_PI;
        }
        let t = CURRENT_ANGLE;
        let x = r * t.sin();
        let z = r * t.sin() * t.cos();
        let y = h + (t * 2.0).sin() * 1.5;
        CAMERA_POS[0] = x;
        CAMERA_POS[1] = y;
        CAMERA_POS[2] = z;
    }
}

#[inline]
fn update_spiral_impl(dt: f32, r: f32, h_base: f32, spd: f32) {
    static mut SPIRAL_PHASE: f32 = 0.0;
    unsafe {
        CURRENT_ANGLE += spd * dt;
        SPIRAL_PHASE += dt * 0.5;
        const TWO_PI: f32 = 6.28318530718;
        while CURRENT_ANGLE >= TWO_PI {
            CURRENT_ANGLE -= TWO_PI;
        }
        let current_radius = r * (1.0 - (SPIRAL_PHASE * 0.1).min(0.8));
        let x = CURRENT_ANGLE.cos() * current_radius;
        let z = CURRENT_ANGLE.sin() * current_radius;
        let y = h_base + SPIRAL_PHASE * 2.0;
        CAMERA_POS[0] = x;
        CAMERA_POS[1] = y.clamp(1.0, 20.0);
        CAMERA_POS[2] = z;
    }
}

// ============================================================================
// 兼容旧接口
// ============================================================================

#[no_mangle]
pub extern "C" fn update_orbital(dt: i32, f: i32, r: i32, h: i32, s: i32) {
    unsafe { CAMERA_MODE = 1u8; }
    update(dt, f, r, h, s, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
}

#[no_mangle]
pub extern "C" fn update_figure8(dt: i32, f: i32, r: i32, h: i32, s: i32) {
    unsafe { CAMERA_MODE = 2u8; }
    update(dt, f, r, h, s, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
}

#[no_mangle]
pub extern "C" fn update_spiral(dt: i32, f: i32, r: i32, h: i32, s: i32) {
    unsafe { CAMERA_MODE = 3u8; }
    update(dt, f, r, h, s, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
}

// ============================================================================
// 相机位置查询接口
// ============================================================================

#[no_mangle]
pub extern "C" fn get_camera_x() -> i32 {
    unsafe { f32_to_i32_bits(CAMERA_POS[0]) }
}

#[no_mangle]
pub extern "C" fn get_camera_y() -> i32 {
    unsafe { f32_to_i32_bits(CAMERA_POS[1]) }
}

#[no_mangle]
pub extern "C" fn get_camera_z() -> i32 {
    unsafe { f32_to_i32_bits(CAMERA_POS[2]) }
}

// ============================================================================
// 生命周期接口
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        DYNAMIC_OBJECT_COUNT = 0;
        SPAWN_TIMER = 0.0;
    }
}

#[no_mangle]
pub extern "C" fn shutdown() {
    // 清理
}

// ============================================================================
// 动态物体管理接口 - 供引擎调用
// ============================================================================

/// 请求创建实体（返回 1 表示请求）
#[no_mangle]
pub extern "C" fn request_spawn() -> i32 {
    unsafe {
        if DYNAMIC_OBJECT_COUNT > 0 {
            1
        } else {
            0
        }
    }
}

/// 获取要创建的实体类型（0=sphere, 1=cube）
#[no_mangle]
pub extern "C" fn get_spawn_type() -> i32 {
    // 目前只支持一种类型，返回 0
    0
}

/// 获取创建实体的 X 位置
#[no_mangle]
pub extern "C" fn get_spawn_x() -> i32 {
    unsafe {
        if DYNAMIC_OBJECT_COUNT > 0 {
            f32_to_i32_bits(DYNAMIC_OBJECTS[0].0)
        } else {
            0
        }
    }
}

/// 获取创建实体的 Y 位置
#[no_mangle]
pub extern "C" fn get_spawn_y() -> i32 {
    unsafe {
        if DYNAMIC_OBJECT_COUNT > 0 {
            f32_to_i32_bits(DYNAMIC_OBJECTS[0].1)
        } else {
            0
        }
    }
}

/// 获取创建实体的 Z 位置
#[no_mangle]
pub extern "C" fn get_spawn_z() -> i32 {
    unsafe {
        if DYNAMIC_OBJECT_COUNT > 0 {
            f32_to_i32_bits(DYNAMIC_OBJECTS[0].2)
        } else {
            0
        }
    }
}

/// 获取创建实体的大小
#[no_mangle]
pub extern "C" fn get_spawn_scale() -> i32 {
    f32_to_i32_bits(0.5)
}

/// 请求删除实体（返回要删除的实体 ID，0 表示无请求）
/// 目前简化实现：每帧处理一个超界物体
static mut NEXT_DESPAWN_INDEX: u8 = 0;

#[no_mangle]
pub extern "C" fn request_despawn() -> i32 {
    // 目前简化实现，返回 0，由引擎处理
    // 实际项目中需要更完善的实体 ID 管理
    0
}
