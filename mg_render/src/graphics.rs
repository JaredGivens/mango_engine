use mg_core::*;
use std::cell::Ref;
use std::cell::RefCell;
use std::rc::Rc;
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;
#[cfg(target_arch = "wasm32")]
fn window_size(_: Ref<Window>) -> (u32, u32) {
    (
        web_sys::window()
            .unwrap()
            .inner_width()
            .unwrap()
            .as_f64()
            .unwrap() as u32,
        web_sys::window()
            .unwrap()
            .inner_height()
            .unwrap()
            .as_f64()
            .unwrap() as u32,
    )
}

#[cfg(target_arch = "wasm32")]
fn web_add_resize(window: Rc<RefCell<Window>>) -> Closure<dyn FnMut(web_sys::Event)> {
    use crate::winit::dpi::LogicalSize;
    use wasm_bindgen::closure::Closure;
    let (width, height) = window_size(window.borrow());
    window
        .borrow_mut()
        .set_inner_size(LogicalSize::new(width, height));

    let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
        let (width, height) = window_size(window.borrow());
        // this triggers window event resize
        window
            .borrow_mut()
            .set_inner_size(LogicalSize::new(width, height));
    }) as Box<dyn FnMut(_)>);
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref());
    closure
}

#[cfg(not(target_arch = "wasm32"))]
fn window_size(window: Ref<Window>) -> (u32, u32) {
    let physical_size = window.inner_size();
    (physical_size.width, physical_size.height)
}

#[cfg(target_arch = "wasm32")]
fn adapter_limits(adapter: &wgpu::Adapter) -> wgpu::Limits {
    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
}

#[cfg(not(target_arch = "wasm32"))]
fn adapter_limits(_: &wgpu::Adapter) -> wgpu::Limits {
    wgpu::Limits::default()
}

pub struct Graphics {
    pub window: Rc<RefCell<Window>>,
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

impl Graphics {
    pub async fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Graphics {
        #[cfg(target_arch = "wasm32")]
        let canvas_element = {
            console_log::init().expect("Initialize logger");
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.get_element_by_id("iced_canvas"))
                .and_then(|element| element.dyn_into::<HtmlCanvasElement>().ok())
                .expect("Get canvas element")
        };

        let window = {
            let builder = winit::window::WindowBuilder::new().with_title("mg");
            #[cfg(target_arch = "wasm32")]
            let builder = builder.with_canvas(Some(canvas_element));
            Rc::new(RefCell::new(builder.build(event_loop).unwrap()))
        };
        #[cfg(target_arch = "wasm32")]
        let default_backend = wgpu::util::backend_bits_from_env()
            .unwrap_or(wgpu::Backends::PRIMARY | wgpu::Backends::GL);
        #[cfg(not(target_arch = "wasm32"))]
        let default_backend = wgpu::Backends::PRIMARY;

        let (width, height) = window_size(window.borrow());
        #[cfg(target_arch = "wasm32")]
        let on_resize = web_add_resize(window.clone());

        let backend = wgpu::util::backend_bits_from_env().unwrap_or(default_backend);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(&*window.borrow()) }.unwrap();

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
            width,
            height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &config);
        Graphics {
            surface,
            device,
            queue,
            config,
            width,
            height,
            tx_format_surface,
            window,
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
