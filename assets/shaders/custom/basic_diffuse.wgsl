// =========================================================
// Custom Shader: Basic Diffuse
// 演示如何使用 core 库
// =========================================================

#include "core/input.wgsl"
#include "core/lighting.wgsl"

// Group 0: 全局 (CameraUniform defined in core::input)
// @group(0) @binding(0) var<uniform> camera: CameraUniform; // 已经在 core/input.wgsl 中定义

// Group 2: 材质 (自定义)
@group(2) @binding(0) var base_texture: texture_2d<f32>;
// @group(2) @binding(1) var base_sampler: sampler; // Removed, use global sampler

struct MaterialUniform {
    color_mod: vec4<f32>,
}
@group(2) @binding(1) var<uniform> material_uniform: MaterialUniform;

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
    // 采样基础色 (使用全局采样器)
    let base_color = textureSample(base_texture, sampler_linear, in.uv);
    
    // 简单的光照 (硬编码光源方向)
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normalize(in.world_normal), light_dir), 0.0);
    
    // 环境光
    let ambient = 0.1;
    
    // Apply material uniform (even if zero, just to use it)
    // If we add it, it does nothing if zero.
    let mod_color = material_uniform.color_mod.rgb; 
    
    let final_color = base_color.rgb * (diffuse + ambient) + mod_color;
    
    return vec4<f32>(final_color, base_color.a);
}
