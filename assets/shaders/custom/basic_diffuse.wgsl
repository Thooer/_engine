// =========================================================
// Custom Shader: Basic Diffuse
// 演示如何使用 core 库
// =========================================================

#include "core/input.wgsl"
#include "core/lighting.wgsl"

// Group 0: 全局 (CameraUniform defined in core::input)
// @group(0) @binding(0) var<uniform> camera: CameraUniform; // 已经在 core/input.wgsl 中定义

// Group 2: 材质 (自定义)
// @group(2) @binding(0) var base_texture: texture_2d<f32>; // Removed - not used

struct MaterialUniform {
    color_mod: vec4<f32>,
}
@group(2) @binding(0) var<uniform> material_uniform: MaterialUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    
    // Apply Model Matrix
    let world_pos = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_pos.xyz;
    
    // Transform Normal (Inverse Transpose for correct non-uniform scaling, but for now simple rotation/uniform scale is fine)
    // Assuming uniform scale for now:
    let world_normal = (model_matrix * vec4<f32>(model.normal, 0.0)).xyz;
    out.world_normal = normalize(world_normal);

    out.uv = model.uv;
    out.clip_position = camera.view_proj * world_pos;
    
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
