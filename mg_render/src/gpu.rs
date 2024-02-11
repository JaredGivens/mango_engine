use wgpu;
pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
}
impl Gpu {
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        let default_backend = wgpu::util::backend_bits_from_env()
            .unwrap_or(wgpu::Backends::PRIMARY | wgpu::Backends::GL);
        #[cfg(not(target_arch = "wasm32"))]
        let default_backend = wgpu::Backends::PRIMARY;

        let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::util::power_preference_from_env()
                    .unwrap_or(wgpu::PowerPreference::HighPerformance),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .or_else(|| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    instance
                        .enumerate_adapters(wgpu::Backends::all())
                        .find(|adapter| {
                            // Check if this adapter supports our surface
                            adapter.is_surface_supported(&surface)
                        })
                }

                #[cfg(target_arch = "wasm32")]
                panic!("no adapter")
            })
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: adapter.features() & wgpu::Features::default(),
                    limits: adapter_limits(&adapter),
                },
                None,
            )
            .await
            .expect("Request device");
        Self {
            device,
            queue,
            instance,
            adapter,
        }
    }
}
