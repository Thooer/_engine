#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use crate::graphics::gpu_shader_loader::ShaderLoader;

    #[test]
    fn test_shader_include() {
        // 1. 设置测试环境
        let test_dir = PathBuf::from("test_assets/shaders");
        let core_dir = test_dir.join("core");
        let custom_dir = test_dir.join("custom");
        
        fs::create_dir_all(&core_dir).unwrap();
        fs::create_dir_all(&custom_dir).unwrap();

        // 2. 创建模拟文件
        let input_wgsl = "
struct VertexInput {
    @location(0) position: vec3<f32>,
};
";
        fs::write(core_dir.join("input.wgsl"), input_wgsl).unwrap();

        let main_wgsl = "
#include \"core/input.wgsl\"

@vertex
fn vs_main(in: VertexInput) -> vec4<f32> {
    return vec4<f32>(in.position, 1.0);
}
";
        fs::write(custom_dir.join("main.wgsl"), main_wgsl).unwrap();

        // 3. 测试加载器
        let loader = ShaderLoader::new(PathBuf::from("test_assets"));
        let result = loader.load_shader_source("custom/main.wgsl").unwrap();

        // 4. 验证结果
        assert!(result.contains("struct VertexInput"));
        assert!(result.contains("fn vs_main"));
        assert!(!result.contains("#include")); // 确保 include 指令被替换

        // 5. 清理
        fs::remove_dir_all("test_assets").unwrap();
    }
}
