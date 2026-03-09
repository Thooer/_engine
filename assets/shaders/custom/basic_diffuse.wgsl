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
    let base_color = vec4<f32>(1.0);//textureSample(base_texture, sampler_linear, in.uv);
    let normal = normalize(in.world_normal);
    let view_dir = normalize(camera.view_pos - in.world_position);
    
    var total_light = vec3<f32>(0.0);

    // 环境光 (简单硬编码)
    let ambient = vec3<f32>(0.05) * base_color.rgb;
    total_light += ambient;
    
    // 遍历点光源
    for (var i: u32 = 0u; i < lights.point_light_info.x; i++) {
        let light = lights.point_lights[i];
        total_light += calculate_point_light(
            light,
            normal,
            view_dir,
            in.world_position,
            base_color.rgb,
            32.0 // Shininess (Hardcoded for now)
        );
    }
    
    // Apply material uniform
    let mod_color = material_uniform.color_mod.rgb; 
    total_light += mod_color;
    
    return vec4<f32>(total_light, base_color.a);
}
