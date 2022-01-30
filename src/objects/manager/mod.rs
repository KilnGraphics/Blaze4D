//! Management of vulkan objects.
//!
//! Contains structs and enums to manage creation, access to and destruction of vulkan objects.
//!
//! Access to objects is controlled using synchronization groups. All objects belonging to a
//! synchronization group are accessed as one unit protected by a single timeline semaphore.
//!
//! Allocation and destruction of objects is managed through object sets. A objects set is a
//! collection of objects that have the same lifetime. All objects are created when creating the set
//! and all objects are destroyed only when the entire set is destroyed. All objects of a set
//! belong to the same synchronization group.
//!
//! Both synchronization groups as well as objects sets are managed by smart pointers eliminating
//! the need for manual lifetime management. Object sets keep a reference to their synchronization
//! group internally meaning that if a synchronization group is needed only for a single objects set
//! it suffices to keep the object set alive to also ensure the synchronization group stays alive.
//!
//! Multiple object sets can be accessed in a sequentially consistent manner by using
//! synchronization group sets. This is required to prevent deadlock situations when trying to
//! access multiple sets for the same operation.

pub(super) mod synchronization_group;
pub(super) mod object_set;

mod resource_object_set;
mod swapchain_object_set;

use std::sync::Arc;

use ash::vk;

use synchronization_group::*;
use crate::objects::allocator::*;
use crate::util::slice_splitter::Splitter;

pub use object_set::ObjectSetProvider;
pub use resource_object_set::ResourceObjectSetBuilder;
pub use swapchain_object_set::SwapchainObjectSetBuilder;

use crate::objects::id::SurfaceId;
use crate::objects::manager::resource_object_set::{ObjectCreateError, ResourceObjectCreateMetadata, ResourceObjectCreator, ResourceObjectData};
use crate::objects::swapchain::SwapchainCreateDesc;
use crate::UUID;