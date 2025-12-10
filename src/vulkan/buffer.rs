use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc};

use crate::vulkan::context::VulkanContext;

#[allow(unused)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub allocation: Option<Allocation>,
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
            context.set_object_name(buffer, name)?;

            let mem_reqs = device.get_buffer_memory_requirements(buffer);

            let mut allocator = context
                .allocator
                .lock()
                .map_err(|_| vk::Result::NOT_READY)?;

            let allocation = allocator
                .allocate(&AllocationCreateDesc {
                    name,
                    requirements: mem_reqs,
                    location,
                    linear: true,
                    allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
                })
                .map_err(|_| vk::Result::ERROR_INITIALIZATION_FAILED)?;

            device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;

            Ok(Self {
                buffer,
                allocation: Some(allocation),
                size,
            })
        }
    }

    pub fn device_local_with_data<T: Copy>(
        context: &VulkanContext,
        usage: vk::BufferUsageFlags,
        name: &str,
        data: &[T],
    ) -> Result<Self, vk::Result> {
        let size = (std::mem::size_of::<T>() * data.len()) as u64;

        let mut staging = Self::new(
            context, 
            size, 
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
            &format!("Staging-{}", name)
        )?;
        
        staging.update_slice(data)?;

        let gpu_buffer = Self::new(
            context, 
            size, 
            usage | vk::BufferUsageFlags::TRANSFER_DST, 
            MemoryLocation::GpuOnly,
            name
        )?;

        context.immediate_submit(|cmd| {
            let copy = vk::BufferCopy { src_offset: 0, dst_offset: 0, size };
            unsafe {
                context.device.cmd_copy_buffer(cmd, staging.buffer, gpu_buffer.buffer, &[copy]);
            }
        })?;

        staging.destroy(context);

        Ok(gpu_buffer)
    }

    pub fn update_slice<T: Copy>(&mut self, data: &[T]) -> Result<(), vk::Result> {
        if let Some(alloc) = &self.allocation {
             if let Some(ptr) = alloc.mapped_ptr() {
                 let size_bytes = (std::mem::size_of::<T>() * data.len()) as u64;
                 if size_bytes > self.size {
                     return Err(vk::Result::ERROR_MEMORY_MAP_FAILED);
                 }
                 unsafe {
                     std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr() as *mut T, data.len());
                 }
                 return Ok(());
             }
        }
        Err(vk::Result::ERROR_OUT_OF_HOST_MEMORY)
    }
    
    // For single struct (Uniforms)
    pub fn update_item<T: Copy>(&mut self, data: T) -> Result<(), vk::Result> {
        self.update_slice(std::slice::from_ref(&data))
    }

    pub fn destroy(&mut self, context: &VulkanContext) {
        let device = &context.device;
        unsafe { device.destroy_buffer(self.buffer, None); }
        if let Some(alloc) = self.allocation.take() {
            let mut allocator = context.allocator.lock().unwrap();
            let _ = allocator.free(alloc);
        }
    }
}
