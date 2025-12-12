use std::collections::HashSet;

use log::*;
use nalgebra::Vector3;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use crate::core::engine::VoxelEngine;

mod core;
mod vulkan;

#[derive(Default)]
struct App {
    pub engine: Option<VoxelEngine>,
    pub input: InputState,
}

#[derive(Default)]
struct InputState {
    keys_held: HashSet<KeyCode>,
}

impl InputState {
    fn is_key_down(&self, code: KeyCode) -> bool {
        self.keys_held.contains(&code)
    }
}

impl App {
    pub fn handle_input(&mut self) {
        if let Some(engine) = self.engine.as_mut() {
            let speed = 20.0;
            let mut dir = Vector3::zeros();

            if self.input.is_key_down(KeyCode::KeyW) {
                dir.z += 1.0;
            }
            if self.input.is_key_down(KeyCode::KeyS) {
                dir.z -= 1.0;
            }
            if self.input.is_key_down(KeyCode::KeyA) {
                dir.x -= 1.0;
            }
            if self.input.is_key_down(KeyCode::KeyD) {
                dir.x += 1.0;
            }
            if self.input.is_key_down(KeyCode::Space) {
                dir.y += 1.0;
            }
            if self.input.is_key_down(KeyCode::ShiftLeft) {
                dir.y -= 1.0;
            }

            if dir.magnitude() > 0.0 {
                engine.camera.input_move(dir, speed);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let engine = VoxelEngine::new(event_loop).expect("Voxel engine initialization failed");
        self.engine = Some(engine);
    }
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(engine) = self.engine.as_mut() {
            if let DeviceEvent::MouseMotion { delta } = event {
                engine
                    .camera
                    .input_rotate(delta.0 as f32, delta.1 as f32, 0.002);
            }
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_input();
                self.engine
                    .as_mut()
                    .unwrap()
                    .draw_frame()
                    .expect("Unable to draw frame");
                self.engine.as_ref().unwrap().window.request_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {
                    return;
                }
                if let Some(engine) = self.engine.as_mut() {
                    engine
                        .rebuild_swapchain(physical_size.width, physical_size.height)
                        .expect("Unable to recreate swapchain");
                    engine
                        .camera
                        .update_aspect(physical_size.width, physical_size.height);
                }
                self.engine.as_ref().unwrap().window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    if key_event.state.is_pressed() {
                        self.input.keys_held.insert(code);
                    } else {
                        self.input.keys_held.remove(&code);
                    }
                }
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
