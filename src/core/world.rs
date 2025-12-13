use ash::vk;
use gpu_allocator::MemoryLocation;
use nalgebra::{Vector2, Vector3};

use crate::{core::generator::VoxelGenerator, vulkan::{buffer::Buffer, context::VulkanContext}};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub const WORLD_CHUNKS: usize = 32;
pub const DIR_SIZE: usize = WORLD_CHUNKS * WORLD_CHUNKS * WORLD_CHUNKS;
pub const MAX_CHUNKS: usize = 2048;

pub struct ChunkedWorld {
    pub dir_buffer: Buffer,
    pub pool_buffer: Buffer,
    pub active_chunk_count: u32,
    pub generator: VoxelGenerator,
}

impl ChunkedWorld {
    pub fn new(context: &VulkanContext) -> Result<Self, vk::Result> {
        let mut dir_data = vec![1u32; DIR_SIZE];

        let range_x = 16;
        let range_y = 8;
        let range_z = 16;
        let start_x = (WORLD_CHUNKS - range_x) / 2;
        let start_y = 0;
        let start_z = (WORLD_CHUNKS - range_z) / 2;
        let mut pool_id_counter = 1;

        for x in 0..range_x {
            for y in 0..range_y {
                for z in 0..range_z {
                    let cx = start_x + x;
                    let cy = start_y + y;
                    let cz = start_z + z;
                    let dir_idx = cx + (cy * WORLD_CHUNKS) + (cz * WORLD_CHUNKS * WORLD_CHUNKS);
                    
                    if pool_id_counter < MAX_CHUNKS {
                        dir_data[dir_idx] = pool_id_counter as u32;
                        pool_id_counter += 1;
                    }
                }
            }
        }


        let pool_size = (CHUNK_VOLUME * MAX_CHUNKS * std::mem::size_of::<u32>()) as u64;
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

        let generator = VoxelGenerator::new(context, &dir_buffer, &pool_buffer)?;
        let start = [start_x as i32, start_y as i32, start_z as i32];
        let size = [range_x as u32, range_y as u32, range_z as u32];
        
        generator.run(context, start, size)?;

        Ok(Self {
            dir_buffer,
            pool_buffer,
            active_chunk_count: 1,
            generator
        })
    }
}
