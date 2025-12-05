use ash::vk;

pub struct SurfaceSync {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub current_frame: usize,
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
            unsafe{
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
