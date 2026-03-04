use crate::renderer::{
    backends_from_env, surface_size_is_zero, wgpu_debug_on, DefaultSurfaceContextNew,
    SurfaceContext, SurfaceContextNew, SurfaceSize,
};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

impl SurfaceContextNew for DefaultSurfaceContextNew {
    async fn surface_context_new<'w, W>(
        window: &'w W,
        size: SurfaceSize,
    ) -> Result<SurfaceContext<'w>, wgpu::RequestDeviceError>
    where
        W: HasWindowHandle + HasDisplayHandle + Sync + ?Sized,
    {
        let backends_list = backends_from_env().unwrap_or_else(|| {
            vec![
                wgpu::Backends::VULKAN,
                wgpu::Backends::DX12,
                wgpu::Backends::GL,
            ]
        });

        let mut last_device_err: Option<wgpu::RequestDeviceError> = None;

        for backends in backends_list {
            if wgpu_debug_on() {
                println!("wgpu: try backends: {backends:?}");
            }
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends,
                ..Default::default()
            });
            let surface = instance
                .create_surface(window)
                .expect("wgpu surface create failed");

            let adapter = match instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    if wgpu_debug_on() {
                        println!("wgpu: request_adapter failed: {e:?}");
                    }
                    continue;
                }
            };

            let info = adapter.get_info();
            if wgpu_debug_on() {
                println!("wgpu: adapter: {:?} {}", info.backend, info.name);
            }

            let (device, queue) = match adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("toyengine device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        experimental_features: wgpu::ExperimentalFeatures::default(),
                        memory_hints: wgpu::MemoryHints::Performance,
                        trace: wgpu::Trace::default(),
                    },
                )
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    if wgpu_debug_on() {
                        println!("wgpu: request_device failed: {e:?}");
                    }
                    last_device_err = Some(e);
                    continue;
                }
            };

            let caps = surface.get_capabilities(&adapter);
            let format = caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(caps.formats[0]);

            let present_mode = caps
                .present_modes
                .iter()
                .copied()
                .find(|m| *m == wgpu::PresentMode::Fifo)
                .unwrap_or(caps.present_modes[0]);

            let alpha_mode = caps.alpha_modes[0];

            let mut config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode,
                alpha_mode,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            if !surface_size_is_zero(size) {
                surface.configure(&device, &config);
            }

            config.width = size.width;
            config.height = size.height;

            return Ok(SurfaceContext {
                size,
                instance,
                surface,
                adapter,
                device,
                queue,
                config,
            });
        }

        if let Some(e) = last_device_err {
            Err(e)
        } else {
            panic!("wgpu adapter not found");
        }
    }
}

