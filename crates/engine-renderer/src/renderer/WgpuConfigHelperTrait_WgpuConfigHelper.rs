use super::{WgpuConfigHelper, WgpuConfigHelperTrait};

impl WgpuConfigHelperTrait for WgpuConfigHelper {
    fn wgpu_debug_on() -> bool {
        std::env::var_os("TOYENGINE_WGPU_DEBUG").is_some()
    }

    fn backends_from_env() -> Option<Vec<wgpu::Backends>> {
        let v = std::env::var("TOYENGINE_WGPU_BACKEND").ok()?;
        let v = v.trim().to_ascii_lowercase();
        let backends = match v.as_str() {
            "vulkan" | "vk" => vec![wgpu::Backends::VULKAN],
            "dx12" | "d3d12" => vec![wgpu::Backends::DX12],
            "gl" | "opengl" => vec![wgpu::Backends::GL],
            "primary" => vec![wgpu::Backends::PRIMARY],
            "all" => vec![wgpu::Backends::all()],
            _ => return None,
        };
        Some(backends)
    }
}
