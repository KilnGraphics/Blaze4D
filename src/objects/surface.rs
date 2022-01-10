use std::cmp::Ordering;
use std::sync::Arc;

use ash::vk;

use crate::objects::id::SurfaceId;
use crate::rosella::InstanceContext;
use crate::util::id::GlobalId;

/// Trait that provides access to a surface object.
///
/// Since many possible surface objects exits and management of these can differ this trait is
/// used to abstract those differences away. Rosella will only access surfaces using a trait object
/// of this type. Once the trait object is dropped it may assume that the surface is no longer used
/// by rosella and is safe to be destroyed.
///
/// Note: While dropping of a surface typically is a rare occurrence it *may* happen synchronously
/// with other engine operations. As such extensive computations or blocking operations should be
/// avoided in the drop function.
pub trait SurfaceProvider : Sync {
    fn get_handle(&self) -> vk::SurfaceKHR;
}

struct SurfaceImpl {
    id: SurfaceId,
    surface: Box<dyn SurfaceProvider>,
}

/// Wrapper struct for surfaces.
///
/// Provides access to a surface provider using a arc.
#[derive(Clone)]
pub struct Surface(Arc<SurfaceImpl>);

impl Surface {
    pub fn new(surface: Box<dyn SurfaceProvider>) -> Self {
        Self(Arc::new(SurfaceImpl{
            id: SurfaceId::new(GlobalId::new(), 0),
            surface
        }))
    }

    pub fn get_handle(&self) -> vk::SurfaceKHR {
        self.0.surface.get_handle()
    }

    pub fn get_id(&self) -> SurfaceId {
        self.0.id
    }
}

impl PartialEq<Self> for Surface {
    fn eq(&self, other: &Self) -> bool {
        self.0.id.eq(&other.0.id)
    }
}

impl Eq for Surface {
}

impl PartialOrd for Surface {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.id.partial_cmp(&other.0.id)
    }
}

impl Ord for Surface {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.id.cmp(&other.0.id)
    }
}

pub struct SurfaceCapabilities {
    presentable_queues: Box<[u32]>,
    surface_formats: Box<[vk::SurfaceFormatKHR]>,
    present_modes: Box<[vk::PresentModeKHR]>,
    capabilities: vk::SurfaceCapabilitiesKHR,
}

impl SurfaceCapabilities {
    pub fn new(instance: &InstanceContext, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Option<Self> {
        let surface_fn = instance.get_extension::<ash::extensions::khr::Surface>()?;
        let family_count = unsafe {
            instance.vk().get_physical_device_queue_family_properties(physical_device).len()
        } as u32;

        let presentable_queues = (0..family_count).filter(|family| unsafe {
            surface_fn.get_physical_device_surface_support(physical_device, *family, surface).unwrap()
        }).collect::<Vec<_>>().into_boxed_slice();

        if presentable_queues.len() == 0 {
            return None;
        }

        let capabilities = unsafe {
            surface_fn.get_physical_device_surface_capabilities(physical_device, surface)
        }.ok()?;

        let surface_formats = unsafe {
            surface_fn.get_physical_device_surface_formats(physical_device, surface)
        }.ok()?.into_boxed_slice();

        let present_modes = unsafe {
            surface_fn.get_physical_device_surface_present_modes(physical_device, surface)
        }.ok()?.into_boxed_slice();

        Some(Self{
            presentable_queues,
            surface_formats,
            present_modes,
            capabilities,
        })
    }

    pub fn get_capabilities(&self) -> &vk::SurfaceCapabilitiesKHR {
        &self.capabilities
    }

    pub fn get_presentable_queue_families(&self) -> &[u32] {
        self.presentable_queues.as_ref()
    }

    pub fn get_surface_formats(&self) -> &[vk::SurfaceFormatKHR] {
        self.surface_formats.as_ref()
    }

    pub fn get_present_modes(&self) -> &[vk::PresentModeKHR] {
        self.present_modes.as_ref()
    }
}