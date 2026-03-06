// =========================================================
// ToyEngine Core Shader Library
// Input Definitions
// =========================================================

// 标准顶点输入 (与 Rust 端 Vertex 定义一致)
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

// 顶点着色器输出 / 片元着色器输入
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
};

// Group(0) - 全局 Uniform & 采样器
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec3<f32>,
};

// 绑定 0: 相机参数
@group(0) @binding(0) var<uniform> camera: CameraUniform;

// 绑定 1: 线性采样器 (Linear filtering, Clamp/Repeat depends on usage, assume Repeat for now)
@group(0) @binding(1) var sampler_linear: sampler;

// 绑定 2: 最近邻采样器 (Nearest filtering)
@group(0) @binding(2) var sampler_nearest: sampler;

// Group(3) - 实例数据
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};
