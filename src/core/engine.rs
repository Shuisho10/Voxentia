use winit::window::Window;

use crate::vulkan::{context::VulkanContext, pipeline::TestPipeline, swapchain::SurfaceSwapchain};


pub struct VoxelEngine{
    pub window: Window,
    pub vkcontext: VulkanContext,
    pub swapchain: SurfaceSwapchain,
    pub pipeline: TestPipeline
}
