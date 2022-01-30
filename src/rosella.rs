use crate::init::device::{create_device, DeviceCreateError};
use crate::init::initialization_registry::InitializationRegistry;
use crate::init::instance::{create_instance, InstanceCreateError};
use crate::window::{RosellaSurface, RosellaWindow};

use crate::init::rosella_features::WindowSurface;

pub use crate::instance::VulkanVersion;
pub use crate::instance::InstanceContext;
pub use crate::device::DeviceContext;
use crate::objects::id::SurfaceId;
use crate::objects::surface::Surface;

pub struct Rosella {
    pub instance: InstanceContext,
    pub surface: SurfaceId,
    pub device: DeviceContext,
}

#[derive(Debug)]
pub enum RosellaCreateError {
    InstanceCreateError(InstanceCreateError),
    DeviceCreateError(DeviceCreateError),
}

impl From<InstanceCreateError> for RosellaCreateError {
    fn from(err: InstanceCreateError) -> Self {
        RosellaCreateError::InstanceCreateError(err)
    }
}

impl From<DeviceCreateError> for RosellaCreateError {
    fn from(err: DeviceCreateError) -> Self {
        RosellaCreateError::DeviceCreateError(err)
    }
}

impl Rosella {
    pub fn new(mut registry: InitializationRegistry, window: &RosellaWindow, application_name: &str) -> Result<Rosella, RosellaCreateError> {
        log::info!("Starting Rosella");

        WindowSurface::register_into(&mut registry, &window.handle, true);

        let now = std::time::Instant::now();

        let instance = create_instance(&mut registry, application_name, 0)?;

        let surface = Surface::new(Box::new(RosellaSurface::new(&instance, window)));
        let surface_id = surface.get_id();

        let device = create_device(&mut registry, instance.clone(), &[surface])?;

        let elapsed = now.elapsed();
        println!("Instance & Device Initialization took: {:.2?}", elapsed);

        Ok(Rosella {
            instance,
            surface: surface_id,
            device,
        })
    }

    pub fn window_update(&self) {}

    pub fn recreate_swapchain(&self, width: u32, height: u32) {
        println!("resize to {}x{}", width, height);
    }
}