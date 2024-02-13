use winit::window::Window;

pub struct Window {
    pub window: Rc<RefCell<Window>>,
}

#[cfg(not(target_arch = "wasm32"))]
fn window_size(window: Ref<Window>) -> (u32, u32) {
    let physical_size = window.inner_size();
    (physical_size.width, physical_size.height)
}

impl Window {
    pub fn new() -> Window {
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
    }
}