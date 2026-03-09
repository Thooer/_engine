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

// 点光源计算
fn calculate_point_light(
    light: PointLight,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    world_pos: vec3<f32>,
    diffuse_color: vec3<f32>,
    shininess: f32
) -> vec3<f32> {
    let light_dir_unnormalized = light.position - world_pos;
    let distance = length(light_dir_unnormalized);
    
    if (distance > light.range) {
        return vec3<f32>(0.0);
    }
    
    let light_dir = normalize(light_dir_unnormalized);
    
    // 简单的线性衰减
    let attenuation = max(1.0 - distance / light.range, 0.0);
    // 或者使用更物理的衰减: 1.0 / (distance * distance)
    
    let diff = calculate_lambert(normal, light_dir);
    let spec = calculate_specular(normal, view_dir, light_dir, shininess);
    
    let diffuse = diff * light.color * diffuse_color * light.intensity;
    // 假设高光强度为 1.0
    let specular = spec * light.color * light.intensity; 
    
    return (diffuse + specular) * attenuation;
}
