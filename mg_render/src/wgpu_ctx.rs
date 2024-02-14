use mg_core::*;
use winit::window::Window;
use std::cell::Ref;
use std::cell::RefCell;
use std::rc::Rc;

use std::borrow::Borrow;
use crate::mango_window::MangoWindow;

#[cfg(target_arch = "wasm32")]
fn adapter_limits(adapter: &wgpu::Adapter) -> wgpu::Limits {
    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
}

#[cfg(not(target_arch = "wasm32"))]
fn adapter_limits(_: &wgpu::Adapter) -> wgpu::Limits {
    wgpu::Limits::default()
}

pub struct WgpuContext {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub tx_format_surface: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
    #[cfg(target_arch = "wasm32")]
    on_resize: Closure<dyn FnMut(web_sys::Event)>,
}

impl WgpuContext {
    pub async fn new(mg_window: &MangoWindow) -> WgpuContext {
        #[cfg(target_arch = "wasm32")]
        let default_backend = wgpu::util::backend_bits_from_env()
            .unwrap_or(wgpu::Backends::PRIMARY | wgpu::Backends::GL);
        #[cfg(not(target_arch = "wasm32"))]
        let default_backend = wgpu::Backends::PRIMARY;

        #[cfg(target_arch = "wasm32")]
        let on_resize = web_add_resize(window.clone());

        let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(&*mg_window.winit.borrow()) }.unwrap();

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

        let capabilities = surface.get_capabilities(&adapter);
        let tx_format_surface = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);

        let present_mode = wgpu::PresentMode::AutoVsync;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: tx_format_surface,
            width: mg_window.width,
            height: mg_window.height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &config);
        WgpuContext {
            surface,
            device,
            queue,
            config,
            width: mg_window.width,
            height: mg_window.height,
            tx_format_surface,
            #[cfg(target_arch = "wasm32")]
            on_resize,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        //let sf = self.window.borrow().scale_factor();

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }
}
