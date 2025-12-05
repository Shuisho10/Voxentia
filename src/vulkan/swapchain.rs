use ash::vk;

use crate::vulkan::context::VulkanContext;

pub struct SurfaceSync {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub current_frame: usize,
}

pub struct SurfaceSwapchain {
    pub swapchain_loader: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
}

impl SurfaceSync {
    pub fn new(device: &ash::Device, count: usize) -> Result<Self, vk::Result> {
        // TODO better allocation
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let mut image_available_semaphores = Vec::<vk::Semaphore>::new();
        let mut render_finished_semaphores = Vec::<vk::Semaphore>::new();
        let mut in_flight_fences = Vec::<vk::Fence>::new();

        for _ in 0..count {
            unsafe {
                image_available_semaphores.push(device.create_semaphore(&semaphore_info, None)?);
                render_finished_semaphores.push(device.create_semaphore(&semaphore_info, None)?);
                in_flight_fences.push(device.create_fence(&fence_info, None)?);
            }
        }
        Ok(Self {
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
        })
    }
}

impl SurfaceSwapchain {
    pub fn new(context: &VulkanContext, width: u32, height: u32) -> Result<Self, vk::Result> {
        let capabilities = unsafe {
            context
                .surface_loader
                .get_physical_device_surface_capabilities(
                    context.physical_device,
                    context.surface,
                )?
        };
        let formats = unsafe {
            context
                .surface_loader
                .get_physical_device_surface_formats(context.physical_device, context.surface)?
        };
        let present_modes = unsafe {
            context
                .surface_loader
                .get_physical_device_surface_present_modes(
                    context.physical_device,
                    context.surface,
                )?
        };
        let format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::R8G8B8A8_UNORM
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&formats[0]);
        let extent = if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        };
        let present_mode = present_modes
            .iter()
            .find(|&p| *p == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(&vk::PresentModeKHR::FIFO);
        let mut usage = vk::ImageUsageFlags::TRANSFER_DST;
        if capabilities
            .supported_usage_flags
            .contains(vk::ImageUsageFlags::STORAGE)
        {
            usage |= vk::ImageUsageFlags::STORAGE
        };
        let image_count = if capabilities.max_image_count > 0 {
            (capabilities.min_image_count + 1).min(capabilities.max_image_count)
        } else {
            capabilities.min_image_count + 1
        };
        let swapchain_loader = ash::khr::swapchain::Device::new(&context.instance, &context.device);
        let swapchain = unsafe {
            let create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(context.surface)
                .min_image_count(image_count)
                .image_format(format.format)
                .image_color_space(format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(usage)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(*present_mode)
                .clipped(true);
            swapchain_loader.create_swapchain(&create_info, None)?
        };
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let image_views = images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format.format)
                    .components(vk::ComponentMapping::default())
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe { context.device.create_image_view(&create_info, None) }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            swapchain_loader,
            swapchain,
            images,
            image_views,
            surface_format: *format,
            extent,
        })
    }
}
