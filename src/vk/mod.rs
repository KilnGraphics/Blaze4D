//! This is a legacy module
//!
//! Some of the structs and functions are still used in places but the ultimate goal is to either
//! remove or move all of those and delete this module.

pub mod objects;

pub use crate::instance::instance::InstanceContext;
pub use crate::device::device::DeviceEnvironment;

#[cfg(any(test, feature = "__internal_doc_test"))]
pub mod test;
