// background.wgsl

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    position: vec3<f32>,
    aspect_ratio: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) view_dir: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // 生成全屏三角形 (Full-screen triangle)
    // 0: (-1, -1), 1: (3, -1), 2: (-1, 3)
    let x = f32((in_vertex_index << 1u) & 2u);
    let y = f32(in_vertex_index & 2u);
    let uv = vec2<f32>(x, y);
    
    out.uv = uv;
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 1.0, 1.0); // Z=1.0 for farthest depth

    // 计算视线方向 (从相机指向屏幕像素)
    // 将屏幕坐标 (NDC) 转换回世界空间方向
    let ndc_pos = vec4<f32>(out.position.xy, 1.0, 1.0);
    let view_pos = camera.inv_proj * ndc_pos;
    let world_pos = camera.inv_view * vec4<f32>(view_pos.xyz, 0.0); // 0.0 w component for direction
    out.view_dir = world_pos.xyz;
    
    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) custom: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let dir = normalize(in.view_dir);
    
    // 简单的程序化天空 (基于高度/方向)
    
    // 1. 基础渐变 (从地平线到天顶)
    let t = 0.5 * (dir.y + 1.0); // [-1, 1] -> [0, 1]
    
    // 颜色定义
    // let bottom_color = vec3<f32>(0.1, 0.1, 0.2); // 深蓝灰 (地平线以下)
    // let horizon_color = vec3<f32>(0.6, 0.7, 0.9); // 淡蓝 (地平线)
    // let top_color = vec3<f32>(0.05, 0.2, 0.5);    // 深蓝 (天顶)
    let bottom_color = vec3<f32>(0.2); // 深蓝灰 (地平线以下)
    let horizon_color = bottom_color;//vec3<f32>(0.8, 0.8, 0.8); // 淡蓝 (地平线)
    let top_color = bottom_color;//vec3<f32>(0.8, 0.8, 0.8);    // 深蓝 (天顶)
    
    var color = vec3<f32>(0.0);
    
    if (dir.y > 0.0) {
        // 天空
        color = mix(horizon_color, top_color, pow(dir.y, 0.5));
    } else {
        // 地面/深渊
        color = mix(horizon_color, bottom_color, -dir.y);
    }
    
    // 2. 简单的太阳 (方向光) - 假设太阳在上方某个位置
    let sun_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let sun_dot = max(dot(dir, sun_dir), 0.0);
    
    // 太阳核心
    let sun_disk = step(0.999, sun_dot);
    let sun_color = vec3<f32>(1.0, 0.9, 0.6) * 2.0;
    
    // 太阳光晕
    let sun_glow = pow(sun_dot, 20.0) * 0.2;
    
    color += sun_color * sun_disk + vec3<f32>(1.0, 0.8, 0.5) * sun_glow;
    
    // 3. 简单的星星 (Grid-based 3D Stars) - 稳定且密度随高度平滑增加
    if (dir.y > 0.0) {
        // 使用离散网格生成星星，确保位置锁定
        let scale = 150.0;
        let grid_pos = dir * scale;
        let cell_id = floor(grid_pos);
        let cell_local = fract(grid_pos) - 0.5; // 单元内坐标 [-0.5, 0.5]
        
        // 3D 哈希函数 (替换不稳定的 sin 噪声)
        var p3 = fract(cell_id * vec3<f32>(.1031, .1030, .0973));
        p3 += dot(p3, p3.yzx + 33.33);
        let hash = fract((p3.xxy + p3.yxx) * p3.zyx); // 返回 vec3 随机数
        
        // 随机偏移星星在网格内的位置 (-0.4 到 0.4)，避免网格感
        let star_offset = (hash.xyz - 0.5) * 0.8;
        let dist = length(cell_local - star_offset);
        
        // 基础概率，越高星星越稀疏
        let base_threshold = 0.98;
        
        // 随高度调整密度：dir.y 越高，density_curve 越大，阈值越低，星星越多
        let density_curve = pow(dir.y, 1.5); 
        let effective_threshold = base_threshold - density_curve * 0.03; // 动态调整概率
        
        // 哈希值的第一个分量决定该网格是否有星星
        if (hash.x > effective_threshold) {
            // 距离越近越亮 (模拟圆形星星)
            // 0.25 是星星半径
            let radius = 0.25 + hash.y * 0.2; // 随机大小
            let star_shape = smoothstep(radius, 0.0, dist);
            
            // 强度随高度渐变，避免地平线突兀
            let fade = smoothstep(0.0, 0.2, dir.y);
            
            // 增加一点基于时间的闪烁 (可选)
            // let twinkle = 0.5 + 0.5 * sin(camera.time * 2.0 + hash.z * 10.0);
            
            // 极其明亮的内核
            let brightness = star_shape * star_shape * fade;
            color += vec3<f32>(brightness);
        }
    }

    // 4. Screen Space Darkening (Top is darker)
    // color *= 0.3 + 0.7 * in.uv.y;

    var out: FragmentOutput;
    out.color = vec4<f32>(color, 1.0);
    out.normal = vec4<f32>(0.0); // Background has no normal
    out.custom = vec4<f32>(0.0);
    return out;
}
