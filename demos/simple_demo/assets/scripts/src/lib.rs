//! 游戏脚本 - 演示多实体控制
//!
//! 导出 update(dt, frame_count, radius, height, speed, input_mask)
//! 使用 raw WASM 导出
//! input_mask: 按键状态位掩码 (bit 0-3 对应数字键 1-4)
//!
//! 功能：
//! - 相机轨道控制（4种模式切换）
//! - 卫星物体环绕运动
//! - 颜色随时间变化

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

// ============================================================================
// 主更新函数
// ============================================================================

/// 主更新函数
/// 参数: dt, frame_count, radius, height, speed, input_mask
#[no_mangle]
pub extern "C" fn update(
    dt: i32,
    _frame_count: i32,
    radius: i32,
    height: i32,
    speed: i32,
    input_mask: i32,
) {
    let dt_sec = i32_to_f32_bits(dt);
    let r = i32_to_f32_bits(radius);
    let h = i32_to_f32_bits(height);
    let spd = i32_to_f32_bits(speed);
    let mask = input_mask as u8;

    unsafe {
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

        // 更新卫星物体
        update_satellites(dt_sec, spd * 2.0);
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
    update(dt, f, r, h, s, 0);
}

#[no_mangle]
pub extern "C" fn update_figure8(dt: i32, f: i32, r: i32, h: i32, s: i32) {
    unsafe { CAMERA_MODE = 2u8; }
    update(dt, f, r, h, s, 0);
}

#[no_mangle]
pub extern "C" fn update_spiral(dt: i32, f: i32, r: i32, h: i32, s: i32) {
    unsafe { CAMERA_MODE = 3u8; }
    update(dt, f, r, h, s, 0);
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
    // 初始化
}

#[no_mangle]
pub extern "C" fn shutdown() {
    // 清理
}
