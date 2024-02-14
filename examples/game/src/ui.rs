use mg_core::*;
use mg_render::mango_window::MangoWindow;
use mg_render::{wgpu_ctx::WgpuContext, texture::Texture};
use egui_wgpu::renderer::Renderer;
use egui_wgpu::renderer::ScreenDescriptor;
use std::cell::Cell;
use std::borrow::Borrow;

#[derive(Copy, Clone)]
enum State {
    Home,
    Editor,
}

pub struct Egui {
    pub renderer: Renderer,
    winit: egui_winit::State,
    egui: egui::Context,
    pub clipped_primitives: Vec<egui::ClippedPrimitive>,
    pub screen_descriptor: ScreenDescriptor,
    //render_pass: Option<wgpu::RenderPass<'_>>,
}

impl Egui {
    pub fn new(w_ctx: &WgpuContext, window: &MangoWindow) -> Egui {
        let egui = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "verdana".to_owned(),
            egui::FontData::from_static(include_bytes!("fonts/Verdana.ttf")),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "verdana".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("verdana".to_owned());

        // Tell egui to use these fonts:
        egui.set_fonts(fonts);
        let winit =
            egui_winit::State::new(egui.viewport_id(), &*window.winit.borrow(), None, None);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [w_ctx.width, w_ctx.height],
            pixels_per_point: egui_winit::pixels_per_point(&egui, &*window.winit.borrow()),
        };
        let renderer = Renderer::new(&w_ctx.device, w_ctx.tx_format_surface, None, 1);
        Egui {
            egui,
            winit,
            renderer,
            screen_descriptor,
            clipped_primitives: vec![],
            //render_pass: None
        }
    }
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.renderer.render(
            render_pass,
            &self.clipped_primitives,
            &self.screen_descriptor,
        )
    }

    pub fn on_window_event(&mut self, event: &winit::event::WindowEvent) {
        let _ = self.winit.on_window_event(&self.egui, &event);
    }

    pub fn update<F>(&mut self, w_ctx: &WgpuContext, window: &MangoWindow, closure: F)
    where
        F: FnOnce(&egui::Context),
    {
        let raw_input = self.winit.take_egui_input(&*window.winit.borrow());
        let full_output = self.egui.run(raw_input.clone(), closure);
        self.winit.handle_platform_output(
            &*window.winit.borrow(),
            &self.egui,
            full_output.platform_output,
        );
        self.clipped_primitives = self
            .egui
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        self.screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [w_ctx.width, w_ctx.height],
            pixels_per_point: egui_winit::pixels_per_point(&self.egui, &*window.winit.borrow()),
        };
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&w_ctx.device, &w_ctx.queue, *id, image_delta);
        }
    }

    pub fn update_buffers(&mut self, encoder: &mut wgpu::CommandEncoder, w_ctx: &WgpuContext) {
        self.renderer.update_buffers(
            &w_ctx.device,
            &w_ctx.queue,
            encoder,
            &self.clipped_primitives,
            &self.screen_descriptor,
        );
    }
}
