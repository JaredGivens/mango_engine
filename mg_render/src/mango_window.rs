use winit::window::{WindowBuilder, Window};

pub struct MangoWindow {
    pub width: u32,
    pub height: u32,
    pub winit: Window,
}

impl MangoWindow {
    pub fn new(title: String, event_loop: &winit::event_loop::EventLoop<()>) -> MangoWindow {
        let window_builder = WindowBuilder::new()
            .with_theme(Some(winit::window::Theme::Light))
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

        let window = window_builder.build(&event_loop).unwrap();
        let size = window.inner_size();

        MangoWindow {
            width: size.width,
            height: size.height,
            winit: window,
        }
    }
}