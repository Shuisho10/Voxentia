use ash::vk;

use crate::vulkan::{context::VulkanContext, swapchain::SurfaceSwapchain};

#[allow(unused)]
pub struct TestPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

impl TestPipeline {
    pub fn new(context: &VulkanContext, swapchain: &SurfaceSwapchain) -> Result<Self, vk::Result> {
        let descriptor_set_layout = unsafe {
            let bindings = [
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE),
                //vk::DescriptorSetLayoutBinding::default()
                //    .binding(1)
                //    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                //    .descriptor_count(1)
                //    .stage_flags(vk::ShaderStageFlags::COMPUTE),
                //vk::DescriptorSetLayoutBinding::default()
                //    .binding(2)
                //    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                //    .descriptor_count(1)
                //    .stage_flags(vk::ShaderStageFlags::COMPUTE),
            ];

            let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

            context.device.create_descriptor_set_layout(&layout_info, None)?
        };

        let layout = unsafe {
            let set_layouts = [descriptor_set_layout];

            let push_constant_ranges = []; //time?

            let create_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(&set_layouts)
                .push_constant_ranges(&push_constant_ranges);

            context.device.create_pipeline_layout(&create_info, None)?
        };

        let shader_module = unsafe {
            let code = include_bytes!("shaders/raytrace.spv");
            let code_u32 = ash::util::read_spv(&mut std::io::Cursor::new(&code))
                .expect("raytrace.spv counld't be read properly");

            let create_info = vk::ShaderModuleCreateInfo::default().code(&code_u32);

            context.device.create_shader_module(&create_info, None)?
        };

        let pipeline = unsafe {
            let entry_point_name = c"main";

            let stage = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::COMPUTE)
                .module(shader_module)
                .name(&entry_point_name);

            let create_info = vk::ComputePipelineCreateInfo::default()
                .stage(stage)
                .layout(layout);
            // TODO cache and allocation callbacks
            context.device.create_compute_pipelines(vk::PipelineCache::null(), &[create_info], None).map_err(|e| e.1)?[0]
        };

        unsafe {
            context.device.destroy_shader_module(shader_module, None);
        }

        let image_len = swapchain.images.len();
        let descriptor_pool = unsafe {
            let pool_sizes = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_IMAGE,
                    descriptor_count: image_len as u32,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: image_len as u32,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: image_len as u32,
                },
            ];

            let create_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                .max_sets(image_len as u32);

            context.device.create_descriptor_pool(&create_info, None)?
        };

        let descriptor_sets = unsafe {
            let set_layouts = vec![descriptor_set_layout; image_len];

            let allocate_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&set_layouts);

            context.device.allocate_descriptor_sets(&allocate_info)?
        };

        let test_pipeline = Self {
            pipeline,
            layout,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,
        };

        test_pipeline.update_descriptors(context, swapchain);

        Ok(test_pipeline)
    }

    pub fn update_descriptors(&self, context: &VulkanContext, swapchain: &SurfaceSwapchain){
        for (i, descriptor_set) in self.descriptor_sets.iter().enumerate().take(swapchain.images.len()) {
            let image_info = [vk::DescriptorImageInfo::default()
                .image_view(swapchain.image_views[i])
                .image_layout(vk::ImageLayout::GENERAL)];

            let write_image = vk::WriteDescriptorSet::default()
                .dst_set(*descriptor_set)
                .dst_binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(&image_info);

            //let buffer_info = vk::DescriptorBufferInfo::default()
            //    .buffer(camera_buffer)
            //    .offset(0)
            //    .range(vk::WHOLE_SIZE);
            //let write_camera = vk::WriteDescriptorSet::default()
            //    .dst_set(*descriptor_set)
            //    .dst_binding(1)
            //    .descriptor_count(1)
            //    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            //    .buffer_info(&buffer_info);
            //let buffer_info = vk::DescriptorBufferInfo::default()
            //    .buffer(camera_buffer)
            //    .offset(0)
            //    .range(vk::WHOLE_SIZE);
            //let write_camera = vk::WriteDescriptorSet::default()
            //    .dst_set(*descriptor_set)
            //    .dst_binding(1)
            //    .descriptor_count(1)
            //    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            //    .buffer_info(&buffer_info);
            unsafe {
                context.device.update_descriptor_sets(&[write_image], &[]);
            }
        }
    }
}
