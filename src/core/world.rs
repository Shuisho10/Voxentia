use ash::vk;
use gpu_allocator::MemoryLocation;
use nalgebra::Vector3;

use crate::vulkan::{buffer::Buffer, context::VulkanContext};

#[repr(C)]
pub struct WorldData {
    raw: [[[bool; 32]; 32]; 32],
}

impl WorldData {
    pub fn new(context: &VulkanContext) -> Result<(Self, Buffer), vk::Result> {
        let buffer = Buffer::new(
            &context,
            std::mem::size_of::<[[[bool; 32]; 32]; 32]>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::GpuOnly,
            "World",
        )?;
        let mut raw = [[[false; 32]; 32]; 32];
        let center = Vector3::<f32>::new(16.0_f32, 16.0_f32, 16.0_f32);
        let radius = 10.0_f32;
        for i in 0..32 {
            for j in 0..32 {
                for k in 0..32 {
                    let d = Vector3::<f32>::new(i as f32, j as f32, k as f32) - center;
                    raw[i][j][k] = d.magnitude() < radius;
                }
            }
        }
        Ok((Self { raw }, buffer))
    }
}
