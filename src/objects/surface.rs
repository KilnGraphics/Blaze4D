use std::cmp::Ordering;
use std::sync::Arc;

use ash::vk;

use crate::objects::id::SurfaceId;
use crate::util::id::GlobalId;

/// Trait that provides access to a surface object.
///
/// Since many possible surface objects exits and management of these can differ this trait can be
/// used to abstract those differences away. Rosella will only access the surface using a trait
/// objects. Once the trait object is dropped it may assume that the surface is no longer used by
/// rosella and is safe to be destroyed.
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