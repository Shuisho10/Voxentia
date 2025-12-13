use crate::vulkan::buffer::Buffer;
use crate::vulkan::context::VulkanContext;
use ash::vk;

pub struct VoxelGenerator {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
}

impl VoxelGenerator {
    pub fn new(
        context: &VulkanContext,
        dir_buffer: &Buffer,
        pool_buffer: &Buffer,
    ) -> Result<Self, vk::Result> {
        let device = &context.device;

        unsafe {
            let bindings = [
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE),
            ];

            let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
            let ds_layout = device.create_descriptor_set_layout(&layout_info, None)?;

            // 2. Pipeline Layout
            let push_constant = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
                .offset(0)
                .size(12); // ivec3 (4*3 bytes)

            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(std::slice::from_ref(&ds_layout))
                .push_constant_ranges(std::slice::from_ref(&push_constant));

            let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_info, None)?;

            // 3. Shader

            let shader_module = unsafe {
                let code = include_bytes!("../vulkan/shaders/generate.spv");
                let code_u32 = ash::util::read_spv(&mut std::io::Cursor::new(&code))
                    .expect("generate.spv counld't be read properly");

                let create_info = vk::ShaderModuleCreateInfo::default().code(&code_u32);

                context.device.create_shader_module(&create_info, None)?
            };
            let stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::COMPUTE)
                .module(shader_module)
                .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap());

            let pipeline_info = vk::ComputePipelineCreateInfo::default()
                .stage(stage_info)
                .layout(pipeline_layout);

            let pipeline = device
                .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|e| e.1)?[0];

            device.destroy_shader_module(shader_module, None);

            // 4. Allocate Descriptor Set
            let pool_size = [vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 2,
            }];
            let pool_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_size)
                .max_sets(1);
            let descriptor_pool = device.create_descriptor_pool(&pool_info, None)?;

            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(std::slice::from_ref(&ds_layout));
            let descriptor_set = device.allocate_descriptor_sets(&alloc_info)?[0];

            // 5. Update Descriptors
            let dir_info = vk::DescriptorBufferInfo::default()
                .buffer(dir_buffer.buffer)
                .range(vk::WHOLE_SIZE);
            let pool_info = vk::DescriptorBufferInfo::default()
                .buffer(pool_buffer.buffer)
                .range(vk::WHOLE_SIZE);

            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(&dir_info)),
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(&pool_info)),
            ];
            device.update_descriptor_sets(&writes, &[]);

            Ok(Self {
                pipeline,
                pipeline_layout,
                descriptor_set_layout: ds_layout,
                descriptor_pool,
                descriptor_set,
            })
        }
    }

    pub fn run(
        &self,
        context: &VulkanContext,
        start_chunk: [i32; 3],
        num_chunks: [u32; 3],
    ) -> Result<(), vk::Result> {
        context.immediate_submit(|cmd| {
            unsafe {
                context.device.cmd_bind_pipeline(
                    cmd,
                    vk::PipelineBindPoint::COMPUTE,
                    self.pipeline,
                );
                context.device.cmd_bind_descriptor_sets(
                    cmd,
                    vk::PipelineBindPoint::COMPUTE,
                    self.pipeline_layout,
                    0,
                    &[self.descriptor_set],
                    &[],
                );

                // Push Constants
                let pc_bytes = std::slice::from_raw_parts(start_chunk.as_ptr() as *const u8, 12);
                context.device.cmd_push_constants(
                    cmd,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::COMPUTE,
                    0,
                    pc_bytes,
                );

                context.device.cmd_dispatch(
                    cmd,
                    num_chunks[0],
                    num_chunks[1],
                    num_chunks[2],
                );
            }
        })
    }
}
