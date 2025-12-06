use winit::window::Window;

use crate::vulkan::{context::VulkanContext, swapchain::SurfaceSwapchain};


pub struct VoxelEngine{
    pub window: Window,
    pub vkcontext: VulkanContext,
    pub swapchain: SurfaceSwapchain,
}
