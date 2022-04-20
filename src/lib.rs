extern crate core;

pub mod init;
pub mod shader;
pub mod objects;
pub mod util;
pub mod window;

mod instance;
mod device;

pub use util::id::UUID;
pub use util::id::NamedUUID;

pub use instance::InstanceContext;
pub use device::DeviceContext;

#[cfg(any(test, feature = "__internal_doc_test"))]
pub use util::test;