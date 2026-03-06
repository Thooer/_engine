// =========================================================
// ToyEngine Core Shader Library
// Shadow Functions
// =========================================================

// 简单的 PCF 阴影采样 (Percentage-Closer Filtering)
fn fetch_shadow(
    shadow_map: texture_depth_2d,
    shadow_sampler: sampler_comparison,
    shadow_coord: vec4<f32>,
    bias: f32
) -> f32 {
    if (shadow_coord.w <= 0.0) {
        return 1.0;
    }
    
    // 透视除法
    let proj_coords = shadow_coord.xyz / shadow_coord.w;
    
    // 变换到 [0, 1] 纹理空间
    // wgpu 的 NDC 是 [-1, 1] for x,y 和 [0, 1] for z
    let current_depth = proj_coords.z - bias;
    
    // 如果超出视锥体范围，不算阴影
    if (proj_coords.z > 1.0 || proj_coords.x < -1.0 || proj_coords.x > 1.0 || proj_coords.y < -1.0 || proj_coords.y > 1.0) {
        return 1.0;
    }
    
    // 简单的 3x3 PCF
    var shadow: f32 = 0.0;
    let size = textureDimensions(shadow_map);
    let texel_size = vec2<f32>(1.0 / f32(size.x), 1.0 / f32(size.y));
    
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            // 注意：wgpu 的坐标系 y 轴向下吗？需要确认 NDC。
            // Vulkan/Metal/DX12: (0,0) top-left
            // Flip Y is handled in projection matrix usually.
            let uv = vec2<f32>(
                proj_coords.x * 0.5 + 0.5 + offset.x,
                -proj_coords.y * 0.5 + 0.5 + offset.y
            );
            
            shadow += textureSampleCompare(
                shadow_map, 
                shadow_sampler, 
                uv, 
                current_depth
            );
        }
    }
    
    return shadow / 9.0;
}
