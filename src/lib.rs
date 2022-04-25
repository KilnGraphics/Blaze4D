#[macro_use]
extern crate static_assertions;

pub mod vk;
pub mod util;
pub mod b4d;

pub mod debug;

pub use util::id::UUID;
pub use util::id::NamedUUID;

mod glfw_surface;
mod renderer;
pub mod window;

pub mod prelude {
    pub type DeviceContext = crate::vk::device::DeviceContext;
    pub type InstanceContext = crate::vk::instance::InstanceContext;

    pub type Vec2f32 = nalgebra::Vector2<f32>;
    pub type Vec3f32 = nalgebra::Vector3<f32>;

    pub type Vec2u32 = nalgebra::Vector2<u32>;
}