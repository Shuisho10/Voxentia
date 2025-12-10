use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use crate::core::engine::VoxelEngine;

mod core;
mod vulkan;

#[derive(Default)]
struct App {
    pub engine: Option<VoxelEngine>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let engine = VoxelEngine::new(event_loop).expect("Voxel engine initialization failed");
        self.engine = Some(engine);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.engine.as_mut().unwrap().draw_frame().expect("Unable to draw frame");
                self.engine.as_ref().unwrap().window.request_redraw();
            },
            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {return;}
                if let Some(engine) = self.engine.as_mut() {
                    engine.rebuild_swapchain(physical_size.width, physical_size.height).expect("Unable to recreate swapchain");
                }
                self.engine.as_ref().unwrap().window.request_redraw();
            },
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
