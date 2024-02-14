use crate::Buttons;
use mg_core::*;
use mg_render::{wgpu_ctx::WgpuContext, mango_window::MangoWindow, scene::Scene};
use std::borrow::BorrowMut;

#[derive(PartialEq)]
pub enum State {
    Menu,
    Edit,
    Test,
    Validation,
}
pub struct Editor {
    pub state: State,
    yaw: f32,
    pitch: f32,
}
impl Editor {
    pub fn new() -> Editor {
        Editor {
            state: State::Menu,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn update(&mut self, scene: &mut Scene, buttons: &Buttons, mouse_delta: Vec2f) {
        if self.state == State::Edit {
            let sensitivity = 0.01;
            self.yaw = (mouse_delta.x * sensitivity + self.yaw) % TAU;
            self.pitch = -mouse_delta.y * sensitivity + self.pitch;

            static SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;
            self.pitch = self.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);

            let (yaw_sin, yaw_cos) = (self.yaw.sin(), self.yaw.cos());
            let forward = Vec3f::new(yaw_cos, 0.0, yaw_sin);
            let right = Vec3f::new(-yaw_sin, 0.0, yaw_cos);
            let speed = 1.0;
            if buttons.contains(Buttons::Forward) {
                scene.camera.eye += forward * speed;
            }
            if buttons.contains(Buttons::Backward) {
                scene.camera.eye -= forward * speed;
            }
            if buttons.contains(Buttons::Right) {
                scene.camera.eye += right * speed;
            }
            if buttons.contains(Buttons::Left) {
                scene.camera.eye -= right * speed;
            }
            if buttons.contains(Buttons::Up) {
                scene.camera.eye.y += speed;
            }
            if buttons.contains(Buttons::Down) {
                scene.camera.eye.y -= speed;
            }
            let (pitch_sin, pitch_cos) = (self.pitch.sin(), self.pitch.cos());
            scene.camera.target =
                scene.camera.eye + Vec3f::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin);
        }
    }

    pub fn start_edit(&mut self, window: &mut MangoWindow) {
        let res = window
            .winit
            .borrow_mut()
            .set_cursor_grab(winit::window::CursorGrabMode::Confined);
        res.or_else(|_e| {
            window
                .winit
                .borrow_mut()
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
        })
        .unwrap();
        window.winit.borrow_mut().set_cursor_visible(false);
        self.state = State::Edit;
    }

    pub fn start_menu(&mut self, window: &mut MangoWindow) {
        window
            .winit
            .borrow_mut()
            .set_cursor_grab(winit::window::CursorGrabMode::None)
            .unwrap();
        window.winit.borrow_mut().set_cursor_visible(true);
        self.state = State::Menu;
    }
}
