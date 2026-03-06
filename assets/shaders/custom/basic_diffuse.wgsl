// =========================================================
// Custom Shader: Basic Diffuse
// 演示如何使用 core 库
// =========================================================

#include "core/input.wgsl"
#include "core/lighting.wgsl"

// Group 0: 全局 (CameraUniform defined in core::input)
// @group(0) @binding(0) var<uniform> camera: CameraUniform; // 已经在 core/input.wgsl 中定义

// Group 1: 材质 (自定义)
@group(1) @binding(0) var base_texture: texture_2d<f32>;
// @group(1) @binding(1) var base_sampler: sampler; // 移除，改用全局 sampler_linear

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // 简化的 MVP，实际可能需要 Model Matrix
    // 这里假设 model matrix 是 identity 或者包含在 uniform 里（仅作示例）
    out.world_position = model.position; 
    out.world_normal = model.normal;
    out.uv = model.uv;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 采样基础色 (使用全局 sampler_linear)
    let base_color = textureSample(base_texture, sampler_linear, in.uv);
    
    // 简单的光照 (硬编码光源方向)
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normalize(in.world_normal), light_dir), 0.0);
    
    // 环境光
    let ambient = 0.1;
    
    let final_color = base_color.rgb * (diffuse + ambient);
    
    return vec4<f32>(final_color, base_color.a);
}
