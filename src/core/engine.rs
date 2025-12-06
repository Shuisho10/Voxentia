use ash::vk;
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::vulkan::{
    context::VulkanContext,
    pipeline::TestPipeline,
    swapchain::{SurfaceSwapchain, SurfaceSync},
};

#[allow(unused)]
pub struct VoxelEngine {
    pub frame: usize,
    pub window: Window,
    pub vkcontext: VulkanContext,
    pub swapchain: SurfaceSwapchain,
    pub pipeline: TestPipeline,
    pub sync: SurfaceSync,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
}

impl VoxelEngine {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self, vk::Result> {
        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("Window not created");
        let vkcontext = VulkanContext::new(&window).expect("Vulkan context not initializated");
        let swapchain = SurfaceSwapchain::new(&vkcontext, 0, 0).expect("Swapchain not created");
        let image_count = swapchain.images.len();
        let sync = SurfaceSync::new(&vkcontext.device, image_count)?;
        let pipeline = TestPipeline::new(&vkcontext, &swapchain).expect("Pipeline not created");
        let command_pool = unsafe {
            let create_info = vk::CommandPoolCreateInfo::default()
                .queue_family_index(vkcontext.compute_queue_fi)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            vkcontext.device.create_command_pool(&create_info, None)?
        };
        let command_buffers = unsafe {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(image_count as u32);
            vkcontext.device.allocate_command_buffers(&allocate_info)?
        };
        Ok(Self {
            frame: 0,
            window,
            vkcontext,
            swapchain,
            pipeline,
            sync,
            command_pool,
            command_buffers,
        })
    }

    pub fn draw_frame(&mut self) -> Result<(), vk::Result> {
        let device = &self.vkcontext.device;
        let current_frame = self.sync.current_frame;
        unsafe {
            device.wait_for_fences(&[self.sync.in_flight_fences[current_frame]], true, u64::MAX)?;

            let (image_index, _) = self.swapchain.swapchain_loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                self.sync.image_available_semaphores[current_frame],
                vk::Fence::null(),
            )?;

            device.reset_fences(&[self.sync.in_flight_fences[current_frame]])?;

            let cmd = self.command_buffers[current_frame];
            let wait_semaphores = [self.sync.image_available_semaphores[current_frame]];
            let signal_semaphores = [self.sync.render_finished_semaphores[image_index as usize]];
            let command_buffers = [cmd];
            let wait_stages = [vk::PipelineStageFlags::COMPUTE_SHADER];
            let swapchains = [self.swapchain.swapchain];
            let image_indices = [image_index];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .signal_semaphores(&signal_semaphores)
                .command_buffers(&command_buffers);
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())?;

            self.record_compute_commands(cmd, image_index as usize)?;

            device.queue_submit(
                self.vkcontext.compute_queue,
                &[submit_info],
                self.sync.in_flight_fences[current_frame],
            )?;

            self.swapchain
                .swapchain_loader
                .queue_present(self.vkcontext.compute_queue, &present_info)?;
        }

        self.frame = (self.frame + 1) % usize::MAX;
        self.sync.current_frame = (self.sync.current_frame + 1) % self.sync.in_flight_fences.len();

        Ok(())
    }

    pub fn rebuild_swapchain(&mut self, width: u32, height: u32) -> Result<(), vk::Result> {
        let device = &self.vkcontext.device;
        unsafe {
            device.device_wait_idle()?;
            for view in &self.swapchain.image_views {
                device.destroy_image_view(*view, None);
            }
            self.swapchain
                .swapchain_loader
                .destroy_swapchain(self.swapchain.swapchain, None);
            let new_swapchain = SurfaceSwapchain::new(&self.vkcontext, width, height)?;
            self.swapchain = new_swapchain;
            // TODO recreate sync objects and command buffers if images.len() changed
            self.pipeline
                .update_descriptors(&self.vkcontext, &self.swapchain);
        }
        Ok(())
    }

    fn record_compute_commands(
        &self,
        cmd: vk::CommandBuffer,
        image_index: usize,
    ) -> Result<(), vk::Result> {
        let device = &self.vkcontext.device;
        let target_image = self.swapchain.images[image_index];
        let x_groups = (self.swapchain.extent.width + 15).div_ceil(16);
        let y_groups = (self.swapchain.extent.height + 15).div_ceil(16);
        let begin_info = vk::CommandBufferBeginInfo::default();
        let subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let barrier_to_compute = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(target_image)
            .subresource_range(subresource_range)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::SHADER_WRITE);
        let barrier_to_present = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(target_image)
            .subresource_range(subresource_range)
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::empty());

        unsafe {
            device.begin_command_buffer(cmd, &begin_info)?;
            device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier_to_compute],
            );
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.pipeline.pipeline);
            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline.layout,
                0,
                &[self.pipeline.descriptor_sets[image_index]],
                &[],
            );
            device.cmd_dispatch(cmd, x_groups, y_groups, 1);
            device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier_to_present],
            );
            device.end_command_buffer(cmd)?;
        }
        Ok(())
    }
}
