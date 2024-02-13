#[macro_use]
mod assets;
mod editor;
mod time;
mod ui;

use assets::Assets;
use bitflags::bitflags;
use editor::Editor;
use mg_core::*;
use mg_render::{mango_window::MangoWindow, graphics::Graphics, instance, scene::Scene};
use winit::{
    event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use std::cell::RefCell;
use std::borrow::Borrow;

bitflags! {
    #[derive(Default)]
    pub struct Buttons: u32{
        const Esc = 1 << 0;
        const Forward = 1 << 1;
        const Backward = 1 << 2;
        const Left = 1 << 3;
        const Right = 1 << 4;
        const Up = 1 << 5;
        const Down = 1 << 6;
        const KeyA = 1 << 7;
        const KeyB = 1 << 8;
        const LMB = 1 << 9;
        const RMB = 1 << 10;
    }
}

enum State {
    Home,
    Editor(Editor),
    Race,
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

struct App {
    window: MangoWindow,
    graphics: Graphics,
    renderer: mg_render::Renderer,
    scene: Scene,
    state: State,
    buttons: Buttons,
    mouse_delta: Vec2f,
    egui: ui::Egui,
    assets: Assets,
}

impl App {
    async fn new(event_loop: &winit::event_loop::EventLoop<()>) -> App {
        let title = String::from("Mango Engine");
        let window = MangoWindow::new(title, event_loop);
        let graphics = Graphics::new(&window).await;

        // let graphics = Graphics::new(event_loop).await;
        let egui = ui::Egui::new(&graphics, &window);
        let renderer = mg_render::Renderer::new(&graphics);
        let mut scene = Scene::new(&graphics);
        let assets = Assets::new(&graphics);
        for mesh in assets.meshes.iter() {
            scene.instantiate_mesh(
                &graphics,
                mesh.clone(),
                instance::Params {
                    amt: 1,
                    bin: None,
                    buffer: None,
                    range: None,
                },
            );
        }
        App {
            window,
            graphics,
            renderer,
            scene,
            egui,
            assets,
            state: State::Home,
            buttons: Buttons::default(),
            mouse_delta: Vec2f::new(0.0, 0.0),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width <= 0 && height <= 0 {
            return;
        }

        self.graphics.resize(width, height);
        self.renderer.resize(&self.graphics);
        self.scene.resize(&self.graphics);
    }

    pub fn update(&mut self) {
        self.update_ui();
        self.window.winit.borrow().request_redraw();
    }

    fn update_ui(&mut self) {
        match &mut self.state {
            State::Home => {
                self.egui.update(&self.graphics, &self.window, |ctx| {
                    egui::Window::new("")
                        .title_bar(false)
                        .movable(false)
                        .collapsible(false)
                        .resizable(false)
                        .show(ctx, |ui| {
                            if ui.button("editor").clicked() {
                                self.state = State::Editor(Editor::new());
                            }
                        });
                });
            }
            State::Editor(editor) => {}
            // State::Editor(editor) => {
            //     let window_ref = RefCell::new(&self.window);

            //     editor.update(&mut self.scene, &self.buttons, self.mouse_delta);
            //     self.mouse_delta = Vec2f::new(0.0, 0.0);
            //     self.egui.update(&self.graphics, &self.window, |ctx| {
            //         egui::Window::new("")
            //             .title_bar(false)
            //             .movable(false)
            //             .collapsible(false)
            //             .resizable(false)
            //             .show(ctx, |ui| match editor.state {
            //                 editor::State::Menu => {
            //                     if ui.button("map").clicked() {
            //                         let mut window_ref_mut = window_ref.borrow_mut();
            //                         editor.start_edit(&mut *window_ref_mut);
            //                     }
            //                 }
            //                 editor::State::Edit => {
            //                     if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            //                         editor.start_menu(&mut self.window);
            //                     }
            //                 }
            //                 editor::State::Test => {}
            //                 editor::State::Validation => {}
            //             });
            //     });
            // }
            State::Race => {}
        }
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .graphics
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.renderer.geometry_pass(&mut encoder, &self.scene);
        self.renderer.ray_pass(&mut encoder, &mut self.scene);
        self.graphics.queue.submit(Some(encoder.finish()));
        let mut encoder = self
            .graphics
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let output = self.graphics.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.egui.update_buffers(&mut encoder, &self.graphics);
        {
            let mut comp_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("comp pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.renderer.compose_pass(&mut comp_pass);
            self.egui.render(&mut comp_pass);
        }

        self.graphics.queue.submit(Some(encoder.finish()));
        output.present();

        self.scene.camera.update(&self.graphics);
        Ok(())
    }
    pub fn on_window_event(&mut self, event: &winit::event::WindowEvent) {
        self.egui.on_window_event(event);
    }
    pub fn set_key_button(&mut self, input: &winit::event::KeyboardInput) {
        let key_code = match input.virtual_keycode {
            Some(key_code) => key_code,
            None => return,
        };

        let input_flag = match key_code {
            VirtualKeyCode::Escape => Buttons::Esc,
            VirtualKeyCode::W => Buttons::Forward,
            VirtualKeyCode::A => Buttons::Left,
            VirtualKeyCode::S => Buttons::Backward,
            VirtualKeyCode::D => Buttons::Right,
            VirtualKeyCode::Space => Buttons::Up,
            VirtualKeyCode::LShift => Buttons::Down,
            _ => return,
        };
        self.buttons
            .set(input_flag, input.state == ElementState::Pressed);
    }
    pub fn set_mouse_button(&mut self, button: &MouseButton, state: &ElementState) {
        let input_flag = match button {
            MouseButton::Left => Buttons::LMB,
            MouseButton::Right => Buttons::RMB,
            _ => return,
        };
        self.buttons
            .set(input_flag, state == &ElementState::Pressed);
    }
    pub fn update_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.mouse_delta = Vec2f::new(dx, dy);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    let event_loop = winit::event_loop::EventLoop::new();
    let mut app = App::new(&event_loop).await;
    event_loop.run(move |event, _, control_flow| match event {
        Event::DeviceEvent {
                event: winit::event::DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => {
                app.update_mouse_delta(delta.0 as f32, delta.1 as f32);
            }
        Event::WindowEvent { event, .. } => {
            match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    app.set_mouse_button(&button, &state);
                }
                WindowEvent::Resized(physical_size) => {
                    app.resize(physical_size.width, physical_size.height);
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    app.set_key_button(&input);
                }
                _ => {}
            }
            app.on_window_event(&event);
        }
        Event::MainEventsCleared => {
            app.update();
        }
        Event::RedrawRequested(_) => {
            match app.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        _ => {}
    })
}
