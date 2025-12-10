use ash::vk;
use gpu_allocator::MemoryLocation;
use nalgebra::Vector3;

use crate::vulkan::{buffer::Buffer, context::VulkanContext};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WorldData {
    raw: [u32;  32*32*32],
}

impl WorldData {
    pub fn new(context: &VulkanContext) -> Result<(Self, Buffer), vk::Result> {
        let mut raw = [0; 32*32*32];
        let center = Vector3::<f32>::new(16.0_f32, 16.0_f32, 16.0_f32);
        let radius = 10.0_f32;
        for i in 0..32 {
            for j in 0..32 {
                for k in 0..32 {
                    let d = Vector3::<f32>::new(i as f32, j as f32, k as f32) - center;
                    if d.magnitude() < radius {
                        let index = i + j * 32 + k * 1024 as usize;
                        raw[index] = 1;
                    }
                }
            }
        }
        let buffer = Buffer::device_local_with_data(context, vk::BufferUsageFlags::STORAGE_BUFFER, "World", std::slice::from_ref(&raw))?;
        Ok((Self { raw }, buffer))
    }
}
