#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use crate::graphics::gpu_pipeline_generator::PipelineGenerator;

    #[test]
    fn test_pipeline_generation() {
        // 使用 pollster 运行 async 代码
        pollster::block_on(async {
            // 1. 初始化 wgpu (Headless)
            // 注意：CI 环境可能没有显卡，需要处理这种情况
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
            
            // 请求 Adapter
            let adapter_result = instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true, // 使用软件渲染以保证测试通过
                compatible_surface: None,
            }).await;

            let adapter = match adapter_result {
                Ok(a) => a,
                Err(e) => {
                    println!("Skipping test: No wgpu adapter found: {:?}", e);
                    return;
                }
            };
            // If adapter is Option<Adapter> inside Result (unlikely for Result return), we'd need another unwrap. 
            // But usually request_adapter returning Result implies Success(Adapter) or Error.
            // Wait, previous wgpu returned Option<Adapter>. 
            // If it returns Result, maybe it's Result<Option<Adapter>, ...>? No, that's weird.
            // Let's assume it returns Result<Adapter, ...>.

            // 请求 Device
            let (device, _queue) = adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    memory_hints: wgpu::MemoryHints::default(),
                    experimental_features: wgpu::ExperimentalFeatures::default(),
                    trace: wgpu::Trace::default(),
                },
            ).await.expect("Failed to create device");

            // 2. 设置测试文件系统
            let root = PathBuf::from("test_assets_pipeline");
            let shaders_dir = root.join("shaders");
            let custom_dir = shaders_dir.join("custom");
            let core_dir = shaders_dir.join("core");
            
            fs::create_dir_all(&custom_dir).unwrap();
            fs::create_dir_all(&core_dir).unwrap();

            // 创建基础 input.wgsl
            let input_wgsl = "
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};
";
            fs::write(core_dir.join("input.wgsl"), input_wgsl).unwrap();

            // 创建测试 shader
            let shader_code = "
#include \"core/input.wgsl\"

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
";
            fs::write(custom_dir.join("test_shader.wgsl"), shader_code).unwrap();

            // 3. 运行 PipelineGenerator
            let generator = PipelineGenerator::new(&root);
            let result = generator.scan_and_generate_pipelines(
                &device, 
                wgpu::TextureFormat::Rgba8UnormSrgb,
                Some(wgpu::TextureFormat::Depth32Float)
            );

            // 4. 验证结果
            assert!(result.is_ok(), "Pipeline generation failed: {:?}", result.err());
            let pipelines = result.unwrap();
            
            assert!(pipelines.contains_key("custom/test_shader.wgsl"), "Pipeline not found in map");
            
            println!("Successfully generated {} pipelines", pipelines.len());

            // 5. 清理
            fs::remove_dir_all(&root).unwrap();
        });
    }
}
