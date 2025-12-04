use winit::window::Window;

use crate::vulkan::context::VulkanContext;


pub struct VoxelEngine{
    pub window: Window,
    pub vkcontext: VulkanContext,
}
