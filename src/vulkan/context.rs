use ash::vk;
use winit::window::Window;

#[allow(unused)]
pub struct VulkanContext {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub device: ash::Device,
}


impl VulkanContext {
    pub fn new(_window: &Window) -> Result<Self, vk::Result>{
        Err(vk::Result::SUCCESS)
    }
}
