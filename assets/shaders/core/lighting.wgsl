// =========================================================
// ToyEngine Core Shader Library
// Lighting Functions
// =========================================================

// 基础 Lambert 漫反射
fn calculate_lambert(normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    return max(dot(normal, light_dir), 0.0);
}

// Blinn-Phong 高光
fn calculate_specular(
    normal: vec3<f32>, 
    view_dir: vec3<f32>, 
    light_dir: vec3<f32>, 
    shininess: f32
) -> f32 {
    let half_dir = normalize(view_dir + light_dir);
    let spec_angle = max(dot(normal, half_dir), 0.0);
    return pow(spec_angle, shininess);
}

// 简单的方向光计算
fn directional_light(
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    light_color: vec3<f32>,
    diffuse_color: vec3<f32>,
    specular_strength: f32,
    shininess: f32
) -> vec3<f32> {
    let diff = calculate_lambert(normal, light_dir);
    let spec = calculate_specular(normal, view_dir, light_dir, shininess);
    
    let ambient = 0.1 * diffuse_color;
    let diffuse = diff * light_color * diffuse_color;
    let specular = spec * light_color * specular_strength;
    
    return ambient + diffuse + specular;
}
