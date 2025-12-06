use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator};
use gpu_allocator::MemoryLocation;
use std::sync::{Arc, Mutex};

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub allocation: Allocation,
    pub size: u64,
}

impl Buffer {
    pub fn new(
        context: &crate::vulkan::context::VulkanContext,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
        name: &str,
    ) -> Result<Self, vk::Result> {
        let device = &context.device;

        unsafe {
            let buffer_info = vk::BufferCreateInfo::default()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let buffer = device.create_buffer(&buffer_info, None)?;

            let mem_reqs = device.get_buffer_memory_requirements(buffer);

            let mut allocator = context.allocator.lock().unwrap();
            
            let allocation = allocator.allocate(&AllocationCreateDesc {
                name,
                requirements: mem_reqs,
                location,
                linear: true,
                allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
            }).map_err(|_| vk::Result::ERROR_INITIALIZATION_FAILED)?;

            device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;

            Ok(Self { buffer, allocation, size })
        }
    }
}
