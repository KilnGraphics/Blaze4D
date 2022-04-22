extern crate core;

pub mod vk;
pub mod util;
pub mod b4d;

pub use util::id::UUID;
pub use util::id::NamedUUID;

mod glfw_surface;
mod renderer;
pub mod window;