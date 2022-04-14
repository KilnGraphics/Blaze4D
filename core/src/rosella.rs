pub use crate::instance::VulkanVersion;
pub use crate::instance::InstanceContextImpl;
pub use crate::device::DeviceContext;
use crate::objects::id::SurfaceId;

pub struct Rosella {
    pub instance: InstanceContextImpl,
    pub surface: SurfaceId,
    pub device: DeviceContext,
}