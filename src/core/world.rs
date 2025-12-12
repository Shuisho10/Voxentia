use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::vulkan::{buffer::Buffer, context::VulkanContext};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

// TODO infinite with render distance
pub const WORLD_CHUNKS: usize = 32;
pub const DIR_SIZE: usize = WORLD_CHUNKS * WORLD_CHUNKS * WORLD_CHUNKS;
pub const MAX_CHUNKS: usize = 2048;

pub struct ChunkedWorld {
    pub dir_buffer: Buffer,
    pub pool_buffer: Buffer,
    pub active_chunk_count: u32,
}

impl ChunkedWorld {
    pub fn new(context: &VulkanContext) -> Result<Self, vk::Result> {
        let mut dir_data = vec![0u32; DIR_SIZE];
        let mut pool_data = vec![0u32; CHUNK_VOLUME];
        let pool_size = (CHUNK_VOLUME * MAX_CHUNKS * std::mem::size_of::<u32>()) as u64;
        // sphere
        let center = 16.0;
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let dx = x as f32 - center;
                    let dy = y as f32 - center;
                    let dz = z as f32 - center;
                    if (dx * dx + dy * dy + dz * dz).sqrt() < 14.0 {
                        let idx = x + (y * CHUNK_SIZE) + (z * CHUNK_SIZE * CHUNK_SIZE);
                        pool_data[idx] = 1;
                    }
                }
            }
        }
        let center_chunk = WORLD_CHUNKS / 2;
        // TODO starting to think i need a helper function for 3d in array
        let dir_idx = center_chunk + (center_chunk * WORLD_CHUNKS) + (center_chunk * WORLD_CHUNKS);
        dir_data[dir_idx] = 1;
        let mut dir_buffer = Buffer::new(
            context,
            (DIR_SIZE * 4) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::CpuToGpu,
            "Chunk Directory",
        )?;
        dir_buffer.update_slice(&dir_data)?;

        let pool_buffer = Buffer::new(
            context,
            pool_size as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
            "Chunk Pool",
        )?;

        let mut upload_data = Vec::with_capacity(CHUNK_VOLUME * 2);
        upload_data.extend(std::iter::repeat(0u32).take(CHUNK_VOLUME));
        upload_data.extend_from_slice(&pool_data);

        let mut staging = Buffer::new(
                context,
                (upload_data.len() * 4) as u64,
                vk::BufferUsageFlags::TRANSFER_SRC,
                MemoryLocation::CpuToGpu,
                "Pool Upload Staging",
            )
            .unwrap(); // i forgor
            staging.update_slice(&upload_data).unwrap();

        context.immediate_submit(|cmd| {
            

            let copy = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: (upload_data.len() * 4) as u64,
            };

            unsafe {
                context
                    .device
                    .cmd_copy_buffer(cmd, staging.buffer, pool_buffer.buffer, &[copy]);
            }
        })?; // empty and sphere
        Ok(Self {
            dir_buffer,
            pool_buffer,
            active_chunk_count: 1,
        })
    }
}
