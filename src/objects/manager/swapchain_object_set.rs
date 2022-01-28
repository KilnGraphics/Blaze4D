use ash::vk;
use crate::objects::ObjectManager;
use crate::objects::id::SurfaceId;
use crate::objects::swapchain::SwapchainCreateDesc;

pub struct SwapchainObjectSetBuilder {
    manager: ObjectManager,
    surface: SurfaceId,
    swapchain: vk::SwapchainKHR,
    image_count: usize,
}

impl SwapchainObjectSetBuilder {
    fn new(manager: ObjectManager, surface: SurfaceId, desc: SwapchainCreateDesc) -> Self {
        todo!()
    }
}