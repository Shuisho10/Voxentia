use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::core::engine::VoxelEngine;
use crate::vulkan::context::VulkanContext;

mod core;
mod vulkan;

#[derive(Default)]
struct App {
    pub engine: Option<VoxelEngine>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("Window not created");
        let vkcontext = VulkanContext::new(&window).expect("Vulkan context not initializated");
        let engine = VoxelEngine { window, vkcontext };
        self.engine = Some(engine);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.

                // Draw.

                self.engine.as_ref().unwrap().window.request_redraw();
            }
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
