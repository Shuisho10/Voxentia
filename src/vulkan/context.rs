use std::sync::Mutex;

use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

#[allow(unused)]
pub struct VulkanContext {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub compute_queue: vk::Queue,
    pub compute_queue_fi: u32,
    pub command_pool: vk::CommandPool,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
    pub allocator: Mutex<Allocator>,
}

impl VulkanContext {
    pub fn new(window: &Window) -> Result<Self, vk::Result> {
        // TODO allocation callbacks
        // TODO better physical device picker
        let entry =
            unsafe { ash::Entry::load().map_err(|_| vk::Result::ERROR_INITIALIZATION_FAILED)? };
        let display_handle = window
            .display_handle()
            .map_err(|_| vk::Result::ERROR_INITIALIZATION_FAILED)?
            .as_raw();
        let window_handle = window
            .window_handle()
            .map_err(|_| vk::Result::ERROR_INITIALIZATION_FAILED)?
            .as_raw();
        let instance = unsafe {
            let app_info = vk::ApplicationInfo::default()
                .engine_name(c"Voxentia")
                .api_version(vk::API_VERSION_1_3)
                .engine_version(vk::make_api_version(0, 0, 1, 0))
                .application_name(c"Voxentia Example")
                .application_version(vk::make_api_version(0, 0, 1, 0));
            let extension_names =
                ash_window::enumerate_required_extensions(display_handle)?.to_vec();
            let validation_layers = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_layer_names(&validation_layers)
                .enabled_extension_names(&extension_names);
            entry
                .create_instance(&create_info, None)
                .map_err(|_| vk::Result::ERROR_INITIALIZATION_FAILED)?
        };
        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
        }?;
        let pdevices = unsafe { instance.enumerate_physical_devices()? };
        let (physical_device, compute_queue_fi) = unsafe {
            pdevices
                .iter()
                .find_map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let supports_compute =
                                info.queue_flags.contains(vk::QueueFlags::COMPUTE);
                            let supports_surface = surface_loader
                                .get_physical_device_surface_support(
                                    *pdevice,
                                    index as u32,
                                    surface,
                                )
                                .unwrap_or(false);
                            if supports_compute && supports_surface {
                                Some((*pdevice, index as u32))
                            } else {
                                None
                            }
                        })
                })
                .ok_or(vk::Result::ERROR_DEVICE_LOST)?
        };
        let mut bda_features = vk::PhysicalDeviceBufferDeviceAddressFeatures::default()
            .buffer_device_address(true)
            .buffer_device_address_capture_replay(false)
            .buffer_device_address_multi_device(false);


        let device = unsafe {
            let queue_priorities = [1.0];
            let queue_infos = [vk::DeviceQueueCreateInfo::default()
                .queue_priorities(&queue_priorities)
                .queue_family_index(compute_queue_fi)];
            let extensions = [ash::khr::swapchain::NAME.as_ptr()];
            let create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&queue_infos)
                .enabled_extension_names(&extensions)
                .push_next(&mut bda_features);
            instance.create_device(physical_device, &create_info, None)
        }?;
        let compute_queue = unsafe { device.get_device_queue(compute_queue_fi, 0) };
        let command_pool = unsafe {
            let create_info = vk::CommandPoolCreateInfo::default()
                .queue_family_index(compute_queue_fi)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            device.create_command_pool(&create_info, None)
        }?;

        let allocator = {
            let desc = AllocatorCreateDesc {
                instance: instance.clone(),
                device: device.clone(),
                physical_device,
                buffer_device_address: true,
                debug_settings: Default::default(),
                allocation_sizes: Default::default(),
            };
            Mutex::new(Allocator::new(&desc).map_err(|_| vk::Result::ERROR_INITIALIZATION_FAILED)?)
        };

        Ok(Self {
            entry,
            instance,
            device,
            physical_device,
            compute_queue_fi,
            compute_queue,
            command_pool,
            surface,
            surface_loader,
            allocator,
        })
    }
}
