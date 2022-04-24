//! Debug overlay

use ash::vk;
use crate::vk::device::VkQueue;
use crate::vk::DeviceContext;

use crate::vk::objects::{BufferDescription, Format, ImageDescription, ImageSize, ImageSpec, ObjectSet, ResourceObjectSet};
use crate::vk::objects::types::{BufferId, ImageId};

pub mod text;

struct DebugRenderer {
    resources: ObjectSet,
    staging_buffer: vk::Buffer,
}

impl DebugRenderer {
    pub fn new(device: DeviceContext) {
        let queue = device.get_main_queue();

        let staging_size = 32000000u64;

        let set = ResourceObjectSet::new(device.clone());
        let staging_id = Self::crete_staging(&set, staging_size);
        let atlas_id = Self::create_images(&set, (256, 256));

        

        let pool = Self::create_command_pool(&device, &queue);
    }

    fn create_command_pool(device: &DeviceContext, queue: &VkQueue) -> vk::CommandPool {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue.get_queue_family_index());

        unsafe { device.vk().create_command_pool(&info, None) }.unwrap()
    }

    fn crete_staging(set: &ResourceObjectSet, size: u64) -> BufferId {
        let desc = BufferDescription::new_simple(size,
            vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST
        );

        set.add_default_gpu_cpu_buffer(&desc)
    }

    fn create_images(set: &ResourceObjectSet, size: (u32, u32)) -> ImageId {
        let desc = ImageDescription::new_simple(
            ImageSpec::new_single_sample(
                ImageSize::make_2d_array(size.0, size.1, 1),
                &Format::R8G8B8_UNORM,
            ),
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT
        );

        set.add_default_gpu_only_image(&desc)
    }

    fn upload_image(buffer: vk::Buffer, image: vk::Image, pool: vk::CommandPool, device: &DeviceContext, queue: &VkQueue, data: &[u8]) {
    }
}