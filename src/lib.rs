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
pub mod transfer;

pub mod prelude {
    pub use crate::UUID;
    pub use crate::NamedUUID;

    pub type DeviceContext = crate::vk::device::DeviceContext;
    pub type InstanceContext = crate::vk::instance::InstanceContext;

    pub type Vec2f32 = nalgebra::Vector2<f32>;
    pub type Vec3f32 = nalgebra::Vector3<f32>;

    pub type Vec2u32 = nalgebra::Vector2<u32>;
    pub type Vec3u32 = nalgebra::Vector3<u32>;

    pub type Vec2i32 = nalgebra::Vector2<i32>;
    pub type Vec3i32 = nalgebra::Vector3<i32>;
}